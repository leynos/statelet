//! A test-only `tracing` subscriber that captures span fields verbatim.
//!
//! `tracing-subscriber` would format spans into text, and text cannot
//! distinguish a declared-but-unrecorded field from a field that was never
//! declared; `tracing-test` asserts on that formatted text. Both would hide
//! the property this suite exists to prove, so this module implements
//! [`Subscriber`] directly and keeps each field's name, text, and delivery
//! method.
//!
//! The subscriber is installed through [`tracing::subscriber::with_default`],
//! which scopes it to the calling thread, so tests run in parallel without
//! global state. [`capture`] rebuilds the interest cache inside that scope,
//! because a call site first reached on a thread with no subscriber can be
//! cached as `never` and would then record nothing.
//!
//! This is not the structured recorder proposed for roadmap task 6.3.1. It is
//! local to `tests/transition_fields/`, and a second suite that needs one
//! should promote it deliberately rather than import this copy.

use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use tracing::Subscriber;
use tracing_core::{
    Metadata,
    span::{Attributes, Current, Id, Record},
};

/// How a value reached the subscriber.
///
/// The contract requires string labels, so the suite asserts on this tag: a
/// `Display` value recorded with `%` arrives as [`Via::Debug`], and a label
/// rendered as `"stay"` is then visibly different from `stay`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// Delivered through [`tracing_core::field::Visit::record_str`].
    Str,
    /// Delivered through [`tracing_core::field::Visit::record_debug`].
    Debug,
}

/// One recorded field value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedValue {
    /// The recorded text.
    pub text: String,
    /// How `tracing` delivered the value.
    pub via: Via,
}

/// Everything the subscriber observed about one span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedSpan {
    /// The span's name, from its metadata.
    pub name: String,
    /// Every field the span declared, in metadata order.
    pub declared: Vec<String>,
    /// The values recorded on the span, keyed by field name.
    pub values: BTreeMap<String, CapturedValue>,
}

impl CapturedSpan {
    /// The recorded value for `field`, or `None` when it was not recorded.
    #[must_use]
    pub fn value(&self, field: &str) -> Option<&CapturedValue> { self.values.get(field) }

    /// Whether the span declared `field`.
    #[must_use]
    pub fn declares(&self, field: &str) -> bool {
        self.declared.iter().any(|declared| declared == field)
    }
}

/// Why [`capture`] could not return exactly one span.
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    /// The closure created no span at all.
    #[error("the closure created no span; expected one named {expected:?}")]
    NoSpan {
        /// The name `capture` was waiting for.
        expected: String,
    },
    /// The closure created more than one span.
    #[error("the closure created {count} spans; expected exactly one")]
    TooManySpans {
        /// How many spans were created.
        count: usize,
    },
    /// The one span had a different name.
    #[error("the closure created a span named {actual:?}; expected {expected:?}")]
    UnexpectedName {
        /// The name `capture` was waiting for.
        expected: String,
        /// The name the span actually carried.
        actual: String,
    },
}

/// One span's live state inside the subscriber.
struct SpanState {
    /// The span's metadata, kept for `current_span`.
    metadata: &'static Metadata<'static>,
    /// The declared field names, in metadata order.
    declared: Vec<String>,
    /// The values recorded so far.
    values: BTreeMap<String, CapturedValue>,
}

/// The mutable state one capture run accumulates.
#[derive(Default)]
struct Capture {
    /// Span state per identifier.
    spans: BTreeMap<u64, SpanState>,
    /// The stack of entered spans; the last entry is the current span.
    stack: Vec<Id>,
    /// The next identifier to hand out; zero is reserved by `tracing`.
    next_id: u64,
}

/// A subscriber that records spans and their field values in memory.
///
/// Cloning shares the same state, so [`capture`] can install one handle in the
/// scoped dispatcher and read through another after the closure returns.
#[derive(Clone)]
pub struct CaptureSubscriber {
    /// Shared state, so `&self` methods can mutate it.
    state: Arc<Mutex<Capture>>,
}

impl Default for CaptureSubscriber {
    fn default() -> Self { Self::new() }
}

