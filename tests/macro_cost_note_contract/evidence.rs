//! Parses and validates macro-cost note registers within this test contract.
//!
//! This module owns only the `Field | Value | Evidence` schema. The exit-register
//! parser models a different five-column ADR table and remains separate.

use std::collections::BTreeMap;

use super::{
    super::{MarkdownFile, REQUIRED_FIELDS},
    Kind,
    full_revision,
};

pub(super) type RegisterRows = BTreeMap<String, Row>;

pub(super) const NOTE_MARKER: &str = "<!-- macro-cost-note -->";

pub(super) fn marker_count(source: &str, marker: &str) -> usize {
    source.lines().filter(|line| line.trim() == marker).count()
}

fn placeholder(value: &str) -> bool {
    let trimmed = value.trim();
    let checked_value = unavailable_reason(trimmed).unwrap_or(trimmed);
    checked_value.is_empty()
        || starts_with_placeholder_marker(checked_value)
        || contains_angle_placeholder(checked_value)
}

fn unavailable_reason(value: &str) -> Option<&str> {
    let (label, reason) = value.trim().split_once(':')?;
    label
        .eq_ignore_ascii_case("unavailable")
        .then_some(reason.trim())
}

fn starts_with_placeholder_marker(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    ["tbd", "todo", "pending", "n/a"].iter().any(|marker| {
        normalized.strip_prefix(marker).is_some_and(|suffix| {
            suffix.is_empty()
                || suffix
                    .chars()
                    .next()
                    .is_some_and(|character| !character.is_ascii_alphanumeric() && character != '_')
        })
    })
}

fn contains_angle_placeholder(value: &str) -> bool {
    let mut search_from = 0;
    while let Some(open_offset) = value
        .get(search_from..)
        .and_then(|remaining| remaining.find('<'))
    {
        let open = search_from + open_offset;
        let candidate_start = open + 1;
        if let Some(close_offset) = value
            .get(candidate_start..)
            .and_then(|remaining| remaining.find('>'))
        {
            let close = candidate_start + close_offset;
            let is_placeholder = match (value.get(candidate_start..close), value.get(..open)) {
                (Some(candidate), Some(prefix)) => {
                    !is_attached_to_path_identifier(prefix) && is_placeholder_token(candidate)
                }
                _ => false,
            };
            if is_placeholder {
                return true;
            }
        }
        // Advancing past only this opener lets malformed markup expose a
        // later standalone placeholder instead of hiding it.
        search_from = candidate_start;
    }
    false
}

fn is_attached_to_path_identifier(prefix: &str) -> bool {
    // This lexical heuristic admits Rust generics. `Type<sha>` is ambiguous,
    // so revision placeholders must remain standalone to be rejected.
    let path = prefix.strip_suffix("::").unwrap_or(prefix);
    path.chars()
        .next_back()
        .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn is_placeholder_token(candidate: &str) -> bool {
    let trimmed_candidate = candidate.trim();
    is_placeholder_identifier(trimmed_candidate) || is_placeholder_instruction(trimmed_candidate)
}

fn is_placeholder_identifier(candidate: &str) -> bool {
    candidate.is_empty()
        || candidate.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
        })
}

fn is_placeholder_instruction(candidate: &str) -> bool {
    ["fill ", "insert ", "replace ", "example ", "your "]
        .iter()
        .any(|prefix| candidate.to_ascii_lowercase().starts_with(prefix))
}

fn has_revision_citation(evidence: &str) -> bool {
    evidence.split_whitespace().any(|token| {
        let Some((repository, rest)) = token.split_once('@') else {
            return false;
        };
        let Some((revision, path)) = rest.split_once(':') else {
            return false;
        };
        !repository.is_empty() && full_revision(revision) && !path.is_empty()
    })
}

pub(super) fn row<'a>(path: &str, rows: &'a RegisterRows, field: &str) -> Result<&'a Row, String> {
    rows.get(field)
        .ok_or_else(|| format!("{path}: required field `{field}` is missing"))
}

pub(super) struct Row {
    pub(super) value: String,
    pub(super) evidence: String,
}

pub(super) struct Register {
    pub(super) rows: BTreeMap<String, Row>,
    pub(super) fields: Vec<String>,
}

pub(super) fn parse_register(file: &MarkdownFile) -> Result<Register, String> {
    let mut register = Register {
        rows: BTreeMap::new(),
        fields: Vec::new(),
    };
    let mut header_seen = false;
    for line in register_body(file)?.lines() {
        let Some(cells) = table_cells(line) else {
            continue;
        };
        if cells == ["Field", "Value", "Evidence"] {
            if header_seen {
                return Err(format!("{}: register repeats its header", file.path));
            }
            header_seen = true;
            continue;
        }
        if is_separator_row(&cells) {
            continue;
        }
        if !header_seen {
            return Err(format!("{}: malformed register row", file.path));
        }
        insert_row(file, &cells, &mut register)?;
    }
    if !header_seen {
        return Err(format!("{}: register has no table header", file.path));
    }
    Ok(register)
}

