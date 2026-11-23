//! Equivalence between the two field-naming forms, and absence semantics.
//!
//! The contract accepts dotted string literals and the exported constants as
//! equal alternatives. Equality is a claim about the fields `tracing` builds,
//! not about the source text, so it is checked by capturing both forms and
//! comparing the whole observation: declared fields, recorded values, and how
//! each value was delivered.
//!
//! The absence cases pin the other direction: a declared field that is never
//! recorded is absent, while the same field recorded with the empty string is
//! present with `""`. Without the second case, "absent" could mean "empty" and
//! a consumer could not tell a missed path from an empty label.

use pretty_assertions::assert_eq;
use rstest::rstest;
use statelet::fields;

use super::{
    capture::{CapturedSpan, Via, capture},
    downstream::{Decision, Mode, handle_line, handle_line_with_literals},
};

/// A capture failure is a test failure with the reason attached.
fn captured(
    span: Result<CapturedSpan, super::capture::CaptureError>,
) -> Result<CapturedSpan, String> {
    span.map_err(|error| error.to_string())
}

/// Both naming forms produce the same span for the same run.
#[rstest]
#[case::success(false)]
#[case::error(true)]
fn literal_and_constant_forms_agree(#[case] fails: bool) -> Result<(), String> {
    let mode = Mode::Normalize;
    let constants = captured(capture("handle_line", || {
        let _ = handle_line(&mode, fails);
    }))?;
    let literals = captured(capture("handle_line", || {
        let _ = handle_line_with_literals(&mode, fails);
    }))?;

    assert_eq!(
        constants.declared.len(),
        5,
        "the constant-form span must declare all five contracted fields"
    );
    assert_eq!(
        literals.declared.len(),
        5,
        "the literal-form span must declare all five contracted fields"
    );
    assert!(
        constants.values.len() >= 4 && literals.values.len() >= 4,
        "each form must record at least four values so two empty captures cannot agree"
    );
    assert_eq!(
        constants, literals,
        "the constant and literal forms must produce identical spans"
    );
    Ok(())
}

/// The success run records the decision label as a string.
#[test]
fn the_success_run_records_the_outcome_as_a_string() -> Result<(), String> {
    let mode = Mode::Normalize;
    let span = captured(capture("handle_line", || {
        let _ = handle_line(&mode, false);
    }))?;

    assert_eq!(
        span.value(fields::TRANSITION_OUTCOME)
            .map(|value| (value.text.as_str(), value.via)),
        Some(("stay", Via::Str)),
        "the outcome must be recorded as the decision's string label"
    );
    Ok(())
}

/// The error run records the error category as a string and no outcome.
#[test]
fn the_error_run_records_the_category_as_a_string() -> Result<(), String> {
    let mode = Mode::Normalize;
    let span = captured(capture("handle_line", || {
        let _ = handle_line(&mode, true);
    }))?;

    assert_eq!(
        span.value(fields::TRANSITION_ERROR)
            .map(|value| (value.text.as_str(), value.via)),
        Some(("unterminated_row", Via::Str)),
        "the error must be recorded as the failure's string category"
    );
    assert_eq!(
        span.value(fields::TRANSITION_OUTCOME),
        None,
        "this fixture records no outcome on the error path"
    );
    Ok(())
}

/// A declared field that is never recorded is absent.
#[test]
fn an_unrecorded_field_is_absent() -> Result<(), String> {
    let span = captured(capture("absence_probe", || {
        let span = tracing::info_span!("absence_probe", late = tracing::field::Empty);
        let _guard = span.enter();
    }))?;

    assert!(
        span.declares("late"),
        "the probe must declare the late field"
    );
    assert_eq!(
        span.value("late"),
        None,
        "a declared field that is never recorded must be absent"
    );
    Ok(())
}

/// The same declared field recorded with `""` is present with `""`.
#[test]
fn an_empty_string_is_recorded_as_present() -> Result<(), String> {
    let span = captured(capture("absence_probe", || {
        let span = tracing::info_span!("absence_probe", late = tracing::field::Empty);
        let _guard = span.enter();
        tracing::Span::current().record("late", "");
    }))?;

    assert_eq!(
        span.value("late")
            .map(|value| (value.text.as_str(), value.via)),
        Some(("", Via::Str)),
        "recording the empty string must produce a present value, not an absent one"
    );
    Ok(())
}

/// The fixtures' decision labels are the documented fixture data.
#[test]
fn decision_labels_are_stable_fixture_data() {
    assert_eq!(
        (Decision::Stay.label(), Decision::Move.label()),
        ("stay", "move"),
        "the fixture decision labels must match the scenarios' expected values"
    );
}
