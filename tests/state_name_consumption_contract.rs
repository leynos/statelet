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

#[path = "state_name_consumption_contract/clauses.rs"]
mod clauses;
#[path = "state_name_consumption_contract/fixtures.rs"]
mod fixtures;
#[path = "state_name_consumption_contract/notes.rs"]
mod notes;
#[path = "state_name_consumption_contract/parse.rs"]
mod parse;
#[path = "state_name_consumption_contract/policy.rs"]
mod policy;
#[path = "state_name_consumption_contract/registers.rs"]
mod registers;
#[path = "state_name_consumption_contract/types.rs"]
mod types;

use camino::Utf8PathBuf;
use fixtures::{
    IDENTIFIER_NEED_ROW,
    METRICS_CARDINALITY_ROW,
    STATE_DISPLAY_NAME_ROW,
    STATUS_HEADING,
    TRACING_USE_ROW,
    aggregation_register,
    benchmark_note,
    gate_table,
    note_with_rows,
    state_name_note,
    status_register,
    status_register_without,
};
use googletest::prelude::*;
use notes::committed_notes;
use parse::{aggregation_rows, gate_rows, note_rows, status_rows, status_rows_or_error};
use policy::{check_exclusions, check_note_cells, check_vocabulary, resolve_note};
use pretty_assertions::assert_eq;
use registers::{
    check_aggregation_total,
    check_aggregation_vocabulary,
    check_deferred_clause,
    check_gate_titles,
    check_success_criterion,
};
use rstest::rstest;
use types::{NoteRow, ParseError, Register, Resolution, StatusRow, field_order};

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

/// Reads the live register and rejects it unless it matches the fixture.
#[test]
fn status_register_matches_fixture() -> Result<(), String> {
    let rows = live_status()?;
    let fixture = status_rows(&status_register()).map_err(|error| error.to_string())?;
    assert_eq!(rows, fixture);
    check_vocabulary(&rows)?;
    Ok(())
}

/// Rejects a register with no delimiters, and reports the repair through the
/// adapter the scenarios that only need vocabulary call.
#[test]
fn status_register_rejects_missing_delimiters() {
    let undelimited = status_register()
        .replace(Register::Status.begin(), "")
        .replace(Register::Status.end(), "");
    assert_eq!(
        status_rows(&undelimited),
        Err(ParseError::MissingDelimiters {
            register: Register::Status
        })
    );
    assert_eq!(
        status_rows_or_error(STATUS_HEADING),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: no status register found between \
             <!-- status-register:begin --> and <!-- status-register:end -->. Repair: add the \
             status register to the ## Status register section."
                .to_owned()
        )
    );
}

/// Rejects a register whose section heading has been renamed.
#[test]
fn status_register_rejects_a_missing_section() {
    let renamed = status_register().replace("## Status register", "## Statuses");
    assert_eq!(
        status_rows(&renamed),
        Err(ParseError::MissingSection {
            register: Register::Status
        })
    );
}

/// Rejects a status token the register's own vocabulary does not define, as the
/// vocabulary check downstream reports it.
#[test]
fn status_register_rejects_an_unknown_status() {
    let mutated = status_register().replace("| yes | nothing |", "| yes | plenty |");
    let rows = status_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_vocabulary(&rows),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: status Enumerated contributes \
             \"plenty\". Repair: use nothing, Sufficient or Insufficient."
                .to_owned()
        )
    );
}

/// Rejects an admissibility cell that is neither `yes` nor `no`.
#[test]
fn status_register_rejects_a_non_boolean_admissibility() {
    let mutated = status_register().replace("| yes | nothing |", "| maybe | nothing |");
    assert_eq!(
        status_rows(&mutated),
        Err(ParseError::UnknownToken {
            register: Register::Status,
            column: "admissibility",
            found: "maybe".to_owned(),
        })
    );
}

/// Reads the live aggregation register and rejects it unless it matches.
#[test]
fn aggregation_register_matches_fixture() -> Result<(), String> {
    let rows = aggregation_rows(ADR).map_err(|error| error.to_string())?;
    let fixture = aggregation_rows(&aggregation_register()).map_err(|error| error.to_string())?;
    assert_eq!(rows, fixture);
    check_aggregation_vocabulary(&live_status()?, &rows)?;
    Ok(())
}

/// Rejects an empty aggregation register rather than passing over zero rows.
#[test]
fn aggregation_register_rejects_an_empty_block() {
    let emptied = aggregation_register()
        .replace(Register::Aggregation.begin(), "")
        .replace(Register::Aggregation.end(), "");
    assert_eq!(
        aggregation_rows(&emptied),
        Err(ParseError::MissingDelimiters {
            register: Register::Aggregation
        })
    );
}

