//! Fence and line grammar shared by the documented-example parser.
//!
//! Markdown fences are recognized here so the parser can concentrate on the
//! tested-example protocol: an opening fence, its matching closer, and the
//! command text between them.

/// Opening-fence details needed to recognize its matching closer.
pub(super) struct Fence {
    delimiter: char,
    length: usize,
    /// Language identifier written after the fence delimiter.
    pub(super) info: String,
}

/// A numbered source line from a user-facing document.
#[derive(Clone, Copy)]
pub(super) struct SourceLine<'source> {
    /// One-based line number used in diagnostics.
    pub(super) number: usize,
    /// The line's text without its terminating newline.
    pub(super) text: &'source str,
}

impl<'source> SourceLine<'source> {
    /// Builds a one-based source line from an iterator's zero-based index.
    pub(super) fn new(index: usize, text: &'source str) -> Self {
        Self {
            number: index + 1,
            text,
        }
    }

    /// Names an existing position when no line text is available.
    pub(super) fn numbered(number: usize) -> Self { Self { number, text: "" } }

    /// Returns the line with surrounding whitespace removed.
    pub(super) fn trimmed_text(self) -> &'source str { self.text.trim() }

    /// Returns whether the line begins a tested-example marker.
    pub(super) fn is_marker(self) -> bool { self.trimmed_text().starts_with("<!-- tested-example") }

    /// Parses a valid opening fence with up to three leading spaces.
    pub(super) fn opening_fence(self) -> Option<Fence> {
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
    pub(super) fn looks_like_fence(self) -> bool {
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
    pub(super) fn closes(self, fence: &Fence) -> bool {
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
