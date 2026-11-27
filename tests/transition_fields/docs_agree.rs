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

use self::section::Section;

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

/// The rank of an ATX heading line, or `None` for any other line.
///
/// A rank of one is `#`, two is `##`, and so on. Leading indentation of up to
/// three spaces is permitted, matching Markdown's ATX rule.
fn heading_rank(line: &str) -> Option<usize> {
    let indent = line
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    if indent > 3 {
        return None;
    }
    let hashes = line
        .chars()
        .skip(indent)
        .take_while(|character| *character == '#')
        .count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = line
        .char_indices()
        .nth(indent + hashes)
        .map(|(_, character)| character);
    if rest.is_none_or(|character| character == ' ') {
        Some(hashes)
    } else {
        None
    }
}

/// The body of the section headed exactly `heading`, at `rank`.
///
/// Returns `None` when no line is that heading. The body runs from the line
/// after it to the next line whose rank is `rank` or higher, so a level-three
/// section stops at its section's next heading and a level-two section stops at
/// the next level-two or level-one heading.
fn section_body<'a>(text: &'a str, heading: &str, rank: usize) -> Option<&'a str> {
    let mut begin = None;
    let mut offset = 0_usize;
    for line in text.split_inclusive('\n') {
        let content = line.trim_end_matches('\n');
        if begin.is_none() {
            if content.trim_end() == heading {
                begin = Some(offset + line.len());
            }
        } else if heading_rank(content).is_some_and(|next| next <= rank) {
            return text.get(begin?..offset);
        }
        offset += line.len();
    }
    text.get(begin?..)
}

#[path = "docs_agree/section.rs"]
mod section;

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
    Section::heading(DESIGN, "### 9.1 Field contract")?.require_the_contract()
}

/// The users' guide section names exactly the contracted fields.
#[test]
fn users_guide_names_the_contract() -> Result<(), String> {
    Section::heading(USERS_GUIDE, "## Transition tracing fields")?.require_the_contract()
}

/// The glossary entry names exactly the contracted fields.
#[test]
fn glossary_names_the_contract() -> Result<(), String> {
    Section::heading(CONTEXT, "### Transition field contract")?.require_the_contract()
}

/// Whether the first-column check rejects `text` as a marked table.
///
/// Both synthetic cases share this runner so a change to the table grammar or
/// the markers touches one place rather than two nearly identical tests.
fn first_column_rejects(text: &'static str) -> bool {
    let document = Document {
        path: "synthetic",
        text,
    };
    Section::between(document, "field table", "<!-- begin -->", "<!-- end -->")
        .and_then(|section| section.require_first_column())
        .is_err()
}

/// A synthetic table whose last row repeats `transition.name`.
const DUPLICATE_ROW_TABLE: &str = concat!(
    "<!-- begin -->\n",
    "| Field | Meaning |\n",
    "| ----- | ------- |\n",
    "| `transition.name` | first |\n",
    "| `transition.state.before` | second |\n",
    "| `transition.event` | third |\n",
    "| `transition.outcome` | fourth |\n",
    "| `transition.error` | fifth |\n",
    "| `transition.name` | duplicate |\n",
    "<!-- end -->\n",
);

/// A synthetic table whose last row names a field without backticks.
const UNBACKTICKED_ROW_TABLE: &str = concat!(
    "<!-- begin -->\n",
    "| Field | Meaning |\n",
    "| ----- | ------- |\n",
    "| `transition.name` | first |\n",
    "| `transition.state.before` | second |\n",
    "| `transition.event` | third |\n",
    "| `transition.outcome` | fourth |\n",
    "| `transition.error` | fifth |\n",
    "| transition.after | sixth, unbackticked |\n",
    "<!-- end -->\n",
);

/// A duplicated first-column row is rejected.
#[test]
fn a_duplicate_table_row_is_rejected() {
    assert!(
        first_column_rejects(DUPLICATE_ROW_TABLE),
        "a duplicate first-column row must be rejected, but the check accepted it"
    );
}

/// A field row without backticks is rejected, not silently ignored.
#[test]
fn an_unbackticked_table_row_is_rejected() {
    assert!(
        first_column_rejects(UNBACKTICKED_ROW_TABLE),
        "an unbackticked field row must be rejected, but the check accepted it"
    );
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
    let section = Section::heading(DESIGN, "### 9.1 Field contract")
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
