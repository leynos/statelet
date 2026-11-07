//! Parser regression and property tests for the documented-example contract.

use proptest::{prelude::*, string::string_regex};
use rstest::rstest;

use super::{Document, Example, ensure_unique_identifiers, parse_examples};

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

/// Generates a valid lower-case kebab-case identifier.
fn identifier() -> impl Strategy<Value = String> {
    string_regex("[a-z0-9]{1,8}(-[a-z0-9]{1,8}){0,2}")
        .expect("the identifier pattern is a valid regular expression")
}

/// Generates command text that cannot begin a fence.
fn command_line() -> impl Strategy<Value = String> {
    string_regex("[!-~ ]{0,30}")
        .expect("the command-line pattern is a valid regular expression")
        .prop_filter("a command line must not begin a fence", |line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with('`') && !trimmed.starts_with('~')
        })
}

/// Generates an identifier the parser must reject.
fn invalid_identifier() -> impl Strategy<Value = String> {
    (
        "[a-z0-9]{1,4}",
        proptest::sample::select(vec!['A', 'Z', '_', ' ', '.', '/']),
        "[a-z0-9]{1,4}",
    )
        .prop_map(|(prefix, invalid, suffix)| format!("{prefix}{invalid}{suffix}"))
}

/// Wraps a command inside a marked Bash fence.
fn marked_example(id: &str, command: &str) -> String {
    format!("<!-- tested-example: {id} -->\n```bash\n{command}\n```\n")
}

proptest! {
    /// A generated valid marker and fence round-trip their identifier and text.
    #[test]
    fn parses_generated_marked_examples(
        id in identifier(),
        lines in prop::collection::vec(command_line(), 0..3),
    ) {
        let command = lines.join("\n");
        let markdown = marked_example(&id, &command);
        let examples = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect("a generated valid example should parse");
        prop_assert_eq!(examples.len(), 1);
        let parsed = examples
            .first()
            .map(|example| (example.id.as_str(), example.command.as_str()));
        prop_assert_eq!(parsed, Some((id.as_str(), command.as_str())));
    }

    /// A generated malformed identifier is rejected.
    #[test]
    fn rejects_generated_invalid_identifiers(id in invalid_identifier()) {
        let markdown = marked_example(&id, "echo ok");
        let error = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect_err("an invalid identifier should be rejected");
        prop_assert!(error.contains("identifier"), "unexpected error: {}", error);
    }

    /// A repeated identifier is rejected wherever it appears in the document.
    #[test]
    fn rejects_duplicate_generated_identifiers(id in identifier()) {
        let block = marked_example(&id, "echo run");
        let markdown = format!("{block}{block}");
        let error = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect_err("a duplicate identifier should be rejected");
        prop_assert!(error.contains("duplicate"), "unexpected error: {}", error);
    }

    /// A marker separated from its fence by prose is rejected.
    #[test]
    fn rejects_markers_detached_from_their_fence(
        id in identifier(),
        filler in command_line().prop_filter("filler must not be blank", |line| {
            !line.trim().is_empty()
        }),
    ) {
        let markdown = format!("<!-- tested-example: {id} -->\n{filler}\n```bash\necho ok\n```\n");
        let error = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect_err("a detached marker should be rejected");
        prop_assert!(error.contains("immediately"), "unexpected error: {}", error);
    }

    /// Every distinct example in a document is extracted with its own command.
    #[test]
    fn preserves_every_distinct_example(
        ids in prop::collection::hash_set(identifier(), 1..5),
    ) {
        let markdown: String = ids
            .iter()
            .map(|id| marked_example(id, &format!("echo {id}")))
            .collect();
        let examples = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect("distinct examples should parse");
        prop_assert_eq!(examples.len(), ids.len());
        for id in &ids {
            let expected = format!("echo {id}");
            prop_assert!(
                examples.iter().any(|example| &example.id == id && example.command == expected),
                "missing example for {}",
                id
            );
        }
    }
}