/// Covers each reachable state of the note multiset exactly once, and to the
/// right outcome.
#[rstest]
#[case::no_admissible_note("None", "n/a", "Blocked")]
#[case::admissible_and_none_insufficient("One or more", "No", "Ratify")]
#[case::admissible_and_some_insufficient("One or more", "Yes", "Amend")]
fn aggregation_register_is_total(
    #[case] notes: &str,
    #[case] insufficient: &str,
    #[case] expected: &str,
) -> Result<(), String> {
    let rows = aggregation_rows(ADR).map_err(|error| error.to_string())?;
    check_aggregation_total(&rows)?;
    // Both columns must be matched in one predicate. Filtering after `find`
    // would test only the first row carrying the notes count, so a case asking
    // for `One or more` *and* `Yes` would never reach the row that answers it.
    let row = rows
        .iter()
        .find(|row| {
            row.admissible_notes == notes
                && (notes == "None" || row.any_insufficient == insufficient)
        })
        .expect("the aggregation register must cover every reachable state");
    assert_that!(row.outcome.as_str(), starts_with(expected));
    Ok(())
}

/// Rejects an aggregation register that ratifies on no evidence.
#[test]
fn aggregation_register_rejects_an_outcome_that_does_not_match() {
    let mutated = aggregation_register().replace(
        "| None | n/a | Blocked: no admissible evidence |",
        "| None | n/a | Ratify the current return type |",
    );
    let rows = aggregation_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_aggregation_total(&rows),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: none admissible notes with any \
             insufficient n/a yields \"Ratify the current return type\" where it must Blocked. \
             Repair: a register that does not Blocked there is not a decision procedure."
                .to_owned()
        )
    );
}

/// Rejects an aggregation register missing a reachable state.
#[test]
fn aggregation_register_rejects_a_missing_state() {
    let mutated =
        aggregation_register().replace("| None | n/a | Blocked: no admissible evidence |\n", "");
    let rows = aggregation_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_aggregation_total(&rows),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: the aggregation register does not \
             cover none admissible notes with any insufficient n/a. Repair: add that row; it must \
             Blocked."
                .to_owned()
        )
    );
}

/// Rejects a register where a status other than the recorded required property
/// is made decisive.
#[test]
fn register_confines_insufficient_to_the_required_property() {
    let mutated = status_register().replace(
        "| metrics-cardinality | Bounded | yes | nothing |",
        "| metrics-cardinality | Bounded | yes | Insufficient |",
    );
    let rows = status_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_exclusions(&rows),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: field metrics-cardinality status \
             Bounded selects Insufficient. Repair: only a recorded required property may overturn \
             the &'static str default."
                .to_owned()
        )
    );
}

/// Rejects a register where the required-property field carries a decisive
/// status that does not record a required property.
///
/// The wrong-field half of `check_exclusions` is covered above; this is the
/// wrong-status half, and the two are separate branches of the check.
#[test]
fn register_confines_insufficient_to_the_required_status() {
    let mutated = status_register()
        .replace(
            "| identifier-need | None | yes | Sufficient |",
            "| identifier-need | None | yes | nothing |",
        )
        .replace(
            "| identifier-need | Property required | yes | Insufficient |",
            "| identifier-need | Overridden | yes | Insufficient |",
        );
    let rows = status_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_exclusions(&rows),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: field identifier-need status \
             Overridden selects Insufficient. Repair: only the Property required status may \
             overturn the &'static str default."
                .to_owned()
        )
    );
}

/// Rejects a status register where every admissible cell overturns the default,
/// leaving the verdict nothing to be a verdict about.
#[test]
fn aggregation_vocabulary_rejects_a_register_without_a_neutral_status() {
    let mutated = status_register().replace("| nothing |", "| Sufficient |");
    let rows = status_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_aggregation_vocabulary(&rows, &[]),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: no admissible status contributes \
             nothing, so every admissible note would overturn the default. Repair: the verdict \
             would no longer be a verdict."
                .to_owned()
        )
    );
}

/// Rejects a register that would overturn the default on evidence other than a
/// recorded required property.
#[test]
fn default_survives_without_a_required_property() {
    let mutated = status_register().replace(
        "| identifier-need | None | yes | Sufficient |",
        "| identifier-need | None | yes | Insufficient |",
    );
    let rows = status_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_exclusions(&rows),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: field identifier-need status None \
             selects Insufficient. Repair: only the Property required status may overturn the \
             &'static str default."
                .to_owned()
        )
    );
}

/// Rejects a register that can no longer overturn the default.
#[test]
fn register_can_select_insufficient() {
    let mutated = status_register().replace(
        "| identifier-need | Property required | yes | Insufficient |",
        "| identifier-need | Property required | yes | Sufficient |",
    );
    let rows = status_rows(&mutated).expect("the mutated register still parses");
    assert_eq!(
        check_exclusions(&rows),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: no row selects Insufficient. \
             Repair: a register that cannot overturn the default is not a decision procedure."
                .to_owned()
        )
    );
}

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

