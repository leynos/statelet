//! Parser regression tests for the documented-example contract.

use rstest::rstest;

use super::{
    Document,
    Example,
    ensure_unique_identifiers,
    find_document,
    load_document,
    load_non_empty_document,
    parse_examples,
};

/// Parses a fixture document that must contain exactly one example.
fn parse_single(markdown: &str) -> Example {
    let mut examples = parse_examples(Document {
        path: "fixture.md",
        markdown,
    })
    .expect("the fixture document should contain a parseable example");
    assert_eq!(examples.len(), 1, "expected exactly one example");
    examples
        .pop()
        .expect("the parser reported exactly one example")
}

/// Loads each published document through the public loader entry point and
/// compares its example inventory with the published text.
#[rstest]
#[case::readme(
    "README.md",
    vec![
        ("readme-quick-start-targets", "make help"),
        (
            "readme-review-the-roadmap",
            "make help\nsed -n '1,120p' docs/roadmap.md"
        ),
    ]
)]
#[case::users_guide(
    "docs/users-guide.md",
    vec![("users-guide-validation-targets", "make help")]
)]
fn loads_each_published_document(
    #[case] path: &'static str,
    #[case] expected: Vec<(&'static str, &'static str)>,
) -> Result<(), String> {
    let examples = load_non_empty_document(find_document(path)?)?;
    let actual: Vec<(&str, &str)> = examples
        .iter()
        .map(|example| (example.id.as_str(), example.command.as_str()))
        .collect();
    if actual != expected {
        return Err(format!(
            "{path} published {actual:?} instead of {expected:?}"
        ));
    }
    Ok(())
}

/// Reports malformed Markdown through the public loader entry point.
#[test]
fn load_document_reports_malformed_markdown() {
    let document = Document {
        path: "fixture.md",
        markdown: "```bash\nprintf 'ok'\n```\n",
    };
    let error = load_document(document).expect_err("an unmarked fence should be rejected");
    assert!(
        error.contains("fixture.md:1"),
        "the diagnostic should name the opening line: {error}"
    );
    assert!(
        error.contains("no tested-example marker"),
        "unexpected error: {error}"
    );
}

/// Rejects a published document whose example inventory is empty.
#[test]
fn rejects_a_published_document_without_examples() {
    let document = Document {
        path: "fixture.md",
        markdown: "# A document with no examples\n",
    };
    let error = load_non_empty_document(document)
        .expect_err("an empty example inventory should be rejected");
    assert_eq!(error, "fixture.md publishes no tested-example blocks");
}

/// Parses a marked Bash block from its source text.
#[test]
fn parses_marked_bash_example() {
    let example =
        parse_single("<!-- tested-example: stable-example -->\n```bash\nprintf 'ok\\n'\n```\n");
    assert_eq!(example.id, "stable-example");
    assert_eq!(example.command, "printf 'ok\\n'");
}

/// Ignores inline code spans that begin a prose line.
#[test]
fn ignores_inline_code_spans() {
    let examples = parse_examples(Document {
        path: "fixture.md",
        markdown: "`typed-fsm` is a state-machine crate.\n",
    })
    .expect("inline code spans should not be treated as fenced blocks");
    assert_eq!(examples.len(), 0);
}

/// Rejects missing, malformed, duplicated, or detached example markers.
#[rstest]
#[case("```bash\nprintf 'ok'\n```", "no tested-example marker")]
#[case(
    "<!-- tested-example: valid-id -->\n```bash\nprintf 'ok'",
    "unterminated"
)]
#[case("<!-- tested-example:  -->\n```bash\nprintf 'ok'\n```", "identifier")]
#[case(
    "<!-- tested-example: Bad_ID -->\n```bash\nprintf 'ok'\n```",
    "identifier"
)]
#[case(
    "<!-- tested-example: same -->\n```bash\na\n```\n<!-- tested-example: same \
     -->\n```bash\nb\n```",
    "duplicate"
)]
#[case("<!-- tested-example: detached -->\n\n```bash\na\n```", "immediately")]
#[case(
    "<!-- tested-example: no-language -->\n```\na\n```",
    "supported `bash`"
)]
fn rejects_invalid_examples(#[case] markdown: &'static str, #[case] expected: &str) {
    let error = parse_examples(Document {
        path: "fixture.md",
        markdown,
    })
    .expect_err("invalid Markdown example should be rejected");
    assert!(error.contains(expected), "unexpected error: {error}");
}

/// Keeps lines that only resemble a closing fence inside the example.
#[rstest]
#[case("shorter fence", "```bash\necho before\n``\necho after\n```")]
#[case("different delimiter", "```bash\necho before\n~~~\necho after\n```")]
#[case(
    "trailing text after a closer",
    "```bash\necho before\n``` not a close\necho after\n```"
)]
fn keeps_non_closing_lines_in_the_command(#[case] description: &str, #[case] body: &str) {
    let markdown = format!("<!-- tested-example: fence-case -->\n{body}\n");
    let example = parse_single(&markdown);
    let expected = body
        .strip_prefix("```bash\n")
        .and_then(|rest| rest.strip_suffix("\n```"))
        .expect("each case body is a marked Bash fence");
    assert_eq!(example.command, expected, "the case {description}");
}

/// Accepts a tilde fence and closes it with a matching tilde line.
#[test]
fn parses_tilde_fenced_example() {
    let example = parse_single("<!-- tested-example: tilde -->\n~~~bash\nprintf 'ok\\n'\n~~~\n");
    assert_eq!(example.id, "tilde");
    assert_eq!(example.command, "printf 'ok\\n'");
}

/// Builds an example with a chosen identifier and source document.
fn example(id: &str, document_path: &'static str) -> Example {
    Example {
        id: id.to_owned(),
        document_path,
        language: "bash".to_owned(),
        command: String::new(),
    }
}

/// Rejects an identifier reused by a different document.
#[test]
fn rejects_duplicate_identifiers_across_documents() {
    let examples = [example("shared", "one.md"), example("shared", "two.md")];
    let error =
        ensure_unique_identifiers(&examples).expect_err("a cross-document duplicate is invalid");
    assert!(
        error.contains("both one.md and two.md"),
        "unexpected error: {error}"
    );
}

/// Accepts distinct identifiers across documents.
#[test]
fn accepts_distinct_identifiers_across_documents() {
    let examples = [example("first", "one.md"), example("second", "two.md")];
    ensure_unique_identifiers(&examples).expect("distinct identifiers are valid");
}

#[path = "loader_property_cases.rs"]
mod properties;
