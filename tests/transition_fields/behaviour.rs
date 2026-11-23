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
use rstest_bdd_macros::{ScenarioState, given, then, when};
use statelet::fields;

use super::{
    capture::{CaptureError, CapturedSpan, Via, capture},
    downstream::{
        Decision,
        Mode,
        ParseFailure,
        handle_early_return,
        handle_fence,
        handle_line,
        handle_undeclared,
    },
};

/// Which fixture boundary a scenario runs.
#[derive(Clone, Copy)]
enum Boundary {
    /// A fallible boundary naming its fields with the exported constants.
    Fallible,
    /// An infallible boundary.
    Infallible,
    /// A boundary that records a late field it never declared.
    Undeclared,
    /// A fallible boundary that returns through `?` before recording.
    EarlyReturn,
}

/// What a `Given` step selected for its `When` step to run.
#[derive(Clone)]
struct Plan {
    /// The fixture boundary to run.
    boundary: Boundary,
    /// The state label the boundary must observe.
    state: String,
    /// The event label the boundary must handle.
    event: String,
}

/// The world each scenario fills.
#[derive(Default, ScenarioState)]
pub(crate) struct TransitionWorld {
    /// The boundary and labels the `Given` step selected.
    plan: Slot<Plan>,
    /// The span the `When` step captured.
    span: Slot<CapturedSpan>,
    /// What the boundary returned.
    result: Slot<Result<Decision, ParseFailure>>,
}

/// The world each scenario owns.
#[fixture]
pub(crate) fn world() -> TransitionWorld {
    // Every slot starts empty; each scenario fills only the slots its steps
    // reach, so a missing step is reported rather than papered over.
    TransitionWorld::default()
}

/// Converts a capture failure into the scenario's error.
fn captured(result: Result<CapturedSpan, CaptureError>) -> Result<CapturedSpan, String> {
    result.map_err(|error| error.to_string())
}

/// The mode matching `label`, or the reason no fixture mode does.
fn mode_for(label: &str) -> Result<Mode, String> {
    match label {
        "Normalize" => Ok(Mode::Normalize),
        "TightCodeSpan" => Ok(Mode::TightCodeSpan),
        other => Err(format!("no fixture mode carries the label {other:?}")),
    }
}

/// Stores a `Given` step's plan in the world.
fn store_plan(world: &TransitionWorld, boundary: Boundary, state: &str, event: &str) {
    world.plan.set(Plan {
        boundary,
        state: state.to_owned(),
        event: event.to_owned(),
    });
}

/// The plan a preceding `Given` stored, or the reason there is none.
fn stored_plan(world: &TransitionWorld) -> Result<Plan, String> {
    world
        .plan
        .get()
        .ok_or_else(|| "the Given step did not select a boundary".to_owned())
}

/// The span a preceding `When` captured, or the reason there is none.
fn stored_span(world: &TransitionWorld) -> Result<CapturedSpan, String> {
    world
        .span
        .get()
        .ok_or_else(|| "the When step did not capture a span".to_owned())
}

/// The result a preceding `When` stored, or the reason there is none.
fn stored_result(world: &TransitionWorld) -> Result<Result<Decision, ParseFailure>, String> {
    world
        .result
        .get()
        .ok_or_else(|| "the When step did not record a return value".to_owned())
}

/// Requires the captured span to carry `transition.name` as a string.
///
/// Every absence check calls this first: a span that recorded nothing would
/// otherwise satisfy "does not record X" for every X.
fn require_name(span: &CapturedSpan) -> Result<&str, String> {
    match span.value(fields::TRANSITION_NAME) {
        Some(value) if value.via == Via::Str => Ok(&value.text),
        Some(_) => Err("transition.name was not recorded as a string".to_owned()),
        None => Err("the captured span does not record transition.name".to_owned()),
    }
}

