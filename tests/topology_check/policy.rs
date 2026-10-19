//! The pre-macro topology policy over a Cargo metadata document.
//!
//! The policy is a pure function of the metadata JSON. It deliberately reads
//! *declarations*, not a resolved graph: format version 1 without `--deps`
//! carries each workspace package's dependency list and no external package
//! records. That is enough to reject a forbidden edge however it is spelled —
//! renamed, optional, build-kind, target-specific — and not enough to prove
//! anything about transitive dependencies, which the success message says.
//!
//! Every rejection names the repair as well as the violation. A contributor who
//! trips the check is usually adding the first macro package, and the message
//! must tell them that admission is an evidence-led roadmap change rather than
//! a Makefile edit.

use std::collections::BTreeMap;

use serde_json::{Map, Value};
use thiserror::Error;

/// The accepted format version of the metadata document.
const FORMAT_VERSION: u64 = 1;

/// The sole accepted workspace package.
pub(crate) const RUNTIME_PACKAGE: &str = "statelet";

/// The package forbidden before the macro baseline gate.
pub(crate) const FORBIDDEN_MACROS: &str = "statelet-macros";

/// The pre-macro policy, printed before the guard runs.
#[must_use]
pub(crate) const fn policy_message() -> &'static str {
    "Topology policy: statelet only; statelet-macros is forbidden before the macro baseline gate."
}

/// The success message, printed only after the guard exits zero.
#[must_use]
pub(crate) const fn success_message() -> &'static str {
    "Topology check passed: one runtime package; no forbidden declared edge."
}

/// The limitation the placeholder admits, printed with the success message.
#[must_use]
pub(crate) const fn limitation_message() -> &'static str {
    "This placeholder does not validate a future multi-crate graph or feature leakage."
}

/// One refused metadata document or workspace shape.
///
/// `Clone` is derived for the behavioural suite's [`rstest_bdd::Slot`], whose
/// `get` clones the stored verdict rather than lending it out.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub(crate) enum TopologyViolation {
    /// The document is not a metadata format this policy can read.
    #[error("metadata is not a JSON object")]
    NotAnObject,

    /// The document declares a format version the policy does not know.
    #[error(
        "metadata format version is {found}; this policy reads version {FORMAT_VERSION}. Repair: \
         regenerate metadata with `cargo metadata --no-deps --format-version 1`."
    )]
    UnsupportedFormatVersion { found: String },

    /// A field the policy reads is absent, misspelled, or of another type.
    #[error(
        "metadata field {field} is not {expected}. Repair: inspect the Cargo output; a renamed or \
         handwritten field is not readable by this policy."
    )]
    MalformedField {
        field: String,
        expected: &'static str,
    },

    /// The workspace does not hold exactly one member.
    #[error(
        "the workspace declares {found} members; this placeholder admits exactly one. Repair: \
         remove the extra member, or change this policy deliberately as part of an approved \
         workspace expansion."
    )]
    WorkspaceMemberCount { found: usize },

    /// The sole member is not named `statelet`.
    #[error(
        "the sole workspace member is a package named {found:?}, not {RUNTIME_PACKAGE:?}. Repair: \
         restore the runtime package as the only member, or revise this policy."
    )]
    UnexpectedRuntimePackage { found: String },

    /// No package record answers the sole member's identifier.
    #[error(
        "workspace member {member} has no package record. Repair: regenerate metadata; a dangling \
         member identifier was not produced by Cargo."
    )]
    DanglingMember { member: String },

    /// Two package records share one identifier.
    #[error(
        "two package records share the identifier {id}. Repair: regenerate metadata; a duplicated \
         identifier was not produced by Cargo."
    )]
    DuplicatePackageId { id: String },

    /// The runtime package declares no library target.
    #[error(
        "the {RUNTIME_PACKAGE} package declares no library target. Repair: restore the runtime \
         library, or revise this policy before removing it."
    )]
    MissingLibraryTarget,

    /// The runtime package carries a procedural-macro target of its own.
    #[error(
        "the {RUNTIME_PACKAGE} package declares a proc-macro target. Repair: the macro is \
         forbidden before the baseline gate; keep the macro out of the runtime package."
    )]
    RuntimePackageIsProcMacro,

    /// A declared dependency names a forbidden package.
    #[error(
        "the {RUNTIME_PACKAGE} package declares a {kind} dependency on {package:?} as \
         {declared:?}. Repair: remove the declaration; {FORBIDDEN_MACROS} is admitted only after \
         the macro baseline gate and a reviewed policy change."
    )]
    ForbiddenDependency {
        declared: String,
        package: String,
        kind: &'static str,
    },
}

