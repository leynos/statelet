//! One delimited-table syntax function, with typed mappers layered over it.
//!
//! Syntax and typing are deliberately separate: `parse_table` knows only about
//! delimiters, pipes and the structural header and divider rows, while each
//! `*_rows` mapper knows one register's vocabulary. A failure therefore names
//! its layer — a malformed row is a syntax defect, an unknown token is a
//! vocabulary defect.

use super::types::{AggRow, GateRow, NoteRow, ParseError, Register, StatusRow};

/// Reads every data row of a delimited table.
///
/// Structural rows — the header and the `---` divider — are recognized exactly
/// rather than by shape, so a cell whose *content* begins with a hyphen is not
/// mistaken for a divider. Prose between the delimiters is ignored.
pub(crate) fn parse_table(
    source: &str,
    register: Register,
) -> Result<Vec<Vec<String>>, ParseError> {
    let section = section_body(source, register)?;
    let Some((_, after_begin)) = section.split_once(register.begin()) else {
        return Err(ParseError::MissingDelimiters { register });
    };
    let Some((block, _)) = after_begin.split_once(register.end()) else {
        return Err(ParseError::MissingDelimiters { register });
    };
    let rows = block
        .lines()
        .filter_map(|line| row_from_line(line, register))
        .collect::<Vec<_>>();
    if rows.is_empty() {
        Err(ParseError::MissingDelimiters { register })
    } else {
        Ok(rows)
    }
}

/// Bounds a register to the section that must contain it.
///
/// The bound is taken with `split_once` on the literal text that ends a section,
/// never by scanning for a leading `#`. `docs/design.md` §6.1 contains
/// `#[derive(StateName)]` at column zero inside a fence, and a scanner looking
/// for `#` truncates the section there. Every register in ADR 004 sits under a
/// top-level `##` heading, so the next such heading ends the section.
fn section_body(source: &str, register: Register) -> Result<&str, ParseError> {
    let Some((_, after_heading)) = source.split_once(register.section()) else {
        return Err(ParseError::MissingSection { register });
    };
    Ok(after_heading
        .split_once(NEXT_SECTION)
        .map_or(after_heading, |(body, _)| body))
}

/// The literal that ends a register's section: the next top-level heading.
const NEXT_SECTION: &str = "\n## ";

/// Converts one table line into trimmed cells, or `None` for a structural row
/// or non-table prose.
///
/// Infallible by construction, and the infallibility is the point: any line
/// beginning with a pipe yields at least one cell, because `split('|')` on any
/// string yields at least one element — `"||"` yields `[""]`. A row cannot
/// therefore be too malformed to parse *here*; it can only carry the wrong
/// number of cells for its register, which the typed mapper rejects several
/// checks later with the register named. A guard for an empty cell list would
/// be unreachable, and a branch that cannot be taken is not a check.
///
/// The `offset` the caller once passed is gone with it: its only use was the
/// row number in that unreachable error.
fn row_from_line(line: &str, register: Register) -> Option<Vec<String>> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect::<Vec<String>>();
    if is_structural_row(&cells, register) {
        return None;
    }
    Some(cells)
}

/// Identifies the header row or the divider row of the named register.
fn is_structural_row(cells: &[String], register: Register) -> bool {
    cells == expected_header(register) || cells.iter().all(|cell| is_divider(cell))
}

/// The exact header row each register's table must carry.
///
/// The aggregation register's outcome column is headed `Outcome if publication
/// proceeds`, not `Outcome`, because every row's outcome is conditional. The
/// header is what identifies the row as structural, so a header this function
/// does not reproduce exactly is parsed as a data row and then reported as a
/// malformed one several checks later, far from the edit that caused it.
fn expected_header(register: Register) -> Vec<String> {
    let columns: &[&str] = match register {
        Register::Status => &["Field", "Status", "Admissible", "Contributes"],
        Register::Aggregation => &[
            "Admissible notes",
            "Any insufficient",
            "Outcome if publication proceeds",
        ],
        Register::Gates => &["Gate", "Roadmap task title fragment"],
        Register::Note => &["Field", "Status", "Evidence"],
        Register::Evidence => &[],
    };
    columns.iter().map(|column| (*column).to_owned()).collect()
}

/// Recognizes a divider cell: one or more hyphens and nothing else.
fn is_divider(cell: &str) -> bool {
    cell.strip_prefix('-')
        .is_some_and(|rest| rest.bytes().all(|byte| byte == b'-'))
}

/// Maps the status register, rejecting a token its own vocabulary lacks.
pub(crate) fn status_rows(adr: &str) -> Result<Vec<StatusRow>, ParseError> {
    let mut rows = Vec::new();
    for (offset, cells) in parse_table(adr, Register::Status)?.iter().enumerate() {
        let [field, status, admissible, contributes] = cells.as_slice() else {
            return Err(ParseError::MalformedRow {
                register: Register::Status,
                row: offset + 1,
            });
        };
        rows.push(StatusRow {
            field: field.clone(),
            status: status.clone(),
            admissible: parse_admissible(admissible)?,
            contributes: contributes.clone(),
        });
    }
    Ok(rows)
}

/// Maps the aggregation register.
pub(crate) fn aggregation_rows(adr: &str) -> Result<Vec<AggRow>, ParseError> {
    let mut rows = Vec::new();
    for (offset, cells) in parse_table(adr, Register::Aggregation)?.iter().enumerate() {
        let [admissible_notes, any_insufficient, outcome] = cells.as_slice() else {
            return Err(ParseError::MalformedRow {
                register: Register::Aggregation,
                row: offset + 1,
            });
        };
        rows.push(AggRow {
            admissible_notes: admissible_notes.clone(),
            any_insufficient: any_insufficient.clone(),
            outcome: outcome.clone(),
        });
    }
    Ok(rows)
}

/// Maps the gate table.
pub(crate) fn gate_rows(adr: &str) -> Result<Vec<GateRow>, ParseError> {
    let mut rows = Vec::new();
    for (offset, cells) in parse_table(adr, Register::Gates)?.iter().enumerate() {
        let [gate, fragment] = cells.as_slice() else {
            return Err(ParseError::MalformedRow {
                register: Register::Gates,
                row: offset + 1,
            });
        };
        rows.push(GateRow {
            gate: gate.clone(),
            fragment: fragment.clone(),
        });
    }
    Ok(rows)
}

/// Maps a validation note's own register.
pub(crate) fn note_rows(note: &str) -> Result<Vec<NoteRow>, ParseError> {
    let mut rows = Vec::new();
    for (offset, cells) in parse_table(note, Register::Note)?.iter().enumerate() {
        let [field, status, evidence] = cells.as_slice() else {
            return Err(ParseError::MalformedRow {
                register: Register::Note,
                row: offset + 1,
            });
        };
        rows.push(NoteRow {
            field: field.clone(),
            status: status.clone(),
            evidence: evidence.clone(),
        });
    }
    Ok(rows)
}

/// Reads the status register, rendering a parse failure as its own repair
/// message.
///
/// Callers that only need the register's vocabulary want the message, not the
/// `ParseError`: a scenario asserting a downstream property must still report
/// the document defect that prevented it from running.
pub(crate) fn status_rows_or_error(adr: &str) -> Result<Vec<StatusRow>, String> {
    status_rows(adr).map_err(|error| error.to_string())
}

/// Parses the `Admissible` cell; `yes` and `no` are the register's vocabulary.
fn parse_admissible(value: &str) -> Result<bool, ParseError> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(ParseError::UnknownToken {
            register: Register::Status,
            column: "admissibility",
            found: value.to_owned(),
        }),
    }
}
