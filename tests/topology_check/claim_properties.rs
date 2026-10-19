//! Invariance properties for the topology policy.
//!
//! `policy_cases.rs` names one edit per partition and asks for the exact
//! violation that edit should produce. What a case table cannot state is the
//! rule that must survive *any* spelling of the same graph: Cargo is free to
//! order packages, dependencies and targets as it likes, to name packages with
//! opaque identifiers, and to alias a forbidden declaration, and none of those
//! may turn a refusal into an acceptance. These properties state that directly
//! over generated documents.
//!
//! The generators are deliberately unfiltered. A generator that only produced
//! documents already admitting a forbidden edge would make "reordering never
//! accepts" true for the wrong reason, so the accepted fixture is generated in
//! its own right, extra packages are drawn from zero to eight, and every
//! property below asserts its own non-vacuity beside the equality it states.
//!
//! Each property compares the verdict on a transformed document against the
//! verdict on the document it was built from. The oracle is therefore the same
//! question asked twice, not a second copy of the policy: a transformation that
//! changed the answer would say the policy is reading something other than the
//! declarations it claims to read. The transformations themselves live in
//! `transformations.rs`, and each reports whether it found the structure it
//! edits, so a silent no-op cannot pass as invariance.

use proptest::prelude::*;
use serde_json::{Value, json};

use super::{
    fixtures::{accepted_metadata, parse},
    policy::{FORBIDDEN_MACROS, RUNTIME_PACKAGE, evaluate_metadata},
    transformations::{
        alias_forbidden_declarations,
        append_declarations,
        append_member_ids,
        append_package_records,
        replace_identifiers_with_opaque,
        reverse_dependencies,
        reverse_kind_lists,
        reverse_packages,
        reverse_targets,
    },
};

/// The greatest number of extra packages or declarations a case generates.
///
/// Zero and several are both inside the range, and the range is bounded so a
/// generated case stays a document a person could read. The bound is stated,
/// not incidental: these properties check robustness over plausible Cargo
/// documents, not every graph the format could express.
const MAX_EXTRAS: usize = 8;

/// The accepted document, parsed once per generated case.
///
/// The fixture is a compile-time constant, so a parse failure is a defect in
/// the fixture rather than a property violation. Asserting it here keeps it out
/// of every generator: threading a `Result` through the property bodies would
/// either reject every case or need the fallible construction `proptest!`'s
/// infallible body cannot express. The scoped expectation is visible beside the
/// failure it guards instead of silencing the lint repository-wide.
#[expect(
    clippy::expect_used,
    reason = "The accepted fixture is constant text; failing loudly here is how a broken fixture \
              is told apart from a policy verdict."
)]
fn accepted_document() -> Value {
    parse(&accepted_metadata()).expect("the accepted metadata fixture is valid JSON")
}

/// A short name Cargo could have produced for an extra package.
///
/// The ASCII branch is drawn from the character set a crate name uses. The
/// Unicode branch is short on purpose: a policy that indexes a name by byte
/// offset rather than by character boundary would panic or slice mid-character,
/// and the fixture refuses to hide that behind long strings. The mandatory
/// suffix keeps every name distinct from the runtime package and from the
/// forbidden macro package.
fn extra_package_name() -> impl Strategy<Value = String> {
    prop_oneof![
        (
            "[a-z][a-z0-9-]{0,6}",
            prop::sample::select(&["-extra", "-util"][..]),
        )
            .prop_map(|(stem, suffix)| format!("{stem}{suffix}")),
        prop::sample::select(&["café-extra", "λ-util", "münchen-extra", "日本-util"][..])
            .prop_map(str::to_owned),
    ]
}

/// One generated alias, ASCII or short Unicode.
fn alias_name() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z][a-z0-9_]{0,6}",
        prop::sample::select(&["café", "λ", "日本"][..]).prop_map(str::to_owned),
    ]
}

