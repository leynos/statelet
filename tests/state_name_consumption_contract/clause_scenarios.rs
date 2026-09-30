//! Clause scenarios: whether ADR 004's quotations still resolve, and against
//! *what* each one resolves.
//!
//! `anchor_scenarios.rs` asks where a fragment resolves; these ask whether the
//! clause resolves at all, and whether it is bound to the record it was quoted
//! from rather than to the section that happens to hold that record. Both
//! belong to `clauses.rs`, and the split out of `anchor_scenarios.rs` is the
//! one the 400-line cap forces rather than a difference of subject.

use pretty_assertions::assert_eq;

use super::{
    ADR,
    ADR_002,
    DESIGN,
    EMPTY_CLAUSE_LIST,
    ROADMAP,
    STRONGER,
    clauses,
    mutated,
    types::Register,
};

/// Resolves the three quoted clauses, and rejects a rewritten clause, a
/// relocated clause, a fabricated one, a mis-attributed one, and an emptied
/// evidence section.
#[test]
fn quoted_passages_still_resolve() -> Result<(), String> {
    clauses::check_quoted_clauses(ADR, DESIGN, ROADMAP, ADR_002)?;
    // The check reports the file a reader must open, not the attribution word
    // the ADR uses for it.
    let drifted = |path: &str, quoted: &str| {
        Err(format!(
            "{path} no longer contains the quoted clause {quoted:?} under \"6.1 State naming\". \
             Repair: update ADR 004 and its contract together."
        ))
    };
    // The source no longer carries the clause the ADR quotes. The reported
    // text is the ADR's quotation, because that is what failed to resolve.
    let rewritten = mutated(DESIGN, "stronger", "a stronger type");
    assert_eq!(
        clauses::check_quoted_clauses(ADR, &rewritten, ROADMAP, ADR_002),
        drifted("docs/design.md", STRONGER)
    );
    // The clause is still in `docs/design.md`, word for word, but a heading now
    // ends §6.1 before it. A resolver that searched the whole document rather
    // than the named section would accept it.
    let relocated = mutated(
        DESIGN,
        "The `mdtablefix` baseline",
        "### 6.1.1 Superseded\n\nThe `mdtablefix` baseline",
    );
    assert_eq!(
        clauses::check_quoted_clauses(ADR, &relocated, ROADMAP, ADR_002),
        drifted("docs/design.md", STRONGER)
    );
    // The ADR quotes a clause its source never carried. The needle is a single
    // word: `mdtablefix --wrap` breaks the quoted clause between "something"
    // and "stronger", so any longer needle would be split by the formatter.
    let fabricated = mutated(ADR, "stronger", "weaker");
    assert_eq!(
        clauses::check_quoted_clauses(&fabricated, DESIGN, ROADMAP, ADR_002),
        drifted(
            "docs/design.md",
            "The default remains `&'static str` until a real example consumes something weaker"
        )
    );
    // The clause is real and unmoved, but the ADR now credits it to a document
    // this contract does not read.
    let misattributed = mutated(ADR, "— design", "— context");
    assert_eq!(
        clauses::check_quoted_clauses(&misattributed, DESIGN, ROADMAP, ADR_002),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md attributes a clause to \"context\", \
             which this contract does not read. Repair: use design, roadmap or adr-002."
                .to_owned()
        )
    );
    // The evidence section still holds its delimiters but no clause at all.
    let emptied = empty_evidence_block(ADR);
    assert_eq!(
        clauses::check_quoted_clauses(&emptied, DESIGN, ROADMAP, ADR_002),
        Err(EMPTY_CLAUSE_LIST.to_owned())
    );
    Ok(())
}

