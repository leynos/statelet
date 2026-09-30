//! The acceptance-criterion scenarios: the sentence roadmap task 1.1.3 is
//! graded on, and the three ways a check can be pointed at the wrong one.
//!
//! `INV-CRITERION`'s controls live here rather than in `anchor_scenarios.rs`
//! because they ask a question of their own — *which* copy of the sentence is
//! the criterion — and because the two modules together must stay under
//! AGENTS.md's 400-line cap. The gate-binding scenarios, which ask where in the
//! roadmap a fragment resolves, stay in `anchor_scenarios.rs`; these ask
//! whether a clause found anywhere at all is the clause the task is graded on.

use pretty_assertions::assert_eq;

use super::{
    ROADMAP,
    fixtures::status_register_without,
    fold_whitespace,
    live_status,
    mutated,
    parse::status_rows,
    roadmap::{CRITERION_TASK_FRAGMENT, check_success_criterion, task_records},
};

/// The clause the three controls below look for, folded.
///
/// Held once rather than repeated in each control, because the three differ
/// only in *where* they plant or remove it. A control that folded its own copy
/// would be free to fold it differently from the check, and a phrase that
/// resolved folded in one and unfolded in another reports a line break as
/// evidence of drift.
fn criterion_clause() -> String {
    fold_whitespace(
        "the Phase 2 validation note template has fields for state display name, optional \
         identifier need, metrics cardinality, and tracing use",
    )
}

/// The failure a roadmap missing the criterion must produce.
fn missing_criterion() -> String {
    "docs/roadmap.md no longer contains the 1.1.3 success criterion. Repair: restore \"the Phase 2 \
     validation note template has fields for state display name, optional identifier need, metrics \
     cardinality, and tracing use\", or revise this contract together with the task."
        .to_owned()
}

/// Checks the acceptance criterion the task is graded on still resolves.
#[test]
fn success_criterion_still_maps() -> Result<(), String> {
    check_success_criterion(&live_status()?, ROADMAP)?;
    let missing = status_register_without("tracing-use");
    let rows = status_rows(&missing).expect("the mutated register still parses");
    assert_eq!(
        check_success_criterion(&rows, ROADMAP),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: no status-register field maps the \
             criterion noun \"tracing use\". Repair: add the tracing-use field, or reword the \
             roadmap task."
                .to_owned()
        )
    );
    let reworded = mutated(ROADMAP, "and tracing use", "and tracing coverage");
    assert_eq!(
        check_success_criterion(&live_status()?, &reworded),
        Err(missing_criterion())
    );
    Ok(())
}

/// Rejects the criterion when the task's own copy of it is gone.
///
/// The companion control to `criterion_outside_a_task_is_not_the_criterion`,
/// and the other half of the same distinction. That control asks whether the
/// check knows a document region; this one asks whether it knows a *task*. The
/// clause is removed from the graded task by a needle unique to that task's
/// bullet, so the only question is whether the check follows the clause to
/// another task's record or holds it to the one it grades.
///
/// The plant is non-vacuity: the sentence really does appear elsewhere in the
/// document when the check runs, and it appears inside a task record rather than
/// in prose, so a check reading any record would accept it. Without the plant
/// this is just the reworded-clause control again, and proves nothing about
/// attribution.
#[test]
fn criterion_in_another_task_is_not_the_criterion() -> Result<(), String> {
    // The graded task's own bullet loses the clause: its "Success:" line still
    // opens as before, and the trailing citation is reworded so the clause's
    // words can no longer be reconstructed from where they were. Both needles
    // are tokens no reflow can split.
    let reworded = mutated(
        ROADMAP,
        "fields for state display",
        "fields for every recorded aspect",
    );
    // Another task's record gains an identical sentence, as its own success
    // bullet. The needle is the sub-bullet 2.2.1 closes with, which no other
    // task shares.
    let planted = mutated(
        &reworded,
        "- See design.md §12.",
        "- Success: the Phase 2 validation note template has fields for state display name, \
         optional identifier need, metrics cardinality, and tracing use.\n  - See design.md §12.",
    );
    let clause = criterion_clause();
    let in_other_tasks = task_records(&planted)
        .iter()
        .filter(|record| !record.title.contains(CRITERION_TASK_FRAGMENT))
        .filter(|record| record.text.contains(&clause))
        .count();
    assert_eq!(
        in_other_tasks, 1,
        "this control needs exactly one copy of the criterion inside a task record other than the \
         graded one, and found {in_other_tasks}. Repair: check that the plant still lands in a \
         record and that the graded task's own copy is still broken; a control with no surviving \
         copy proves nothing about attribution."
    );
    assert_eq!(
        check_success_criterion(&live_status()?, &planted),
        Err(missing_criterion())
    );
    Ok(())
}

/// Rejects the criterion when its sentence appears only *outside* a task.
///
/// The clause is a task's success bullet. The same sentence in the roadmap's
/// framing prose is not the criterion, and a check that scanned the document's
/// lines would accept it — reporting the task as still graded on a sentence
/// the task had lost. The control plants an identical sentence in the page's
/// introduction, then breaks the task's own copy with a needle unique to its
/// bullet, so exactly one copy of the clause survives and the only question is
/// whether the check knows which document region it is in.
///
/// Both needles are phrases no reflow can split. The clause itself is never a
/// needle: `mdtablefix --wrap` breaks it across lines, and the replacements are
/// applied to the raw text, so a needle spanning one of those breaks would stop
/// applying the next time the document was formatted.
#[test]
fn criterion_outside_a_task_is_not_the_criterion() -> Result<(), String> {
    let planted = mutated(
        ROADMAP,
        "# Statelet roadmap\n",
        "# Statelet roadmap\n\nRecall that the Phase 2 validation note template has fields for \
         state display name, optional identifier need, metrics cardinality, and tracing use.\n",
    );
    let broken_in_the_task = mutated(&planted, "Success: the Phase 2", "Success: the note form");
    let clause = criterion_clause();
    let copies = fold_whitespace(&broken_in_the_task)
        .matches(&clause)
        .count();
    assert_eq!(
        copies, 1,
        "this control needs exactly one copy of the criterion in the document — the one planted \
         in the introduction — and found {copies}. Repair: check that the planted sentence still \
         matches the clause the check looks for, and that the task's own copy is still broken by \
         the reword; a control with no surviving copy proves nothing about where the clause lives."
    );
    assert_eq!(
        check_success_criterion(&live_status()?, &broken_in_the_task),
        Err(missing_criterion())
    );
    Ok(())
}
