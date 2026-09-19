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

use camino::{Utf8Path, Utf8PathBuf};

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
pub(crate) fn committed_notes(root: &Utf8Path) -> Vec<CommittedNote> {
    let directory = root.join(NOTES_DIR);
    let Ok(entries) = directory.read_dir_utf8() else {
        return Vec::new();
    };
    let mut found = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().ends_with(".md"))
        .filter_map(|entry| read_marked_note(entry.into_path()))
        .collect::<Vec<CommittedNote>>();
    found.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    found
}

/// Reads one file, returning it only when it declares the marker.
///
/// The marker must be a line of its own, not a substring of prose. This
/// directory's own `README.md` documents the marker inside a code span, and a
/// substring test would read the README as a note and then reject it for lacking
/// a note register.
fn read_marked_note(path: Utf8PathBuf) -> Option<CommittedNote> {
    let text = fs::read_to_string(&path).ok()?;
    if !text.lines().any(|line| line.trim() == MARKER) {
        return None;
    }
    Some(CommittedNote {
        file_name: path.file_name()?.to_owned(),
        text,
    })
}
