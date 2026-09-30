//! Anchor scenarios: the template and the roadmap bindings.
//!
//! Each of these checks a *link* between two documents: the blank form against
//! the register it instantiates, and the gate table against the live roadmap. A
//! drift on either side of either pair fails here. The clauses ADR 004 quotes
//! are checked in `clause_scenarios.rs`, and the success criterion's own
//! controls, which ask *which* copy of a sentence is the criterion rather than
//! where a fragment resolves, live in `criterion_scenarios.rs`.

use pretty_assertions::assert_eq;
use rstest::rstest;

use super::{
    ADR,
    DESIGN,
    ROADMAP,
    STRONGER,
    fixtures::{TEMPLATE, gate_table},
    live_status,
    mutated,
    parse::{gate_rows, note_rows},
    registers::check_deferred_clause,
    roadmap::{check_gate_titles, task_records},
    types::field_order,
};

/// Checks the blank form field-by-field against the register.
///
/// The form is read from `TEMPLATE`, the `include_str!` of
/// `docs/phase-2-validation-note-template.md`, so a check can fail only when
/// the live document and the register disagree.
#[test]
fn template_matches_the_status_register() -> Result<(), String> {
    let rows = live_status()?;
    let template = note_rows(TEMPLATE).map_err(|error| error.to_string())?;
    assert_eq!(
        template
            .iter()
            .map(|row| row.field.clone())
            .collect::<Vec<String>>(),
        field_order(&rows)
    );
    for row in &template {
        assert_eq!(row.status, "TBD", "field {} must ship blank", row.field);
        assert_eq!(row.evidence, "TBD", "field {} must ship blank", row.field);
    }
    Ok(())
}

/// The number of roadmap task titles naming a fragment, counted as the check
/// counts.
///
/// The span is the *title*, not the record, because that is the span the check
/// reads: a gate binds by task title. Counting records would disagree with the
/// check for exactly the fragment this helper exists to describe — "baseline"
/// names three titles and six whole records — and a helper that disagrees with
/// the check it predicts is worse than a literal, because the assertion would
/// pass while pinning the wrong number.
fn roadmap_fragment_matches(fragment: &str) -> usize {
    task_records(ROADMAP)
        .iter()
        .filter(|record| record.title.contains(fragment))
        .count()
}

/// The ambiguity failure, with its count taken from the live roadmap.
///
/// A literal count would freeze the roadmap. Binding gates by fragment exists
/// so that completing a bound task or renumbering the roadmap does not break
/// the build; a literal "6" reintroduces exactly that breakage one line later,
/// and fails for a reason a reader would have to diff two documents to see.
///
/// Deriving it costs something, and the cost is worth naming: because this
/// helper counts the way the check counts, the *number* in the message can no
/// longer disagree with `check_gate_titles`, so that one digit is no longer
/// independently pinned. What remains pinned is everything the control is for —
/// that the fragment still matches more than one record (asserted separately by
/// `the_ambiguity_fragment_still_matches_many_records`, so this control cannot
/// quietly decay into a single match and pass), that the check rejects rather
/// than accepts, and that it takes the ambiguous branch rather than the
/// "no task" one. The counting *method* is pinned as far as it can be: a check
/// counting raw document lines would now report a *different* number here, so
/// this assertion still fails if the check regresses to line counting.
fn ambiguous_fragment_message(gate: &str, fragment: &str) -> String {
    let matches = roadmap_fragment_matches(fragment);
    format!(
        "docs/roadmap.md: gate {gate} names task fragment {fragment:?}, which matches {matches} \
         tasks. Repair: use a fragment specific to one task."
    )
}

/// Holds the ambiguity control's precondition: its fragment matches many
/// records.
///
/// The control asserts an exact message, and that message is derived from the
/// live roadmap, so it cannot fail merely because the number moved. What it
/// *must* still fail on is the precondition that there is an ambiguity to
/// demonstrate at all. Without this, a roadmap edit leaving one "baseline"
/// record would turn `#[case::ambiguous]` into a test of nothing: the check
/// would accept the fragment, the case would fail for an unrelated reason, and
/// the only thing the suite would be saying is that the fixture went stale.
#[test]
fn the_ambiguity_fragment_still_matches_many_records() {
    let matches = roadmap_fragment_matches("baseline");
    assert!(
        matches > 1,
        "the ambiguity control's fragment matches {matches} roadmap task record(s), so its case \
         proves nothing about ambiguity. Repair: choose a fragment the live roadmap still repeats."
    );
}

/// The "no task" failure for a fragment no task title carries.
///
/// A fragment is only a task binding when it names a *task*. Each of the
/// deceiving fragments below is present in `docs/roadmap.md` and would satisfy a
/// scan of the document's lines: `kill gates` appears in a phase heading,
/// `adr-004-state-name-consumption-evidence.md` inside a task's link, and
/// `Requires 1.1.2` inside one of its sub-bullets. None is a task title, so
/// each must be reported as matching no task.
fn unresolved_fragment_message(gate: &str, fragment: &str) -> String {
    format!(
        "docs/roadmap.md: gate {gate} names task fragment {fragment:?}, which matches no task. \
         Repair: restore that task's title, or revise ADR 004's gate table."
    )
}