/// Requires `field` to be recorded as the string `expected`.
fn require_string_value(span: &CapturedSpan, field: &str, expected: &str) -> Result<(), String> {
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
fn require_absent(span: &CapturedSpan, field: &str) -> Result<(), String> {
    require_name(span)?;
    span.value(field).map_or(Ok(()), |value| {
        Err(format!(
            "{field} must be absent, but was recorded as {:?}",
            value.text
        ))
    })
}

/// Runs the fallible fixture named by the plan.
fn run_fallible(
    plan: &Plan,
    fails: bool,
) -> Result<(CapturedSpan, Result<Decision, ParseFailure>), String> {
    let mode = mode_for(&plan.state)?;
    let mut returned = None;
    let span = captured(capture("handle_line", || {
        returned = Some(handle_line(&mode, fails));
    }))?;
    let result = returned.ok_or_else(|| "the fallible boundary did not run".to_owned())?;
    Ok((span, result))
}

/// A fallible boundary handling a named input.
#[given("a fallible boundary in state \"{state}\" handling a \"{event}\" input")]
fn fallible_boundary(world: &TransitionWorld, state: String, event: String) {
    store_plan(world, Boundary::Fallible, &state, &event);
}

/// An infallible boundary handling a named input.
#[given("an infallible boundary in state \"{state}\" handling a \"{event}\" input")]
fn infallible_boundary(world: &TransitionWorld, state: String, event: String) {
    store_plan(world, Boundary::Infallible, &state, &event);
}

/// A boundary that declares only its name.
#[given("a boundary that declares only the transition name")]
fn undeclared_boundary(world: &TransitionWorld) { store_plan(world, Boundary::Undeclared, "", ""); }

/// A fallible boundary that propagates errors with `?`.
#[given("a fallible boundary that propagates errors with the question-mark operator")]
fn early_return_boundary(world: &TransitionWorld) {
    store_plan(world, Boundary::EarlyReturn, "Normalize", "table_row");
}

/// Runs the selected boundary to success and keeps its span and return value.
#[when("the boundary returns the outcome \"{label}\"")]
fn returns_outcome(world: &TransitionWorld, label: String) -> Result<(), String> {
    let plan = stored_plan(world)?;
    match plan.boundary {
        Boundary::Fallible => {
            let (span, result) = run_fallible(&plan, false)?;
            match &result {
                Ok(decision) if decision.label() == label => {}
                Ok(decision) => {
                    return Err(format!(
                        "the boundary returned {:?}; the scenario expected {label:?}",
                        decision.label()
                    ));
                }
                Err(failure) => {
                    return Err(format!(
                        "the boundary failed with {:?}; the scenario expected {label:?}",
                        failure.category()
                    ));
                }
            }
            world.span.set(span);
            world.result.set(result);
            Ok(())
        }
        Boundary::Infallible => {
            let mode = mode_for(&plan.state)?;
            let mut returned = None;
            let span = captured(capture("handle_fence", || {
                returned = Some(handle_fence(&mode));
            }))?;
            let decision = returned.ok_or_else(|| "the boundary did not run".to_owned())?;
            if decision.label() != label {
                return Err(format!(
                    "the boundary returned {:?}; the scenario expected {label:?}",
                    decision.label()
                ));
            }
            world.span.set(span);
            world.result.set(Ok(decision));
            Ok(())
        }
        Boundary::Undeclared | Boundary::EarlyReturn => Err("this step runs a fallible or \
                                                             infallible boundary, not the \
                                                             selected control"
            .to_owned()),
    }
}

/// Runs the fallible boundary to failure and keeps its span and return value.
#[when("the boundary returns the error category \"{category}\"")]
fn returns_error(world: &TransitionWorld, category: String) -> Result<(), String> {
    let plan = stored_plan(world)?;
    let (span, result) = run_fallible(&plan, true)?;
    match &result {
        Err(failure) if failure.category() == category => {}
        Err(failure) => {
            return Err(format!(
                "the boundary failed with {:?}; the scenario expected {category:?}",
                failure.category()
            ));
        }
        Ok(decision) => {
            return Err(format!(
                "the boundary succeeded with {:?}; the scenario expected the error {category:?}",
                decision.label()
            ));
        }
    }
    world.span.set(span);
    world.result.set(result);
    Ok(())
}

/// Runs the undeclared-field control and keeps its span.
#[when("the boundary records the outcome \"{label}\"")]
fn records_undeclared_outcome(world: &TransitionWorld, label: String) -> Result<(), String> {
    let span = captured(capture("handle_undeclared", || {
        handle_undeclared(&label);
    }))?;
    world.span.set(span);
    Ok(())
}

/// Runs the early-return control and keeps its span and return value.
#[when("the boundary fails before recording its error category")]
fn fails_before_recording(world: &TransitionWorld) -> Result<(), String> {
    let plan = stored_plan(world)?;
    let mode = mode_for(&plan.state)?;
    let mut returned = None;
    let span = captured(capture("handle_early_return", || {
        returned = Some(handle_early_return(&mode, true));
    }))?;
    let result = returned.ok_or_else(|| "the early-return boundary did not run".to_owned())?;
    world.span.set(span);
    world.result.set(result);
    Ok(())
}

/// Requires the captured span to be named `name` and to record that name.
#[then("the boundary span records the transition name \"{name}\"")]
fn span_records_name(world: &TransitionWorld, name: String) -> Result<(), String> {
    let span = stored_span(world)?;
    if span.name != name {
        return Err(format!(
            "the captured span is named {:?}; the scenario expected {name:?}",
            span.name
        ));
    }
    require_string_value(&span, fields::TRANSITION_NAME, &name)
}

/// Requires the state, event, and outcome fields as strings.
#[then(
    "the span records state \"{state}\", event \"{event}\", and outcome \"{outcome}\" as strings"
)]
fn span_records_state_event_outcome(
    world: &TransitionWorld,
    state: String,
    event: String,
    outcome: String,
) -> Result<(), String> {
    let span = stored_span(world)?;
    require_name(&span)?;
    require_string_value(&span, fields::TRANSITION_STATE_BEFORE, &state)?;
    require_string_value(&span, fields::TRANSITION_EVENT, &event)?;
    require_string_value(&span, fields::TRANSITION_OUTCOME, &outcome)
}