/// Accepts the fixture note and rejects the seven documented note defects.
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

/// Accepts the fixture note as the suite's accepting witness.
#[test]
fn committed_state_name_note_is_usable() -> Result<(), String> {
    let rows = live_status()?;
    let cells = note_rows(&state_name_note()).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &cells)?;
    assert_eq!(resolve_note(&rows, &cells)?, Resolution::Sufficient);
    Ok(())
}

/// Ignores a note without the marker rather than reading its fields as a
/// `StateName` note's.
///
/// This is the accepting end of `INV-FILLED`'s marker control. The note is
/// well-formed Markdown and parses like any other; the marker is what makes it
/// a `StateName` note, and the scan reads the marker rather than globbing the
/// directory, so a note belonging to roadmap task 1.2.3 cannot fail this
/// suite's checks by arriving.
///
/// The assertions return through the error channel rather than panicking, so
/// that every way this control can fail names the artefact it read.
#[test]
fn unmarked_notes_are_ignored() -> Result<(), String> {
    let unmarked = note_rows(&state_name_note().replace(notes::MARKER, ""))
        .map_err(|error| error.to_string())?;
    if unmarked.len() != 4 {
        return Err(format!(
            "the note without its marker no longer parses as a four-row table; it yields {} rows. \
             Repair: keep the fixture note well-formed so the control isolates the marker.",
            unmarked.len()
        ));
    }
    if benchmark_note().contains(notes::MARKER) {
        return Err(
            "the benchmark note carries the StateName marker, so the control cannot show that an \
             unmarked note is ignored."
                .to_owned(),
        );
    }
    // `docs/validation-notes/README.md` *mentions* the marker inside a code
    // span. A scan reading the marker as a substring would treat the README as
    // a note and then reject it for carrying no note register; a scan reading
    // it as a line of its own does not.
    if committed_notes(&workspace_root())
        .iter()
        .any(|note| note.file_name == "README.md")
    {
        return Err(
            "docs/validation-notes/README.md was read as a committed note. It documents the \
             marker inside a code span without declaring it, so the scan must match the marker as \
             a line of its own."
                .to_owned(),
        );
    }
    Ok(())
}

/// Scans the live notes directory and checks every marked note.
///
/// A failure names the note it came from. A message saying only that some note
/// holds a `TBD` would send a Phase 2 engineer to the directory rather than to
/// the file.
///
/// An empty directory passes: no note can honestly exist until roadmap task
/// 2.2.1 has annotated something, so a failure there would demand a fabricated
/// observation. `committed_state_name_notes_are_rejected` and the string
/// fixtures are this invariant's non-vacuity, not the directory's contents.
#[test]
fn committed_state_name_notes_are_usable() -> Result<(), String> {
    let rows = live_status()?;
    for note in committed_notes(&workspace_root()) {
        let name = note.file_name.as_str();
        let cells = note_rows(&note.text).map_err(|error| format!("{name}: {error}"))?;
        check_note_cells(&rows, &cells).map_err(|error| format!("{name}: {error}"))?;
        resolve_note(&rows, &cells).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

/// Checks the blank form field-by-field against the register.
///
/// The form is read from `fixtures::TEMPLATE`, which Step 6 replaces with the
/// `include_str!` of `docs/phase-2-validation-note-template.md`.
#[test]
fn template_matches_the_status_register() -> Result<(), String> {
    let rows = live_status()?;
    let template = note_rows(fixtures::TEMPLATE).map_err(|error| error.to_string())?;
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
    Some(
        "docs/roadmap.md: gate S4 names task fragment \"Do the thing\", which matches no task. \
         Repair: restore that task's title, or revise ADR 004's gate table."
    )
)]
#[case::ambiguous(
    "S3",
    "baseline",
    Some(
        "docs/roadmap.md: gate S3 names task fragment \"baseline\", which matches 15 tasks. \
         Repair: use a fragment specific to one task."
    )
)]
fn gate_titles_resolve(
    #[case] gate: &str,
    #[case] fragment: &str,
    #[case] failure: Option<&str>,
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
        Some(expected) => assert_eq!(
            check_gate_titles(&mutated, ROADMAP),
            Err(expected.to_owned())
        ),
    }
    Ok(())
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
        Err(
            "docs/roadmap.md no longer contains the 1.1.3 success criterion. Repair: restore \
             \"the Phase 2 validation note template has fields for state display name, optional \
             identifier need, metrics cardinality, and tracing use\", or revise this contract \
             together with the task."
                .to_owned()
        )
    );
    Ok(())
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
