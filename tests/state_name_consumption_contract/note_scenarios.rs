//! Note-cell and committed-note scenarios.
//!
//! These exercise what a *note* must satisfy: one row per register field,
//! citation-shaped evidence, the `identifier-need` obligations, and the verdict
//! that follows — including the rejection of a note whose cells select two
//! different contributions. The directory scan's own controls live in
//! `scan_scenarios.rs`, because they ask what the scan reads rather than what a
//! note says.

use googletest::prelude::*;
use pretty_assertions::assert_eq;
use rstest::rstest;

use super::{
    fixtures::{
        IDENTIFIER_NEED_ROW,
        METRICS_CARDINALITY_ROW,
        STATE_DISPLAY_NAME_ROW,
        TRACING_USE_ROW,
        note_with_rows,
        state_name_note,
        status_register,
    },
    live_status,
    parse::{note_rows, status_rows},
    policy::{check_note_cells, resolve_note},
    registers::{check_exclusions, check_vocabulary},
    types::{NoteRow, Resolution, StatusRow},
};

/// Rejects a note whose cells name a status under a field the register defines
/// under a different field.
#[test]
fn note_cells_reject_a_status_borrowed_from_another_field() -> Result<(), String> {
    let rows = live_status()?;
    let cells = note_rows(&state_name_note().replace(
        "| metrics-cardinality | Bounded |",
        "| metrics-cardinality | Enumerated |",
    ))
    .map_err(|error| error.to_string())?;
    assert_eq!(
        check_note_cells(&rows, &cells),
        Err(
            "a StateName note records status \"Enumerated\" for field \"metrics-cardinality\", \
             which the status register does not define. Repair: use a status ADR 004 lists for \
             that field."
                .to_owned()
        )
    );
    Ok(())
}

/// Presents a note to the register with one cell replaced by a blocking one.
///
/// Fallible rather than `expect`ing: `clippy.toml` sets
/// `allow-expect-in-tests`, which reaches `#[test]` bodies and not the helpers
/// they call, so a helper that panics on a parse failure is a lint error. It is
/// also the better behaviour — the parse failure it would have panicked on is a
/// real answer, and it belongs in the caller's error channel with the fixture
/// named.
fn blocked_by(rows: &[StatusRow], blocking: &StatusRow) -> Result<Resolution, String> {
    let note = note_rows(&state_name_note()).map_err(|error| error.to_string())?;
    let mutated = note
        .iter()
        .map(|cell| {
            if cell.field == blocking.field {
                NoteRow {
                    status: blocking.status.clone(),
                    ..cell.clone()
                }
            } else {
                cell.clone()
            }
        })
        .collect::<Vec<NoteRow>>();
    resolve_note(rows, &mutated)
}

/// Resolves a fixture note for every blocking register row, and one admissible
/// control so the check cannot pass by blocking everything.
#[test]
fn blocked_notes_resolve_to_not_resolved() -> Result<(), String> {
    let rows = live_status()?;
    let blocking = rows
        .iter()
        .filter(|row| !row.admissible)
        .collect::<Vec<&StatusRow>>();
    assert_that!(blocking.len(), gt(0));
    for row in blocking {
        assert_eq!(
            blocked_by(&rows, row)?,
            Resolution::NotResolved {
                field: row.field.clone(),
                status: row.status.clone(),
            }
        );
    }
    let admissible = note_rows(&state_name_note()).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &admissible)?;
    assert_eq!(resolve_note(&rows, &admissible)?, Resolution::Sufficient);
    Ok(())
}

