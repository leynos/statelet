//! Public-API tests for `statelet::StateName`.
//!
//! These tests treat the trait as a downstream consumer sees it. Both example
//! enums are local fixtures that deliberately do not implement `Debug`, so a
//! `Debug` supertrait added to the trait, or a label derived from formatting,
//! would fail to compile rather than pass silently. The expected labels are
//! stated in the test data, never read back from the implementation's own
//! match arms.

use googletest::prelude::*;
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use rstest::rstest;
use statelet::StateName;

/// Fieldless illustration enum mirroring the design's continuation modes.
enum FieldlessState {
    /// A normalization step.
    Normalize,
    /// A tight code-span step.
    TightCodeSpan,
    /// A verbatim-flush step.
    VerbatimFlush,
}

impl StateName for FieldlessState {
    fn state_name(&self) -> &'static str {
        match self {
            Self::Normalize => "Normalize",
            Self::TightCodeSpan => "TightCodeSpan",
            Self::VerbatimFlush => "VerbatimFlush",
        }
    }
}

/// Payload-bearing illustration enum used for independently labelled states.
enum PayloadState {
    /// No read is in progress.
    Idle,
    /// A read is in progress with the given byte count.
    Reading(u64),
}

impl StateName for PayloadState {
    fn state_name(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Reading(_) => "Reading",
        }
    }
}

/// Reads a state's label through the public trait alone.
#[must_use]
fn name_of<T: StateName>(state: &T) -> &'static str { state.state_name() }

/// Returns a payload only while the state is reading.
#[must_use]
const fn payload_of(state: &PayloadState) -> Option<u64> {
    match state {
        PayloadState::Idle => None,
        PayloadState::Reading(value) => Some(*value),
    }
}

/// Every fieldless variant reports its exact independently stated label.
#[rstest]
#[case::normalize(FieldlessState::Normalize, "Normalize")]
#[case::tight_code_span(FieldlessState::TightCodeSpan, "TightCodeSpan")]
#[case::verbatim_flush(FieldlessState::VerbatimFlush, "VerbatimFlush")]
fn fieldless_labels_are_exact(#[case] state: FieldlessState, #[case] expected: &'static str) {
    assert_that!(state.state_name(), eq(expected));
}

/// Both payload-example variants report their exact stated labels.
#[rstest]
#[case::idle(PayloadState::Idle, "Idle")]
#[case::reading(PayloadState::Reading(7), "Reading")]
fn payload_labels_are_exact(#[case] state: PayloadState, #[case] expected: &'static str) {
    assert_that!(state.state_name(), eq(expected));
}

/// The complete fieldless label collection is ordered as declared.
#[test]
fn all_fieldless_labels_match_the_declared_table() {
    let actual: Vec<&'static str> = [
        FieldlessState::Normalize,
        FieldlessState::TightCodeSpan,
        FieldlessState::VerbatimFlush,
    ]
    .iter()
    .map(StateName::state_name)
    .collect();

    assert_eq!(
        actual,
        ["Normalize", "TightCodeSpan", "VerbatimFlush"],
        "fieldless labels must follow the declared table order"
    );
}

/// The label is `'static`, repeatable, and leaves the state usable.
#[test]
fn label_outlives_the_state_and_does_not_consume_it() {
    let label: &'static str = {
        let state = FieldlessState::TightCodeSpan;
        assert_that!(name_of(&state), eq("TightCodeSpan"));
        assert_that!(name_of(&state), eq("TightCodeSpan"));
        assert_that!(state.state_name(), eq("TightCodeSpan"));
        state.state_name()
    };

    assert_that!(label, eq("TightCodeSpan"));
}

/// Boundary payloads are neither clamped nor normalized by naming.
#[rstest]
#[case::zero(0)]
#[case::maximum(u64::MAX)]
fn naming_preserves_boundary_payloads(#[case] value: u64) {
    let state = PayloadState::Reading(value);
    assert_that!(state.state_name(), eq("Reading"));
    assert_that!(payload_of(&state), some(eq(value)));
}

proptest! {
    /// Reading labels ignore the payload across the generated `u64` space.
    #[test]
    fn reading_label_is_independent_of_payload(value in any::<u64>()) {
        let state = PayloadState::Reading(value);
        prop_assert_eq!(state.state_name(), "Reading");
        prop_assert_eq!(payload_of(&state), Some(value));
    }
}
