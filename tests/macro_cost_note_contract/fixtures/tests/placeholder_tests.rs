//! Exercises placeholder rejection and the values that must remain valid.

use rstest::rstest;

use super::{FixtureField, Kind, check, note, open_roadmap, replace_value, template};

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