/// Requires the error field to be absent from a non-empty span.
#[then("the span does not record a transition error")]
fn span_has_no_error(world: &TransitionWorld) -> Result<(), String> {
    let span = stored_span(world)?;
    require_absent(&span, fields::TRANSITION_ERROR)
}

/// Requires the error field as a string with the given category.
#[then("the boundary span records the transition error \"{category}\" as a string")]
fn span_records_error(world: &TransitionWorld, category: String) -> Result<(), String> {
    let span = stored_span(world)?;
    require_name(&span)?;
    require_string_value(&span, fields::TRANSITION_ERROR, &category)
}

/// Requires the fixture to have recorded no outcome.
#[then("this fixture records no transition outcome")]
fn span_has_no_outcome(world: &TransitionWorld) -> Result<(), String> {
    let span = stored_span(world)?;
    require_absent(&span, fields::TRANSITION_OUTCOME)
}

/// Requires the span to omit the error field from its declaration.
#[then("the span does not declare a transition error")]
fn span_does_not_declare_error(world: &TransitionWorld) -> Result<(), String> {
    let span = stored_span(world)?;
    require_name(&span)?;
    let plan = stored_plan(world)?;
    require_string_value(&span, fields::TRANSITION_EVENT, &plan.event)?;
    if span.declares(fields::TRANSITION_ERROR) {
        return Err(
            "the span declares transition.error, but this boundary is infallible".to_owned(),
        );
    }
    Ok(())
}

/// Requires a non-empty span that recorded no outcome.
#[then("the boundary span exists but does not record a transition outcome")]
fn span_exists_without_outcome(world: &TransitionWorld) -> Result<(), String> {
    let span = stored_span(world)?;
    require_absent(&span, fields::TRANSITION_OUTCOME)
}

/// Requires the boundary to have returned an error.
#[then("the boundary returns an error")]
fn boundary_returns_error(world: &TransitionWorld) -> Result<(), String> {
    stored_result(world)?.map_or(Ok(()), |decision| {
        Err(format!(
            "the boundary returned {:?}; the scenario expected an error",
            decision.label()
        ))
    })
}

/// Requires a non-empty span that recorded no error.
#[then("the boundary span exists but does not record a transition error")]
fn span_exists_without_error(world: &TransitionWorld) -> Result<(), String> {
    let span = stored_span(world)?;
    require_absent(&span, fields::TRANSITION_ERROR)
}
