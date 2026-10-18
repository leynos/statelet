//! Behavioural scenarios for the topology policy.
//!
//! `tests/features/topology_check.feature` states the three behaviours the
//! policy exists for, in the words a reviewer of the roadmap item would use:
//! the runtime package alone passes, an inactive renamed macro edge is still
//! refused, and a second workspace member needs a deliberate policy change.
//!
//! The steps call the same [`evaluate_metadata`] the partition cases and the
//! Make target call. There is no second policy here: the scenarios are a
//! readable statement of the rule, and the verdict they assert is the verdict
//! the guard produces. The world carries the evaluated document as a
//! [`Slot`], because a `Given` may build one of three documents and the `Then`
//! step must not be able to run without one.

use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, given, scenario, then, when};
use serde_json::Value;

use super::{
    fixtures::{accepted_metadata, accepted_with, parse},
    policy::{TopologyViolation, evaluate_metadata},
};

/// The document under test and the verdict it produced.
///
/// The verdict is a slot rather than a field read back through the policy: a
/// scenario that never reached its `When` step would otherwise evaluate a
/// document it never built, and the `Then` step would pass for the wrong
/// reason.
#[derive(Default, ScenarioState)]
struct TopologyWorld {
    document: Slot<Value>,
    verdict: Slot<Result<(), TopologyViolation>>,
}

/// The world each scenario owns.
#[fixture]
fn world() -> TopologyWorld {
    // Every slot starts empty; each scenario fills only the slots its steps
    // reach, so a missing step is reported rather than papered over.
    TopologyWorld::default()
}

/// Stores a parsed document in the world.
///
/// Parsing happens in the `Given` step rather than here, so a broken fixture is
/// reported against the step that built it rather than against a helper the
/// scenario never named. The fallible signature keeps that report an error
/// return instead of a panic inside the scenario.
fn store_document(world: &TopologyWorld, parsed: Result<Value, String>) -> Result<(), String> {
    let document = parsed?;
    world.document.set(document);
    Ok(())
}

/// The document a preceding `Given` stored, or the reason there is none.
///
/// A `Then` step that ran without its `Given` is a scenario bug, and this
/// message names it rather than letting the step assert against nothing.
fn stored_document(world: &TopologyWorld) -> Result<Value, String> {
    world
        .document
        .get()
        .ok_or_else(|| "the Given step did not store a document".to_owned())
}

/// The verdict a preceding `When` step stored, or the reason there is none.
fn stored_verdict(world: &TopologyWorld) -> Result<Result<(), TopologyViolation>, String> {
    world
        .verdict
        .get()
        .ok_or_else(|| "the When step did not record a verdict".to_owned())
}

/// The accepted single-package document.
#[given("metadata for the single statelet runtime package")]
fn single_runtime_package(world: &TopologyWorld) -> Result<(), String> {
    store_document(world, parse(&accepted_metadata()))
}

/// The accepted document plus an inactive, renamed macro declaration.
#[given("runtime metadata declaring an optional renamed statelet-macros dependency")]
fn renamed_macro_edge(world: &TopologyWorld) -> Result<(), String> {
    store_document(
        world,
        accepted_with(
            "/packages/0/dependencies",
            serde_json::json!([
                {"name": "statelet-macros", "kind": "dev", "rename": "macros", "optional": true}
            ]),
        ),
    )
}

/// The accepted document plus a second member and its package record.
#[given("metadata with an additional workspace member")]
fn additional_member(world: &TopologyWorld) -> Result<(), String> {
    store_document(
        world,
        accepted_with(
            "/workspace_members",
            serde_json::json!([
                "path+file:///repo#statelet@0.1.0",
                "path+file:///repo#statelet-extra@0.1.0"
            ]),
        ),
    )
}

/// Runs the policy over the stored document and keeps its verdict.
#[when("the topology policy is checked")]
fn check_policy(world: &TopologyWorld) -> Result<(), String> {
    let document = stored_document(world)?;
    world.verdict.set(evaluate_metadata(&document));
    Ok(())
}

