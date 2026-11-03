//! Invariance properties for the lexical predicates the register parser uses.
//!
//! `fixtures/tests/placeholder_tests.rs` names one spelling per rule and asks
//! for the exact diagnostic. What a case table cannot state is the rule that
//! must survive *any* cell text: a marker word only matches at a word
//! boundary, a standalone placeholder is rejected wherever it appears, and an
//! attached Rust path generic is never mistaken for one. These properties
//! state that directly over generated text.
//!
//! The generators are deliberately unfiltered. A generator that only produced
//! text already containing a placeholder would make "a longer word is not a
//! placeholder" true for the wrong reason, so each property draws the
//! surrounding text independently and asserts the verdict it expects rather
//! than an implication that a degenerate input could satisfy vacuously.

use proptest::prelude::*;

use super::{
    evidence::table_cells,
    placeholders::{has_revision_citation, placeholder, unavailable_reason},
};
use crate::{
    MarkdownFile,
    types::{CellText, SourceLine},
};

/// The marker words the template documents as placeholders.
const MARKERS: [&str; 4] = ["tbd", "todo", "pending", "n/a"];

fn marker_strategy() -> impl Strategy<Value = &'static str> {
    proptest::sample::select(MARKERS.to_vec())
}

fn note_file() -> MarkdownFile {
    MarkdownFile {
        path: "note.md".into(),
        text: String::new(),
    }
}

