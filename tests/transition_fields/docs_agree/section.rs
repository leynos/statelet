//! Extraction of the document regions the agreement checks compare.
//!
//! A `Section` is one region of one document with the name it is reported
//! under. Extraction returns an error rather than an empty region when a
//! heading or marker is missing, so a renamed section fails loudly instead of
//! comparing nothing with nothing.

use super::{Document, EXPECTED, EXPECTED_SPANS, heading_rank, section_body};

/// Whether a first cell is a Markdown table separator (`---`/`:--:`).
fn is_separator(cell: &str) -> bool {
    !cell.is_empty()
        && cell
            .chars()
            .all(|character| matches!(character, '-' | ':' | ' '))
}

/// One extracted region of a document, with the name it is reported under.
#[derive(Debug, Clone, Copy)]
pub(super) struct Section<'a> {
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
    pub(super) fn between(
        document: Document,
        name: &'a str,
        begin: &str,
        end: &str,
    ) -> Result<Self, String> {
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

    /// The body of the section whose ATX heading is exactly `heading`.
    ///
    /// The body ends at the next heading of the same or higher rank. Matching
    /// the whole heading line, and bounding by rank rather than by the next
    /// `## `, keeps a level-three section from swallowing its siblings: a name
    /// moved from §9.1 into §9.2, or from one glossary entry into the next,
    /// must leave the checked region rather than remain visible to it.
    ///
    /// # Errors
    ///
    /// Returns a message naming the missing heading when no line is that exact
    /// heading, and a message when the section body is empty.
    pub(super) fn heading(document: Document, heading: &'a str) -> Result<Self, String> {
        let rank = heading_rank(heading)
            .ok_or_else(|| format!("{}: {heading:?} is not an ATX heading", document.path))?;
        let body = section_body(document.text, heading, rank)
            .ok_or_else(|| format!("{}: heading {heading:?} is missing", document.path))?;
        Self::require_non_empty(document, heading, body)
    }

    /// Accepts the region, rejecting an empty one.
    ///
    /// A section that exists but says nothing would compare "no names" with
    /// "no names" and pass, so emptiness is an error rather than a result.
    pub(super) fn require_non_empty(
        document: Document,
        name: &'a str,
        body: &'a str,
    ) -> Result<Self, String> {
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
    pub(super) fn folded(&self) -> String {
        self.body.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Every `transition.*` token in the region, deduplicated and sorted.
    ///
    /// Tokens are trimmed of surrounding punctuation so a sentence-final full
    /// stop does not become part of a name. A bare `transition.` prefix
    /// mention is not a field name and is ignored.
    pub(super) fn tokens(&self) -> Vec<String> {
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
    pub(super) fn first_column(&self) -> Result<Vec<String>, String> {
        let path = self.document.path;
        let mut names = Vec::new();
        for (row, line) in self
            .body
            .lines()
            .filter(|line| line.trim_start().starts_with('|'))
            .enumerate()
        {
            let cells = line.trim().trim_matches('|');
            let first = cells.split('|').next().unwrap_or("").trim();
            // Row zero is the header and a separator row is dashes only.
            if row == 0 || is_separator(first) {
                continue;
            }
            let name = first
                .strip_prefix('`')
                .and_then(|rest| rest.split_once('`'))
                .map(|(name, _)| name.to_owned());
            match name {
                Some(field) => names.push(field),
                None => {
                    return Err(format!(
                        concat!(
                            "{} table row {} has the unbackticked first cell {:?}; ",
                            "every field row must name its field in backticks"
                        ),
                        path, row, first
                    ));
                }
            }
        }
        Ok(names)
    }

    /// Rejects a table whose first column is not exactly the contract.
    ///
    /// # Errors
    ///
    /// Returns a message naming the missing, unexpected, or duplicated field.
    pub(super) fn require_first_column(&self) -> Result<(), String> {
        let path = self.document.path;
        let names = self.first_column()?;
        let mut sorted = names.clone();
        sorted.sort();
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
    pub(super) fn require_the_contract(&self) -> Result<(), String> {
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
    pub(super) fn require_phrase(&self, phrase: &str) -> Result<(), String> {
        if self.folded().contains(phrase) {
            return Ok(());
        }
        Err(format!(
            "{} {} must still contain the quoted phrase {phrase:?}",
            self.document.path, self.name
        ))
    }
}
