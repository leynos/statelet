//! Step definitions for the transition-field feature file.
//!
//! The steps read and write the world declared in `behaviour.rs`, which also
//! owns the pure helpers they share, so the world module stays a readable
//! statement of what a scenario knows and this module a statement of what it
//! does.

use pretty_assertions::Comparison;
use rstest_bdd_macros::{StepArgs, given, then, when};
use statelet::fields;

use super::{
    behaviour::{
        Boundary,
        Event,
        TransitionWorld,
        captured,
        expected_values,
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

/// The state and event labels a boundary `Given` step supplies.
///
/// Grouping them keeps the step signature to one value and names the pair the
/// plan stores, rather than passing two interchangeable strings.
#[derive(StepArgs)]
struct BoundaryLabels {
    /// The state the boundary observes.
    state: String,
    /// The event the boundary handles.
    event: String,
}

/// The state, event, and outcome labels an outcome `Then` step supplies.
#[derive(StepArgs)]
struct OutcomeLabels {
    /// The state the boundary observed.
    state: String,
    /// The event the boundary handled.
    event: String,
    /// The outcome the scenario expects.
    outcome: String,
}

/// Parses a `Given` step's labels and stores the plan for `boundary`.
fn select_boundary(
    world: &TransitionWorld,
    boundary: Boundary,
    state: &str,
    event: &str,
) -> Result<(), String> {
    let mode = parse_mode(state)?;
    let handled = Event::from_label(event)?;
    store_plan(world, boundary, mode, handled);
    Ok(())
}

/// A fallible boundary handling a named input.
#[given("a fallible boundary in state \"{state}\" handling a \"{event}\" input")]
fn fallible_boundary(
    #[step_args] labels: BoundaryLabels,
    world: &TransitionWorld,
) -> Result<(), String> {
    select_boundary(world, Boundary::Fallible, &labels.state, &labels.event)
}

/// An infallible boundary handling a named input.
#[given("an infallible boundary in state \"{state}\" handling a \"{event}\" input")]
fn infallible_boundary(
    #[step_args] labels: BoundaryLabels,
    world: &TransitionWorld,
) -> Result<(), String> {
    select_boundary(world, Boundary::Infallible, &labels.state, &labels.event)
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
            world
                .expected
                .set(expected_values(plan, "handle_line", Some(&label), None));
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
            world
                .expected
                .set(expected_values(plan, "handle_fence", Some(&label), None));
            world.span.set(span);
            world.result.set(Ok(decision));
            Ok(())
        }
        Boundary::Undeclared | Boundary::EarlyReturn => Err(concat!(
            "this step runs a fallible or infallible boundary, ",
            "not the selected control"
        )
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
    world
        .expected
        .set(expected_values(plan, "handle_line", None, Some(&category)));
    world.span.set(span);
    world.result.set(result);
    Ok(())
}

/// Runs the undeclared-field control and keeps its span.
#[when("the boundary records the outcome \"{label}\"")]
fn records_undeclared_outcome(world: &TransitionWorld, label: String) -> Result<(), String> {
    let plan = stored_plan(world)?;
    let span = captured(capture("handle_undeclared", || {
        handle_undeclared(&label);
    }))?;
    // The recording call is the same one the declared fixtures make, but this
    // span never declared the field, so the value is absent from the map.
    world
        .expected
        .set(expected_values(plan, "handle_undeclared", None, None));
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
    // The failure path returns before recording, so neither the error nor the
    // success outcome is in the map.
    world
        .expected
        .set(expected_values(plan, "handle_early_return", None, None));
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
    #[step_args] labels: OutcomeLabels,
    world: &TransitionWorld,
) -> Result<(), String> {
    let span = stored_span(world)?;
    require_name(&span)?;
    require_string_value(&span, fields::TRANSITION_STATE_BEFORE, &labels.state)?;
    require_string_value(&span, fields::TRANSITION_EVENT, &labels.event)?;
    require_string_value(&span, fields::TRANSITION_OUTCOME, &labels.outcome)
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

/// Requires the captured field map to equal the scenario's full expectation.
///
/// This is the whole-map comparison the plan's verification method requires:
/// keys, values, and visitor tags must match exactly, so an extra
/// `transition.*` value or a `Debug`-delivered label fails even when the
/// individually checked fields are right.
#[then("the span records exactly the expected fields")]
fn span_records_exactly(world: &TransitionWorld) -> Result<(), String> {
    let span = stored_span(world)?;
    require_name(&span)?;
    let expected = world
        .expected
        .get()
        .ok_or_else(|| "the When step did not record its expectation".to_owned())?;
    if span.values == expected {
        return Ok(());
    }
    Err(format!(
        "the captured fields differ from the scenario's expectation:\n{}",
        Comparison::new(&expected, &span.values)
    ))
}

/// Requires a non-empty span that recorded no error.
#[then("the boundary span exists but does not record a transition error")]
fn span_exists_without_error(world: &TransitionWorld) -> Result<(), String> {
    let span = stored_span(world)?;
    require_absent(&span, fields::TRANSITION_ERROR)
}
