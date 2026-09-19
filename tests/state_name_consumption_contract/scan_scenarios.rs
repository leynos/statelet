//! Directory-scan scenarios over `docs/validation-notes/`.
//!
//! These are `INV-FILLED`'s controls on the *scan* rather than on a note's
//! cells: which files it reads, what it does when a read fails, and what an
//! absent directory means. They live apart from `note_scenarios.rs` because
//! each answers a question about the directory as a whole, and because the
//! scan's four controls would otherwise push the note-cell module past the
//! 400-line cap with no room for the next one.

use pretty_assertions::assert_eq;

use super::{
    fixtures::{benchmark_note, state_name_note},
    live_status,
    notes::{self, committed_notes},
    parse::note_rows,
    policy::{check_note_cells, resolve_note},
    workspace_root,
};

/// Ignores a note without the marker rather than reading its fields as a
/// `StateName` note's.
///
/// This is the accepting end of `INV-FILLED`'s marker control. The note is
/// well-formed Markdown and parses like any other; the marker is what makes it
/// a `StateName` note, and the scan reads the marker rather than globbing the
/// directory, so a note belonging to roadmap task 1.2.3 cannot fail this
/// suite's checks by arriving.
///
/// The assertions return through the error channel rather than panicking, so
/// that every way this control can fail names the artefact it read.
#[test]
fn unmarked_notes_are_ignored() -> Result<(), String> {
    let unmarked = note_rows(&state_name_note().replace(notes::MARKER, ""))
        .map_err(|error| error.to_string())?;
    if unmarked.len() != 4 {
        return Err(format!(
            "the note without its marker no longer parses as a four-row table; it yields {} rows. \
             Repair: keep the fixture note well-formed so the control isolates the marker.",
            unmarked.len()
        ));
    }
    if benchmark_note().contains(notes::MARKER) {
        return Err(
            "the benchmark note carries the StateName marker, so the control cannot show that an \
             unmarked note is ignored."
                .to_owned(),
        );
    }
    // `docs/validation-notes/README.md` *mentions* the marker inside a code
    // span. A scan reading the marker as a substring would treat the README as
    // a note and then reject it for carrying no note register; a scan reading
    // it as a line of its own does not.
    let committed = committed_notes(&workspace_root())?;
    if committed.iter().any(|note| note.file_name == "README.md") {
        return Err(
            "docs/validation-notes/README.md was read as a committed note. It documents the \
             marker inside a code span without declaring it, so the scan must match the marker as \
             a line of its own."
                .to_owned(),
        );
    }
    Ok(())
}

/// Rejects a directory entry that cannot be read, naming the path.
///
/// `INV-FILLED`'s third control. The scan's other controls all read a note that
/// *is* readable, so without this one the module's doc comment — that a failed
/// read is an error and never `None` — is an unchecked claim. The entry passed
/// here is a directory, which every filesystem this repository targets refuses
/// to read as a file.
///
/// The path is the fixture's own directory, so the array of argument types a
/// successful read would require is discarded rather than marshalled.
#[test]
fn unreadable_entries_are_an_error_not_a_skip() -> Result<(), String> {
    let directory = workspace_root().join("docs/validation-notes");
    if !directory.exists() {
        return Err(format!(
            "{directory} does not exist, so the control cannot show that an unreadable entry is \
             an error rather than a skip."
        ));
    }
    match notes::read_marked_note(&directory) {
        Err(message) if message.starts_with(&format!("{directory}: ")) => Ok(()),
        Err(message) => Err(format!(
            "the read failure was reported as {message:?}, which does not name {directory}. \
             Repair: name the path in every read failure, so an engineer is sent to the file."
        )),
        Ok(_) => Err(format!(
            "{directory} was read as a note, so this control is vacuous. Repair: pass a path that \
             cannot be read as a file."
        )),
    }
}

/// Accepts an absent notes directory rather than failing over it.
///
/// The other half of the set: the scan must distinguish an absent directory
/// (no note can honestly exist yet) from a present one it cannot read. The root
/// here has no `docs/validation-notes`, standing for a checkout before task
/// 2.2.1 has annotated anything. Cargo creates nothing at that path, so the
/// precondition holds by construction rather than by luck.
#[test]
fn an_absent_notes_directory_yields_no_notes() -> Result<(), String> {
    let absent = workspace_root().join("target/state-name-contract-absent-root");
    if absent.exists() {
        return Err(format!(
            "{absent} exists, so this control cannot show that an absent directory is not a \
             failure. Repair: name a path under `target/` that nothing creates."
        ));
    }
    assert_eq!(committed_notes(&absent), Ok(Vec::new()));
    Ok(())
}

/// Scans the live notes directory and checks every marked note.
///
/// A failure names the note it came from. A message saying only that some note
/// holds a `TBD` would send a Phase 2 engineer to the directory rather than to
/// the file.
///
/// An empty directory passes: no note can honestly exist until roadmap task
/// 2.2.1 has annotated something, so a failure there would demand a fabricated
/// observation. `committed_state_name_notes_are_rejected` and the string
/// fixtures are this invariant's non-vacuity, not the directory's contents.
#[test]
fn committed_state_name_notes_are_usable() -> Result<(), String> {
    let rows = live_status()?;
    for note in committed_notes(&workspace_root())? {
        let name = note.file_name.as_str();
        let cells = note_rows(&note.text).map_err(|error| format!("{name}: {error}"))?;
        check_note_cells(&rows, &cells).map_err(|error| format!("{name}: {error}"))?;
        resolve_note(&rows, &cells).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
