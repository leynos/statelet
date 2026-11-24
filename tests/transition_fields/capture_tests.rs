//! Unit tests for the capture subscriber, through the real macros.
//!
//! These prove the subscriber itself is faithful before the downstream
//! scenarios rely on it: a late field recorded through `Span::current`
//! arrives, an unrecorded one stays absent, and `capture` reports a missing,
//! extra, or misnamed span instead of returning an empty one.

use pretty_assertions::assert_eq;
use rstest::rstest;
use statelet::fields;
use tracing::{info_span, level_filters::LevelFilter};
use tracing_core::{
    Metadata,
    span::{Attributes, Current, Id, Record},
};

use super::capture::{CaptureError, CaptureSubscriber, Via, capture};

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

/// A subscriber that rejects one named span while capturing everything else.
///
/// It models a level or target filter that disables a boundary's own span but
/// leaves an enclosing span enabled, which is the case the documentation
/// warns about.
struct FilteredSubscriber {
    /// The capture subscriber the wrapper delegates to.
    inner: CaptureSubscriber,
    /// The span name the filter rejects.
    rejected: &'static str,
}

impl tracing::Subscriber for FilteredSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.name() != self.rejected && self.inner.enabled(metadata)
    }

    fn max_level_hint(&self) -> Option<LevelFilter> { self.inner.max_level_hint() }

    fn new_span(&self, attributes: &Attributes<'_>) -> Id { self.inner.new_span(attributes) }

    fn record(&self, span: &Id, values: &Record<'_>) { self.inner.record(span, values) }

    fn record_follows_from(&self, span: &Id, follows: &Id) {
        self.inner.record_follows_from(span, follows);
    }

    fn event(&self, event: &tracing::Event<'_>) { self.inner.event(event) }

    fn enter(&self, span: &Id) { self.inner.enter(span); }

    fn exit(&self, span: &Id) { self.inner.exit(span); }

    fn current_span(&self) -> Current { self.inner.current_span() }
}

/// A boundary whose own span the filter rejects.
#[expect(
    clippy::cognitive_complexity,
    reason = "the span guard `tracing-attributes` generates for an INFO-level span is counted as \
              boundary complexity; this fixture's body fits the ceiling but the guard alone \
              scores 10"
)]
#[tracing::instrument(name = "filtered_boundary", skip_all, fields(
    {fields::TRANSITION_NAME} = "filtered_boundary",
    {fields::TRANSITION_OUTCOME} = tracing::field::Empty,
))]
fn filtered_boundary() {
    tracing::Span::current().record(fields::TRANSITION_OUTCOME, "child_outcome");
}

/// A filtered boundary's late record lands on an enabled ancestor span.
///
/// This is the hazard the rustdoc and ADR 0008 warn about, not a behaviour
/// the contract endorses: the boundary's own span is disabled, so
/// `Span::current()` resolves to the parent that declares the same field.
/// Keeping the observation explicit stops a later change from silently
/// altering what the documentation promises.
#[test]
fn a_filtered_boundary_records_into_an_enabled_ancestor() {
    let inner = CaptureSubscriber::new();
    let subscriber = FilteredSubscriber {
        inner: inner.clone(),
        rejected: "filtered_boundary",
    };
    tracing::subscriber::with_default(subscriber, || {
        tracing::callsite::rebuild_interest_cache();
        let parent = info_span!("enabled_parent", transition.outcome = tracing::field::Empty);
        let guard = parent.enter();
        filtered_boundary();
        drop(guard);
    });

    let spans = inner.into_spans();
    assert_eq!(
        spans.len(),
        1,
        "only the enabled parent span must be captured"
    );
    let parent = spans
        .first()
        .expect("the enabled parent span must be captured");
    assert_eq!(
        parent.name, "enabled_parent",
        "the captured span must be the enabled ancestor"
    );
    assert_eq!(
        parent
            .value("transition.outcome")
            .map(|value| value.text.as_str()),
        Some("child_outcome"),
        "the filtered boundary's record must land on the enabled ancestor, as documented"
    );
}
