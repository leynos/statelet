//! Exercises the V2 claim-state and note-collection matrix exhaustively.
//!
//! The `ExecPlan` requires every combination of open and completed roadmap
//! tasks against zero notes, each comparison alone, both complete
//! comparisons, a placeholder-bearing note, and an unselected historical
//! note. Enumerating the matrix here keeps the required cross-product
//! reviewable in one place instead of scattered across single-case tests.

use rstest::rstest;

use super::{
    FixtureField,
    Kind,
    MarkdownFile,
    check,
    note,
    replace_value,
    roadmap_with_claims,
    template,
};

#[derive(Clone, Copy, Debug)]
enum Notes {
    None,
    Real,
    Synthetic,
    Both,
    PlaceholderSynthetic,
    UnselectedHistorical,
}

impl Notes {
    /// Return the note collection for this matrix case.
    fn build(self) -> Result<Vec<MarkdownFile>, String> {
        match self {
            Self::None => Ok(Vec::new()),
            Self::Real => Ok(vec![note(Kind::Mdtablefix, 3)]),
            Self::Synthetic => Ok(vec![note(Kind::Synthetic20, 20)]),
            Self::Both => Ok(vec![note(Kind::Mdtablefix, 3), note(Kind::Synthetic20, 20)]),
            Self::PlaceholderSynthetic => Ok(vec![placeholder_synthetic()?]),
            Self::UnselectedHistorical => Ok(vec![unselected_historical()]),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Expectation {
    Accepted,
    Rejected(&'static str),
}

/// Check each claim state accepts exactly the note collections the V2 matrix allows.
#[rstest]
#[case(false, false, Notes::None, Expectation::Accepted)]
#[case(false, false, Notes::Real, Expectation::Accepted)]
#[case(false, false, Notes::Synthetic, Expectation::Accepted)]
#[case(false, false, Notes::Both, Expectation::Accepted)]
#[case(
    false,
    false,
    Notes::PlaceholderSynthetic,
    Expectation::Rejected("placeholder")
)]
#[case(false, false, Notes::UnselectedHistorical, Expectation::Accepted)]
#[case(true, false, Notes::None, Expectation::Rejected("links missing note"))]
#[case(true, false, Notes::Real, Expectation::Rejected("links missing note"))]
#[case(true, false, Notes::Synthetic, Expectation::Accepted)]
#[case(true, false, Notes::Both, Expectation::Accepted)]
#[case(
    true,
    false,
    Notes::PlaceholderSynthetic,
    Expectation::Rejected("placeholder")
)]
#[case(
    true,
    false,
    Notes::UnselectedHistorical,
    Expectation::Rejected("links missing note")
)]
#[case(false, true, Notes::None, Expectation::Rejected("links missing note"))]
#[case(false, true, Notes::Real, Expectation::Rejected("links missing note"))]
#[case(
    false,
    true,
    Notes::Synthetic,
    Expectation::Rejected("links missing note")
)]
#[case(false, true, Notes::Both, Expectation::Accepted)]
#[case(
    false,
    true,
    Notes::PlaceholderSynthetic,
    Expectation::Rejected("placeholder")
)]
#[case(
    false,
    true,
    Notes::UnselectedHistorical,
    Expectation::Rejected("links missing note")
)]
#[case(true, true, Notes::None, Expectation::Rejected("links missing note"))]
#[case(true, true, Notes::Real, Expectation::Rejected("links missing note"))]
#[case(
    true,
    true,
    Notes::Synthetic,
    Expectation::Rejected("links missing note")
)]
#[case(true, true, Notes::Both, Expectation::Accepted)]
#[case(
    true,
    true,
    Notes::PlaceholderSynthetic,
    Expectation::Rejected("placeholder")
)]
#[case(
    true,
    true,
    Notes::UnselectedHistorical,
    Expectation::Rejected("links missing note")
)]
fn claim_states_accept_exactly_the_v2_note_collections(
    #[case] claim_422: bool,
    #[case] claim_431: bool,
    #[case] notes: Notes,
    #[case] expected: Expectation,
) -> Result<(), String> {
    let roadmap = roadmap_with_claims(claim_422, claim_431);
    let files = notes.build()?;
    match expected {
        Expectation::Accepted => check(&template(), &roadmap, &files)
            .map_err(|error| format!("expected acceptance: {error}")),
        Expectation::Rejected(message) => match check(&template(), &roadmap, &files) {
            Err(error) if error.contains(message) => Ok(()),
            Err(error) => Err(format!("expected `{message}` in: {error}")),
            Ok(()) => Err(format!("expected rejection containing `{message}`")),
        },
    }
}

/// Return a synthetic note whose baseline value is still a placeholder.
fn placeholder_synthetic() -> Result<MarkdownFile, String> {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::CLEAN_DEBUG_BASELINE_SECONDS,
        "TBD",
    )?;
    Ok(evidence)
}

fn unselected_historical() -> MarkdownFile {
    let mut historical = note(Kind::Synthetic20, 20);
    historical.path = "docs/validation-notes/4.2.2-earlier-candidate-macro-cost.md".into();
    historical
}
