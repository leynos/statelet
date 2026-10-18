//! Partition cases for the topology policy.
//!
//! Every rejection case differs from the accepted document in exactly one
//! policy-relevant field, and its expected violation names that field's repair.
//! The accepted document is a case too, so a table that has drifted toward
//! refusal cannot pass by rejecting everything.
//!
//! Cases build their documents through `accepted_with`, which parses the
//! accepted text and applies a JSON-Pointer edit. A pointer typo fails the case
//! instead of leaving a valid document behind, which is the failure mode a
//! textual `replace` would have. The case table holds the *edited* document and
//! the expected verdict; the document is parsed once per row by the runner.

use googletest::prelude::*;
use rstest::rstest;
use serde_json::{Value, json};

use super::{
    fixtures::{accepted_metadata, accepted_with, parse},
    policy::{TopologyViolation, evaluate_metadata},
};

/// Builds a case document from the accepted document plus one edit.
///
/// The edit is applied eagerly here, so a pointer that no longer resolves is
/// reported as that case's failure rather than as a policy verdict. Returning
/// the edited document and the expected violation keeps the table a literal.
fn edited(
    pointer: &str,
    value: Value,
    expected: TopologyViolation,
) -> (Result<Value, String>, TopologyViolation) {
    (accepted_with(pointer, value), expected)
}

/// The accepted document yields the accepted verdict.
#[test]
fn accepted_metadata_is_accepted() -> Result<(), String> {
    let document = parse(&accepted_metadata())?;
    assert_that!(evaluate_metadata(&document), ok(eq(&())));
    Ok(())
}

/// Every rejection differs from the accepted document in one field.
#[rstest]
#[case::extra_workspace_member(
    edited(
        "/workspace_members",
        json!(["/packages/0", "/packages/1"]),
        TopologyViolation::WorkspaceMemberCount { found: 2 },
    )
)]
#[case::duplicate_workspace_member(
    edited(
        "/workspace_members",
        json!(["/packages/0", "/packages/0"]),
        TopologyViolation::WorkspaceMemberCount { found: 2 },
    )
)]
#[case::dangling_workspace_member(
    edited(
        "/workspace_members",
        json!(["/packages/1"]),
        TopologyViolation::DanglingMember { member: "/packages/1".to_owned() },
    )
)]
#[case::empty_workspace_members(
    edited(
        "/workspace_members",
        json!([]),
        TopologyViolation::MalformedField {
            field: "workspace_members".to_owned(),
            expected: "a non-empty array",
        },
    )
)]
#[case::missing_workspace_members(
    edited(
        "/workspace_members",
        Value::Null,
        TopologyViolation::MalformedField {
            field: "workspace_members".to_owned(),
            expected: "an array",
        },
    )
)]
#[case::runtime_package_renamed(
    edited(
        "/packages/0/name",
        json!("statelet-renamed"),
        TopologyViolation::UnexpectedRuntimePackage { found: "statelet-renamed".to_owned() },
    )
)]
#[case::runtime_package_without_library(
    edited(
        "/packages/0/targets",
        json!([{"name": "statelet", "kind": ["bin"], "crate_types": ["bin"]}]),
        TopologyViolation::MissingLibraryTarget,
    )
)]
#[case::runtime_package_is_a_proc_macro(
    edited(
        "/packages/0/targets",
        json!([{"name": "statelet", "kind": ["proc-macro"], "crate_types": ["proc-macro"]}]),
        TopologyViolation::RuntimePackageIsProcMacro,
    )
)]
#[case::forbidden_registry_dependency(
    edited(
        "/packages/0/dependencies",
        json!([{"name": "statelet-macros", "kind": null, "rename": null}]),
        TopologyViolation::ForbiddenDependency {
            declared: "statelet-macros".to_owned(),
            package: "statelet-macros".to_owned(),
            kind: "normal",
        },
    )
)]
#[case::forbidden_build_dependency(
    edited(
        "/packages/0/dependencies",
        json!([{"name": "statelet-macros", "kind": "build", "rename": null}]),
        TopologyViolation::ForbiddenDependency {
            declared: "statelet-macros".to_owned(),
            package: "statelet-macros".to_owned(),
            kind: "build",
        },
    )
)]
#[case::forbidden_renamed_optional_dependency(
    edited(
        "/packages/0/dependencies",
        json!([{"name": "statelet-macros", "kind": "dev", "rename": "macros", "optional": true}]),
        TopologyViolation::ForbiddenDependency {
            declared: "macros".to_owned(),
            package: "statelet-macros".to_owned(),
            kind: "development",
        },
    )
)]
#[case::self_dependency(
    edited(
        "/packages/0/dependencies",
        json!([{"name": "statelet", "kind": null, "rename": null}]),
        TopologyViolation::ForbiddenDependency {
            declared: "statelet".to_owned(),
            package: "statelet".to_owned(),
            kind: "normal",
        },
    )
)]
#[case::unsupported_format_version(
    edited(
        "/version",
        json!(2),
        TopologyViolation::UnsupportedFormatVersion { found: "2".to_owned() },
    )
)]
#[case::version_is_not_an_integer(
    edited(
        "/version",
        json!("1"),
        TopologyViolation::MalformedField { field: "version".to_owned(), expected: "an integer" },
    )
)]
#[case::dependencies_is_not_an_array(
    edited(
        "/packages/0/dependencies",
        json!({}),
        TopologyViolation::MalformedField {
            field: "packages[].dependencies".to_owned(),
            expected: "an array",
        },
    )
)]
fn rejected_documents_report_their_violation(
    #[case] (parsed, expected): (Result<Value, String>, TopologyViolation),
) -> Result<(), String> {
    let document = parsed?;
    match evaluate_metadata(&document) {
        Err(violation) if violation == expected => Ok(()),
        other => Err(format!("expected {expected:?}, got {other:?}")),
    }
}

/// A document that is not an object at all is named as such.
#[test]
fn a_non_object_document_is_refused() {
    assert_eq!(
        evaluate_metadata(&json!([])),
        Err(TopologyViolation::NotAnObject)
    );
}

/// Fields the policy does not read are additive extensions, not violations.
#[test]
fn unrelated_fields_are_ignored() -> Result<(), String> {
    let document = accepted_with("/packages/0/metadata", json!({"docs": "extension"}))?;
    evaluate_metadata(&document)
        .map_err(|violation| format!("the extension was refused: {violation}"))
}
