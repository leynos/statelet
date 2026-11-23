//! Checks that every document naming the transition fields agrees with the
//! published constants.
//!
//! ADR 0008 is the single source of each field's rules, and three other places
//! name the same five fields: the design's §9.1, the users' guide's
//! "Transition tracing fields" section, and the glossary entry in
//! `docs/context.md`. Each is a list an author edits by hand, so each can drift
//! from the constants independently. This module extracts the names from every
//! such list and rejects both a missing contracted name and a name the
//! contract does not publish.
//!
//! Extraction is deliberately name-set-only outside the ADR: the ADR table is
//! the source of meaning, presence, and value rules, and checking its prose
//! from here would be a brittle second copy of it. The extractors return an
//! error rather than an empty slice when a heading or marker is missing, so a
//! renamed section fails loudly instead of comparing nothing with nothing.

use pretty_assertions::assert_eq;
use rstest::rstest;
use statelet::fields;

/// The ADR that owns the contract.
const ADR: &str =
    include_str!("../../docs/adrs/adr-0008-publish-the-transition-tracing-field-contract.md");
/// The design document whose §9 lists the names.
const DESIGN: &str = include_str!("../../docs/design.md");
/// The users' guide section that lists the names.
const USERS_GUIDE: &str = include_str!("../../docs/users-guide.md");
/// The glossary that defines the contract.
const CONTEXT: &str = include_str!("../../docs/context.md");
/// The source that publishes the constants.
const FIELDS_SOURCE: &str = include_str!("../../src/fields.rs");

/// The marker opening the contract table in ADR 0008.
const TABLE_BEGIN: &str = "<!-- transition-fields:begin -->";
/// The marker closing the contract table in ADR 0008.
const TABLE_END: &str = "<!-- transition-fields:end -->";

/// The five names every document must list, written independently of the
/// constants so a drifted constant cannot validate the documents.
const EXPECTED: [&str; 5] = [
    "transition.name",
    "transition.state.before",
    "transition.event",
    "transition.outcome",
    "transition.error",
];

/// Folds whitespace runs to single spaces, so a rewrapped paragraph matches.
fn fold(text: &str) -> String { text.split_whitespace().collect::<Vec<_>>().join(" ") }

/// The text between two marker lines, or an error naming the missing marker.
fn marked_section<'a>(text: &'a str, begin: &str, end: &str) -> Result<&'a str, String> {
    let after_begin = text
        .split_once(begin)
        .ok_or_else(|| format!("the contract table's opening marker {begin:?} is missing"))?
        .1;
    let section = after_begin
        .split_once(end)
        .ok_or_else(|| format!("the contract table's closing marker {end:?} is missing"))?
        .0;
    if section.trim().is_empty() {
        return Err("the marker-delimited contract table is empty".to_owned());
    }
    Ok(section)
}

/// The body of the level-two section whose heading starts with `prefix`.
///
/// The body ends at the next heading of the same or higher rank, so the check
/// follows a restructured section rather than silently reading to the end of
/// the document.
fn section<'a>(text: &'a str, prefix: &str) -> Result<&'a str, String> {
    let marker = format!("## {prefix}");
    let after_heading = text
        .split_once(&marker)
        .ok_or_else(|| format!("the section heading {marker:?} is missing"))?
        .1;
    let body = after_heading
        .split_once("\n## ")
        .map_or(after_heading, |(body, _)| body);
    if body.trim().is_empty() {
        return Err(format!("the section {marker:?} is empty"));
    }
    Ok(body)
}

/// Every `transition.*` token in `text`, deduplicated and sorted.
///
/// Tokens are trimmed of surrounding punctuation so a sentence-final full stop
/// does not become part of a name. A bare `transition.` prefix mention is not
/// a field name and is ignored.
fn transition_tokens(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for raw in text.split(|character: char| {
        character.is_whitespace() || matches!(character, '`' | '(' | ')' | ',' | '"' | '\'')
    }) {
        let token = raw.trim_end_matches(['.', ':', ';']);
        if token.starts_with("transition.") && token.len() > "transition.".len() {
            found.push(token.to_owned());
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Rejects a section that omits a contracted name or invents another.
fn check_section(document: &str, section_name: &str, body: &str) -> Result<(), String> {
    let missing: Vec<&str> = EXPECTED
        .iter()
        .copied()
        .filter(|name| !body.contains(&format!("`{name}`")))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{document} {section_name} does not name the contracted field(s) {missing:?}"
        ));
    }
    let unexpected: Vec<String> = transition_tokens(body)
        .into_iter()
        .filter(|token| !EXPECTED.contains(&token.as_str()))
        .collect();
    if !unexpected.is_empty() {
        return Err(format!(
            "{document} {section_name} names {unexpected:?}, which the contract does not publish"
        ));
    }
    Ok(())
}

/// The names published by the `pub const` items in `src/fields.rs`.
fn constant_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub const "))
        .filter_map(|rest| rest.split_once(" = \""))
        .filter_map(|(_, value)| value.split_once('"'))
        .map(|(name, _)| name.to_owned())
        .collect()
}

/// The ADR table's first column names exactly the contracted fields.
#[test]
fn adr_table_lists_the_contract() -> Result<(), String> {
    let table = marked_section(ADR, TABLE_BEGIN, TABLE_END)?;
    check_section("ADR 0008", "field table", table)
}

/// The design's §9 names exactly the contracted fields.
#[test]
fn design_names_the_contract() -> Result<(), String> {
    let body = section(DESIGN, "9. Observability")?;
    check_section("docs/design.md", "§9", body)
}

/// The users' guide section names exactly the contracted fields.
#[test]
fn users_guide_names_the_contract() -> Result<(), String> {
    let body = section(USERS_GUIDE, "Transition tracing fields")?;
    check_section("docs/users-guide.md", "field section", body)
}

/// The glossary entry names exactly the contracted fields.
#[test]
fn glossary_names_the_contract() -> Result<(), String> {
    let body = section(CONTEXT, "Transition field contract")?;
    check_section("docs/context.md", "glossary entry", body)
}

/// The exported constants are exactly the contracted names.
#[test]
fn constants_match_the_contract_table() {
    assert_eq!(
        constant_names(FIELDS_SOURCE),
        EXPECTED.map(str::to_owned),
        "src/fields.rs must export exactly the contracted field names"
    );
}

/// The design's §9 still carries the phrases other documents quote.
#[rstest]
#[case::public_api("public operational API")]
#[case::semver("semver-relevant")]
fn design_keeps_quoted_phrases(#[case] phrase: &str) {
    let body = section(DESIGN, "9. Observability")
        .expect("the design must still have its observability section");
    assert!(
        fold(body).contains(phrase),
        "design §9 must still contain the quoted phrase {phrase:?}"
    );
}

/// The contracted names equal the published constants.
#[test]
fn constants_are_the_expected_names() {
    let published = [
        fields::TRANSITION_NAME,
        fields::TRANSITION_STATE_BEFORE,
        fields::TRANSITION_EVENT,
        fields::TRANSITION_OUTCOME,
        fields::TRANSITION_ERROR,
    ];
    assert_eq!(
        published, EXPECTED,
        "the exported constants must equal the names the documents list"
    );
}
