//! Shared parser and runner for executable examples in published Markdown.

use std::{
    collections::{HashMap, HashSet},
    iter::{Enumerate, Peekable},
    process::Command,
    str::Lines,
};

/// A user-facing Markdown document whose fenced examples are part of the test
/// contract.
#[derive(Clone, Copy)]
pub(super) struct Document {
    /// The repository-relative path used in diagnostics.
    pub(super) path: &'static str,
    /// The Markdown as compiled from the published source file.
    pub(super) markdown: &'static str,
}

/// A shell example parsed from its published Markdown document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Example {
    /// Stable identifier attached to the fence in the source document.
    pub(super) id: String,
    /// Repository-relative source document path.
    pub(super) document_path: &'static str,
    /// Exact language identifier on the opening fence.
    language: String,
    /// Shell script inside the published fence.
    command: String,
}

/// Documents whose examples are executable during the test suite.
pub(super) const USER_FACING_DOCUMENTS: [Document; 2] = [
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
pub(super) fn find_document(path: &str) -> Result<Document, String> {
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
pub(super) fn load_document(document: Document) -> Result<Vec<Example>, String> {
    parse_examples(document.path, document.markdown)
}

/// Parses marked fenced blocks and rejects malformed or duplicate markers.
///
/// # Errors
///
/// Returns a path- and line-specific diagnostic when a fence or marker is
/// malformed, or when an identifier appears more than once in the document.
fn parse_examples(path: &'static str, markdown: &str) -> Result<Vec<Example>, String> {
    let mut examples = Vec::new();
    let mut identifiers = HashSet::new();
    let mut pending_identifier = None;
    let mut lines = markdown.lines().enumerate().peekable();

    while let Some((line_index, line)) = lines.next() {
        let line_number = line_index + 1;

        if is_marker_line(line) {
            if pending_identifier.is_some() {
                return Err(diagnostic(
                    path,
                    line_number,
                    "a tested-example marker is not followed immediately by a fence",
                ));
            }
            pending_identifier = Some(parse_marker(path, line_number, line)?);
            continue;
        }

        if let Some(fence) = opening_fence(line) {
            let identifier = pending_identifier.take().ok_or_else(|| {
                diagnostic(
                    path,
                    line_number,
                    "fenced block has no tested-example marker",
                )
            })?;
            if !identifiers.insert(identifier.clone()) {
                return Err(diagnostic(
                    path,
                    line_number,
                    &format!("duplicate tested-example identifier `{identifier}`"),
                ));
            }
            examples.push(read_example(
                ExampleStart {
                    document_path: path,
                    line_number,
                    identifier,
                },
                fence,
                &mut lines,
            )?);
        } else if pending_identifier.is_some() {
            return Err(diagnostic(
                path,
                line_number,
                "a tested-example marker is not followed immediately by a fenced block",
            ));
        } else if looks_like_fence(line) {
            return Err(diagnostic(
                path,
                line_number,
                "fenced block is malformed or uses an unsupported delimiter",
            ));
        }
    }

    if pending_identifier.is_some() {
        return Err(diagnostic(
            path,
            markdown.lines().count(),
            "tested-example marker is not followed by a fenced block",
        ));
    }

    Ok(examples)
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
            start.line_number,
            &format!(
                "example `{}` must use the supported `bash` language, got `{}`",
                start.identifier, fence.info
            ),
        ));
    }

    let mut command_lines = Vec::new();
    loop {
        let Some((_, line)) = lines.next() else {
            return Err(diagnostic(
                start.document_path,
                start.line_number,
                &format!(
                    "example `{}` has an unterminated fenced block",
                    start.identifier
                ),
            ));
        };
        if closes_fence(line, &fence) {
            break;
        }
        command_lines.push(line);
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

/// Executes each published Bash example from the checkout root.
///
/// # Errors
///
/// Returns an error if the shell cannot be started or if an example exits
/// unsuccessfully. The diagnostic includes captured standard output and error.
pub(super) fn run_examples(examples: &[Example]) -> Result<(), String> {
    for example in examples {
        if example.language != "bash" {
            return Err(format!(
                "{} example `{}` uses unsupported language `{}`",
                example.document_path, example.id, example.language
            ));
        }
        let output = Command::new("bash")
            .args(["-e", "-u", "-o", "pipefail", "-c"])
            .arg(&example.command)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .map_err(|error| {
                format!(
                    "could not run {} example `{}`: {error}",
                    example.document_path, example.id
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "{} example `{}` exited with {}\nstdout:\n{}\nstderr:\n{}",
                example.document_path,
                example.id,
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
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

/// Returns whether a line begins a tested-example marker.
fn is_marker_line(line: &str) -> bool { line.trim().starts_with("<!-- tested-example") }

/// Parses and validates the identifier from a marker line.
fn parse_marker(path: &str, line_number: usize, line: &str) -> Result<String, String> {
    let marker = line.trim();
    let identifier = marker
        .strip_prefix("<!-- tested-example:")
        .and_then(|value| value.strip_suffix("-->"))
        .map(str::trim)
        .filter(|value| is_valid_identifier(value))
        .ok_or_else(|| {
            diagnostic(
                path,
                line_number,
                "tested-example marker must contain a lowercase kebab-case identifier",
            )
        })?;
    Ok(identifier.to_owned())
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

/// Parses a valid opening fence with up to three leading spaces.
fn opening_fence(line: &str) -> Option<Fence> {
    let content = fence_content(line)?;
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

/// Recognizes lines that look like fences but fail the supported syntax.
fn looks_like_fence(line: &str) -> bool {
    fence_content(line).is_some_and(|content| {
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
fn fence_content(line: &str) -> Option<&str> {
    let indentation = line.len() - line.trim_start_matches(' ').len();
    if indentation > 3 {
        return None;
    }
    line.get(indentation..)
}

/// Recognizes a closing fence with the same delimiter and at least its length.
fn closes_fence(line: &str, fence: &Fence) -> bool {
    let Some(content) = fence_content(line) else {
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

/// Adds the source path and line number to a parser diagnostic.
fn diagnostic(path: &str, line_number: usize, message: &str) -> String {
    format!("{path}:{line_number}: {message}")
}

#[cfg(test)]
#[path = "loader_cases.rs"]
mod tests;
