//! Application-shaped fixtures used by the transition-field tests.
//!
//! Everything here looks like a downstream crate: a project-owned mode enum
//! implementing [`statelet::StateName`], a project-local decision label, and a
//! boundary instrumented with the plain `#[tracing::instrument]` attribute.
//! The suite compiles as an external crate against the public API, so these
//! fixtures prove the published surface is usable without special support.

use statelet::{StateName, fields};

/// The fixture application's continuation mode.
enum Mode {
    /// A normalization step.
    Normalize,
}

impl StateName for Mode {
    fn state_name(&self) -> &'static str {
        match self {
            Self::Normalize => "Normalize",
        }
    }
}

/// The fixture application's decision for the handled input.
enum Decision {
    /// Keep the current state.
    Stay,
}

impl Decision {
    /// The stable label of this decision.
    const fn label(&self) -> &'static str {
        match self {
            Self::Stay => "stay",
        }
    }
}

/// A transition boundary instrumented with the exported field constants.
#[expect(
    clippy::cognitive_complexity,
    reason = "the guard `tracing-attributes` 0.1.31 generates for an INFO-level span is counted \
              as boundary complexity none of this code wrote"
)]
#[tracing::instrument(skip_all, fields(
    {fields::TRANSITION_NAME} = "handle_line",
    {fields::TRANSITION_STATE_BEFORE} = mode.state_name(),
    {fields::TRANSITION_EVENT} = "table_row",
    {fields::TRANSITION_OUTCOME} = tracing::field::Empty,
    {fields::TRANSITION_ERROR} = tracing::field::Empty,
))]
fn handle_line(mode: &Mode) -> Decision {
    let _ = mode;
    Decision::Stay
}

/// The instrumented fixture boundary compiles and runs.
#[test]
fn the_instrumented_boundary_runs() {
    let mode = Mode::Normalize;
    assert_eq!(
        handle_line(&mode).label(),
        "stay",
        "the fixture boundary must return its project-local decision"
    );
}
