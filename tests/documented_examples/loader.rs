//! Shared parser for executable examples in published Markdown.

use std::{
    collections::{HashMap, HashSet},
    iter::{Enumerate, Peekable},
    str::Lines,
};

/// A user-facing Markdown document whose fenced examples are part of the test
/// contract.
#[derive(Clone, Copy)]
pub(super) struct Document<'source> {
    /// The repository-relative path used in diagnostics.
    pub(super) path: &'static str,
    /// The Markdown as compiled from the published source file.
    pub(super) markdown: &'source str,
}

/// A shell example parsed from its published Markdown document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Example {
    /// Stable identifier attached to the fence in the source document.
    pub(super) id: String,
    /// Repository-relative source document path.
    pub(super) document_path: &'static str,
    /// Exact language identifier on the opening fence.
    pub(super) language: String,
    /// Shell script inside the published fence.
    pub(super) command: String,
}

/// Documents whose examples are executable during the test suite.
pub(super) const USER_FACING_DOCUMENTS: [Document<'static>; 2] = [
    Document {
        path: "README.md",
        markdown: include_str!("../../README.md"),
    },
    Document {
        path: "docs/users-guide.md",
        markdown: include_str!("../../docs/users-guide.md"),
    },
];

/// Finds one published document by its repository-relative path.
///
/// # Errors
///
/// Returns an error when a behavioural scenario names a document outside the
/// explicit user-facing inventory.
pub(super) fn find_document(path: &str) -> Result<Document<'static>, String> {
    USER_FACING_DOCUMENTS
        .iter()
        .find(|document| document.path == path)
        .copied()
        .ok_or_else(|| format!("{path} is not in the user-facing example inventory"))
}

/// Parses every marked fenced example in a user-facing document.
///
/// # Errors
///
/// Returns an error for an unmarked or unterminated fence, an invalid or
/// missing identifier, duplicate identifiers within the document, or a
/// language other than `bash`.
pub(super) fn load_document(document: Document<'_>) -> Result<Vec<Example>, String> {
    parse_examples(document)
}

/// Parses marked fenced blocks and rejects malformed or duplicate markers.
///
/// # Errors
///
/// Returns a path- and line-specific diagnostic when a fence or marker is
/// malformed, or when an identifier appears more than once in the document.
fn parse_examples(document: Document<'_>) -> Result<Vec<Example>, String> {
    ExampleParser::new(document).parse()
}

/// Holds the state needed to parse one user-facing Markdown source.
struct ExampleParser<'source> {
    document: Document<'source>,
    lines: Peekable<Enumerate<Lines<'source>>>,
    examples: Vec<Example>,
    identifiers: HashSet<String>,
    pending_identifier: Option<String>,
}

impl<'source> ExampleParser<'source> {
    fn new(document: Document<'source>) -> Self {
        Self {
            document,
            lines: document.markdown.lines().enumerate().peekable(),
            examples: Vec::new(),
            identifiers: HashSet::new(),
            pending_identifier: None,
        }
    }

    fn parse(mut self) -> Result<Vec<Example>, String> {
        while let Some((line_index, text)) = self.lines.next() {
            self.parse_line(SourceLine::new(line_index, text))?;
        }
        self.finish()
    }

    fn parse_line(&mut self, line: SourceLine<'_>) -> Result<(), String> {
        if line.is_marker() {
            self.read_marker(line)
        } else if let Some(fence) = line.opening_fence() {
            self.read_fenced_example(line, fence)
        } else {
            self.reject_unrecognized_fence(line)
        }
    }

    fn read_marker(&mut self, line: SourceLine<'_>) -> Result<(), String> {
        if self.pending_identifier.is_some() {
            return Err(diagnostic(
                self.document.path,
                line,
                "a tested-example marker is not followed immediately by a fence",
            ));
        }
        self.pending_identifier = Some(self.parse_marker(line)?);
        Ok(())
    }

    fn read_fenced_example(&mut self, line: SourceLine<'_>, fence: Fence) -> Result<(), String> {
        let identifier = self.pending_identifier.take().ok_or_else(|| {
            diagnostic(
                self.document.path,
                line,
                "fenced block has no tested-example marker",
            )
        })?;
        if !self.identifiers.insert(identifier.clone()) {
            return Err(diagnostic(
                self.document.path,
                line,
                &format!("duplicate tested-example identifier `{identifier}`"),
            ));
        }
        let example = read_example(
            ExampleStart {
                document_path: self.document.path,
                line_number: line.number,
                identifier,
            },
            fence,
            &mut self.lines,
        )?;
        self.examples.push(example);
        Ok(())
    }

    fn reject_unrecognized_fence(&self, line: SourceLine<'_>) -> Result<(), String> {
        let message = if self.pending_identifier.is_some() {
            "a tested-example marker is not followed immediately by a fenced block"
        } else if line.looks_like_fence() {
            "fenced block is malformed or uses an unsupported delimiter"
        } else {
            return Ok(());
        };
        Err(diagnostic(self.document.path, line, message))
    }

    /// Parses and validates the identifier from a marker line.
    fn parse_marker(&self, line: SourceLine<'_>) -> Result<String, String> {
        line.trimmed_text()
            .strip_prefix("<!-- tested-example:")
            .and_then(|value| value.strip_suffix("-->"))
            .map(str::trim)
            .filter(|value| is_valid_identifier(value))
            .map(str::to_owned)
            .ok_or_else(|| {
                diagnostic(
                    self.document.path,
                    line,
                    "tested-example marker must contain a lowercase kebab-case identifier",
                )
            })
    }

