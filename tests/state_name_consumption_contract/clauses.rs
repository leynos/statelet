//! Resolution of the source clauses ADR 004 quotes.
//!
//! The rule ADR 004 records rests on three statements in other documents. Each
//! is quoted in its evidence section and must still resolve, whitespace-folded,
//! within its named section of its source. Folding is mandatory rather than
//! cosmetic: `mdtablefix --wrap` rewraps ADR 004's prose at different break
//! points from its sources, and clauses that resolve in the source do not
//! resolve unfolded.

use super::types::{ParseError, Register};

/// Where each supported source document stands in for a clause's attribution.
///
/// The first element is the name ADR 004 uses in its attribution — `design`,
/// not `docs/design.md` — because that is what the resolver reads out of the
/// clause. The second is the file the clause is checked against. Comparing the
/// attribution against the path would reject every well-formed clause.
const SOURCES: [(&str, &str); 3] = [
    ("design", "docs/design.md"),
    ("roadmap", "docs/roadmap.md"),
    ("adr-002", "docs/adr-002-transition-boundary-scope.md"),
];

/// Checks that each quoted clause still resolves in its named section.
///
/// Failures name the *file* the clause was quoted from, not the attribution word
/// the ADR uses for it: a reader told only that "design" has drifted has to
/// guess which file to open, and the two differ by a path.
pub(crate) fn check_quoted_clauses(
    adr: &str,
    design: &str,
    roadmap: &str,
    adr_002: &str,
) -> Result<(), String> {
    let sources = [design, roadmap, adr_002];
    for clause in quoted_clauses(adr)? {
        let (document, section, quoted) = resolve_clause(&clause)?;
        let Some(index) = SOURCES.iter().position(|(name, _)| *name == document) else {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md attributes a clause to \
                 {document:?}, which this contract does not read. Repair: use design, roadmap or \
                 adr-002."
            ));
        };
        let path = SOURCES.get(index).map_or("", |(_, path)| *path);
        let source = sources.get(index).copied().unwrap_or_default();
        let Some(body) = locate_section(source, &section) else {
            return Err(format!(
                "{path} has no section {section:?}. Repair: restore it, or revise ADR 004's \
                 quoted clause."
            ));
        };
        if !fold_whitespace(body).contains(&quoted) {
            return Err(format!(
                "{path} no longer contains the quoted clause {quoted:?} under {section:?}. \
                 Repair: update ADR 004 and its contract together."
            ));
        }
    }
    Ok(())
}

/// The body of the named section: what follows its heading, up to the next
/// heading of the same or higher rank.
///
/// The section is named in ADR 004 as rendered prose — `6.1 State naming`,
/// `wireframe: primary proving ground` — while the source may carry it as a
/// heading that wraps a word in a code span and prefixes it with `#` marks.
/// Comparing the two therefore normalizes each source line to plain text and
/// looks for the section name as a substring of it.
///
/// Matching on the heading's *rendered* text rather than on its literal source
/// line is what lets a source mark a word as code without the ADR having to
/// reproduce the backticks inside a code span of its own, which `MD038` forbids
/// and which would nest two spans.
///
/// The body is bounded by rank rather than by the next top-level heading, so
/// that a clause quoted from `### 6.1` cannot be satisfied by text that has
/// drifted into `### 6.2`. A match on a line that is not a heading at all — the
/// roadmap states one clause's location as a list item — is bounded by the next
/// `##`.
fn locate_section<'a>(source: &'a str, section: &str) -> Option<&'a str> {
    let mut start = 0;
    let mut found = None;
    let mut rank = 0;
    for line in source.split_inclusive('\n') {
        if let Some(body_start) = found {
            if heading_rank(line).is_some_and(|level| level <= rank) {
                return source.get(body_start..start);
            }
        } else if plain_text(line).contains(section) {
            found = Some(start + line.len());
            rank = heading_rank(line).unwrap_or(PROSE_RANK);
        }
        start += line.len();
    }
    found.and_then(|body_start| source.get(body_start..))
}

/// The rank a match on prose inherits: it is bounded by the next `##`.
const PROSE_RANK: usize = 2;