fn register_body(file: &MarkdownFile) -> Result<&str, String> {
    const BEGIN: &str = "<!-- macro-cost-register:begin -->";
    const END: &str = "<!-- macro-cost-register:end -->";
    if [BEGIN, END]
        .iter()
        .any(|marker| marker_count(&file.text, marker) != 1)
    {
        return Err(format!(
            "{}: register needs one ordered begin/end marker pair",
            file.path
        ));
    }
    let (_, tail) = file
        .text
        .split_once(BEGIN)
        .ok_or_else(|| format!("{}: register begin marker missing", file.path))?;
    tail.split_once(END).map(|(body, _)| body).ok_or_else(|| {
        format!(
            "{}: register end marker is missing or out of order",
            file.path
        )
    })
}

fn is_separator_row(cells: &[&str]) -> bool {
    cells
        .iter()
        .all(|cell| !cell.is_empty() && cell.chars().all(|ch| ch == '-' || ch == ':'))
}

fn insert_row(file: &MarkdownFile, cells: &[&str], register: &mut Register) -> Result<(), String> {
    let [field_cell, value_cell, evidence_cell] = cells else {
        return Err(format!("{}: malformed register row", file.path));
    };
    let field = field_cell.to_owned();
    if register
        .rows
        .insert(
            (*field_cell).to_owned(),
            Row {
                value: (*value_cell).to_owned(),
                evidence: (*evidence_cell).to_owned(),
            },
        )
        .is_some()
    {
        return Err(format!(
            "{}: duplicate macro-cost field `{field}`",
            file.path
        ));
    }
    register.fields.push(field.to_owned());
    Ok(())
}

fn table_cells(raw_line: &str) -> Option<Vec<&str>> {
    let line = raw_line.trim();
    if !line.starts_with('|') || !line.ends_with('|') {
        return None;
    }
    line.get(1..line.len().checked_sub(1)?)
        .map(|cells| cells.split('|').map(str::trim).collect())
}

pub(super) fn check_fields(file: &MarkdownFile, fields: &[String]) -> Result<(), String> {
    if fields.iter().map(String::as_str).eq(REQUIRED_FIELDS) {
        return Ok(());
    }
    if let Some(field) = REQUIRED_FIELDS
        .iter()
        .find(|field| !fields.iter().any(|actual| actual == **field))
    {
        return Err(format!("{}: missing required field `{field}`", file.path));
    }
    if let Some(field) = fields
        .iter()
        .find(|field| !REQUIRED_FIELDS.contains(&field.as_str()))
    {
        return Err(format!("{}: unexpected field `{field}`", file.path));
    }
    Err(format!(
        "{}: fields must appear in the template order",
        file.path
    ))
}
pub(super) fn check_required_values(
    file: &MarkdownFile,
    register: &Register,
    template: &Register,
    kind: Kind,
) -> Result<(), String> {
    for field in REQUIRED_FIELDS {
        let field_row = row(&file.path, &register.rows, field)?;
        let template_row = row(&file.path, &template.rows, field)?;
        if field_row.value.trim() == template_row.value.trim() {
            return Err(format!(
                "{}: `{field}` still matches the template value",
                file.path
            ));
        }
        check_required_field(file, kind, field, field_row)?;
    }
    if register
        .rows
        .values()
        .any(|field_row| unavailable_reason(&field_row.value).is_some())
        && !row(&file.path, &register.rows, "budget-verdict")?
            .value
            .contains("Inconclusive")
    {
        return Err(format!(
            "{}: unavailable evidence requires an Inconclusive verdict",
            file.path
        ));
    }
    Ok(())
}

fn check_required_field(
    file: &MarkdownFile,
    kind: Kind,
    field: &str,
    field_row: &Row,
) -> Result<(), String> {
    if required_value_is_placeholder(field, field_row, kind) {
        return Err(format!(
            "{}: `{field}` is empty or still a placeholder",
            file.path
        ));
    }
    if placeholder(&field_row.evidence) || !has_revision_citation(&field_row.evidence) {
        return Err(format!(
            "{}: `{field}` needs revision-bound evidence",
            file.path
        ));
    }
    Ok(())
}

fn required_value_is_placeholder(field: &str, row: &Row, kind: Kind) -> bool {
    let value = row.value.trim();
    match field {
        "stress-fixture-results" if kind == Kind::Mdtablefix => {
            value != "Not applicable: real comparison" && placeholder(value)
        }
        "review" => value != "Pending: G3 decision not yet recorded" && placeholder(value),
        _ => placeholder(value),
    }
}

pub(super) fn check_measurement_values(
    file: &MarkdownFile,
    register: &Register,
    kind: Kind,
) -> Result<(), String> {
    let count = row(&file.path, &register.rows, "annotation-count")?
        .value
        .parse::<u32>()
        .map_err(|_| format!("{}: annotation-count must be a positive integer", file.path))?;
    let is_valid_count = match kind {
        Kind::Mdtablefix => count > 0,
        Kind::Synthetic20 => count == 20,
    };
    if !is_valid_count {
        return Err(format!(
            "{}: annotation-count is invalid for {}",
            file.path,
            kind.label()
        ));
    }
    if kind == Kind::Mdtablefix {
        let stress_results = row(&file.path, &register.rows, "stress-fixture-results")?;
        if stress_results.value.trim() != "Not applicable: real comparison" {
            return Err(format!(
                "{}: real comparison must mark stress results not applicable",
                file.path
            ));
        }
    } else if row(&file.path, &register.rows, "stress-fixture-results")?
        .value
        .trim()
        == "Not applicable: real comparison"
    {
        return Err(format!(
            "{}: synthetic-20 must record its 20-transition results",
            file.path
        ));
    }
    Ok(())
}