/// Applies the pre-macro policy to one metadata document.
///
/// The checks are ordered from the document's envelope inwards: format, then
/// workspace membership, then packages, then targets and declarations. A later
/// check therefore always reads a field an earlier check has established, and a
/// document with two faults reports the outermost one, which is the repair a
/// contributor should make first.
///
/// # Errors
///
/// Returns the first [`TopologyViolation`] found. The caller prints it and exits
/// non-zero; nothing here prints or decides a process status.
///
/// # Examples
///
/// A metadata document for the single runtime package is accepted, while the
/// same document declaring a second member and its package record is refused:
///
/// ```rust,no_run
/// let runtime = serde_json::json!({
///     "name": "statelet",
///     "id": "path+file:///repo#statelet@0.1.0",
///     "targets": [{"name": "statelet", "kind": ["lib"], "crate_types": ["lib"]}],
///     "dependencies": [],
/// });
/// let accepted = serde_json::json!({
///     "packages": [runtime],
///     "workspace_members": ["path+file:///repo#statelet@0.1.0"],
///     "version": 1,
/// });
/// assert!(evaluate_metadata(&accepted).is_ok());
///
/// let expanded = serde_json::json!({
///     "packages": [
///         accepted["packages"][0].clone(),
///         {
///             "name": "statelet-extra",
///             "id": "path+file:///repo/extra#statelet-extra@0.1.0",
///             "targets": [
///                 {"name": "statelet_extra", "kind": ["lib"], "crate_types": ["lib"]}
///             ],
///             "dependencies": [],
///         },
///     ],
///     "workspace_members": [
///         "path+file:///repo#statelet@0.1.0",
///         "path+file:///repo/extra#statelet-extra@0.1.0",
///     ],
///     "version": 1,
/// });
/// assert_eq!(
///     evaluate_metadata(&expanded),
///     Err(TopologyViolation::WorkspaceMemberCount { found: 2 })
/// );
/// ```
pub(crate) fn evaluate_metadata(document: &Value) -> Result<(), TopologyViolation> {
    let root = document.as_object().ok_or(TopologyViolation::NotAnObject)?;
    require_format_version(root)?;
    let member_ids = workspace_member_ids(root)?;
    let packages = package_index(root)?;
    let runtime = sole_runtime_package(&member_ids, &packages)?;
    require_library_target(runtime)?;
    reject_forbidden_declarations(runtime)
}

/// Requires the format version this policy understands.
fn require_format_version(root: &Map<String, Value>) -> Result<(), TopologyViolation> {
    let version = root
        .get("version")
        .ok_or_else(|| malformed("version", "an integer"))?;
    match version.as_u64() {
        Some(FORMAT_VERSION) => Ok(()),
        Some(found) => Err(TopologyViolation::UnsupportedFormatVersion {
            found: found.to_string(),
        }),
        None => Err(malformed("version", "an integer")),
    }
}

/// Returns the workspace member identifiers, requiring a non-empty string list.
///
/// Membership is decided by this list alone, never by the package records: a
/// full-metadata document also carries external packages, and reading those as
/// members would reject every resolved document for a reason unrelated to the
/// policy.
fn workspace_member_ids(root: &Map<String, Value>) -> Result<Vec<String>, TopologyViolation> {
    let members = array_field(root, "workspace_members")?;
    let ids = members
        .iter()
        .map(|member| {
            member
                .as_str()
                .filter(|id| !id.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| malformed("workspace_members[]", "a non-empty string identifier"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if ids.is_empty() {
        return Err(malformed("workspace_members", "a non-empty array"));
    }
    Ok(ids)
}

/// Builds the package index, rejecting missing, empty, or duplicate identifiers.
///
/// `BTreeMap` rather than the JSON map: the index is keyed by identifier, not
/// by a document field, and its ordering makes a duplicate report deterministic.
fn package_index(root: &Map<String, Value>) -> Result<BTreeMap<String, &Value>, TopologyViolation> {
    let packages = array_field(root, "packages")?;
    let mut index = BTreeMap::new();
    for package in packages {
        let id = package
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| malformed("packages[].id", "a non-empty string identifier"))?;
        if index.insert(id.to_owned(), package).is_some() {
            return Err(TopologyViolation::DuplicatePackageId { id: id.to_owned() });
        }
    }
    Ok(index)
}

/// Resolves the sole member to its package record and requires its name.
fn sole_runtime_package<'a>(
    member_ids: &[String],
    packages: &BTreeMap<String, &'a Value>,
) -> Result<&'a Value, TopologyViolation> {
    let [member] = member_ids else {
        return Err(TopologyViolation::WorkspaceMemberCount {
            found: member_ids.len(),
        });
    };
    let package =
        packages
            .get(member)
            .copied()
            .ok_or_else(|| TopologyViolation::DanglingMember {
                member: member.clone(),
            })?;
    let name = package
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| malformed("packages[].name", "a non-empty string"))?;
    if name != RUNTIME_PACKAGE {
        return Err(TopologyViolation::UnexpectedRuntimePackage {
            found: name.to_owned(),
        });
    }
    Ok(package)
}

