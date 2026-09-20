//! Checks that bind ADR 004's registers to each other.
//!
//! The status register records vocabulary; this module checks that the
//! aggregation register speaks it, and owns the status register's internal
//! consistency — that its vocabulary is closed and that the default can still
//! fall — because both are claims about the *register*, not about any note read
//! through it. The two checks that bind ADR 004 to `docs/roadmap.md` live in
//! `roadmap.rs` instead: both read the roadmap as *task records*, which is a
//! different job from reading a register, and neither is a claim about the
//! register at all.

use super::{
    fold_whitespace,
    policy::{IDENTIFIER_NEED, INSUFFICIENT, NOTHING, PROPERTY_REQUIRED, SUFFICIENT},
    types::{AggRow, StatusRow},
};

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

/// Checks that `docs/design.md` §6.1 still carries the clause the whole
/// instrument exists to defer to.
pub(crate) fn check_deferred_clause(design: &str) -> Result<(), String> {
    if !fold_whitespace(design).contains(&fold_whitespace(DEFERRED_CLAUSE)) {
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
                    "docs/adr-004-state-name-consumption-evidence.md: {} contributing notes with \
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
                     does not cover {} contributing notes with any insufficient {}. Repair: add \
                     that row; it must {expected}.",
                    notes.to_lowercase(),
                    insufficient.to_lowercase()
                ));
            }
            _ => {
                return Err(format!(
                    "docs/adr-004-state-name-consumption-evidence.md: the aggregation register is \
                     ambiguous for {} contributing notes with any insufficient {}. Repair: keep \
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
    row.contributing_notes == notes && (notes == "None" || row.any_insufficient == insufficient)
}

/// Checks that the aggregation register's preconditions are reachable, and that
/// its vocabulary is closed.
pub(crate) fn check_aggregation_vocabulary(
    status: &[StatusRow],
    aggregation: &[AggRow],
) -> Result<(), String> {
    for row in aggregation {
        if !["None", "One or more"].contains(&row.contributing_notes.as_str()) {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: the aggregation register names \
                 {:?} contributing notes. Repair: use None or One or more.",
                row.contributing_notes
            ));
        }
        if !["n/a", "No", "Yes"].contains(&row.any_insufficient.as_str()) {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: the aggregation register names \
                 {:?} for any insufficient. Repair: use n/a, No, or Yes.",
                row.any_insufficient
            ));
        }
        if row.contributing_notes == "None" && row.any_insufficient != "n/a" {
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

/// Checks that the default holds without a required property, and that it can
/// still fall.
///
/// An `Insufficient` contribution is the register's one power to overturn
/// `&'static str`, so it is confined to a single field *and* a single status: a
/// second form of sufficient cause would let a later edit make some other
/// observation decisive without that being visible as a decision.
///
/// `INV-REGISTERS` pins the live document, so both halves are implied for it.
/// They are not redundant against the real threat model: an editor who changes
/// ADR 004 and updates the fixture in the same commit keeps that check green and
/// trips these.
pub(crate) fn check_exclusions(rows: &[StatusRow]) -> Result<(), String> {
    for row in rows.iter().filter(|row| row.contributes == INSUFFICIENT) {
        let cause = if row.field == IDENTIFIER_NEED {
            PROPERTY_REQUIRED
        } else {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: field {} status {} selects \
                 {INSUFFICIENT}. Repair: only a recorded required property may overturn the \
                 &'static str default.",
                row.field, row.status
            ));
        };
        if row.status != cause {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: field {} status {} selects \
                 {INSUFFICIENT}. Repair: only the {cause} status may overturn the &'static str \
                 default.",
                row.field, row.status
            ));
        }
    }
    if !rows.iter().any(|row| row.contributes == INSUFFICIENT) {
        return Err(
            "docs/adr-004-state-name-consumption-evidence.md: no row selects Insufficient. \
             Repair: a register that cannot overturn the default is not a decision procedure."
                .to_owned(),
        );
    }
    Ok(())
}

/// Checks that the register's vocabulary is closed, that an inadmissible cell
/// contributes nothing, and that a note's verdict is unambiguous.
pub(crate) fn check_vocabulary(rows: &[StatusRow]) -> Result<(), String> {
    for row in rows {
        if ![NOTHING, SUFFICIENT, INSUFFICIENT].contains(&row.contributes.as_str()) {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: status {} contributes {:?}. \
                 Repair: use {NOTHING}, {SUFFICIENT} or {INSUFFICIENT}.",
                row.status, row.contributes
            ));
        }
        if !row.admissible && row.contributes != NOTHING {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: inadmissible status {} still \
                 contributes {}. Repair: an inadmissible cell blocks the note, so it contributes \
                 {NOTHING}.",
                row.status, row.contributes
            ));
        }
    }
    let deciding = rows
        .iter()
        .filter(|row| row.admissible && row.contributes == INSUFFICIENT)
        .count();
    if deciding != 1 {
        return Err(format!(
            "docs/adr-004-state-name-consumption-evidence.md: {deciding} admissible rows select \
             {INSUFFICIENT}. Repair: exactly one admissible row may overturn the default, so that \
             a note's verdict is unambiguous."
        ));
    }
    Ok(())
}