impl CaptureSubscriber {
    /// Creates an empty subscriber.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Capture {
                next_id: 1,
                ..Capture::default()
            })),
        }
    }

    /// Locks the shared state, recovering a poisoned lock.
    ///
    /// A panicking assertion inside another test thread must not turn every
    /// later observation into a second panic that hides the first failure, so
    /// the poison marker is deliberately ignored.
    fn state(&self) -> MutexGuard<'_, Capture> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Reports every span the run created, in creation order.
    fn into_spans(self) -> Vec<CapturedSpan> {
        let state = self.state();
        state
            .spans
            .values()
            .map(|span| CapturedSpan {
                name: span.metadata.name().to_owned(),
                declared: span.declared.clone(),
                values: span.values.clone(),
            })
            .collect()
    }
}

/// Collects a span's field values into a map.
struct FieldVisitor<'a> {
    /// The map the visitor fills.
    values: &'a mut BTreeMap<String, CapturedValue>,
}

impl tracing_core::field::Visit for FieldVisitor<'_> {
    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        self.values.insert(
            field.name().to_owned(),
            CapturedValue {
                text: value.to_owned(),
                via: Via::Str,
            },
        );
    }

    fn record_debug(&mut self, field: &tracing_core::Field, value: &dyn fmt::Debug) {
        self.values.insert(
            field.name().to_owned(),
            CapturedValue {
                text: format!("{value:?}"),
                via: Via::Debug,
            },
        );
    }
}

impl Subscriber for CaptureSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool { true }

    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::TRACE)
    }

    fn new_span(&self, attributes: &Attributes<'_>) -> Id {
        let metadata = attributes.metadata();
        let mut values = BTreeMap::new();
        attributes.record(&mut FieldVisitor {
            values: &mut values,
        });

        let mut state = self.state();
        let id = state.next_id;
        state.next_id += 1;
        let declared = metadata
            .fields()
            .iter()
            .map(|field| field.name().to_owned())
            .collect();
        state.spans.insert(
            id,
            SpanState {
                metadata: metadata as &'static Metadata<'static>,
                declared,
                values,
            },
        );
        Id::from_u64(id)
    }

    fn record(&self, span: &Id, values: &Record<'_>) {
        let mut state = self.state();
        let Some(entry) = state.spans.get_mut(&span.into_u64()) else {
            return;
        };
        values.record(&mut FieldVisitor {
            values: &mut entry.values,
        });
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, _event: &tracing::Event<'_>) {}

    fn enter(&self, span: &Id) { self.state().stack.push(span.clone()); }

    fn exit(&self, span: &Id) {
        let mut state = self.state();
        if let Some(position) = state.stack.iter().position(|entered| entered == span) {
            state.stack.remove(position);
        }
    }

    fn current_span(&self) -> Current {
        let state = self.state();
        state
            .stack
            .last()
            .and_then(|id| {
                let span = state.spans.get(&id.into_u64())?;
                Some(Current::new(id.clone(), span.metadata))
            })
            .unwrap_or_else(Current::none)
    }
}

/// Runs `body` under a fresh capture subscriber and returns its one span.
///
/// # Errors
///
/// Returns [`CaptureError::NoSpan`] when the closure created no span,
/// [`CaptureError::TooManySpans`] when it created more than one, and
/// [`CaptureError::UnexpectedName`] when the single span's name differs from
/// `expected`. An absence assertion needs the first and last: without them a
/// test asserting a field is missing passes when nothing was captured at all.
pub fn capture<F>(expected: &str, body: F) -> Result<CapturedSpan, CaptureError>
where
    F: FnOnce(),
{
    let subscriber = CaptureSubscriber::new();
    tracing::subscriber::with_default(subscriber.clone(), || {
        // A call site first reached on a thread with no subscriber can be
        // cached as `never`, so rebuild the cache inside the scoped dispatcher.
        tracing::callsite::rebuild_interest_cache();
        body();
    });

    let spans = subscriber.into_spans();
    match spans.as_slice() {
        [] => Err(CaptureError::NoSpan {
            expected: expected.to_owned(),
        }),
        [span] if span.name == expected => Ok(span.clone()),
        [span] => Err(CaptureError::UnexpectedName {
            expected: expected.to_owned(),
            actual: span.name.clone(),
        }),
        _ => Err(CaptureError::TooManySpans { count: spans.len() }),
    }
}