/// Requires a library target and forbids a proc-macro target on the member.
///
/// A proc-macro target in the runtime package would make the runtime crate the
/// macro crate under another name, which the pre-macro boundary forbids just as
/// firmly as a declared edge.
fn require_library_target(package: &Value) -> Result<(), TopologyViolation> {
    let targets = package
        .get("targets")
        .and_then(Value::as_array)
        .ok_or_else(|| malformed("packages[].targets", "an array"))?;
    if targets.is_empty() {
        return Err(malformed("packages[].targets", "a non-empty array"));
    }
    let kinds = targets
        .iter()
        .map(target_kinds)
        .collect::<Result<Vec<_>, _>>()?;
    let has_kind = |wanted: &str| kinds.iter().any(|kind| kind.contains(&wanted));
    if has_kind("proc-macro") {
        return Err(TopologyViolation::RuntimePackageIsProcMacro);
    }
    if !has_kind("lib") {
        return Err(TopologyViolation::MissingLibraryTarget);
    }
    Ok(())
}

/// Returns one target's kind list, requiring non-empty strings.
fn target_kinds(target: &Value) -> Result<Vec<&str>, TopologyViolation> {
    target
        .get("kind")
        .and_then(Value::as_array)
        .filter(|kinds| !kinds.is_empty())
        .ok_or_else(|| malformed("packages[].targets[].kind", "a non-empty array"))?
        .iter()
        .map(|kind| {
            kind.as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| malformed("packages[].targets[].kind[]", "a non-empty string"))
        })
        .collect()
}

/// Rejects every declared dependency whose package name is forbidden.
///
/// The `name` field is Cargo's package name, not the alias, so an alias cannot
/// hide the edge; a rename is reported beside it to make a disguised
/// declaration visible in the diagnostic. Kind, optionality, target condition,
/// and source are deliberately not consulted: a declaration is a declaration
/// even when a feature or platform would leave it inactive.
fn reject_forbidden_declarations(package: &Value) -> Result<(), TopologyViolation> {
    let dependencies = package
        .get("dependencies")
        .and_then(Value::as_array)
        .ok_or_else(|| malformed("packages[].dependencies", "an array"))?;
    for dependency in dependencies {
        let package_name = dependency
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .ok_or_else(|| malformed("packages[].dependencies[].name", "a non-empty string"))?;
        if package_name != FORBIDDEN_MACROS && package_name != RUNTIME_PACKAGE {
            continue;
        }
        let alias = dependency.get("rename").and_then(Value::as_str);
        return Err(TopologyViolation::ForbiddenDependency {
            declared: alias.map_or_else(|| package_name.to_owned(), str::to_owned),
            package: package_name.to_owned(),
            kind: dependency_kind(dependency),
        });
    }
    Ok(())
}

/// Names a declaration's kind for the diagnostic.
fn dependency_kind(dependency: &Value) -> &'static str {
    match dependency.get("kind").and_then(Value::as_str) {
        Some("build") => "build",
        Some("dev") => "development",
        Some(_) => "declared",
        None => "normal",
    }
}

/// Returns an array field, reporting an absent or mistyped value by name.
fn array_field<'a>(
    root: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Vec<Value>, TopologyViolation> {
    root.get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| malformed(field, "an array"))
}

/// Builds the malformed-field violation for one JSON path.
fn malformed(field: &str, expected: &'static str) -> TopologyViolation {
    TopologyViolation::MalformedField {
        field: field.to_owned(),
        expected,
    }
}
