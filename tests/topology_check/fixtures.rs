//! Metadata documents the topology policy suite reads.
//!
//! The accepted document is Cargo's own format-version-1 shape with one runtime
//! package. Cases derive from it by changing exactly the field under test, so a
//! rejection is attributable to that field rather than to a fixture that was
//! never valid. Text rather than a programmatic builder keeps Cargo's field
//! names visible beside the policy that reads them.
//!
//! The fixture is checked against the live checkout by
//! `repository_obeys_topology` in `tests/topology_check.rs`, which runs the same
//! policy over `cargo metadata`'s own output for this workspace. A drift
//! between the fixture's shape and the shape the command actually produces
//! therefore fails the guard even when every in-memory case still passes.

use serde_json::Value;

/// The template's marker for the dependency list the accepted document holds.
const DEPENDENCY_MARKER: &str = "##DEPENDENCIES##";

/// One dependency declaration, as `cargo metadata` prints it.
///
/// `name` is the *package* name even when `rename` aliases it, which is why the
/// policy reads `name` rather than the alias. The remaining fields describe how
/// the declaration is admitted, so a case can move the same forbidden package
/// between kinds, optionality, and target conditions without rewriting it.
fn external_dev_dependency(name: &str, requirement: &str) -> String {
    format!(
        r#"{{
          "name": "{name}",
          "source": "registry+https://github.com/rust-lang/crates.io-index",
          "req": "{requirement}",
          "kind": "dev",
          "rename": null,
          "optional": false,
          "uses_default_features": true,
          "features": [],
          "target": null,
          "registry": null
        }}"#
    )
}

/// The accepted document, with external test proc macros permitted.
///
/// The dependency list names the real development dependencies that provide
/// procedural macros. Their admission is the case that distinguishes this
/// policy from "no proc macros anywhere": the policy constrains Statelet's
/// workspace topology, not what external test tooling is built from.
pub(crate) fn accepted_metadata() -> String {
    let dependencies = [
        external_dev_dependency("rstest", "^0.27.0"),
        external_dev_dependency("rstest-bdd", "^0.6.0"),
        external_dev_dependency("rstest-bdd-macros", "^0.6.0"),
    ]
    .join(",\n        ");
    TEMPLATE.replace(DEPENDENCY_MARKER, &dependencies)
}

/// The accepted document's shape: one member, one package, two library targets.
///
/// The runtime target declares two kinds, `["lib", "rlib"]`, because Cargo
/// genuinely emits more than one kind for some targets. That shape is what makes
/// a kind-order transformation observable: with one kind per target, reversing
/// a kind list changes nothing, and an evaluator that read only the first kind
/// would satisfy every property vacuously. The second target keeps the test
/// kind, so target order is exercised independently of kind order.
///
/// Each placeholder is replaced once by [`accepted_metadata`]; the template is
/// a private constant so no case can silently hold marked-up text that happens
/// to parse as a different document.
const TEMPLATE: &str = r#"{
  "packages": [
    {
      "name": "statelet",
      "version": "0.1.0",
      "id": "path+file:///repo#statelet@0.1.0",
      "manifest_path": "/repo/Cargo.toml",
      "metadata": {},
      "targets": [
        {"name": "statelet", "kind": ["lib", "rlib"], "crate_types": ["lib"]},
        {"name": "topology_check", "kind": ["test"], "crate_types": ["bin"]}
      ],
      "dependencies": [
        ##DEPENDENCIES##
      ]
    }
  ],
  "workspace_members": ["path+file:///repo#statelet@0.1.0"],
  "workspace_default_members": ["path+file:///repo#statelet@0.1.0"],
  "resolve": null,
  "workspace_root": "/repo",
  "target_directory": "/repo/target",
  "version": 1
}"#;

/// Parses a metadata document, reporting a syntax failure as a message.
///
/// This is the acquisition boundary: malformed JSON never reaches the policy,
/// which is why its case asserts a parse failure rather than a violation.
///
/// # Errors
///
/// Returns the JSON syntax error with its position, so a fixture case says what
/// it broke rather than only that the document was refused.
pub(crate) fn parse(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|error| error.to_string())
}

/// Parses the accepted document and then applies one JSON-Pointer edit to it.
///
/// A case that rewrites a nested field needs the parsed document, and a
/// textual `replace` on the template would silently stop matching after an
/// unrelated reformat. The pointer reports a typo in the case as a failure
/// instead of leaving the accepted document untouched.
///
/// # Errors
///
/// Returns the parse failure or an edit failure, because a control that did not
/// apply must not be read as a policy verdict.
pub(crate) fn accepted_with(pointer: &str, value: Value) -> Result<Value, String> {
    let mut document = parse(&accepted_metadata())?;
    let slot = document
        .pointer_mut(pointer)
        .ok_or_else(|| format!("the accepted fixture has no value at {pointer}"))?;
    *slot = value;
    Ok(document)
}

/// Parses the accepted document, then appends one value to a named array.
///
/// A duplicate package identifier cannot be produced by [`accepted_with`],
/// because the pointer addresses a whole array: replacing `/packages` would
/// need a second copy of the entire record. Appending is also closer to the
/// failure being described, since a duplicated identifier arrives as one extra
/// record rather than as a rewritten document.
///
/// # Errors
///
/// Returns the parse failure or the absent-array failure.
pub(crate) fn accepted_extended(pointer: &str, value: Value) -> Result<Value, String> {
    let mut document = parse(&accepted_metadata())?;
    let array = document
        .pointer_mut(pointer)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("the accepted fixture has no array at {pointer}"))?;
    array.push(value);
    Ok(document)
}

/// Parses the accepted document, then removes one object field.
///
/// Assigning `null` tests a wrong *type*, not an absent field, and the policy
/// distinguishes them in its message. A required field is absent in the way a
/// handwritten or filtered document omits it, so the control has to delete the
/// key rather than shadow its value.
///
/// # Errors
///
/// Returns the parse failure or the absent-field failure.
pub(crate) fn accepted_removed(pointer: &str) -> Result<Value, String> {
    let mut document = parse(&accepted_metadata())?;
    let (parent, field) = pointer
        .rsplit_once('/')
        .ok_or_else(|| format!("{pointer} does not name an object field"))?;
    let object = document
        .pointer_mut(parent)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| format!("the accepted fixture has no object at {parent}"))?;
    if object.remove(field).is_none() {
        return Err(format!(
            "the accepted fixture has no field {field} at {parent}"
        ));
    }
    Ok(document)
}
