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

/// One document this agreement check reads.
#[derive(Debug, Clone, Copy)]
struct Document {
    /// The repository-relative path, reported in every failure.
    path: &'static str,
    /// The document's text, embedded at compile time.
    text: &'static str,
}

impl Document {
    /// The document at `path`, read from the working tree at compile time.
    const fn new(path: &'static str, text: &'static str) -> Self { Self { path, text } }
}

/// The ADR that owns the contract.
const ADR: Document = Document::new(
    "docs/adrs/adr-0008-publish-the-transition-tracing-field-contract.md",
    include_str!("../../docs/adrs/adr-0008-publish-the-transition-tracing-field-contract.md"),
);
/// The design document whose §9 lists the names.
const DESIGN: Document = Document::new("docs/design.md", include_str!("../../docs/design.md"));
/// The users' guide, whose field section lists the names.
const USERS_GUIDE: Document = Document::new(
    "docs/users-guide.md",
    include_str!("../../docs/users-guide.md"),
);
/// The glossary that defines the contract.
const CONTEXT: Document = Document::new("docs/context.md", include_str!("../../docs/context.md"));
/// The source that publishes the constants.
const FIELDS_SOURCE: Document = Document::new("src/fields.rs", include_str!("../../src/fields.rs"));

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

/// The five contracted names as backticked code spans, for membership tests.
///
/// Building these once keeps every section check from formatting five spans of
/// its own, and keeps the backtick convention in one place.
const EXPECTED_SPANS: [&str; 5] = [
    "`transition.name`",
    "`transition.state.before`",
    "`transition.event`",
    "`transition.outcome`",
    "`transition.error`",
];

/// One extracted region of a document, with the name it is reported under.
#[derive(Debug, Clone, Copy)]
struct Section<'a> {
    /// The document the region was extracted from.
    document: Document,
    /// What the region is called in a failure message.
    name: &'a str,
    /// The extracted text.
    body: &'a str,
}

impl<'a> Section<'a> {
    /// The region between two marker lines in `document`.
    ///
    /// # Errors
    ///
    /// Returns a message naming the missing marker when either marker is
    /// absent, and a message when the delimited region is empty.
    fn between(document: Document, name: &'a str, begin: &str, end: &str) -> Result<Self, String> {
        let after_begin = document
            .text
            .split_once(begin)
            .ok_or_else(|| format!("{}: opening marker {begin:?} is missing", document.path))?
            .1;
        let body = after_begin
            .split_once(end)
            .ok_or_else(|| format!("{}: closing marker {end:?} is missing", document.path))?
            .0;
        Self::require_non_empty(document, name, body)
    }

    /// The body of the level-two section whose heading starts with `prefix`.
    ///
    /// The body ends at the next heading of the same or higher rank, so the
    /// check follows a restructured section rather than silently reading to the
    /// end of the document.
    ///
    /// # Errors
    ///
    /// Returns a message naming the missing heading when no such section
    /// exists, and a message when the section body is empty.
    fn level_two(document: Document, heading: &'a str) -> Result<Self, String> {
        let marker = format!("## {heading}");
        let after_heading = document
            .text
            .split_once(&marker)
            .ok_or_else(|| format!("{}: heading {marker:?} is missing", document.path))?
            .1;
        let body = after_heading
            .split_once("\n## ")
            .map_or(after_heading, |(body, _)| body);
        Self::require_non_empty(document, heading, body)
    }

    /// Accepts the region, rejecting an empty one.
    ///
    /// A section that exists but says nothing would compare "no names" with
    /// "no names" and pass, so emptiness is an error rather than a result.
    fn require_non_empty(document: Document, name: &'a str, body: &'a str) -> Result<Self, String> {
        if body.trim().is_empty() {
            return Err(format!("{}: the {name} region is empty", document.path));
        }
        Ok(Self {
            document,
            name,
            body,
        })
    }

    /// Folds the region's whitespace runs to single spaces.
    ///
    /// `mdtablefix --wrap` rewraps each document at its own column, so a phrase
    /// quoted in one and looked for in another arrives with different line
    /// breaks.
    fn folded(&self) -> String { self.body.split_whitespace().collect::<Vec<_>>().join(" ") }

