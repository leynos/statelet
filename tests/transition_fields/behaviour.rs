//! Behavioural scenarios for the transition tracing fields.
//!
//! `tests/features/transition_fields.feature` states what a reviewer of the
//! roadmap item would ask: does an ordinary `#[tracing::instrument(fields(...))]`
//! boundary record the contracted fields, does an infallible boundary leave
//! the error field undeclared, and what happens when a late field is not
//! declared or an error returns before it is recorded?
//!
//! The steps call the same fixtures the unit tests use, under the same capture
//! subscriber. Every absence assertion first requires `transition.name` in the
//! captured span, so a scenario that captured nothing fails rather than passing
//! because the field it checks is also missing.

use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::ScenarioState;
use statelet::fields;

use super::{
    capture::{CaptureError, CapturedSpan, Via, capture},
    downstream::{Decision, Mode, ParseFailure, handle_line},
};

/// Which fixture boundary a scenario runs.
#[derive(Clone, Copy)]
pub(crate) enum Boundary {
    /// A fallible boundary naming its fields with the exported constants.
    Fallible,
    /// An infallible boundary.
    Infallible,
    /// A boundary that records a late field it never declared.
    Undeclared,
    /// A fallible boundary that returns through `?` before recording.
    EarlyReturn,
}

/// The fixture event a boundary handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Event {
    /// A Markdown table row.
    TableRow,
    /// A fenced code marker.
    Fence,
}

impl Event {
    /// The label the fixture records for this event.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::TableRow => "table_row",
            Self::Fence => "fence",
        }
    }

    /// The event carrying `label`, or the reason no fixture event does.
    pub(crate) fn from_label(label: &str) -> Result<Self, String> {
        match label {
            "table_row" => Ok(Self::TableRow),
            "fence" => Ok(Self::Fence),
            other => Err(format!("no fixture event carries the label {other:?}")),
        }
    }
}

/// What a `Given` step selected for its `When` step to run.
#[derive(Clone, Copy)]
pub(crate) struct Plan {
    /// The fixture boundary to run.
    pub(crate) boundary: Boundary,
    /// The state the boundary must observe.
    pub(crate) mode: Mode,
    /// The event the boundary must handle.
    pub(crate) event: Event,
}

/// The world each scenario fills.
#[derive(Default, ScenarioState)]
pub(crate) struct TransitionWorld {
    /// The boundary and labels the `Given` step selected.
    pub(crate) plan: Slot<Plan>,
    /// The span the `When` step captured.
    pub(crate) span: Slot<CapturedSpan>,
    /// What the boundary returned.
    pub(crate) result: Slot<Result<Decision, ParseFailure>>,
}

/// The world each scenario owns.
#[fixture]
pub(crate) fn world() -> TransitionWorld {
    // Every slot starts empty; each scenario fills only the slots its steps
    // reach, so a missing step is reported rather than papered over.
    TransitionWorld::default()
}

/// Converts a capture failure into the scenario's error.
pub(crate) fn captured(result: Result<CapturedSpan, CaptureError>) -> Result<CapturedSpan, String> {
    result.map_err(|error| error.to_string())
}

/// The mode named by `label`, or the reason no fixture mode does.
pub(crate) fn parse_mode(label: &str) -> Result<Mode, String> {
    match label {
        "Normalize" => Ok(Mode::Normalize),
        "TightCodeSpan" => Ok(Mode::TightCodeSpan),
        other => Err(format!("no fixture mode carries the label {other:?}")),
    }
}

/// Stores a `Given` step's plan in the world.
pub(crate) fn store_plan(world: &TransitionWorld, boundary: Boundary, mode: Mode, event: Event) {
    world.plan.set(Plan {
        boundary,
        mode,
        event,
    });
}

/// The plan a preceding `Given` stored, or the reason there is none.
pub(crate) fn stored_plan(world: &TransitionWorld) -> Result<Plan, String> {
    world
        .plan
        .get()
        .ok_or_else(|| "the Given step did not select a boundary".to_owned())
}

/// The span a preceding `When` captured, or the reason there is none.
pub(crate) fn stored_span(world: &TransitionWorld) -> Result<CapturedSpan, String> {
    world
        .span
        .get()
        .ok_or_else(|| "the When step did not capture a span".to_owned())
}

/// The result a preceding `When` stored, or the reason there is none.
pub(crate) fn stored_result(
    world: &TransitionWorld,
) -> Result<Result<Decision, ParseFailure>, String> {
    world
        .result
        .get()
        .ok_or_else(|| "the When step did not record a return value".to_owned())
}

/// Requires the captured span to carry `transition.name` as a string.
///
/// Every absence check calls this first: a span that recorded nothing would
/// otherwise satisfy "does not record X" for every X.
pub(crate) fn require_name(span: &CapturedSpan) -> Result<&str, String> {
    match span.value(fields::TRANSITION_NAME) {
        Some(value) if value.via == Via::Str => Ok(&value.text),
        Some(_) => Err("transition.name was not recorded as a string".to_owned()),
        None => Err("the captured span does not record transition.name".to_owned()),
    }
}

/// Requires `field` to be recorded as the string `expected`.
pub(crate) fn require_string_value(
    span: &CapturedSpan,
    field: &str,
    expected: &str,
) -> Result<(), String> {
    match span.value(field) {
        Some(value) if value.via == Via::Str && value.text == expected => Ok(()),
        Some(value) => Err(format!(
            "{field} was recorded as {:?} via {:?}; expected the string {expected:?}",
            value.text, value.via
        )),
        None => Err(format!("{field} was not recorded; expected {expected:?}")),
    }
}

/// Requires `field` to be absent from a span that is otherwise non-empty.
pub(crate) fn require_absent(span: &CapturedSpan, field: &str) -> Result<(), String> {
    require_name(span)?;
    span.value(field).map_or(Ok(()), |value| {
        Err(format!(
            "{field} must be absent, but was recorded as {:?}",
            value.text
        ))
    })
}

/// Runs the fallible fixture named by the plan.
pub(crate) fn run_fallible(
    plan: Plan,
    fails: bool,
) -> Result<(CapturedSpan, Result<Decision, ParseFailure>), String> {
    let mode = plan.mode;
    let mut returned = None;
    let span = captured(capture("handle_line", || {
        returned = Some(handle_line(mode, fails));
    }))?;
    let result = returned.ok_or_else(|| "the fallible boundary did not run".to_owned())?;
    Ok((span, result))
}