/// The declaration fields that must never exempt a forbidden package.
#[derive(Debug, Clone)]
struct DeclarationShape {
    /// The alias Cargo would report in `rename`, if any.
    alias: Option<String>,
    /// The dependency kind as metadata spells it.
    kind: Value,
    /// Whether a feature could leave the declaration inactive.
    is_optional: bool,
    /// Whether a platform condition could leave it inactive.
    has_target: bool,
}

/// Independently draws each field that must not exempt a declaration.
///
/// Alias, kind, optionality, and target condition are generated independently
/// precisely because the policy must ignore all four. A fixed shape would let a
/// witness pass by carrying the one spelling the literal happened to have.
fn declaration_shape() -> impl Strategy<Value = DeclarationShape> {
    (
        prop::option::of(alias_name()),
        prop::sample::select(vec![Value::Null, json!("dev"), json!("build")]),
        any::<bool>(),
        any::<bool>(),
    )
        .prop_map(|(alias, kind, is_optional, has_target)| DeclarationShape {
            alias,
            kind,
            is_optional,
            has_target,
        })
}

/// A declaration naming `name`, with the generated admission fields.
fn declaration(name: &str, shape: &DeclarationShape) -> Value {
    json!({
        "name": name,
        "source": "registry+https://github.com/rust-lang/crates.io-index",
        "req": "^1.0.0",
        "kind": shape.kind,
        "rename": shape.alias,
        "optional": shape.is_optional,
        "uses_default_features": true,
        "features": [],
        "target": shape.has_target.then_some("cfg(unix)"),
        "registry": null,
    })
}

/// A generated declaration whose name may be forbidden or permitted.
///
/// This is the generator the verdict-equality properties rest on. It draws the
/// package name from the two names the policy refuses and two it permits, so a
/// generated document can land on either side of the verdict and both
/// directions of "the transformation changed nothing" are exercised.
fn generated_dependency() -> impl Strategy<Value = Value> {
    (
        prop::sample::select(&[RUNTIME_PACKAGE, FORBIDDEN_MACROS, "serde", "rstest"][..]),
        declaration_shape(),
    )
        .prop_map(|(name, shape)| declaration(name, &shape))
}

/// A generated declaration naming a package the policy permits.
///
/// Used beside the explicit forbidden witness, so a refusal in that property
/// can only come from the witness and not from a surrounding declaration that
/// happened to draw a forbidden name.
fn generated_permitted_dependency() -> impl Strategy<Value = Value> {
    (
        prop::sample::select(&["serde", "rstest", "anyhow", "thiserror"][..]),
        declaration_shape(),
    )
        .prop_map(|(name, shape)| declaration(name, &shape))
}

/// A generated declaration naming the forbidden macro package.
///
/// The *name* is the one thing held fixed, because it is the policy's subject.
/// Everything that must not exempt the declaration is generated, so a witness
/// cannot pass by carrying the alias or kind a fixed literal happened to have.
fn generated_forbidden_declaration() -> impl Strategy<Value = Value> {
    declaration_shape().prop_map(|shape| declaration(FORBIDDEN_MACROS, &shape))
}

/// Returns the verdict as a boolean, so a property need not name a variant.
fn is_accepted(document: &Value) -> bool { evaluate_metadata(document).is_ok() }

