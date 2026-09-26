//! Contract tests for ADR 004, the `StateName` consumption evidence record.
//!
//! The record is the artefact under test. ADR 004 defines what a validation note
//! must contain and the rule that reads one note, or several, into an outcome;
//! this contract checks that the record still says what it said, that the blank
//! form instantiates it, and that every committed note under
//! `docs/validation-notes/` is usable evidence. It pronounces no verdict:
//! whether `&'static str` survives is roadmap task 3.2.1's decision, taken
//! against evidence this contract cannot foresee.
//!
//! Every negative control asserts an exact repair message. A control asserting
//! only `is_err()` would not discharge its obligation, because it could pass for
//! the wrong reason.
//!
//! This file holds the shared constants and helpers; the scenarios themselves
//! live in `anchor_scenarios.rs` (template and roadmap bindings),
//! `note_scenarios.rs` (note cells and the verdict they yield),
//! `register_scenarios.rs` (register parsing and consistency) and
//! `scan_scenarios.rs` (the directory scan over committed notes). Each module
//! owns one invariant class, and each is a child rather than a share of this
//! file so that the 400-line cap binds every part of the contract alike. The
//! non-scenario modules divide the same way: `claims.rs` reads what a cell
//! says, `policy.rs` decides what that obliges, and `registers.rs`,
//! `clauses.rs` and `roadmap.rs` own one bound document each.

#[path = "state_name_consumption_contract/anchor_scenarios.rs"]
mod anchor_scenarios;
#[path = "state_name_consumption_contract/claims.rs"]
mod claims;
#[path = "state_name_consumption_contract/clauses.rs"]
mod clauses;
#[path = "state_name_consumption_contract/fixtures.rs"]
mod fixtures;
#[path = "state_name_consumption_contract/note_scenarios.rs"]
mod note_scenarios;
#[path = "state_name_consumption_contract/notes.rs"]
mod notes;
#[path = "state_name_consumption_contract/parse.rs"]
mod parse;
#[path = "state_name_consumption_contract/policy.rs"]
mod policy;
#[path = "state_name_consumption_contract/register_scenarios.rs"]
mod register_scenarios;
#[path = "state_name_consumption_contract/registers.rs"]
mod registers;
#[path = "state_name_consumption_contract/roadmap.rs"]
mod roadmap;
#[path = "state_name_consumption_contract/scan_scenarios.rs"]
mod scan_scenarios;
#[path = "state_name_consumption_contract/types.rs"]
mod types;

use camino::Utf8PathBuf;
use parse::status_rows_or_error;
use types::StatusRow;

const ADR: &str = include_str!("../docs/adr-004-state-name-consumption-evidence.md");
const DESIGN: &str = include_str!("../docs/design.md");
const ROADMAP: &str = include_str!("../docs/roadmap.md");
const ADR_002: &str = include_str!("../docs/adr-002-transition-boundary-scope.md");

/// The clause `docs/design.md` §6.1 must carry, as ADR 004 quotes it.
const STRONGER: &str =
    "The default remains `&'static str` until a real example consumes something stronger";

/// The failure an ADR whose evidence section quotes nothing must produce.
const EMPTY_CLAUSE_LIST: &str = "docs/adr-004-state-name-consumption-evidence.md cites no \
                                 clauses. Repair: quote each load-bearing source clause as an \
                                 italic line in the evidence section.";

/// Parses the live status register, from which most scenarios read vocabulary.
fn live_status() -> Result<Vec<StatusRow>, String> { status_rows_or_error(ADR) }

/// Folds a string's whitespace runs to single spaces.
///
/// Every check that resolves a phrase written by hand needs this, and all of
/// them need the *same* folding: a phrase that resolves folded in one check but
/// unfolded in another reports a line break as evidence of drift. `mdtablefix
/// --wrap` rewraps each document at its own column, so a phrase quoted from a
/// source and the same phrase looked for in one arrive with different breaks.
/// One definition, here, is what keeps the several folders from disagreeing.
fn fold_whitespace(text: &str) -> String { text.split_whitespace().collect::<Vec<_>>().join(" ") }

/// The workspace root, derived from the manifest directory so the scan works
/// regardless of the runner's working directory.
fn workspace_root() -> Utf8PathBuf { Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Applies a source substitution, refusing to apply one that matches nothing.
///
/// A control that mutates a *live* document is only as good as its needle, and
/// `mdtablefix --wrap` moves a phrase's line break whenever it reflows the
/// paragraph around it. A `replace` whose needle no longer matches is a silent
/// no-op: the control then asserts a failure the check no longer produces, and
/// passes for the wrong reason or fails for a confusing one. So the needle is
/// held to a token that wrapping cannot split.
fn mutated(source: &str, needle: &str, replacement: &str) -> String {
    assert!(
        source.contains(needle),
        "this control's needle {needle:?} is no longer present in the document it mutates. The \
         document has been rewrapped and the control has stopped applying; narrow the needle to a \
         single word, which no reflow can split."
    );
    source.replace(needle, replacement)
}