/// Rejects a clause the ADR credits to a task that does not carry it.
///
/// The roadmap quotation resolves against task 3.2.1's own record, so moving it
/// into 3.2.2 — where it still sits under the same `### 3.2` section and inside
/// the same `## 3` part — must be refused. A resolver reading the section rather
/// than the record accepts it, because every task below 3.2.1 is still within
/// that section's body: the ADR would be free to credit the sentence to 3.2.2
/// while quoting it from 3.2.1, which is the binding the contract keeps.
#[test]
fn a_clause_moved_to_another_task_is_rejected() -> Result<(), String> {
    let moved = quotation_moved_to_the_next_task(ROADMAP);
    // The refusal travels through the error channel rather than `assert!`,
    // because `panic_in_result_fn` is denied — a control that returns `Result`
    // reports rather than crashes.
    let still_carries_it = clauses::task_record(&moved, TASK_3_2_1)
        .is_some_and(|record| record.text.contains("observed example consumption"));
    if still_carries_it {
        return Err(
            "the relocation control's move did not apply, so its case would prove nothing about \
             which task a clause is bound to"
                .to_owned(),
        );
    }
    assert_eq!(
        clauses::check_quoted_clauses(ADR, DESIGN, &moved, ADR_002),
        Err(
            "docs/roadmap.md no longer contains the quoted clause \"backed by observed example \
             consumption, not anticipation\" under \"3.2.1. Finalize the StateName return \
             shape\". Repair: update ADR 004 and its contract together."
                .to_owned()
        ),
        "the clause is no longer in task 3.2.1's own record, so the check must refuse it rather \
         than read the section the record happens to sit in"
    );
    Ok(())
}

/// The section name ADR 004 attributes its roadmap quotation to.
const TASK_3_2_1: &str = "3.2.1. Finalize the StateName return shape";

/// Relocates ADR 004's roadmap quotation from task 3.2.1 into task 3.2.2.
///
/// The clause occupies two lines once `mdtablefix --wrap` has reflowed it, and
/// `mutated` refuses any needle a reflow can split, so the move is made by line
/// rather than by substring. Both records stay inside the same `### 3.2`
/// section, which is the whole point: the clause remains resolvable *by
/// section*, and only a check bound to the record refuses it.
///
/// A roadmap whose shape has changed is returned untouched, and the control's
/// own precondition then fails with a message naming what it needed. Falling
/// back to a no-op is what `empty_evidence_block` does for the same reason: a
/// helper that panicked would report a fixture change as a predicate defect.
fn quotation_moved_to_the_next_task(roadmap: &str) -> String {
    let mut lines = roadmap.lines().map(str::to_owned).collect::<Vec<String>>();
    let (Some(from), Some(to)) = (task_opens(&lines, "3.2.1"), task_opens(&lines, "3.2.2")) else {
        return roadmap.to_owned();
    };
    let sub_bullet = "- Success: the public trait shape is backed by observed example";
    let Some(start) = (from..to).find(|index| {
        lines
            .get(*index)
            .is_some_and(|line| line.trim() == sub_bullet)
    }) else {
        return roadmap.to_owned();
    };
    let end = (start + 1..to)
        .find(|index| {
            lines
                .get(*index)
                .is_some_and(|line| line.trim_start().starts_with("- "))
        })
        .unwrap_or(to);
    let relocated = lines.drain(start..end).collect::<Vec<String>>();
    // The drain shifted every index after it, so the next task is looked up
    // again; reusing the stale `to` would splice into the wrong place.
    let Some(next_task) = task_opens(&lines, "3.2.2") else {
        return roadmap.to_owned();
    };
    let insert_at = (next_task + 1..lines.len())
        .find(|index| {
            lines
                .get(*index)
                .is_some_and(|line| line.starts_with("- ["))
        })
        .unwrap_or(lines.len());
    lines.splice(insert_at..insert_at, relocated);
    lines.join("\n")
}

/// The index of the line opening the record numbered `number`, if it is open.
fn task_opens(lines: &[String], number: &str) -> Option<usize> {
    let prefix = format!("- [ ] {number}. ");
    lines.iter().position(|line| line.starts_with(&prefix))
}

/// Empties ADR 004's evidence block while leaving both delimiters in place.
///
/// Replacing the quoted text would leave the italic markers and so leave an
/// (empty-bodied) clause behind; the empty-list failure needs a block that
/// genuinely holds none.
fn empty_evidence_block(adr: &str) -> String {
    let begin = Register::Evidence.begin();
    let end = Register::Evidence.end();
    let Some((head, rest)) = adr.split_once(begin) else {
        return adr.to_owned();
    };
    let Some((_, tail)) = rest.split_once(end) else {
        return adr.to_owned();
    };
    format!("{head}{begin}\n\n{end}{tail}")
}
