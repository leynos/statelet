//! Exercises placeholder rejection and the values that must remain valid.

use rstest::rstest;

use super::{
    FixtureField,
    Kind,
    check,
    note,
    open_roadmap,
    replace_value,
    roadmap_with_claims,
    template,
};

const SYNTHETIC_SUCCESS: &str =
    "  - Success: [synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)";

#[rstest]
#[case("tbd")]
#[case("TODO")]
#[case("pending")]
#[case("N/A")]
#[case("<fill this>")]
#[case("TBD: later")]
#[case("Pending: run")]
#[case("Unavailable: TODO later")]
#[case("Measured on <target> hardware")]
#[case("Malformed markup < remains before <target> hardware")]
#[case("repo@<sha>:path")]
#[case("revision: <sha>")]
#[case("macro=<sha>")]
#[case("Reviewed `<sha>` expansion")]
#[case("Reviewed `Option<Foo>` expansion `<fill this>`")]
fn marked_note_rejects_placeholder_values(#[case] value: &str) {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(&mut evidence, FixtureField::ENVIRONMENT, value).expect("fixture field exists");
    let error = check(&template(), &open_roadmap(), &[evidence])
        .expect_err("invalid test input must be rejected");
    assert!(
        error.contains("environment") && error.contains("is empty or still a placeholder"),
        "{error}"
    );
}

#[rstest]
#[case("Reviewed `Option<Foo>` expansion")]
#[case("Reviewed `Vec<u8>` expansion")]
#[case("Reviewed `Option<Vec<T>>` expansion")]
#[case("Reviewed `std::vec::Vec<T>` expansion")]
#[case("Reviewed `foo::<T>` expansion")]
fn marked_note_accepts_attached_generic_syntax(#[case] value: &str) {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(&mut evidence, FixtureField::FALLIBLE_EXPANSION, value)
        .expect("fixture field exists");

    check(&template(), &open_roadmap(), &[evidence])
        .expect("attached Rust path generics are not placeholders");
}

#[test]
fn marked_note_accepts_marker_words_inside_larger_identifiers() {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::ENVIRONMENT,
        "pending_count=0; todoist=available",
    )
    .expect("fixture field exists");

    check(&template(), &open_roadmap(), &[evidence])
        .expect("marker words inside identifiers are ordinary values");
}

#[test]
fn marked_note_accepts_budget_inequalities_with_both_angle_characters() {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::BUDGET_VERDICT,
        "Marginal cost < 0.5%; baseline > 0",
    )
    .expect("fixture field exists");

    check(&template(), &open_roadmap(), &[evidence])
        .expect("budget inequalities are not angle-bracket placeholders");
}

#[rstest]
#[case(
    "  - Selection: [synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)\n  - Success: \
     complete"
)]
#[case(
    "  - Success: complete\n  - Related: \
     [synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)"
)]
fn completed_claim_rejects_links_in_sibling_bullets(#[case] replacement: &str) {
    let roadmap = roadmap_with_claims(true, false).replace(SYNTHETIC_SUCCESS, replacement);
    let error = check(&template(), &roadmap, &[note(Kind::Synthetic20, 20)])
        .expect_err("a sibling link must not select evidence");
    assert!(error.contains("must link exactly 1 cost note"), "{error}");
}

#[rstest]
#[case("  - Selection: complete")]
#[case(concat!("  - Success: [synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)",
    "\n  - Success: duplicate"))]
fn completed_claim_requires_one_success_criterion(#[case] replacement: &str) {
    let roadmap = roadmap_with_claims(true, false).replace(SYNTHETIC_SUCCESS, replacement);
    let error = check(&template(), &roadmap, &[note(Kind::Synthetic20, 20)])
        .expect_err("a completed claim needs one success criterion");
    assert!(error.contains("exactly one Success criterion"), "{error}");
}

#[test]
fn completed_claim_accepts_wrapped_success_link_and_ignores_other_links() {
    let replacement = concat!(
        "  - Context: [other](validation-notes/other.md)\n",
        "  - Success: selected\n",
        "    [synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)",
    );
    let roadmap = roadmap_with_claims(true, false).replace(SYNTHETIC_SUCCESS, replacement);
    check(&template(), &roadmap, &[note(Kind::Synthetic20, 20)])
        .expect("only the wrapped success criterion selects evidence");
}

#[test]
fn publication_claim_rejects_one_note_link_outside_success() {
    let link = "[synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)";
    let moved = format!("synthetic note\n  - Related: {link}");
    let roadmap = roadmap_with_claims(false, true).replace(link, &moved);
    let notes = [note(Kind::Mdtablefix, 3), note(Kind::Synthetic20, 20)];
    let error = check(&template(), &roadmap, &notes)
        .expect_err("both selected links must occur in the success criterion");
    assert!(error.contains("must link exactly 2 cost note"), "{error}");
}
