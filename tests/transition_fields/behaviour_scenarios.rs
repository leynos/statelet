//! Scenario bindings for the transition-field feature file.
//!
//! `rstest-bdd` resolves a scenario's `world` parameter as an implicit fixture
//! that must be in scope in the test module, so the type and its fixture are
//! imported from the step module rather than duplicated here.

use rstest_bdd_macros::scenario;

use super::behaviour::{TransitionWorld, world};

/// A fallible success records its outcome.
#[scenario(
    path = "tests/features/transition_fields.feature",
    name = "A fallible transition that succeeds records its outcome"
)]
fn fallible_success_records_outcome(world: TransitionWorld) { let _ = world; }

/// A fallible failure records its error category.
#[scenario(
    path = "tests/features/transition_fields.feature",
    name = "A fallible transition that fails records its error category"
)]
fn fallible_failure_records_error(world: TransitionWorld) { let _ = world; }

/// An infallible boundary declares no error field.
#[scenario(
    path = "tests/features/transition_fields.feature",
    name = "An infallible transition does not declare an error field"
)]
fn infallible_declares_no_error(world: TransitionWorld) { let _ = world; }

/// An undeclared late field is lost.
#[scenario(
    path = "tests/features/transition_fields.feature",
    name = "An undeclared late field is lost"
)]
fn undeclared_late_field_is_lost(world: TransitionWorld) { let _ = world; }

/// An error propagated before recording leaves no error field.
#[scenario(
    path = "tests/features/transition_fields.feature",
    name = "An error propagated before recording leaves no error field"
)]
fn early_return_leaves_no_error(world: TransitionWorld) { let _ = world; }