/// Accepts the fixture note and rejects the eight documented note defects.
///
/// Each case supplies the note's four rows directly. A control that instead
/// edited the assembled note with `String::replace` would be silently skipped
/// the moment the fixture was rewrapped, and would then assert a message for a
/// defect it no longer introduces.
#[rstest]
#[case::residual_tbd(
    note_with_rows(&[STATE_DISPLAY_NAME_ROW, IDENTIFIER_NEED_ROW, METRICS_CARDINALITY_ROW,
                     "| tracing-use | TBD | `mdtablefix@abc1234:src/process.rs` |"]),
    "a StateName note still holds TBD in field \"tracing-use\". Repair: the note records an \
     observation, so every cell must be filled."
)]
#[case::unknown_status(
    note_with_rows(&["| state-display-name | Invented | `mdtablefix@abc1234:src/process.rs` |",
                     IDENTIFIER_NEED_ROW, METRICS_CARDINALITY_ROW, TRACING_USE_ROW]),
    "a StateName note records status \"Invented\" for field \"state-display-name\", which the \
     status register does not define. Repair: use a status ADR 004 lists for that field."
)]
#[case::prose_evidence(
    note_with_rows(&[STATE_DISPLAY_NAME_ROW, IDENTIFIER_NEED_ROW,
                     "| metrics-cardinality | Bounded | looks bounded from the enum |",
                     TRACING_USE_ROW]),
    "a StateName note records evidence for field \"metrics-cardinality\" that is not a citation of \
     the shape <repo>@<sha>:<path>. Repair: cite the revision the observation was made against."
)]
#[case::no_search_set(
    note_with_rows(&[STATE_DISPLAY_NAME_ROW,
                     "| identifier-need | None | `mdtablefix@abc1234:src/process.rs` and nothing \
                      else |",
                     METRICS_CARDINALITY_ROW, TRACING_USE_ROW]),
    "a StateName note's identifier-need cell names no consumer from the search set. Repair: name \
     the tracing subscriber, the metrics recorder or its documented absence, a model checker, or \
     generated documentation — or state that none exist."
)]
#[case::status_and_evidence_disagree(
    note_with_rows(&[STATE_DISPLAY_NAME_ROW,
                     "| identifier-need | None | subscriber and a model checker considered; \
                      `mdtablefix@abc1234:src/process.rs` needs stability across releases |",
                     METRICS_CARDINALITY_ROW, TRACING_USE_ROW]),
    "a StateName note records identifier-need: None but its evidence names a required property. \
     Repair: record Property required, or drop the claim — a status and its evidence must agree."
)]
#[case::missing_a_field(
    note_with_rows(&[STATE_DISPLAY_NAME_ROW, IDENTIFIER_NEED_ROW, METRICS_CARDINALITY_ROW]),
    "a StateName note records 3 fields; the status register defines 4. Repair: fill one row per \
     register field, naming each exactly as ADR 004 does."
)]
#[case::reordered_fields(
    note_with_rows(&[TRACING_USE_ROW, IDENTIFIER_NEED_ROW, METRICS_CARDINALITY_ROW,
                     STATE_DISPLAY_NAME_ROW]),
    "a StateName note records field \"tracing-use\" where the status register defines \
     \"state-display-name\". Repair: use the register's field names, in the register's order."
)]
#[case::citation_without_a_path(
    note_with_rows(&[STATE_DISPLAY_NAME_ROW, IDENTIFIER_NEED_ROW,
                     "| metrics-cardinality | Bounded | `mdtablefix@abc1234` |",
                     TRACING_USE_ROW]),
    "a StateName note records evidence for field \"metrics-cardinality\" that is not a citation of \
     the shape <repo>@<sha>:<path>. Repair: cite the revision the observation was made against."
)]
fn committed_state_name_notes_are_rejected(
    #[case] note: String,
    #[case] expected: &str,
) -> Result<(), String> {
    let rows = live_status()?;
    let cells = note_rows(&note).map_err(|error| error.to_string())?;
    let outcome =
        check_note_cells(&rows, &cells).and_then(|()| resolve_note(&rows, &cells).map(|_| ()));
    assert_eq!(outcome, Err(expected.to_owned()));
    Ok(())
}

