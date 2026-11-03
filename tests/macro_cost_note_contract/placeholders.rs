//! Lexical placeholder and citation predicates over register cell text.
//!
//! These checks are deliberately heuristic: they reject the template's
//! stand-ins without trying to understand arbitrary Markdown. Keeping them
//! separate leaves the register parser in `evidence.rs` owning only the
//! `Field | Value | Evidence` schema.

use super::{
    super::types::{CellText, Revision},
    full_revision,
};

/// Return whether `value` is empty or still a template placeholder.
pub(super) fn placeholder(value: CellText<'_>) -> bool {
    let trimmed = value.0.trim();
    let checked_value = unavailable_reason(CellText(trimmed)).unwrap_or(trimmed);
    checked_value.is_empty()
        || starts_with_placeholder_marker(CellText(checked_value))
        || contains_angle_placeholder(CellText(checked_value))
}

pub(super) fn unavailable_reason(value: CellText<'_>) -> Option<&str> {
    // A bare marker is a documented unavailable form with an empty reason, so
    // it reaches the same incomplete-value and `Inconclusive` checks as
    // `Unavailable:` rather than passing as ordinary text.
    let trimmed = value.0.trim();
    let (label, reason) = trimmed.split_once(':').unwrap_or((trimmed, ""));
    label
        .trim()
        .eq_ignore_ascii_case("unavailable")
        .then_some(reason.trim())
}

/// Return whether `value` begins with a standalone placeholder marker word.
fn starts_with_placeholder_marker(value: CellText<'_>) -> bool {
    let normalized = value.to_ascii_lowercase();
    ["tbd", "todo", "pending", "n/a"].iter().any(|marker| {
        normalized.strip_prefix(marker).is_some_and(|suffix| {
            suffix.is_empty()
                || suffix
                    .chars()
                    .next()
                    .is_some_and(|character| !character.is_ascii_alphanumeric() && character != '_')
        })
    })
}

fn contains_angle_placeholder(value: CellText<'_>) -> bool {
    // Test each opener independently so malformed markup cannot hide a later
    // standalone placeholder.
    value
        .match_indices('<')
        .any(|(open, _)| placeholder_opening_at(value, open))
}

/// Return whether the `<` at `open` starts a standalone placeholder token.
fn placeholder_opening_at(value: CellText<'_>, open: usize) -> bool {
    let candidate_start = open + 1;
    let Some(close_offset) = value.get(candidate_start..).and_then(|tail| tail.find('>')) else {
        return false;
    };
    matches!(
        (value.get(candidate_start..candidate_start + close_offset), value.get(..open)),
        (Some(candidate), Some(prefix))
            if !is_attached_to_path_identifier(CellText(prefix))
                && is_placeholder_token(CellText(candidate))
    )
}

/// Return whether `prefix` ends in a path identifier, so a following `<` is generic syntax.
fn is_attached_to_path_identifier(prefix: CellText<'_>) -> bool {
    // This lexical heuristic admits Rust generics. `Type<sha>` is ambiguous,
    // so revision placeholders must remain standalone to be rejected.
    let path = prefix.0.strip_suffix("::").unwrap_or(prefix.0);
    path.chars()
        .next_back()
        .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
}

/// Return whether `candidate` is a placeholder identifier or instruction.
fn is_placeholder_token(candidate: CellText<'_>) -> bool {
    let trimmed_candidate = candidate.trim();
    is_placeholder_identifier(CellText(trimmed_candidate))
        || is_placeholder_instruction(CellText(trimmed_candidate))
}

/// Return whether `candidate` is empty or only identifier-like characters.
fn is_placeholder_identifier(candidate: CellText<'_>) -> bool {
    candidate.is_empty()
        || candidate.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
        })
}

fn is_placeholder_instruction(candidate: CellText<'_>) -> bool {
    ["fill ", "insert ", "replace ", "example ", "your "]
        .iter()
        .any(|prefix| candidate.to_ascii_lowercase().starts_with(prefix))
}

/// Return whether `evidence` contains a `repo@revision:path` citation.
pub(super) fn has_revision_citation(evidence: CellText<'_>) -> bool {
    evidence.split_whitespace().any(|token| {
        let Some((repository, rest)) = token.split_once('@') else {
            return false;
        };
        let Some((revision, path)) = rest.split_once(':') else {
            return false;
        };
        !repository.is_empty() && full_revision(Revision(revision)) && !path.is_empty()
    })
}
