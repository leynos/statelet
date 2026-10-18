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

/// The accepted document's shape: one member, one package, one library target.
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
        {"name": "statelet", "kind": ["lib"], "crate_types": ["lib"]},
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
