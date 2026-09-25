//! Register-parsing and register-consistency scenarios.
//!
//! These read ADR 004's status and aggregation registers directly: the fixture
//! pin, the delimiter and section defects, and the predicates that keep the
//! register a decision procedure. Everything here is a claim about the
//! *document*, not about a note read through it.

use googletest::prelude::*;
use pretty_assertions::assert_eq;
use rstest::rstest;

use super::{
    ADR,
    fixtures::{STATUS_HEADING, aggregation_register, status_register},
    live_status,
    parse::{aggregation_rows, status_rows, status_rows_or_error},
    registers::{
        check_aggregation_total,
        check_aggregation_vocabulary,
        check_exclusions,
        check_vocabulary,
    },
    types::{ParseError, Register},
};

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
#[case::no_admissible_note("None", "n/a", "Block")]
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
            row.contributing_notes == notes
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
            "docs/adr-004-state-name-consumption-evidence.md: none contributing notes with any \
             insufficient n/a yields \"Ratify the current return type\" where it must Block. \
             Repair: a register that does not Block there is not a decision procedure."
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
             cover none contributing notes with any insufficient n/a. Repair: add that row; it \
             must Block."
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
