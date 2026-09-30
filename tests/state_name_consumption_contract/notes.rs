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

use super::path_exists;

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

/// The directory every scratch root sits under.
///
/// Extracted so that the boundary control can place its fault at the same base
/// `fresh_tree` builds from, rather than restating the path: a control that
/// restated it would silently stop testing the real root the first time the
/// construction changed.
fn scratch_base() -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("state-name-contract")
}

impl ScratchNotes {
    /// Creates a fresh scratch tree at `name`, deleting any existing tree at
    /// that scratch root.
    ///
    /// The name says what the call does, because the deletion is the part a
    /// reader has to know: this returns an *empty* root, never an existing one,
    /// and a caller that assumed otherwise would be reasoning about a directory
    /// it does not control.
    pub(crate) fn fresh_tree(name: &str) -> Result<Self, String> {
        let root = scratch_base().join(name);
        // Rebuilt rather than reused: a note left by an earlier run would be
        // scanned as though this run had written it, and the scenario would then
        // be asserting over a directory it does not control.
        //
        // `try_exists` rather than `exists`, for the reason `committed_notes`
        // gives one branch up. `exists()` answers "false" both for a root that
        // is genuinely absent and for one that cannot be inspected at all, and
        // reading the second as the first would skip the rebuild. The danger
        // there is not the error itself but its transience: a root that fails
        // to inspect once may be removable a moment later, so `create_dir_all`
        // would then succeed *over* the stale directory, leaving the previous
        // run's notes in place for a scan that believes it wrote them.
        if root
            .try_exists()
            .map_err(|error| format!("{root}: the scratch root cannot be inspected: {error}"))?
        {
            fs::remove_dir_all(&root).map_err(|error| {
                format!("{root}: the previous scratch root cannot be removed: {error}")
            })?;
        }
        let directory = root.join(NOTES_DIR);
        fs::create_dir_all(&directory).map_err(|error| {
            format!("{directory}: the scratch notes directory cannot be created: {error}")
        })?;
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

/// Refuses to reset a scratch root whose existing tree cannot be inspected.
///
/// The boundary control for `fresh_tree`'s `try_exists` inspection. It is not
/// covered by `unreadable_entries_are_an_error_not_a_skip`: that control drives
/// `read_marked_note`, a different function, whose failure it reaches by handing
/// it a directory and asking it to read one *note*. This one never calls the
/// scan at all — its subject is the reset `fresh_tree` performs on its own root,
/// which that control cannot reach however many notes it reads.
///
/// The fault is a *file at a parent component* of the inspected root, and the
/// placement is what makes the control discriminating. A file at the root
/// itself would not: `try_exists` answers `Ok(true)` for a path that *is* a
/// file, so the inspection succeeds, the removal proceeds and the control never
/// reaches the branch it exists to test. A file one level up makes the lookup
/// of the root itself fail with `ENOTDIR`, because resolving the path has to
/// traverse the file, and that is the only shape in which inspection *errors*.
///
/// It sits in this module rather than in `scan_scenarios.rs` because the fault
/// is this module's to place: `scan_scenarios` is the loader, and `fresh_tree`'s
/// pre-reset inspection is internal to this module. Every write this control
/// needs also needs the module's `dylint.toml` exemption from
/// `no_std_fs_operations`; moving the control would force a second exempted
/// path and narrow that lint's coverage across the scenario modules, which is
/// the one property the exemption's comment claims for it.
///
/// The two obligations then separate, and only the first is discriminating:
///
/// 1. **The failure names the inspected root.** The message must open with the root path, so an
///    engineer is sent to the component that a previous run left behind. This is what a regression
///    to `exists()` fails: `exists()` answers `Ok(false)` for the same path, so the removal is
///    skipped and `create_dir_all` fails instead, with a message opening on the *notes* directory
///    beneath the root. Both are `Err`, so an `is_err()` assertion could not tell them apart — the
///    prefix can, and does so for a reason the implementation states rather than one this comment
///    assumes.
/// 2. **The reset does not proceed.** The file survives, and no directory appears beneath the root.
///    This assertion holds under a correct implementation and under a reverted one, so it is stated
///    as the invariant the reset owes rather than counted as a second discriminator. The fault that
///    would give it teeth independently — a root whose inspection is denied by permissions — is out
///    of this module's reach, and its own doc comment says why: the write it would need is not
///    covered by the `dylint.toml` exemption, and it would pass vacuously wherever the suite runs
///    as root.
///
/// `fresh_tree` fails here for the reason the module documents: the failure is
/// transient-looking and would otherwise be read as "no tree exists", skipping
/// the reset.
#[test]
fn an_uninspectable_scratch_root_is_refused_before_the_reset() -> Result<(), String> {
    // The parent component the fault is planted at, and the root beneath it
    // that `fresh_tree` is asked to build. The name is a *path*, not a single
    // component, because the fault has to sit above the root rather than on it.
    let base = scratch_base();
    let blocked = base.join("uninspectable-parent");
    let root = blocked.join("uninspectable");
    let notes_dir = root.join(NOTES_DIR);

    // The precondition: the base exists, and nothing is left at the blocked
    // component. A parent left as a *file* by an earlier failed run is cleared
    // as a file, so a failure does not make this control un-runnable a second
    // time.
    fs::create_dir_all(&base)
        .map_err(|error| format!("{base}: the control cannot create its scratch base: {error}"))?;
    if blocked.is_dir() {
        fs::remove_dir_all(&blocked).map_err(|error| {
            format!("{blocked}: the control cannot clear its blocked component: {error}")
        })?;
    } else if path_exists(&blocked)? {
        fs::remove_file(&blocked).map_err(|error| {
            format!("{blocked}: the control cannot clear its leftover fault: {error}")
        })?;
    }

    // A *file* at the blocked component. This is the fault: resolving any path
    // beneath it fails with `ENOTDIR`, so the root cannot be inspected at all.
    fs::write(&blocked, "not a directory\n")
        .map_err(|error| format!("{blocked}: the control cannot plant its fault: {error}"))?;

    let outcome = ScratchNotes::fresh_tree("uninspectable-parent/uninspectable");

    // Both obligations are inspected before this control returns, and the fault
    // is cleared before it does however it does. Leaving the shared scratch base
    // as it was found is not a step conditional on success: the fault is a
    // *parent* component, so a failing run that skipped the cleanup would block
    // `create_dir_all` for every other scratch tree in this test binary, not
    // merely this one, and would do so precisely when something else has already
    // gone wrong.
    let failure = check_refusal_obligations(&outcome, &blocked, &root, &notes_dir).err();
    let cleanup = fs::remove_file(&blocked)
        .map_err(|error| format!("{blocked}: the control cannot clear its fault: {error}"));
    // The arm bindings are named apart from the values they destructure, because
    // `shadow_reuse` is denied repository-wide and rebinding these two would be
    // the very shadowing it denies.
    match (failure, cleanup) {
        (None, Ok(())) => Ok(()),
        (Some(obligation), Ok(())) => Err(obligation),
        (None, Err(tidying)) => Err(tidying),
        // Two things went wrong, and neither excuses the other: the obligation
        // was broken and the tidying failed. Both are reported.
        (Some(obligation), Err(tidying)) => Err(format!("{obligation} {tidying}")),
    }
}

/// The two obligations the uninspectable-root control asserts, extracted so the
/// control can clear its fault before it returns however it returns.
///
/// Obligation 1 — the failure names the inspected root — is the discriminating
/// one, because it is what a reverted implementation cannot produce. Obligation
/// 2 — the reset does not proceed — is the invariant the reset owes. The
/// caller's doc comment states what each is worth and why the second is the
/// weaker discriminator.
fn check_refusal_obligations(
    outcome: &Result<ScratchNotes, String>,
    blocked: &Utf8Path,
    root: &Utf8Path,
    notes_dir: &Utf8Path,
) -> Result<(), String> {
    let Err(message) = outcome else {
        return Err(format!(
            "{blocked} is a file, yet the reset proceeded. Repair: inspect the root with \
             `try_exists` and fail on an inspection error, rather than falling through to \
             `create_dir_all` over a root that was never read."
        ));
    };
    if !message.starts_with(&format!("{root}: ")) {
        return Err(format!(
            "the inspection failure was reported as {message:?}, which does not name {root}. \
             Repair: name the inspected root in the message — a message opening on {notes_dir} \
             means the failure came from `create_dir_all` and not from the inspection branch, so \
             the root was never inspected. An engineer must be sent to the component a previous \
             run left behind."
        ));
    }
    // Obligation 2: the reset did not proceed. The blocked component is still a
    // file, and no directory was created beneath it.
    if !blocked.is_file() {
        return Err(format!(
            "{blocked} is no longer a file, so the reset proceeded past an inspection error. \
             Repair: return before `remove_dir_all`; a root that cannot be inspected must not be \
             reset."
        ));
    }
    // The infallible query, and it has to be: `path_exists` would return `Err`,
    // because the fault that makes the root uninspectable makes everything
    // beneath it uninspectable too, so the `?` would report a probe error as a
    // failure of the subject. `is_dir` answers `false` for exactly this case —
    // no such directory, whether absent or unreadable — which is the question
    // obligation 2 asks.
    if notes_dir.is_dir() {
        return Err(format!(
            "{notes_dir} exists beneath a root that could not be inspected, so the reset \
             proceeded. Repair: return before `create_dir_all`."
        ));
    }
    Ok(())
}
