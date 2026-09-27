//! Directory scan over `docs/validation-notes/`, and the scratch roots its
//! controls scan instead.
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
//!
//! It also *writes*, for one purpose: a scenario that scans a populated
//! directory needs a populated directory, and `docs/validation-notes/` holds no
//! note until roadmap task 2.2.1 has annotated something. Writing the scratch
//! trees here rather than in the scenario module keeps the exemption to a single
//! module, which is the property `dylint.toml` claims; the alternative — a
//! second exempted path — would narrow the coverage the exemption withdraws
//! from nothing while making that claim false.

use std::fs;

use camino::{Utf8Path, Utf8PathBuf};

/// The directory holding committed validation notes.
const NOTES_DIR: &str = "docs/validation-notes";

/// The marker a note declares to opt into the `StateName` contract.
pub(crate) const MARKER: &str = "<!-- state-name-note -->";

/// Whether a note's text *declares* the marker, as opposed to mentioning it.
///
/// The marker must be a line of its own, not a substring of prose. This
/// directory's own `README.md` documents the marker inside a code span, and a
/// substring test would read the README as a note and then reject it for
/// lacking a note register.
///
/// A pure function of the text, so the rule can be tested against all three
/// shapes it must tell apart — a declared marker, a note that declares some
/// other contract's marker, and a marker merely mentioned — without a file
/// for each. `read_marked_note` is the caller, and the scan scenarios supply
/// the end-to-end evidence.
pub(crate) fn declares_marker(text: &str) -> bool { text.lines().any(|line| line.trim() == MARKER) }

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
    // `try_exists` rather than `exists`, because the two differ exactly where
    // this function must not be silent. `exists()` answers "false" both for a
    // path that is genuinely absent and for one that cannot be inspected at
    // all — a component of the path being a regular file yields `ENOTDIR`, and
    // a permission denial yields `EACCES`. Reading the second as an absent
    // directory would return "no notes" for a checkout whose notes were merely
    // unreadable, which is the silent skip this function's own doc comment
    // exists to refuse, arrived at one step earlier.
    if !directory
        .try_exists()
        .map_err(|error| format!("{directory}: the notes directory cannot be inspected: {error}"))?
    {
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
/// The marker rule itself lives in `declares_marker`, so that it can be tested
/// directly against the three shapes it separates; this function is the one
/// caller, and adds the read.
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
    if !declares_marker(&text) {
        return Ok(None);
    }
    let Some(file_name) = path.file_name().map(str::to_owned) else {
        return Err(format!("{path}: the note's file name cannot be read"));
    };
    Ok(Some(CommittedNote { file_name, text }))
}

/// A populated notes directory on disk, and the root `committed_notes` reads
/// through it.
///
/// The scenario that needs this cannot build its subject any other way: the
/// live `docs/validation-notes/` holds no note until roadmap task 2.2.1 has
/// annotated something, so the scan's end-to-end behaviour over *notes* is
/// unreachable from the repository as it stands. Driving `check_note_cells` and
/// `resolve_note` directly, as the other scenarios do, proves what those
/// functions answer; it cannot prove the scan reaches them, which is what the
/// two review findings asked to see.
///
/// The root is `<workspace>/target/...`, which `.gitignore` already excludes and
/// Cargo already treats as its own build directory, so nothing in the tree is
/// touched. The directory is rebuilt on every run rather than reused, so a
/// scenario never reads a previous run's leftovers.
///
/// Every filesystem failure travels as an error. A scratch tree that could not
/// be created is not a finding about the scan, and a control passing because its
/// subject was never written would be exactly the vacuity this suite's controls
/// are written to avoid.
pub(crate) struct ScratchNotes {
    root: Utf8PathBuf,
}

impl ScratchNotes {
    /// Creates an empty root with `docs/validation-notes/` beneath it.
    pub(crate) fn new(name: &str) -> Result<Self, String> {
        let root = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("state-name-contract")
            .join(name);
        // Rebuilt rather than reused: a note left by an earlier run would be
        // scanned as though this run had written it, and the scenario would then
        // be asserting over a directory it does not control.
        if root.exists() {
            fs::remove_dir_all(&root)
                .map_err(|error| format!("{root}: the previous scratch root cannot be removed: {error}"))?;
        }
        let directory = root.join(NOTES_DIR);
        fs::create_dir_all(&directory)
            .map_err(|error| format!("{directory}: the scratch notes directory cannot be created: {error}"))?;
        Ok(Self { root })
    }

    /// The root to hand `committed_notes`.
    pub(crate) fn root(&self) -> &Utf8Path { self.root.as_path() }

    /// Writes one file into the scratch notes directory.
    ///
    /// The body is written verbatim, so a caller that plants a defect writes the
    /// defect; and a caller that plants nothing writes a *valid* note, which is
    /// what keeps a rejection case from passing because its subject was
    /// malformed in some other way.
    pub(crate) fn write(&self, file_name: &str, text: &str) -> Result<(), String> {
        let path = self.root.join(NOTES_DIR).join(file_name);
        fs::write(&path, text)
            .map_err(|error| format!("{path}: the scratch note cannot be written: {error}"))
    }
}