    fn finish(self) -> Result<Vec<Example>, String> {
        if self.pending_identifier.is_some() {
            return Err(diagnostic(
                self.document.path,
                SourceLine::numbered(self.document.markdown.lines().count()),
                "tested-example marker is not followed by a fenced block",
            ));
        }
        Ok(self.examples)
    }
}

/// Reads one fenced block and preserves its command text for execution.
fn read_example(
    start: ExampleStart,
    fence: Fence,
    lines: &mut Peekable<Enumerate<Lines<'_>>>,
) -> Result<Example, String> {
    if fence.info != "bash" {
        return Err(diagnostic(
            start.document_path,
            SourceLine::numbered(start.line_number),
            &format!(
                "example `{}` must use the supported `bash` language, got `{}`",
                start.identifier, fence.info
            ),
        ));
    }

    let mut command_lines = Vec::new();
    loop {
        let Some((line_index, text)) = lines.next() else {
            return Err(diagnostic(
                start.document_path,
                SourceLine::numbered(start.line_number),
                &format!(
                    "example `{}` has an unterminated fenced block",
                    start.identifier
                ),
            ));
        };
        let line = SourceLine::new(line_index, text);
        if line.closes(&fence) {
            break;
        }
        command_lines.push(line.text);
    }

    Ok(Example {
        id: start.identifier,
        document_path: start.document_path,
        language: fence.info,
        command: command_lines.join("\n"),
    })
}

/// Checks that identifiers remain unique across the complete inventory.
///
/// # Errors
///
/// Returns the duplicate identifier and both source paths when documents
/// reuse a marker.
pub(super) fn ensure_unique_identifiers(examples: &[Example]) -> Result<(), String> {
    let mut seen = HashMap::new();
    for example in examples {
        if let Some(previous_path) = seen.insert(&example.id, example.document_path) {
            return Err(format!(
                "tested-example identifier `{}` appears in both {} and {}",
                example.id, previous_path, example.document_path
            ));
        }
    }
    Ok(())
}

/// Opening-fence details needed to recognize its matching closer.
struct Fence {
    delimiter: char,
    length: usize,
    info: String,
}

/// Source location and marker identity for one opening fence.
struct ExampleStart {
    document_path: &'static str,
    line_number: usize,
    identifier: String,
}

/// A numbered source line from a user-facing document.
#[derive(Clone, Copy)]
struct SourceLine<'source> {
    /// One-based line number used in diagnostics.
    number: usize,
    /// The line's text without its terminating newline.
    text: &'source str,
}

impl<'source> SourceLine<'source> {
    /// Builds a one-based source line from an iterator's zero-based index.
    fn new(index: usize, text: &'source str) -> Self {
        Self {
            number: index + 1,
            text,
        }
    }

    /// Names an existing position when no line text is available.
    fn numbered(number: usize) -> Self { Self { number, text: "" } }

    /// Returns the line with surrounding whitespace removed.
    fn trimmed_text(self) -> &'source str { self.text.trim() }

    /// Returns whether the line begins a tested-example marker.
    fn is_marker(self) -> bool { self.trimmed_text().starts_with("<!-- tested-example") }

    /// Parses a valid opening fence with up to three leading spaces.
    fn opening_fence(self) -> Option<Fence> {
        let content = self.content()?;
        let delimiter = content.chars().next()?;
        if delimiter != '`' && delimiter != '~' {
            return None;
        }
        let length = content
            .chars()
            .take_while(|character| *character == delimiter)
            .count();
        if length < 3 {
            return None;
        }
        let info = content.get(length..)?.trim().to_owned();
        if delimiter == '`' && info.contains('`') {
            return None;
        }
        Some(Fence {
            delimiter,
            length,
            info,
        })
    }

    /// Recognizes a line that looks like a fence but fails the supported syntax.
    fn looks_like_fence(self) -> bool {
        self.content().is_some_and(|content| {
            content.chars().next().is_some_and(|delimiter| {
                matches!(delimiter, '`' | '~')
                    && content
                        .chars()
                        .take_while(|character| *character == delimiter)
                        .count()
                        >= 3
            })
        })
    }

    /// Removes up to three leading spaces from a potential fence line.
    fn content(self) -> Option<&'source str> {
        let indentation = self.text.len() - self.text.trim_start_matches(' ').len();
        if indentation > 3 {
            return None;
        }
        self.text.get(indentation..)
    }

    /// Recognizes a closing fence with the same delimiter and at least its length.
    fn closes(self, fence: &Fence) -> bool {
        let Some(content) = self.content() else {
            return false;
        };
        let length = content
            .chars()
            .take_while(|character| *character == fence.delimiter)
            .count();
        content
            .get(length..)
            .is_some_and(|remaining| length >= fence.length && remaining.trim().is_empty())
    }
}

/// Accepts stable lower-case identifiers separated by single hyphens.
fn is_valid_identifier(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

/// Adds the source path and line number to a parser diagnostic.
fn diagnostic(path: &str, line: SourceLine<'_>, message: &str) -> String {
    format!("{path}:{}: {message}", line.number)
}

#[cfg(test)]
#[path = "loader_cases.rs"]
mod tests;