/// Requires one transformation to apply and to leave a refusal standing.
///
/// The transformation reports whether it found the structure it edits. A
/// silent no-op would make the verdict comparison below compare the same
/// document with itself, so a missing structure is a failure rather than a
/// skipped check.
fn still_refused_after(
    document: &mut Value,
    name: &str,
    transform: impl FnOnce(&mut Value) -> bool,
) -> Result<(), TestCaseError> {
    prop_assert!(transform(document), "{} found nothing to edit", name);
    prop_assert!(
        !is_accepted(document),
        "{} admitted the forbidden declaration",
        name
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        ..ProptestConfig::default()
    })]

    /// The accepted document stays accepted under every generated extra record.
    ///
    /// The member list is the only thing that decides membership, so a package
    /// *record* added without its identifier is an invisible extension. Zero
    /// extras is generated too, and the accepted assertion is stated before the
    /// generated one so a policy that refused everything could not pass by
    /// satisfying the implication vacuously.
    #[test]
    fn package_records_outside_the_member_list_are_ignored(
        names in prop::collection::vec(extra_package_name(), 0..=MAX_EXTRAS),
    ) {
        let mut document = accepted_document();
        prop_assert!(is_accepted(&document), "the base fixture was refused");
        prop_assert!(append_package_records(&mut document, &names));
        prop_assert!(
            is_accepted(&document),
            "records {:?} changed the verdict",
            names
        );
    }

    /// A generated extra *member* is refused however the document is ordered.
    ///
    /// Both records and identifiers are appended, so the generated document has
    /// one more member than the accepted one by construction. The reversal is
    /// applied to the whole package list, and the accepted base is asserted
    /// first so the refusal below cannot come from a fixture that was already
    /// invalid.
    #[test]
    fn an_extra_workspace_member_is_refused_after_reordering(
        names in prop::collection::vec(extra_package_name(), 1..=4),
    ) {
        let mut document = accepted_document();
        prop_assert!(is_accepted(&document), "the base fixture was refused");
        prop_assert!(append_package_records(&mut document, &names));
        prop_assert!(append_member_ids(&mut document, &names));
        prop_assert!(!is_accepted(&document), "members {:?} were accepted", names);
        prop_assert!(reverse_packages(&mut document), "the package list is absent");
        prop_assert!(
            !is_accepted(&document),
            "reordering admitted members {:?}",
            names
        );
    }

    /// A generated forbidden declaration survives every graph transformation.
    ///
    /// The witness is inserted at a generated position among declarations that
    /// name only permitted packages, so the refusal is attributable to the
    /// witness alone. Each following assertion applies one transformation the
    /// plan promises must not rescue it: reversing declarations, packages, and
    /// target lists, renaming identifiers consistently, and re-aliasing the
    /// declaration.
    #[test]
    fn a_generated_forbidden_declaration_is_refused_however_spelled(
        dependencies in prop::collection::vec(generated_permitted_dependency(), 0..=MAX_EXTRAS),
        witness in generated_forbidden_declaration(),
        position in 0usize..=MAX_EXTRAS,
        alias in alias_name(),
        names in prop::collection::vec(extra_package_name(), 0..=MAX_EXTRAS),
    ) {
        let mut document = accepted_document();
        let mut generated = dependencies;
        let at = position.min(generated.len());
        generated.insert(at, witness);
        prop_assert!(append_declarations(&mut document, generated));
        prop_assert!(append_package_records(&mut document, &names));
        prop_assert!(!is_accepted(&document), "the forbidden declaration was accepted");
        still_refused_after(&mut document, "reversing declarations", reverse_dependencies)?;
        still_refused_after(&mut document, "reversing packages", reverse_packages)?;
        still_refused_after(&mut document, "reversing targets", reverse_targets)?;
        still_refused_after(&mut document, "reversing kind lists", reverse_kind_lists)?;
        still_refused_after(
            &mut document,
            "renaming identifiers",
            replace_identifiers_with_opaque,
        )?;
        still_refused_after(&mut document, "re-aliasing declarations", |edited| {
            alias_forbidden_declarations(edited, &alias)
        })?;
    }

    /// Reordering a refused document never turns it into an acceptance.
    ///
    /// The converse direction of the same rule, over arbitrary generated
    /// extras and declarations. The document is refused by construction — it
    /// holds an explicitly generated forbidden declaration — and the property
    /// is that permuting both the package list and the declaration list cannot
    /// make it acceptable. An implementation that read only the first
    /// declaration, or that keyed the policy on positional shape, would pass
    /// every exact case in `policy_cases.rs` and fail here.
    #[test]
    fn reordering_never_turns_a_refusal_into_an_acceptance(
        dependencies in prop::collection::vec(generated_dependency(), 0..=MAX_EXTRAS),
        witness in generated_forbidden_declaration(),
        names in prop::collection::vec(extra_package_name(), 0..=4),
    ) {
        let mut document = accepted_document();
        let mut generated = dependencies;
        generated.push(witness);
        prop_assert!(append_declarations(&mut document, generated));
        prop_assert!(append_package_records(&mut document, &names));
        prop_assert!(!is_accepted(&document), "the deliberately refused document was accepted");
        prop_assert!(reverse_packages(&mut document), "the package list is absent");
        prop_assert!(reverse_dependencies(&mut document), "the declaration list is absent");
        prop_assert!(!is_accepted(&document), "reordering accepted a refused document");
    }

    /// Renaming every identifier consistently never changes the verdict.
    ///
    /// Both verdicts are exercised: the unfiltered generator can produce a
    /// document that is already refused, and a document with no forbidden
    /// declaration is accepted. The member lists are rewritten through the same
    /// map, so the runtime package is still the sole member afterwards. A
    /// policy that keyed on identifier text rather than on reference identity
    /// would flip its answer here.
    #[test]
    fn consistent_opaque_identifiers_never_change_the_verdict(
        dependencies in prop::collection::vec(generated_dependency(), 0..=MAX_EXTRAS),
        names in prop::collection::vec(extra_package_name(), 0..=MAX_EXTRAS),
    ) {
        let mut document = accepted_document();
        prop_assert!(is_accepted(&document), "the base fixture was refused");
        prop_assert!(append_declarations(&mut document, dependencies));
        prop_assert!(append_package_records(&mut document, &names));
        let before = is_accepted(&document);
        prop_assert!(
            replace_identifiers_with_opaque(&mut document),
            "no package identifier was found"
        );
        prop_assert_eq!(
            before,
            is_accepted(&document),
            "renaming identifiers changed the verdict"
        );
    }

    /// Reversing target order never changes the verdict.
    ///
    /// The policy reads targets as a set, so their order carries no meaning; a
    /// policy that read only the first target would answer differently once the
    /// list is reversed. Kind order is a separate property, because one flag
    /// covering both reversals would let either fall silent unnoticed.
    #[test]
    fn target_order_never_changes_the_verdict(
        dependencies in prop::collection::vec(generated_dependency(), 0..=MAX_EXTRAS),
        names in prop::collection::vec(extra_package_name(), 0..=MAX_EXTRAS),
    ) {
        let mut document = accepted_document();
        prop_assert!(is_accepted(&document), "the base fixture was refused");
        prop_assert!(append_declarations(&mut document, dependencies));
        prop_assert!(append_package_records(&mut document, &names));
        let before = is_accepted(&document);
        prop_assert!(reverse_targets(&mut document), "no target list was found");
        prop_assert_eq!(
            before,
            is_accepted(&document),
            "reversing target order changed the verdict"
        );
    }

    /// Reversing each target's kind order never changes the verdict.
    ///
    /// The policy reads kinds as a set. The accepted fixture's runtime target
    /// declares `["lib", "rlib"]` — a pair Cargo genuinely emits for
    /// `crate-type = ["lib", "rlib"]` — so the transformation reports `true`
    /// only because a real order change happened. A one-kind fixture would make
    /// `reverse_kind_lists` return `false`, and this assertion would fail
    /// instead of passing vacuously.
    #[test]
    fn kind_order_never_changes_the_verdict(
        dependencies in prop::collection::vec(generated_dependency(), 0..=MAX_EXTRAS),
        names in prop::collection::vec(extra_package_name(), 0..=MAX_EXTRAS),
    ) {
        let mut document = accepted_document();
        prop_assert!(is_accepted(&document), "the base fixture was refused");
        prop_assert!(append_declarations(&mut document, dependencies));
        prop_assert!(append_package_records(&mut document, &names));
        let before = is_accepted(&document);
        prop_assert!(reverse_kind_lists(&mut document), "no kind list was reversed");
        prop_assert_eq!(
            before,
            is_accepted(&document),
            "reversing kind order changed the verdict"
        );
    }
}
