//! Property tests for the documented-example fence grammar.

use proptest::{prelude::*, string::string_regex};

use crate::loader::{Document, parse_examples};

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

/// Generates command text that cannot close a triple-backtick Bash fence.
///
/// Unlike [`command_line`], this keeps tilde lines, so fence-like text that
/// the closer rules must reject stays inside the generated command.
fn bash_command_line() -> impl Strategy<Value = String> {
    string_regex("[!-~ ]{0,30}")
        .expect("the command-line pattern is a valid regular expression")
        .prop_filter("a backtick fence line would close the block", |line| {
            !line.trim_start().starts_with("```")
        })
}

/// Generates one of the two delimiters a fenced block may use.
fn fence_delimiter() -> impl Strategy<Value = char> { proptest::sample::select(vec!['`', '~']) }

/// Generates an opening fence as its delimiter and its run length.
fn opening_fence() -> impl Strategy<Value = (char, usize)> { (fence_delimiter(), 3usize..7) }

/// Generates a closer that legally closes `fence`: the same delimiter, at
/// least as long as the opener, indented by at most three spaces, and padded
/// only with whitespace.
fn legal_closer(fence: (char, usize)) -> impl Strategy<Value = String> {
    let (delimiter, opener_length) = fence;
    (opener_length..opener_length + 4, 0usize..4, "[ \t]{0,4}").prop_map(
        move |(closer_length, indentation, padding)| {
            format!(
                "{}{}{padding}",
                " ".repeat(indentation),
                delimiter.to_string().repeat(closer_length)
            )
        },
    )
}

/// Generates a fence-like line that must not close `fence`, because it uses
/// another delimiter, a shorter run, or trailing non-whitespace.
fn non_closing_line(fence: (char, usize)) -> impl Strategy<Value = String> {
    let (delimiter, opener_length) = fence;
    let other_delimiter = if delimiter == '`' { '~' } else { '`' };
    prop_oneof![
        (3usize..7).prop_map(move |length| other_delimiter.to_string().repeat(length)),
        (1usize..opener_length).prop_map(move |length| delimiter.to_string().repeat(length)),
        (opener_length..opener_length + 3).prop_map(move |length| format!(
            "{} not-a-close",
            delimiter.to_string().repeat(length)
        )),
    ]
}

/// Wraps generated body lines in a marked example with the chosen fence and
/// closer.
fn fenced_example(id: &str, fence: (char, usize), body: &[String], closer: &str) -> String {
    let (delimiter, length) = fence;
    let opener = format!("{}bash", delimiter.to_string().repeat(length));
    let mut markdown = format!("<!-- tested-example: {id} -->\n{opener}\n");
    for line in body {
        markdown.push_str(line);
        markdown.push('\n');
    }
    markdown.push_str(closer);
    markdown.push('\n');
    markdown
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
        lines in prop::collection::vec(bash_command_line(), 0..3),
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

    /// A legal closer ends the block and leaves the command text unchanged.
    #[test]
    fn closes_generated_fences_with_legal_closers(
        case in opening_fence().prop_flat_map(|fence| {
            (
                Just(fence),
                prop::collection::vec(
                    prop_oneof![command_line(), non_closing_line(fence)],
                    0..3,
                ),
                legal_closer(fence),
            )
        }),
    ) {
        let (fence, body, closer) = case;
        let markdown = fenced_example("fence-case", fence, &body, &closer);
        let examples = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect("a legal closer should end the block");
        prop_assert_eq!(examples.len(), 1);
        let command = body.join("\n");
        let parsed = examples.first().map(|example| example.command.as_str());
        prop_assert_eq!(parsed, Some(command.as_str()));
    }

    /// Fence-like lines that cannot close the block stay in the command text.
    #[test]
    fn keeps_generated_fence_like_lines_in_the_command(
        case in opening_fence().prop_flat_map(|fence| {
            (
                Just(fence),
                prop::collection::vec(non_closing_line(fence), 1..4),
                legal_closer(fence),
            )
        }),
    ) {
        let (fence, body, closer) = case;
        let markdown = fenced_example("fence-case", fence, &body, &closer);
        let examples = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect("fence-like lines should not end the block");
        prop_assert_eq!(examples.len(), 1);
        let command = body.join("\n");
        let parsed = examples.first().map(|example| example.command.as_str());
        prop_assert_eq!(parsed, Some(command.as_str()));
    }

    /// A block without a legal closer is reported as unterminated.
    #[test]
    fn rejects_generated_blocks_without_a_closer(
        case in opening_fence().prop_flat_map(|fence| {
            (
                Just(fence),
                prop::collection::vec(
                    prop_oneof![command_line(), non_closing_line(fence)],
                    0..3,
                ),
            )
        }),
    ) {
        let (fence, body) = case;
        let (delimiter, length) = fence;
        let opener = format!("{}bash", delimiter.to_string().repeat(length));
        let mut markdown = format!("<!-- tested-example: fence-case -->\n{opener}\n");
        for line in &body {
            markdown.push_str(line);
            markdown.push('\n');
        }
        let error = parse_examples(Document { path: "fixture.md", markdown: &markdown })
            .expect_err("a block without a closer should be rejected");
        prop_assert!(error.contains("unterminated"), "unexpected error: {}", error);
    }
}
