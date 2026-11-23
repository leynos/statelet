//! Application-shaped fixtures used by the transition-field tests.
//!
//! Everything here looks like a downstream crate: project-owned enums
//! implementing [`statelet::StateName`], a project-local decision label, a
//! project-local error category, and boundaries instrumented with the plain
//! `#[tracing::instrument]` attribute. The suite compiles as an external crate
//! against the public API, so these fixtures prove the published surface is
//! usable without special support.
//!
//! The boundaries come in pairs that differ in exactly one respect — the field
//! naming form, or whether a late field is declared — so each behavioural
//! scenario can attribute its observation to that difference. The labels are
//! fixture data, not contract vocabulary.

use statelet::{StateName, fields};

/// The fixture application's continuation mode.
pub enum Mode {
    /// A normalization step.
    Normalize,
    /// A tight code-span step.
    TightCodeSpan,
}

impl StateName for Mode {
    fn state_name(&self) -> &'static str {
        match self {
            Self::Normalize => "Normalize",
            Self::TightCodeSpan => "TightCodeSpan",
        }
    }
}

/// The fixture application's decision for the handled input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Keep the current state.
    Stay,
    /// Move to the next state.
    Move,
}

impl Decision {
    /// The stable label of this decision.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stay => "stay",
            Self::Move => "move",
        }
    }
}

/// The fixture application's error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseFailure {
    /// The row had no terminating pipe.
    UnterminatedRow,
}

impl ParseFailure {
    /// The stable category of this failure.
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::UnterminatedRow => "unterminated_row",
        }
    }
}

/// A fallible boundary naming its fields through the exported constants.
///
/// Records the outcome on success and the error category on failure. Both late
/// fields are declared `tracing::field::Empty`, so the error path can fill its
/// field after the span exists.
#[tracing::instrument(skip_all, fields(
    {fields::TRANSITION_NAME} = "handle_line",
    {fields::TRANSITION_STATE_BEFORE} = mode.state_name(),
    {fields::TRANSITION_EVENT} = "table_row",
    {fields::TRANSITION_OUTCOME} = tracing::field::Empty,
    {fields::TRANSITION_ERROR} = tracing::field::Empty,
))]
pub fn handle_line(mode: &Mode, fails: bool) -> Result<Decision, ParseFailure> {
    if fails {
        let failure = ParseFailure::UnterminatedRow;
        tracing::Span::current().record(fields::TRANSITION_ERROR, failure.category());
        return Err(failure);
    }
    let decision = Decision::Stay;
    tracing::Span::current().record(fields::TRANSITION_OUTCOME, decision.label());
    Ok(decision)
}

/// The same boundary as [`handle_line`], naming its fields with literals.
///
/// Written out rather than parameterized: the two must differ only in how the
/// field names are spelled, so the form-equivalence test compares like with
/// like.
#[tracing::instrument(name = "handle_line", skip_all, fields(
    transition.name = "handle_line",
    transition.state.before = mode.state_name(),
    transition.event = "table_row",
    transition.outcome = tracing::field::Empty,
    transition.error = tracing::field::Empty,
))]
pub fn handle_line_with_literals(mode: &Mode, fails: bool) -> Result<Decision, ParseFailure> {
    if fails {
        let failure = ParseFailure::UnterminatedRow;
        tracing::Span::current().record(fields::TRANSITION_ERROR, failure.category());
        return Err(failure);
    }
    let decision = Decision::Stay;
    tracing::Span::current().record(fields::TRANSITION_OUTCOME, decision.label());
    Ok(decision)
}

/// An infallible boundary, which declares no error field.
#[expect(
    clippy::cognitive_complexity,
    reason = "the span guard `tracing-attributes` generates for an INFO-level span is counted as \
              boundary complexity; this fixture's body fits the ceiling but the guard alone \
              scores 10"
)]
#[tracing::instrument(skip_all, fields(
    {fields::TRANSITION_NAME} = "handle_fence",
    {fields::TRANSITION_STATE_BEFORE} = mode.state_name(),
    {fields::TRANSITION_EVENT} = "fence",
    {fields::TRANSITION_OUTCOME} = tracing::field::Empty,
))]
pub fn handle_fence(mode: &Mode) -> Decision {
    let decision = Decision::Move;
    tracing::Span::current().record(fields::TRANSITION_OUTCOME, decision.label());
    decision
}

/// A boundary that declares only the name and records an undeclared field.
///
/// This is the negative control for the declaration rule: its `record` call is
/// the same one [`handle_line`] makes, and `tracing` discards it because the
/// span never declared the field.
#[expect(
    clippy::cognitive_complexity,
    reason = "the span guard `tracing-attributes` generates for an INFO-level span is counted as \
              boundary complexity; this fixture's body fits the ceiling but the guard alone \
              scores 10"
)]
#[tracing::instrument(skip_all, fields({fields::TRANSITION_NAME} = "handle_undeclared"))]
pub fn handle_undeclared(outcome: &str) {
    tracing::Span::current().record(fields::TRANSITION_OUTCOME, outcome);
}

/// A fallible boundary that returns through `?` before recording its error.
///
/// The `record` call is unreachable when the guard fails, so the span never
/// learns about the error. This is the control for "absence never implies
/// success": the call really did fail.
#[tracing::instrument(skip_all, fields(
    {fields::TRANSITION_NAME} = "handle_early_return",
    {fields::TRANSITION_STATE_BEFORE} = mode.state_name(),
    {fields::TRANSITION_EVENT} = "table_row",
    {fields::TRANSITION_ERROR} = tracing::field::Empty,
))]
pub fn handle_early_return(mode: &Mode, fails: bool) -> Result<Decision, ParseFailure> {
    let decision = check(fails)?;
    tracing::Span::current().record(fields::TRANSITION_OUTCOME, decision.label());
    Ok(decision)
}

/// Returns the failure or the decision, so the boundary can use `?`.
const fn check(fails: bool) -> Result<Decision, ParseFailure> {
    if fails {
        return Err(ParseFailure::UnterminatedRow);
    }
    Ok(Decision::Stay)
}

/// The instrumented fixture boundary compiles and runs.
#[test]
fn the_instrumented_boundary_runs() {
    let mode = Mode::Normalize;
    let decision = handle_line(&mode, false)
        .expect("the fixture boundary must return its decision without an error");
    assert_eq!(
        decision.label(),
        "stay",
        "the fixture boundary must return its project-local decision"
    );
}