/// Requires the stored verdict to be an acceptance.
#[then("the topology is accepted")]
fn topology_accepted(world: &TopologyWorld) -> Result<(), String> {
    let message = match stored_verdict(world)? {
        Ok(()) => return Ok(()),
        Err(violation) => violation.to_string(),
    };
    Err(format!("the topology was refused: {message}"))
}

/// What one refusal scenario expects the stored verdict to say.
///
/// The four fields are separate because a scenario that matched the wrong
/// partition can still mention the right words: `partition` proves the policy
/// chose the branch the `Given` built, and `repair` proves the message tells a
/// contributor the repair that branch needs.
struct RefusalExpectation {
    /// The message when the policy accepted the document the `Given` built.
    accepted: &'static str,
    /// Whether the refusal came from the partition the `Given` built.
    partition: fn(&TopologyViolation) -> bool,
    /// What that partition is called in a diagnosis.
    partition_name: &'static str,
    /// The repair the refusal must name.
    repair: &'static str,
}

/// Requires the stored verdict to refuse as `expected` describes.
///
/// # Errors
///
/// Returns the expectation's own description when the stored verdict accepted,
/// when it refused through another partition, or when its message does not name
/// the expected repair.
fn require_refusal(world: &TopologyWorld, expected: &RefusalExpectation) -> Result<(), String> {
    let violation = match stored_verdict(world)? {
        Ok(()) => return Err(expected.accepted.to_owned()),
        Err(violation) => violation,
    };
    if !(expected.partition)(&violation) {
        return Err(format!(
            "the refusal named {violation:?} rather than the {}",
            expected.partition_name
        ));
    }
    let message = violation.to_string();
    if !message.contains(expected.repair) {
        return Err(format!(
            "the refusal did not name the {}: {message}",
            expected.repair
        ));
    }
    Ok(())
}

/// What the renamed-macro scenario expects from the stored verdict.
const MACRO_GATE: RefusalExpectation = RefusalExpectation {
    accepted: "the renamed macro declaration was accepted",
    partition: |violation| matches!(violation, TopologyViolation::ForbiddenDependency { .. }),
    partition_name: "forbidden declaration",
    repair: "macro baseline gate",
};

/// What the extra-member scenario expects from the stored verdict.
const WORKSPACE_EXPANSION: RefusalExpectation = RefusalExpectation {
    accepted: "the additional workspace member was accepted",
    partition: |violation| matches!(violation, TopologyViolation::WorkspaceMemberCount { .. }),
    partition_name: "member count",
    repair: "workspace expansion",
};

/// Requires a refusal that names the macro baseline gate as the repair.
#[then("the topology is rejected with the macro baseline gate explanation")]
fn macro_gate_explanation(world: &TopologyWorld) -> Result<(), String> {
    require_refusal(world, &MACRO_GATE)
}

/// Requires a refusal that names the workspace expansion as the repair.
#[then("the topology is rejected with a workspace expansion explanation")]
fn workspace_expansion_explanation(world: &TopologyWorld) -> Result<(), String> {
    require_refusal(world, &WORKSPACE_EXPANSION)
}

/// The initial runtime package passes.
#[scenario(
    path = "tests/features/topology_check.feature",
    name = "The initial runtime package passes"
)]
fn initial_runtime_package_passes(world: TopologyWorld) { let _ = world; }

/// An inactive renamed macro edge is still forbidden.
#[scenario(
    path = "tests/features/topology_check.feature",
    name = "An inactive renamed macro edge is still forbidden"
)]
fn inactive_renamed_macro_edge_is_forbidden(world: TopologyWorld) { let _ = world; }

/// A second workspace package needs a policy change.
#[scenario(
    path = "tests/features/topology_check.feature",
    name = "A second workspace package needs a policy change"
)]
fn second_workspace_package_needs_a_policy_change(world: TopologyWorld) { let _ = world; }
