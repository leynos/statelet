//! Step definitions for the transition-field feature file.
//!
//! The steps read and write the world declared in `behaviour.rs`, which also
//! owns the pure helpers they share, so the world module stays a readable
//! statement of what a scenario knows and this module a statement of what it
//! does.

use rstest_bdd_macros::{given, then, when};
use statelet::fields;

use super::{
    behaviour::{
        Boundary,
        Event,
        TransitionWorld,
        captured,
        parse_mode,
        require_absent,
        require_name,
        require_string_value,
        run_fallible,
        store_plan,
        stored_plan,
        stored_result,
        stored_span,
    },
    capture::capture,
    downstream::{Mode, handle_early_return, handle_fence, handle_undeclared},
};

/// A fallible boundary handling a named input.
#[given("a fallible boundary in state \"{state}\" handling a \"{event}\" input")]
fn fallible_boundary(world: &TransitionWorld, state: String, event: String) -> Result<(), String> {
    let observed_mode = parse_mode(&state)?;
    let handled_event = Event::from_label(&event)?;
    store_plan(world, Boundary::Fallible, observed_mode, handled_event);
    Ok(())
}

/// An infallible boundary handling a named input.
#[given("an infallible boundary in state \"{state}\" handling a \"{event}\" input")]
fn infallible_boundary(
    world: &TransitionWorld,
    state: String,
    event: String,
) -> Result<(), String> {
    let observed_mode = parse_mode(&state)?;
    let handled_event = Event::from_label(&event)?;
    store_plan(world, Boundary::Infallible, observed_mode, handled_event);
    Ok(())
}

/// A boundary that declares only its name.
#[given("a boundary that declares only the transition name")]
fn undeclared_boundary(world: &TransitionWorld) {
    store_plan(
        world,
        Boundary::Undeclared,
        Mode::Normalize,
        Event::TableRow,
    );
}

/// A fallible boundary that propagates errors with `?`.
#[given("a fallible boundary that propagates errors with the question-mark operator")]
fn early_return_boundary(world: &TransitionWorld) {
    store_plan(
        world,
        Boundary::EarlyReturn,
        Mode::Normalize,
        Event::TableRow,
    );
}

/// Runs the selected boundary to success and keeps its span and return value.
#[when("the boundary returns the outcome \"{label}\"")]
fn returns_outcome(world: &TransitionWorld, label: String) -> Result<(), String> {
    let plan = stored_plan(world)?;
    match plan.boundary {
        Boundary::Fallible => {
            let (span, result) = run_fallible(plan, false)?;
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
            let mode = plan.mode;
            let mut returned = None;
            let span = captured(capture("handle_fence", || {
                returned = Some(handle_fence(mode));
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
    let (span, result) = run_fallible(plan, true)?;
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
    let mode = plan.mode;
    let mut returned = None;
    let span = captured(capture("handle_early_return", || {
        returned = Some(handle_early_return(mode, true));
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
    require_string_value(&span, fields::TRANSITION_EVENT, plan.event.label())?;
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
