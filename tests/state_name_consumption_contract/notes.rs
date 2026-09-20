//! Directory scan over `docs/validation-notes/`.
//!
//! This is the one module of the contract that touches the ambient filesystem,
//! and it is exempted by name in the repository's `dylint.toml`; every other
//! module stays under Whitaker's `no_std_fs_operations`. It reads and decides
//! nothing: whether a note is *admissible* is `policy.rs`'s question.
//!
//! Enumeration itself goes through `camino`, the repository's path crate. Only
//! the content read needs `std::fs`, because `camino` has no content-read API
//! and the note set is deliberately open — a Phase 2 engineer adds a note months
//! from now, and its arrival must not require a Rust edit, so `include_str!`
//! cannot serve it.

use std::fs;

use camino::Utf8Path;

/// The directory holding committed validation notes.
const NOTES_DIR: &str = "docs/validation-notes";

/// The marker a note declares to opt into the `StateName` contract.
pub(crate) const MARKER: &str = "<!-- state-name-note -->";

/// A note read from disk: its file name and its complete text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommittedNote {
    pub(crate) file_name: String,
    pub(crate) text: String,
}

/// Every committed note declaring the marker, ordered by file name.
///
/// A note without the marker is ignored rather than rejected. The directory is
/// shared: roadmap task 1.2.3 produces a benchmark note, task 2.2.3 an exit
/// note, and task 3.1.3 a decision note, and none of those is a `StateName`
/// note. Keying on a declared marker keeps each one's arrival from becoming a
/// build failure.
///
/// The marker is a declaration, so it is read as one: a line consisting of the
/// marker and nothing else. The template on disk declares the marker the same
/// way, which is what a Phase 2 engineer copies.
///
/// An absent or empty directory yields an empty vector. That is deliberate: no
/// honest note can exist until task 2.2.1 has annotated something, so an empty
/// directory is not a failure. The suite's accepting witness is a string
/// fixture instead.
///
/// A *present* directory that cannot be enumerated or whose file cannot be read
/// is a different thing, and is an error naming the path. Discarding it would
/// skip the note silently, and a skipped note is indistinguishable from no note
/// at all — the same defect the marker's line-of-its-own rule exists to avoid,
/// one level down.
pub(crate) fn committed_notes(root: &Utf8Path) -> Result<Vec<CommittedNote>, String> {
    let directory = root.join(NOTES_DIR);
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let entries = directory
        .read_dir_utf8()
        .map_err(|error| format!("{directory}: the notes directory cannot be read: {error}"))?;
    let mut found = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|error| format!("{directory}: an entry cannot be read: {error}"))?
            .into_path();
        if !has_markdown_extension(path.as_path()) {
            continue;
        }
        if let Some(note) = read_marked_note(&path)? {
            found.push(note);
        }
    }
    found.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    Ok(found)
}

/// Whether a path names a Markdown file.
///
/// Asked of the path's *extension* rather than of its final characters, so that
/// the comparison is case-insensitive: a note committed as `2.2.1-mdtablefix.MD`
/// is still a note, and an `ends_with(".md")` test would silently skip it.
/// Skipping is the dangerous direction here — an unread note is an unguarded
/// note, and nothing reports it.
fn has_markdown_extension(path: &Utf8Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}

/// Reads one file, returning it only when it declares the marker.
///
/// The marker must be a line of its own, not a substring of prose. This
/// directory's own `README.md` documents the marker inside a code span, and a
/// substring test would read the README as a note and then reject it for lacking
/// a note register.
///
/// The read failure travels as an error rather than as `None`, because `None`
/// means "this file is not a `StateName` note" and a failed read is not that.
/// Returning `None` there would let a note that exists, declares the marker and
/// cannot be read be treated as a note that does not exist.
///
/// Crate-visible so this branch has a control. Forcing a read failure by
/// permissions would need a write from a module the `dylint.toml` exemption
/// does not cover, and would pass vacuously wherever the suite runs as root;
/// the control passes a directory instead, which is present by construction and
/// cannot be read as a file.
pub(crate) fn read_marked_note(path: &Utf8Path) -> Result<Option<CommittedNote>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("{path}: the note cannot be read: {error}"))?;
    if !text.lines().any(|line| line.trim() == MARKER) {
        return Ok(None);
    }
    let Some(file_name) = path.file_name().map(str::to_owned) else {
        return Err(format!("{path}: the note's file name cannot be read"));
    };
    Ok(Some(CommittedNote { file_name, text }))
}