/// Rejects a note whose cells select two different contributions.
///
/// `INV-CONSISTENCY`'s control. The live register's only decisive status sits on
/// `identifier-need`, so this pair is unreachable through it: the register is
/// rebuilt with a second field contributing `Sufficient`, and a note selecting
/// `Property required` is the only admissible way to reach the other
/// contribution. Every document-level check accepts the rebuilt register — its
/// vocabulary is still closed, exactly one admissible row selects
/// `Insufficient`, and the default can still fall — so only a note-level guard
/// can notice that reading a note through it has become contradictory.
///
/// The supporting controls matter as much as the rejection, because the
/// control's register is one cell away from a perfectly usable one. It still
/// resolves a note whose two decisive cells *agree*, so a guard that rejected
/// every note selecting two cells would fail that; and it still lets an
/// inadmissible cell block a note ahead of any contribution, so a guard that
/// reached the contradiction branch first would fail that, reporting a
/// disagreement where the repairable defect is the blocked cell.
#[test]
fn contradictory_notes_are_rejected() -> Result<(), String> {
    let live = status_register();
    let mutated = live.replace(
        "| metrics-cardinality | Bounded | yes | nothing |",
        "| metrics-cardinality | Bounded | yes | Sufficient |",
    );
    if mutated == live {
        return Err(
            "the fixture substitution matched nothing, so the control's register is the live one. \
             Repair: keep the needle a table row the fixture still carries."
                .to_owned(),
        );
    }
    let rows = status_rows(&mutated).map_err(|error| error.to_string())?;
    check_vocabulary(&rows)?;
    check_exclusions(&rows)?;

    // Two admissible fields now contribute, and on this note they agree: the
    // register is still a decision procedure, and the note still ratifies.
    let agreed = note_rows(&state_name_note()).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &agreed)?;
    assert_eq!(
        resolve_note(&rows, &agreed),
        Ok(Resolution::Sufficient),
        "the control's register must still be usable when its cells agree, or the rejection below \
         proves nothing about contradictions"
    );

    // A note whose decisive cells agree but which carries one inadmissible
    // cell. Blocking takes precedence over the contribution: the note is
    // refused, naming the cell to repair, rather than ratified on the two
    // cells that happen to agree.
    let blocked = note_rows(&note_with_rows(&[
        "| state-display-name | Not a named type | `mdtablefix@abc1234:src/process.rs` lists \
         LineMode |",
        IDENTIFIER_NEED_ROW,
        METRICS_CARDINALITY_ROW,
        TRACING_USE_ROW,
    ]))
    .map_err(|error| error.to_string())?;
    check_note_cells(&rows, &blocked)?;
    assert_eq!(
        resolve_note(&rows, &blocked),
        Ok(Resolution::NotResolved {
            field: "state-display-name".to_owned(),
            status: "Not a named type".to_owned(),
        }),
        "an inadmissible cell blocks the note, so it must not be ratified on its other cells"
    );

    // The contradiction: both cells are admissible and both are decisive, so
    // neither can yield to the other. The note is iterated in register order,
    // so the message names `identifier-need` first — the field on which the
    // default hangs — and the second selection as the one contradicting it.
    let cells = note_rows(&note_with_rows(&[
        STATE_DISPLAY_NAME_ROW,
        "| identifier-need | Property required | subscriber and metrics considered; \
         `mdtablefix@abc1234:src/process.rs` needs stability across releases |",
        METRICS_CARDINALITY_ROW,
        TRACING_USE_ROW,
    ]))
    .map_err(|error| error.to_string())?;
    check_note_cells(&rows, &cells)?;
    assert_eq!(
        resolve_note(&rows, &cells),
        Err(
            "a StateName note selects Insufficient in field \"identifier-need\" and Sufficient in \
             field \"metrics-cardinality\"; a note is read as the single non-nothing contribution \
             its cells select, and two different contributions are contradictory. Repair: leave \
             one cell decisive and make the other contribute nothing."
                .to_owned()
        )
    );
    Ok(())
}

/// Accepts the fixture note as the suite's accepting witness.
#[test]
fn committed_state_name_note_is_usable() -> Result<(), String> {
    let rows = live_status()?;
    let cells = note_rows(&state_name_note()).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &cells)?;
    assert_eq!(resolve_note(&rows, &cells)?, Resolution::Sufficient);
    Ok(())
}
