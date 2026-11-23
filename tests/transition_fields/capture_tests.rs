//! Unit tests for the capture subscriber, through the real macros.
//!
//! These prove the subscriber itself is faithful before the downstream
//! scenarios rely on it: a late field recorded through `Span::current`
//! arrives, an unrecorded one stays absent, and `capture` reports a missing,
//! extra, or misnamed span instead of returning an empty one.

use pretty_assertions::assert_eq;
use rstest::rstest;
use tracing::info_span;

use super::capture::{CaptureError, Via, capture};

/// A late field declared `Empty` is recorded through `Span::current()`.
#[test]
fn a_late_field_records_through_the_current_span() {
    let span = capture("capture_unit", || {
        let span = info_span!("capture_unit", late = tracing::field::Empty);
        let _guard = span.enter();
        tracing::Span::current().record("late", "recorded");
    })
    .expect("the capture subscriber must observe exactly one span");

    assert!(
        span.declares("late"),
        "the span must declare the late field"
    );
    assert_eq!(
        span.value("late")
            .map(|value| (value.text.as_str(), value.via)),
        Some(("recorded", Via::Str)),
        "the late field must arrive as a string value"
    );
}

/// A declared field that is never recorded stays absent.
#[test]
fn an_unrecorded_late_field_is_absent() {
    let span = capture("capture_unit", || {
        let span = info_span!("capture_unit", late = tracing::field::Empty);
        let _guard = span.enter();
    })
    .expect("the capture subscriber must observe exactly one span");

    assert!(
        span.declares("late"),
        "the span must declare the late field"
    );
    assert_eq!(
        span.value("late"),
        None,
        "an unrecorded field must be absent rather than defaulted"
    );
}

/// A closure that creates no span is an error, not an empty capture.
#[test]
fn capture_rejects_no_span() {
    match capture("capture_unit", || {}) {
        Err(CaptureError::NoSpan { expected }) => {
            assert_eq!(
                expected, "capture_unit",
                "the error must name the span that was expected"
            );
        }
        other => panic!("expected NoSpan, got {other:?}"),
    }
}

/// A closure that creates two spans is an error.
#[test]
fn capture_rejects_two_spans() {
    match capture("capture_unit", || {
        drop(info_span!("capture_unit"));
        drop(info_span!("capture_unit"));
    }) {
        Err(CaptureError::TooManySpans { count }) => {
            assert_eq!(count, 2, "the error must report both spans");
        }
        other => panic!("expected TooManySpans, got {other:?}"),
    }
}

/// A single span with another name is an error naming both names.
#[rstest]
#[case::other_name("other")]
fn capture_rejects_a_misnamed_span(#[case] actual: &str) {
    match capture("capture_unit", || {
        drop(info_span!("other"));
    }) {
        Err(CaptureError::UnexpectedName {
            expected,
            actual: seen,
        }) => {
            assert_eq!(
                expected, "capture_unit",
                "the error must name the expectation"
            );
            assert_eq!(
                seen, actual,
                "the error must name the span that was created"
            );
        }
        other => panic!("expected UnexpectedName, got {other:?}"),
    }
}
