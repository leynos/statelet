//! Checks that bind ADR 004's registers to the roadmap and to each other.
//!
//! The status register records vocabulary; this module checks that the
//! aggregation register speaks it, that the gate table resolves to live tasks,
//! and that the acceptance criterion the task is graded on still maps.

use super::{
    parse::gate_rows,
    policy::{IDENTIFIER_NEED, INSUFFICIENT, NOTHING},
    types::{AggRow, StatusRow},
};

/// The four roadmap nouns of task 1.1.3's success criterion, and the register
/// field each must map to.
const CRITERION_NOUNS: [(&str, &str); 4] = [
    ("state display name", "state-display-name"),
    ("optional identifier need", IDENTIFIER_NEED),
    ("metrics cardinality", "metrics-cardinality"),
    ("tracing use", "tracing-use"),
];

/// The clause `docs/roadmap.md` must still carry for the criterion to resolve.
const SUCCESS_CRITERION_CLAUSE: &str = "the Phase 2 validation note template has fields for state \
                                        display name, optional identifier need, metrics \
                                        cardinality, and tracing use";

/// The clause the deferred-decisions item in `docs/design.md` must carry.
const DEFERRED_CLAUSE: &str =
    "The default remains `&'static str` until a real example consumes something stronger";

/// The three reachable states of the note multiset, and what each must yield.
///
/// Every outcome is conditional on publication proceeding, so the expected text
/// is a prefix rather than the whole cell.
const AGGREGATION_STATES: [(&str, &str, &str); 3] = [
    ("None", "n/a", "Blocked"),
    ("One or more", "No", "Ratify"),
    ("One or more", "Yes", "Amend"),
];

/// Checks that roadmap task 1.1.3's success bullet still resolves and that each
/// of its four nouns maps to exactly one register field.
pub(crate) fn check_success_criterion(rows: &[StatusRow], roadmap: &str) -> Result<(), String> {
    if !fold(roadmap).contains(&fold(SUCCESS_CRITERION_CLAUSE)) {
        return Err(format!(
            "docs/roadmap.md no longer contains the 1.1.3 success criterion. Repair: restore \
             {SUCCESS_CRITERION_CLAUSE:?}, or revise this contract together with the task."
        ));
    }
    for (noun, field) in CRITERION_NOUNS {
        if !rows.iter().any(|row| row.field == field) {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: no status-register field maps \
                 the criterion noun {noun:?}. Repair: add the {field} field, or reword the \
                 roadmap task."
            ));
        }
    }
    Ok(())
}

/// Checks that `docs/design.md` §6.1 still carries the clause the whole
/// instrument exists to defer to.
pub(crate) fn check_deferred_clause(design: &str) -> Result<(), String> {
    if !fold(design).contains(&fold(DEFERRED_CLAUSE)) {
        return Err(format!(
            "docs/design.md §6.1 no longer contains {DEFERRED_CLAUSE:?}. Repair: restore the \
             default's deferral, or revise ADR 004 and its contract together."
        ));
    }
    Ok(())
}

/// Checks that the aggregation register maps each reachable state of the note
/// multiset to exactly one outcome, and to the right one.
pub(crate) fn check_aggregation_total(rows: &[AggRow]) -> Result<(), String> {
    for (notes, insufficient, expected) in AGGREGATION_STATES {
        let matches = rows
            .iter()
            .filter(|row| matches_state(row, notes, insufficient))
            .collect::<Vec<&AggRow>>();
        match matches.as_slice() {
            [row] if row.outcome.starts_with(expected) => {}
            [row] => {
                return Err(format!(
                    "docs/adr-004-state-name-consumption-evidence.md: {} admissible notes with \
                     any insufficient {} yields {:?} where it must {expected}. Repair: a register \
                     that does not {expected} there is not a decision procedure.",
                    notes.to_lowercase(),
                    insufficient.to_lowercase(),
                    row.outcome
                ));
            }
            [] => {
                return Err(format!(
                    "docs/adr-004-state-name-consumption-evidence.md: the aggregation register \
                     does not cover {} admissible notes with any insufficient {}. Repair: add \
                     that row; it must {expected}.",
                    notes.to_lowercase(),
                    insufficient.to_lowercase()
                ));
            }
            _ => {
                return Err(format!(
                    "docs/adr-004-state-name-consumption-evidence.md: the aggregation register is \
                     ambiguous for {} admissible notes with any insufficient {}. Repair: keep \
                     exactly one row for that state.",
                    notes.to_lowercase(),
                    insufficient.to_lowercase()
                ));
            }
        }
    }
    if rows.len() != AGGREGATION_STATES.len() {
        return Err(format!(
            "docs/adr-004-state-name-consumption-evidence.md: the aggregation register has {} \
             rows and {} states are reachable. Repair: provide exactly one row per state.",
            rows.len(),
            AGGREGATION_STATES.len()
        ));
    }
    Ok(())
}

/// Whether an aggregation row covers one reachable state.
fn matches_state(row: &AggRow, notes: &str, insufficient: &str) -> bool {
    row.admissible_notes == notes && (notes == "None" || row.any_insufficient == insufficient)
}

/// Checks that the aggregation register's preconditions are reachable, and that
/// its vocabulary is closed.
pub(crate) fn check_aggregation_vocabulary(
    status: &[StatusRow],
    aggregation: &[AggRow],
) -> Result<(), String> {
    for row in aggregation {
        if !["None", "One or more"].contains(&row.admissible_notes.as_str()) {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: the aggregation register names \
                 {:?} admissible notes. Repair: use None or One or more.",
                row.admissible_notes
            ));
        }
        if !["n/a", "No", "Yes"].contains(&row.any_insufficient.as_str()) {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: the aggregation register names \
                 {:?} for any insufficient. Repair: use n/a, No, or Yes.",
                row.any_insufficient
            ));
        }
        if row.admissible_notes == "None" && row.any_insufficient != "n/a" {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: the no-evidence row cannot be \
                 qualified by {:?}. Repair: use n/a, because no note exists to insufficient it.",
                row.any_insufficient
            ));
        }
    }
    if !status
        .iter()
        .any(|row| row.admissible && row.contributes == NOTHING)
        && status.iter().any(|row| row.contributes == INSUFFICIENT)
    {
        return Err(
            "docs/adr-004-state-name-consumption-evidence.md: no admissible status contributes \
             nothing, so every admissible note would overturn the default. Repair: the verdict \
             would no longer be a verdict."
                .to_owned(),
        );
    }
    Ok(())
}

/// Checks each gate's title fragment against `docs/roadmap.md`.
pub(crate) fn check_gate_titles(adr: &str, roadmap: &str) -> Result<(), String> {
    for row in gate_rows(adr).map_err(|error| error.to_string())? {
        let matches = roadmap
            .lines()
            .filter(|line| line.contains(&row.fragment))
            .count();
        match matches {
            1 => {}
            0 => {
                return Err(format!(
                    "docs/roadmap.md: gate {} names task fragment {:?}, which matches no task. \
                     Repair: restore that task's title, or revise ADR 004's gate table.",
                    row.gate, row.fragment
                ));
            }
            _ => {
                return Err(format!(
                    "docs/roadmap.md: gate {} names task fragment {:?}, which matches {matches} \
                     tasks. Repair: use a fragment specific to one task.",
                    row.gate, row.fragment
                ));
            }
        }
    }
    Ok(())
}

/// Folds a string's whitespace runs to single spaces.
fn fold(text: &str) -> String { text.split_whitespace().collect::<Vec<_>>().join(" ") }
