//! Parser regression tests for the documented-example contract.

use rstest::rstest;

use super::parse_examples;

/// Parses a marked Bash block from its source text.
#[test]
fn parses_marked_bash_example() {
    let examples = parse_examples(
        "fixture.md",
        "<!-- tested-example: stable-example -->\n```bash\nprintf 'ok\\n'\n```\n",
    )
    .expect("the marked Bash example should parse");
    assert_eq!(examples.len(), 1);
    let example = examples
        .first()
        .expect("the parser should return one example");
    assert_eq!(example.id, "stable-example");
    assert_eq!(example.command, "printf 'ok\\n'");
}

/// Ignores inline code spans that begin a prose line.
#[test]
fn ignores_inline_code_spans() {
    let examples = parse_examples("fixture.md", "`typed-fsm` is a state-machine crate.\n")
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
fn rejects_invalid_examples(#[case] markdown: &str, #[case] expected: &str) {
    let error = parse_examples("fixture.md", markdown)
        .expect_err("invalid Markdown example should be rejected");
    assert!(error.contains(expected), "unexpected error: {error}");
}