/// A path identifier that cannot itself start with a placeholder marker.
///
/// The marker rule is case-insensitive and matches at the start of the cell,
/// so a path such as `tbd` would be rejected before the angle-token rule is
/// consulted. Ending the first segment in `x` keeps the generator
/// construction-based: no documented marker ends in that character.
fn non_marker_path() -> impl Strategy<Value = String> {
    (
        "[a-z_][a-z0-9_]{0,8}",
        proptest::collection::vec("[a-z_][a-z0-9_]{0,8}", 0..3),
    )
        .prop_map(|(first, rest)| {
            let mut path = format!("{first}x");
            for segment in rest {
                path.push_str("::");
                path.push_str(&segment);
            }
            path
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        ..ProptestConfig::default()
    })]

    /// A marker word inside a longer word is ordinary text.
    ///
    /// The suffix starts with an identifier character, so the marker is not
    /// followed by a boundary and the cell must survive as a real value.
    #[test]
    fn a_marker_word_inside_a_longer_word_is_not_a_placeholder(
        marker in marker_strategy(),
        suffix in "[A-Za-z0-9_][a-z0-9 _./-]{0,10}",
    ) {
        let value = format!("{marker}{suffix}");
        prop_assert!(
            !placeholder(CellText(&value)),
            "{value} must not be a placeholder"
        );
    }

    /// A marker word followed by a separator is a placeholder.
    ///
    /// This is the boundary the previous property excludes, stated in its own
    /// right so neither property can pass by generating only one side.
    #[test]
    fn a_marker_word_followed_by_a_separator_is_a_placeholder(
        marker in marker_strategy(),
        suffix in "[ :;,.!?()\\[\\]-][a-z0-9 _./-]{0,10}",
    ) {
        let value = format!("{marker}{suffix}");
        prop_assert!(
            placeholder(CellText(&value)),
            "{value} must be a placeholder"
        );
    }

    /// A standalone placeholder is rejected after any non-identifier prefix.
    ///
    /// The prefix is empty or ends with a space, so the angle token is not
    /// attached to a path identifier and must be read as a placeholder.
    #[test]
    fn a_standalone_placeholder_is_rejected_after_any_prefix(
        prefix in "([a-z]{1,8} )?",
        token in "[a-z0-9_-]{1,10}",
    ) {
        let value = format!("{prefix}<{token}>");
        prop_assert!(
            placeholder(CellText(&value)),
            "{value} must be a placeholder"
        );
    }

    /// An attached Rust path generic is not a placeholder.
    ///
    /// The path ends with an identifier character, which is the documented
    /// ambiguity: `Type<sha>` is a generic, so only a standalone token is
    /// rejected. The path generator excludes marker words because the
    /// leading-marker rule is checked before the angle-token exemption.
    #[test]
    fn an_attached_path_generic_is_not_a_placeholder(
        path in non_marker_path(),
        token in "[A-Za-z][A-Za-z0-9]{0,6}",
    ) {
        let value = format!("{path}<{token}>");
        prop_assert!(
            !placeholder(CellText(&value)),
            "{value} must not be a placeholder"
        );
    }

    /// A malformed opener does not hide a later standalone placeholder.
    ///
    /// Each `<` is tested independently, so the first opener consuming the
    /// final `>` must not mask the second token.
    #[test]
    fn a_malformed_opener_does_not_hide_a_later_placeholder(
        junk in "[a-z ]{0,10}",
        token in "[a-z0-9_-]{1,10}",
    ) {
        let value = format!("<{junk} <{token}>");
        prop_assert!(
            placeholder(CellText(&value)),
            "{value} must be a placeholder"
        );
    }

    /// An unavailable marker yields its trimmed reason.
    #[test]
    fn unavailable_reason_returns_the_trimmed_reason(reason in "[a-z0-9 ]{0,12}") {
        let value = format!("Unavailable: {reason}");
        prop_assert_eq!(unavailable_reason(CellText(&value)), Some(reason.trim()));
    }

    /// A full lowercase revision citation is accepted.
    #[test]
    fn a_full_lowercase_revision_citation_is_accepted(
        repository in "[a-z][a-z0-9-]{0,10}",
        revision in "[0-9a-f]{40}",
        path in "[a-z][a-z0-9/._-]{0,20}",
    ) {
        let evidence = format!("{repository}@{revision}:{path}");
        prop_assert!(has_revision_citation(CellText(&evidence)));
    }

    /// A revision shorter than forty characters is rejected.
    #[test]
    fn a_short_revision_citation_is_rejected(
        repository in "[a-z][a-z0-9-]{0,10}",
        revision in "[0-9a-f]{0,39}",
        path in "[a-z][a-z0-9/._-]{0,20}",
    ) {
        let evidence = format!("{repository}@{revision}:{path}");
        prop_assert!(!has_revision_citation(CellText(&evidence)));
    }

    /// A forty-character revision containing an uppercase letter is rejected.
    ///
    /// The length check passes, so only the lowercase rule can reject it.
    #[test]
    fn an_uppercase_revision_citation_is_rejected(
        repository in "[a-z][a-z0-9-]{0,10}",
        revision in "[0-9a-f]{39}[A-F]",
        path in "[a-z][a-z0-9/._-]{0,20}",
    ) {
        let evidence = format!("{repository}@{revision}:{path}");
        prop_assert!(!has_revision_citation(CellText(&evidence)));
    }

    /// A forty-character revision containing a non-hexadecimal character is
    /// rejected.
    ///
    /// The length check passes, so only the hexadecimal rule can reject it.
    #[test]
    fn a_non_hexadecimal_revision_citation_is_rejected(
        repository in "[a-z][a-z0-9-]{0,10}",
        revision in "[0-9a-f]{19}[g-z][0-9a-f]{20}",
        path in "[a-z][a-z0-9/._-]{0,20}",
    ) {
        let evidence = format!("{repository}@{revision}:{path}");
        prop_assert!(!has_revision_citation(CellText(&evidence)));
    }

    /// A citation with an empty repository component is rejected.
    #[test]
    fn a_citation_with_an_empty_repository_is_rejected(
        revision in "[0-9a-f]{40}",
        path in "[a-z][a-z0-9/._-]{0,20}",
    ) {
        let evidence = format!("@{revision}:{path}");
        prop_assert!(!has_revision_citation(CellText(&evidence)));
    }

    /// A citation with an empty path component is rejected.
    #[test]
    fn a_citation_with_an_empty_path_is_rejected(
        repository in "[a-z][a-z0-9-]{0,10}",
        revision in "[0-9a-f]{40}",
    ) {
        let evidence = format!("{repository}@{revision}:");
        prop_assert!(!has_revision_citation(CellText(&evidence)));
    }

    /// A three-cell row splits into its trimmed cells.
    #[test]
    fn a_three_cell_row_splits_into_trimmed_cells(
        field in "[a-z-]{1,10}",
        value in "[a-z0-9 .]{0,10}",
        evidence in "[a-z0-9 .]{0,10}",
    ) {
        let file = note_file();
        let line = format!("| {field} | {value} | {evidence} |");
        let cells = table_cells(&file, SourceLine(&line));
        prop_assert_eq!(
            cells,
            Ok(Some([field.as_str(), value.trim(), evidence.trim()]))
        );
    }

    /// A row without three cells is rejected rather than silently truncated.
    #[test]
    fn a_row_without_three_cells_is_rejected(
        first in "[a-z]{1,8}",
        second in "[a-z]{1,8}",
    ) {
        let file = note_file();
        let line = format!("| {first} | {second} |");
        prop_assert!(table_cells(&file, SourceLine(&line)).is_err());
    }
}

/// A marker word before an angle token is still a placeholder.
///
/// The angle-token exemption requires an attached path identifier, but the
/// leading-marker rule is checked first. `tbd<A>` therefore stays rejected;
/// this control records that interaction so a future refactor cannot invert
/// the order silently.
#[test]
fn a_marker_word_before_an_angle_token_is_still_a_placeholder() {
    assert!(placeholder(CellText("tbd<A>")));
}
