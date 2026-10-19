//! Graph transformations the invariance properties apply.
//!
//! Each function here performs one edit that must leave the policy's verdict
//! alone: appending package records or dependency declarations, reversing
//! package, declaration, or target lists, renaming package identifiers
//! consistently, and re-aliasing a forbidden declaration. None of them decides
//! a verdict; the property asserts the verdict against the transformed
//! document.
//!
//! Each returns whether it found the structure it edits. A transformation that
//! silently did nothing would leave a property comparing the same document
//! with itself, so the caller asserts the returned flag beside the invariant it
//! protects.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::policy::{FORBIDDEN_MACROS, RUNTIME_PACKAGE};

/// One generated package record, with an identifier derived from its position.
///
/// Positional identifiers are unique within one generated document, and the
/// `generated-` prefix keeps them clear of the runtime package's identifier.
/// Uniqueness matters: the policy refuses duplicate identifiers, and a
/// generator that produced them would turn every property into a test of that
/// refusal instead of the invariance under study.
pub(super) fn extra_package_record(position: usize, name: &str) -> Value {
    json!({
        "name": name,
        "version": "0.1.0",
        "id": format!("path+file:///repo#generated-{position}@0.1.0"),
        "manifest_path": format!("/repo/generated-{position}/Cargo.toml"),
        "targets": [{"name": name, "kind": ["lib"], "crate_types": ["lib"]}],
        "dependencies": [],
    })
}

/// Appends one package record per generated name.
pub(super) fn append_package_records(document: &mut Value, names: &[String]) -> bool {
    let Some(packages) = document.get_mut("packages").and_then(Value::as_array_mut) else {
        return false;
    };
    for (position, name) in names.iter().enumerate() {
        packages.push(extra_package_record(position, name));
    }
    true
}

/// Appends one member identifier per generated name, in the same order.
pub(super) fn append_member_ids(document: &mut Value, names: &[String]) -> bool {
    let Some(members) = document
        .get_mut("workspace_members")
        .and_then(Value::as_array_mut)
    else {
        return false;
    };
    for (position, _name) in names.iter().enumerate() {
        members.push(Value::String(format!(
            "path+file:///repo#generated-{position}@0.1.0"
        )));
    }
    true
}

/// The sole workspace member's identifier, as the document declares it.
fn runtime_member_id(document: &Value) -> Option<&str> {
    document
        .get("workspace_members")
        .and_then(Value::as_array)
        .and_then(|members| members.first())
        .and_then(Value::as_str)
}

/// The runtime package record, found by its member identifier.
///
/// A positional lookup would transform whichever package the list happens to
/// start with, which is a different package as soon as the list is reversed.
fn runtime_package_mut(document: &mut Value) -> Option<&mut Value> {
    let member = runtime_member_id(document)?.to_owned();
    document
        .get_mut("packages")
        .and_then(Value::as_array_mut)?
        .iter_mut()
        .find(|package| package.get("id").and_then(Value::as_str) == Some(member.as_str()))
}

/// Applies one edit to the runtime package's declaration list.
///
/// Both declaration transformations need the same lookup and the same
/// found-or-not contract, and differ only in the edit. Sharing the lookup keeps
/// the "which package is the runtime package" rule in one place; a second copy
/// is how a positional lookup came back before.
fn edit_declarations(document: &mut Value, edit: impl FnOnce(&mut Vec<Value>)) -> bool {
    let Some(declarations) = runtime_package_mut(document)
        .and_then(|package| package.get_mut("dependencies"))
        .and_then(Value::as_array_mut)
    else {
        return false;
    };
    edit(declarations);
    true
}

/// Appends declarations to the runtime package's declaration list.
pub(super) fn append_declarations(document: &mut Value, declarations: Vec<Value>) -> bool {
    edit_declarations(document, |existing| existing.extend(declarations))
}

/// Reverses the top-level package list.
pub(super) fn reverse_packages(document: &mut Value) -> bool {
    let Some(packages) = document.get_mut("packages").and_then(Value::as_array_mut) else {
        return false;
    };
    packages.reverse();
    true
}

/// Reverses the runtime package's dependency declarations.
pub(super) fn reverse_dependencies(document: &mut Value) -> bool {
    edit_declarations(document, |declarations| declarations.reverse())
}

/// Reverses every package's target list and each target's kind list.
///
/// The policy reads target kinds as a set, so neither order carries meaning; a
/// policy that read the first kind, or the first target, would answer
/// differently here.
pub(super) fn reverse_targets(document: &mut Value) -> bool {
    let Some(packages) = document.get_mut("packages").and_then(Value::as_array_mut) else {
        return false;
    };
    let mut applied = false;
    for package in packages {
        let Some(targets) = package.get_mut("targets").and_then(Value::as_array_mut) else {
            continue;
        };
        targets.reverse();
        for target in targets {
            if let Some(kinds) = target.get_mut("kind").and_then(Value::as_array_mut) {
                kinds.reverse();
            }
        }
        applied = true;
    }
    applied
}

/// Replaces every package identifier, and every reference to it, with an
/// opaque identifier.
///
/// Identity stays consistent: the member lists are rewritten through the same
/// map, so the runtime package is still the sole member afterwards. A policy
/// that keyed on identifier text rather than on reference identity would flip
/// its verdict.
pub(super) fn replace_identifiers_with_opaque(document: &mut Value) -> bool {
    let Some(packages) = document.get_mut("packages").and_then(Value::as_array_mut) else {
        return false;
    };
    let mut renames = BTreeMap::new();
    for (position, package) in packages.iter_mut().enumerate() {
        let Some(identifier) = package.get_mut("id") else {
            continue;
        };
        let Some(old) = identifier.as_str().map(str::to_owned) else {
            continue;
        };
        let opaque = format!("opaque-{position}");
        *identifier = Value::String(opaque.clone());
        renames.insert(old, opaque);
    }
    if renames.is_empty() {
        return false;
    }
    rename_member_lists(document, &renames);
    true
}

/// Rewrites member identifiers in both member lists through `renames`.
fn rename_member_lists(document: &mut Value, renames: &BTreeMap<String, String>) {
    for field in ["workspace_members", "workspace_default_members"] {
        let Some(members) = document.get_mut(field).and_then(Value::as_array_mut) else {
            continue;
        };
        for member in members {
            let replacement = member
                .as_str()
                .and_then(|identifier| renames.get(identifier).cloned());
            if let Some(opaque) = replacement {
                *member = Value::String(opaque);
            }
        }
    }
}

/// Rewrites the alias of every declaration naming a forbidden package.
///
/// Every other field of the declaration is untouched, so an alias that
/// admitted it would mean the policy read `rename` instead of `name`.
pub(super) fn alias_forbidden_declarations(document: &mut Value, alias: &str) -> bool {
    let Some(dependencies) = runtime_package_mut(document)
        .and_then(|package| package.get_mut("dependencies"))
        .and_then(Value::as_array_mut)
    else {
        return false;
    };
    let mut applied = false;
    for dependency in dependencies {
        let name = dependency.get("name").and_then(Value::as_str);
        if name == Some(RUNTIME_PACKAGE) || name == Some(FORBIDDEN_MACROS) {
            dependency["rename"] = Value::String(alias.to_owned());
            applied = true;
        }
    }
    applied
}
