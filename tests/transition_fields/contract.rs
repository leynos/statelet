//! Contract tests for the published transition tracing field names.
//!
//! The five names are public operational API: dashboards and log queries may
//! depend on them, and the compiler cannot detect a rename. These tests pin
//! each exported constant to a literal written here rather than read back from
//! the constant, check the dot-separated grammar the prefix rule depends on,
//! and prove the runtime manifest stays free of `tracing`. The manifest is
//! embedded with `include_str!` so the test performs no filesystem access.

use pretty_assertions::assert_eq;
use rstest::rstest;

/// The repository manifest, embedded so the test never touches the filesystem.
const MANIFEST: &str = include_str!("../../Cargo.toml");

/// The five contracted names, used only by the pairwise namespace check.
const NAMES: [&str; 5] = [
    statelet::fields::TRANSITION_NAME,
    statelet::fields::TRANSITION_STATE_BEFORE,
    statelet::fields::TRANSITION_EVENT,
    statelet::fields::TRANSITION_OUTCOME,
    statelet::fields::TRANSITION_ERROR,
];

/// Whether `name` is a lowercase, dot-separated path below `transition.`.
///
/// Every segment must be non-empty and consist of ASCII lowercase letters and
/// underscores. The predicate is written independently of the exported
/// constants, so a malformed constant cannot validate itself.
#[must_use]
fn is_well_formed(name: &str) -> bool {
    name.starts_with("transition.")
        && name.split('.').all(|segment| {
            !segment.is_empty()
                && segment
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character == '_')
        })
}

/// Whether `shorter` is a dot-separated prefix of `longer`.
#[must_use]
fn is_dot_prefix(shorter: &str, longer: &str) -> bool {
    longer
        .strip_prefix(shorter)
        .is_some_and(|rest| rest.starts_with('.'))
}

/// Every exported constant carries its independently stated field name.
#[rstest]
#[case::name(statelet::fields::TRANSITION_NAME, "transition.name")]
#[case::state_before(statelet::fields::TRANSITION_STATE_BEFORE, "transition.state.before")]
#[case::event(statelet::fields::TRANSITION_EVENT, "transition.event")]
#[case::outcome(statelet::fields::TRANSITION_OUTCOME, "transition.outcome")]
#[case::error(statelet::fields::TRANSITION_ERROR, "transition.error")]
fn names_are_exact(#[case] actual: &str, #[case] expected: &str) {
    assert_eq!(
        actual, expected,
        "the exported constant must carry the contracted field name"
    );
}

/// Every exported constant satisfies the contracted name grammar.
#[rstest]
#[case::name(statelet::fields::TRANSITION_NAME)]
#[case::state_before(statelet::fields::TRANSITION_STATE_BEFORE)]
#[case::event(statelet::fields::TRANSITION_EVENT)]
#[case::outcome(statelet::fields::TRANSITION_OUTCOME)]
#[case::error(statelet::fields::TRANSITION_ERROR)]
fn exported_names_are_well_formed(#[case] name: &str) {
    assert!(
        is_well_formed(name),
        "{name:?} must satisfy the contracted name grammar"
    );
}

/// The grammar predicate rejects shapes the contract forbids.
///
/// These cases establish the predicate is not vacuously true, so the accepted
/// cases above mean something.
#[rstest]
#[case::capitalized_prefix("Transition.name")]
#[case::empty_segment("transition..event")]
#[case::trailing_dot("transition.state.before.")]
#[case::missing_prefix("state.before")]
fn the_grammar_rejects_malformed_names(#[case] name: &str) {
    assert!(
        !is_well_formed(name),
        "{name:?} must fail the contracted name grammar"
    );
}

/// No contracted name is a dot-prefix of another.
///
/// The rule keeps every name a leaf field: a name that was also a namespace
/// would map ambiguously in object-versus-leaf document stores.
#[test]
fn no_name_is_a_namespace_of_another() {
    for (index, first) in NAMES.iter().enumerate() {
        for second in NAMES.iter().skip(index + 1) {
            assert!(
                !is_dot_prefix(first, second) && !is_dot_prefix(second, first),
                "{first:?} and {second:?} must be siblings, not a field and a namespace"
            );
        }
    }
}

/// The namespace predicate rejects a genuine prefix pair.
///
/// The five contracted names are all siblings, so the pairwise check alone
/// would pass for a predicate that always returned `false`. This case supplies
/// the pair the plan names and requires the predicate to see the prefix.
#[test]
fn the_namespace_rule_rejects_a_prefix_pair() {
    assert!(
        is_dot_prefix("transition.state", "transition.state.before"),
        "a dot-prefix must be recognized as a namespace of its extension"
    );
    assert!(
        !is_dot_prefix("transition.state.before", "transition.state"),
        "the longer name must not be a prefix of the shorter one"
    );
    assert!(
        !is_dot_prefix("transition.state", "transition.stateful"),
        "a shared prefix without a dot separator is not a namespace"
    );
}

/// The runtime manifest declares no `tracing` dependency and no feature table.
#[test]
fn runtime_has_no_tracing_dependency() -> Result<(), String> {
    let manifest: toml::Value = toml::from_str(MANIFEST)
        .map_err(|error| format!("Cargo.toml must parse as TOML: {error}"))?;
    let table = manifest
        .as_table()
        .ok_or_else(|| "Cargo.toml must be a TOML table".to_owned())?;

    let dependencies = table
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "Cargo.toml must declare a [dependencies] table".to_owned())?;
    for crate_name in ["tracing", "tracing-core"] {
        if dependencies.contains_key(crate_name) {
            return Err(format!("the runtime crate must not depend on {crate_name}"));
        }
    }
    if table.contains_key("features") {
        return Err("this task must not introduce a [features] table".to_owned());
    }

    let dev_dependencies = table
        .get("dev-dependencies")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "Cargo.toml must declare a [dev-dependencies] table".to_owned())?;
    if !dev_dependencies.contains_key("tracing") {
        return Err(
            "the test must read the manifest that declares its own tracing dev-dependency"
                .to_owned(),
        );
    }
    Ok(())
}
