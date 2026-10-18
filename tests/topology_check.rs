//! Dependency-topology guard for the pre-macro single-crate boundary.
//!
//! Roadmap task 1.2.1 needs a check that the workspace is still the `statelet`
//! runtime package alone. The check reads Cargo's own metadata document and
//! applies a closed declaration policy to it. This entrypoint holds the guard
//! the Make target selects and the shared fixtures the policy, property, and
//! behavioural suites read, so the policy implementation exists once.
//!
//! The policy is deliberately narrow. It can see declared dependencies without
//! resolving them, which is enough to forbid a `statelet-macros` edge that is
//! renamed, optional, or target-specific, and not enough to prove anything
//! about a resolved graph. The success message says so; a future multi-crate
//! policy must replace this one deliberately rather than inherit its approval.

#[path = "topology_check/behaviour.rs"]
mod behaviour;
#[path = "topology_check/claim_properties.rs"]
mod claim_properties;
#[path = "topology_check/fixtures.rs"]
mod fixtures;
#[path = "topology_check/metadata.rs"]
mod metadata;
#[path = "topology_check/policy.rs"]
mod policy;
#[path = "topology_check/policy_cases.rs"]
mod policy_cases;

use policy::{evaluate_metadata, limitation_message, policy_message, success_message};

/// The guard `make topology-check` runs against the real checkout.
///
/// Its test name is the `-E` filter in the Makefile, so renaming it without
/// updating that filter selects zero tests. The target's `--no-tests fail`
/// turns that into a failure instead of a silent pass.
///
/// # Errors
///
/// Returns the acquisition or policy failure as a message, because the test
/// body is the place a failure becomes a diagnostic for a contributor.
#[test]
fn repository_obeys_topology() -> Result<(), String> {
    let inputs = metadata::process_inputs().map_err(|error| error.to_string())?;
    let document = metadata::read_metadata(&inputs.cargo, &inputs.manifest)
        .map_err(|error| error.to_string())?;
    evaluate_metadata(&document).map_err(|violation| violation.to_string())
}

/// The three stable lines a successful run prints, in order.
#[test]
fn success_messages_name_the_policy_and_its_limit() {
    assert_eq!(
        [policy_message(), success_message(), limitation_message()],
        [
            "Topology policy: statelet only; statelet-macros is forbidden before the macro \
             baseline gate.",
            "Topology check passed: one runtime package; no forbidden declared edge.",
            "This placeholder does not validate a future multi-crate graph or feature leakage.",
        ]
    );
}