/// The rank of a Markdown ATX heading, or `None` for any other line.
///
/// A leading `#` alone is not enough. `docs/design.md` carries
/// `#[derive(StateName)]` at column zero inside a fence, and an ATX heading
/// requires whitespace after its hashes.
fn heading_rank(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let hashes = trimmed.bytes().take_while(|byte| *byte == b'#').count();
    (trimmed.as_bytes().get(hashes) == Some(&b' ')).then_some(hashes)
}

/// Strips a line's heading marks and code spans, leaving its rendered text.
fn plain_text(line: &str) -> String {
    line.trim_start_matches('#')
        .trim()
        .replace('`', "")
        .trim()
        .to_owned()
}

/// The clauses ADR 004's evidence section quotes, as it renders them.
///
/// Each clause is one italic *block* rather than one line. `mdtablefix --wrap`
/// rewraps prose at the column limit, so a clause long enough to be worth
/// quoting spans several lines once the document has been formatted, and a
/// collector keyed on a leading `*` would silently take only the first line of
/// each — a check that then passes over half a clause. Blocks are therefore
/// separated by blank lines and folded after collection.
///
/// An empty list is a failure rather than a vacuous pass: a check over zero
/// clauses would agree with an ADR whose evidence section had been emptied.
fn quoted_clauses(adr: &str) -> Result<Vec<String>, String> {
    let Some((_, after_heading)) = adr.split_once(Register::Evidence.section()) else {
        return Err(
            "docs/adr-004-state-name-consumption-evidence.md lacks its evidence section. Repair: \
             restore the quoted source clauses."
                .to_owned(),
        );
    };
    let body = after_heading
        .split_once("\n## ")
        .map_or(after_heading, |(body, _)| body);
    let missing = || {
        ParseError::MissingDelimiters {
            register: Register::Evidence,
        }
        .to_string()
    };
    let (_, after_begin) = body
        .split_once(Register::Evidence.begin())
        .ok_or_else(missing)?;
    let (block, _) = after_begin
        .split_once(Register::Evidence.end())
        .ok_or_else(missing)?;
    let clauses = block
        .split("\n\n")
        .map(fold_whitespace)
        .filter_map(|paragraph| {
            paragraph
                .strip_prefix('*')
                .and_then(|inner| inner.strip_suffix('*'))
                .map(str::to_owned)
        })
        .collect::<Vec<String>>();
    if clauses.is_empty() {
        return Err(
            "docs/adr-004-state-name-consumption-evidence.md cites no clauses. Repair: quote each \
             load-bearing source clause as an italic line in the evidence section."
                .to_owned(),
        );
    }
    Ok(clauses)
}

/// Splits one rendered clause into the document, section and text it names.
///
/// The ADR renders each clause as `*"<text>" — <document> <section>.*`, so the
/// quoted span lies between the first pair of double quotes and the remainder
/// names where it came from.
///
/// The document and the section are each wrapped in a single code span, and the
/// spans are stripped *before* the split. Splitting first would put the split
/// inside the document's own span, because the section it must be separated from
/// is itself a phrase containing spaces: `` `design` `6.1 State naming` `` would
/// yield the document `` `6.1 ``.
pub(crate) fn resolve_clause(clause: &str) -> Result<(String, String, String), String> {
    let attributed = |near: &str| {
        format!(
            "docs/adr-004-state-name-consumption-evidence.md quotes a clause without {near}. \
             Repair: render each clause as `\"<text>\" — <document> <section>.`"
        )
    };
    let (_, after_open) = clause
        .split_once('"')
        .ok_or_else(|| attributed("a quoted span"))?;
    let (text, attribution) = after_open
        .split_once('"')
        .ok_or_else(|| attributed("a closing double quote"))?;
    let (_, attributed_to) = attribution
        .split_once('—')
        .ok_or_else(|| attributed("naming its source after an em dash"))?;
    let named = attributed_to
        .trim()
        .trim_end_matches('.')
        .trim()
        .replace('`', "");
    let (document, section) = named
        .split_once(' ')
        .ok_or_else(|| attributed("both a document and a section"))?;
    Ok((
        document.to_owned(),
        section.trim().to_owned(),
        fold_whitespace(text),
    ))
}

/// Folds a string's whitespace runs to single spaces.
pub(crate) fn fold_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
