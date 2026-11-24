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
    capture::{Via, capture},
    downstream::{Decision, Mode, handle_early_return, handle_line, handle_line_with_literals},
};

/// Both naming forms produce the same span for the same run.
#[rstest]
#[case::success(false)]
#[case::error(true)]
fn literal_and_constant_forms_agree(#[case] fails: bool) {
    let mode = Mode::Normalize;
    let constants = capture("handle_line", || {
        let _outcome = handle_line(mode, fails);
    })
    .expect("the constant-form boundary must create exactly one span");
    let literals = capture("handle_line", || {
        let _outcome = handle_line_with_literals(mode, fails);
    })
    .expect("the literal-form boundary must create exactly one span");

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
}

/// The success run records the decision label as a string.
#[test]
fn the_success_run_records_the_outcome_as_a_string() {
    let mode = Mode::Normalize;
    let span = capture("handle_line", || {
        let _outcome = handle_line(mode, false);
    })
    .expect("the success run must create exactly one span");

    assert_eq!(
        span.value(fields::TRANSITION_OUTCOME)
            .map(|value| (value.text.as_str(), value.via)),
        Some(("stay", Via::Str)),
        "the outcome must be recorded as the decision's string label"
    );
}

/// The error run records the error category as a string and no outcome.
#[test]
fn the_error_run_records_the_category_as_a_string() {
    let mode = Mode::Normalize;
    let span = capture("handle_line", || {
        let _outcome = handle_line(mode, true);
    })
    .expect("the error run must create exactly one span");

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
}

/// A declared field that is never recorded is absent.
#[test]
fn an_unrecorded_field_is_absent() {
    let span = capture("absence_probe", || {
        let span = tracing::info_span!("absence_probe", late = tracing::field::Empty);
        let _guard = span.enter();
    })
    .expect("the absence probe must create exactly one span");

    assert!(
        span.declares("late"),
        "the probe must declare the late field"
    );
    assert_eq!(
        span.value("late"),
        None,
        "a declared field that is never recorded must be absent"
    );
}

/// The same declared field recorded with `""` is present with `""`.
#[test]
fn an_empty_string_is_recorded_as_present() {
    let span = capture("absence_probe", || {
        let span = tracing::info_span!("absence_probe", late = tracing::field::Empty);
        let _guard = span.enter();
        tracing::Span::current().record("late", "");
    })
    .expect("the absence probe must create exactly one span");

    assert_eq!(
        span.value("late")
            .map(|value| (value.text.as_str(), value.via)),
        Some(("", Via::Str)),
        "recording the empty string must produce a present value, not an absent one"
    );
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

/// The early-return fixture's success path records its outcome.
///
/// The error path leaves `transition.error` absent; this case proves the
/// same fixture does record an outcome when it succeeds, so the absence in
/// the error scenario is a missed capture rather than a fixture that never
/// records anything.
#[test]
fn the_early_return_fixture_records_its_outcome_on_success() {
    let span = capture("handle_early_return", || {
        let _outcome = handle_early_return(Mode::Normalize, false);
    })
    .expect("the early-return success run must create exactly one span");

    assert_eq!(
        span.value(fields::TRANSITION_OUTCOME)
            .map(|value| (value.text.as_str(), value.via)),
        Some(("stay", Via::Str)),
        "the early-return fixture must record its outcome when it succeeds"
    );
    assert_eq!(
        span.value(fields::TRANSITION_ERROR),
        None,
        "a successful run must not record an error"
    );
}