/// Resolves each gate's title fragment to exactly one live roadmap task, and
/// rejects a fragment matching none and one matching many.
#[rstest]
#[case::records_process_buffer("S1", "Annotate `mdtablefix` `ProcessBuffer`", None)]
#[case::records_continuation("S2", "Annotate `mdtablefix` continuation", None)]
#[case::records_baseline("S3", "Apply the conventions-only baseline", None)]
#[case::decides("S4", "Finalize the `StateName` return shape", None)]
#[case::unresolved(
    "S4",
    "Do the thing",
    Some(unresolved_fragment_message("S4", "Do the thing"))
)]
#[case::prose_is_not_a_task(
    "S4",
    "kill gates",
    Some(unresolved_fragment_message("S4", "kill gates"))
)]
#[case::a_link_is_not_a_task(
    "S4",
    "adr-004-state-name-consumption-evidence.md",
    Some(unresolved_fragment_message("S4", "adr-004-state-name-consumption-evidence.md"))
)]
#[case::a_sub_bullet_is_not_a_task(
    "S4",
    "Requires 1.1.2",
    Some(unresolved_fragment_message("S4", "Requires 1.1.2"))
)]
#[case::ambiguous("S3", "baseline", Some(ambiguous_fragment_message("S3", "baseline")))]
fn gate_titles_resolve(
    #[case] gate: &str,
    #[case] fragment: &str,
    #[case] failure: Option<String>,
) -> Result<(), String> {
    let mutated = gate_table().replace(&gate_table_fragment(gate)?, fragment);
    let row = gate_rows(&mutated)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|row| row.gate == gate)
        .expect("the fixture gate table must name every gate");
    assert_eq!(row.fragment, fragment);
    match failure {
        None => assert_eq!(check_gate_titles(&mutated, ROADMAP), Ok(())),
        Some(expected) => assert_eq!(check_gate_titles(&mutated, ROADMAP), Err(expected)),
    }
    Ok(())
}

/// Resolves the live gate table against the live roadmap.
///
/// Every `gate_titles_resolve` case runs over the *fixture* table, which is what
/// makes each rejection control possible: a case must plant a fragment in a table
/// it owns. That leaves the live pairing unread. The fixture and the document
/// are compared by `INV-REGISTERS`-style parsing in `register_scenarios.rs` for
/// the status and aggregation registers, but the gate table has no equivalent
/// there, so without this scenario the ADR's own table could be deleted, or a
/// fragment edited, and every gate case would still pass over the fixture.
///
/// The two checks are complementary and neither subsumes the other: this one
/// binds the live document to the live roadmap, and `gate_titles_resolve` binds
/// each rejection branch to the message it must produce.
#[test]
fn live_gate_table_binds_the_live_roadmap() -> Result<(), String> {
    // The table must be non-empty before the binding means anything: an ADR
    // whose gate block had been emptied parses to no rows, and a check over no
    // rows would pass while binding nothing at all. The refusal travels through
    // the error channel rather than `assert!`, because `panic_in_result_fn` is
    // denied — a control that returns `Result` reports rather than crashes.
    let gates = gate_rows(ADR).map_err(|error| error.to_string())?;
    if gates.is_empty() {
        return Err(
            "ADR 004's gate table parsed to no rows, so this scenario would pass while binding \
             nothing. Repair: restore the gate-table block, or revise this contract with it."
                .to_owned(),
        );
    }
    check_gate_titles(ADR, ROADMAP)
}

/// The fragment the fixture gate table ships for one gate, so that a case can
/// replace it without the table being written out twice.
///
/// Fallible rather than `expect`ing, for the reason given on `blocked_by`. An
/// absent gate is not a fixture defect either: `gate_titles_resolve` asserts the
/// fragment it receives after calling this, so returning an empty string for an
/// unnamed gate keeps the case's own assertion as the place the failure lands.
fn gate_table_fragment(gate: &str) -> Result<String, String> {
    Ok(gate_rows(&gate_table())
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|row| row.gate == gate)
        .map_or_else(String::new, |row| row.fragment))
}

/// Holds the gate controls' preconditions against the live roadmap.
///
/// Those controls assert fragments that appear in the roadmap *outside* every
/// task title, which is a property of the roadmap rather than of the contract.
/// A roadmap edit can therefore retire a control silently: the fragment stops
/// appearing, `check_gate_titles` still rejects it for the same reason, the
/// case still passes, and the suite stops demonstrating anything about prose.
/// This test fails instead, naming what to re-choose.
#[test]
fn the_deceiving_fragments_still_appear_outside_task_titles() {
    for fragment in [
        "kill gates",
        "adr-004-state-name-consumption-evidence.md",
        "Requires 1.1.2",
    ] {
        let in_document = ROADMAP.lines().any(|line| line.contains(fragment));
        let in_a_title = task_records(ROADMAP)
            .iter()
            .any(|record| record.title.contains(fragment));
        assert!(
            in_document && !in_a_title,
            "{fragment:?} no longer appears in the roadmap outside every task title (present in \
             the document: {in_document}; present in a title: {in_a_title}), so its control no \
             longer demonstrates that prose is not a task. Repair: choose a fragment the live \
             roadmap still carries outside its task titles."
        );
    }
}

/// Checks that `docs/design.md` still defers the default to evidence.
#[test]
fn deferred_clause_still_resolves() -> Result<(), String> {
    check_deferred_clause(DESIGN)?;
    let reworded = mutated(DESIGN, "stronger", "a stronger type");
    assert_eq!(
        check_deferred_clause(&reworded),
        Err(format!(
            "docs/design.md §6.1 no longer contains {STRONGER:?}. Repair: restore the default's \
             deferral, or revise ADR 004 and its contract together."
        ))
    );
    Ok(())
}