    /// Every `transition.*` token in the region, deduplicated and sorted.
    ///
    /// Tokens are trimmed of surrounding punctuation so a sentence-final full
    /// stop does not become part of a name. A bare `transition.` prefix
    /// mention is not a field name and is ignored.
    fn tokens(&self) -> Vec<String> {
        let mut found: Vec<String> = Vec::new();
        for raw in self.body.split(|character: char| {
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

    /// The first cell of every table row in the region.
    ///
    /// The contract table's first column is the field name; a name that moved
    /// into the meaning or value cell would still satisfy a whole-row search,
    /// so the ADR check reads this column alone.
    fn first_column(&self) -> Vec<String> {
        self.body
            .lines()
            .filter(|line| line.trim_start().starts_with('|'))
            .filter_map(|line| {
                let cells = line.trim().trim_matches('|');
                let mut split = cells.split('|');
                let first = split.next()?.trim();
                // The header and separator rows carry no backticked field.
                first
                    .strip_prefix('`')
                    .and_then(|rest| rest.split_once('`'))
                    .map(|(name, _)| name.to_owned())
            })
            .collect()
    }

    /// Rejects a table whose first column is not exactly the contract.
    ///
    /// # Errors
    ///
    /// Returns a message naming the missing, unexpected, or duplicated field.
    fn require_first_column(&self) -> Result<(), String> {
        let path = self.document.path;
        let names = self.first_column();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        let mut expected = EXPECTED.map(str::to_owned);
        expected.sort();
        if sorted != expected {
            return Err(format!(
                "{path} field table first column is {names:?}; expected exactly the contracted \
                 fields in any order"
            ));
        }
        Ok(())
    }

    /// Rejects a region that omits a contracted name or invents another.
    ///
    /// # Errors
    ///
    /// Returns a message listing the contracted names the region does not
    /// mention, or the `transition.*` tokens that the contract does not publish.
    fn require_the_contract(&self) -> Result<(), String> {
        let path = self.document.path;
        let name = self.name;
        let missing: Vec<&str> = EXPECTED_SPANS
            .iter()
            .copied()
            .filter(|span| !self.body.contains(span))
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "{path} {name} does not name the contracted field(s) {missing:?}"
            ));
        }
        let unexpected: Vec<String> = self
            .tokens()
            .into_iter()
            .filter(|token| !EXPECTED.contains(&token.as_str()))
            .collect();
        if !unexpected.is_empty() {
            return Err(format!(
                "{path} {name} names {unexpected:?}, which the contract does not publish"
            ));
        }
        Ok(())
    }

    /// Requires the region to carry `phrase`, ignoring line breaks.
    ///
    /// # Errors
    ///
    /// Returns a message naming the region and the absent phrase.
    fn require_phrase(&self, phrase: &str) -> Result<(), String> {
        if self.folded().contains(phrase) {
            return Ok(());
        }
        Err(format!(
            "{} {} must still contain the quoted phrase {phrase:?}",
            self.document.path, self.name
        ))
    }
}

/// The names published by the `pub const` items in `src/fields.rs`.
///
/// A constant whose value is built by `concat!`, line-wrapped, or otherwise
/// spelled without a direct literal still has to appear here, so the check
/// counts every `pub const` item and then requires its field name to be one of
/// the contracted values. A sixth constant of any spelling therefore fails,
/// which a direct-literal parser would miss.
fn constant_names(source: &str) -> Vec<String> {
    let declarations = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub const "))
        .count();
    let literal_names: Vec<String> = source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub const "))
        .filter_map(|rest| rest.split_once(" = \""))
        .filter_map(|(_, value)| value.split_once('"'))
        .map(|(name, _)| name.to_owned())
        .collect();
    // Every declaration must also be readable as a field name. Comparing the
    // declaration count with the literal count catches a constant whose value
    // uses `concat!` or a line break, which the literal pass cannot read.
    if declarations != literal_names.len() {
        return literal_names;
    }
    literal_names
}

/// The ADR table's first column names exactly the contracted fields.
#[test]
fn adr_table_lists_the_contract() -> Result<(), String> {
    Section::between(ADR, "field table", TABLE_BEGIN, TABLE_END)?.require_first_column()
}

/// The design's §9 names exactly the contracted fields.
#[test]
fn design_names_the_contract() -> Result<(), String> {
    Section::level_two(DESIGN, "9. Observability")?.require_the_contract()
}

/// The users' guide section names exactly the contracted fields.
#[test]
fn users_guide_names_the_contract() -> Result<(), String> {
    Section::level_two(USERS_GUIDE, "Transition tracing fields")?.require_the_contract()
}

/// The glossary entry names exactly the contracted fields.
#[test]
fn glossary_names_the_contract() -> Result<(), String> {
    Section::level_two(CONTEXT, "Transition field contract")?.require_the_contract()
}

/// The exported constants are exactly the contracted names.
#[test]
fn constants_match_the_contract_table() {
    let source = FIELDS_SOURCE.text;
    let declarations = source
        .lines()
        .filter(|line| line.trim().starts_with("pub const "))
        .count();
    assert_eq!(
        declarations,
        EXPECTED.len(),
        "src/fields.rs must declare exactly five public constants"
    );
    assert_eq!(
        constant_names(source),
        EXPECTED.map(str::to_owned),
        "src/fields.rs must export exactly the contracted field names in declaration order"
    );
}

/// The design's §9 still carries the phrases other documents quote.
#[rstest]
#[case::public_api("public operational API")]
#[case::semver("semver-relevant")]
fn design_keeps_quoted_phrases(#[case] phrase: &str) {
    let section = Section::level_two(DESIGN, "9. Observability")
        .expect("the design must still have its observability section");
    section
        .require_phrase(phrase)
        .expect("the design must still quote the phrase");
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
