//! Parses and validates macro-cost note registers within this test contract.
//!
//! This module owns only the `Field | Value | Evidence` schema. The exit-register
//! parser models a different five-column ADR table and remains separate.

use std::collections::BTreeMap;

use super::{
    super::{
        MarkdownFile,
        REQUIRED_FIELDS,
        types::{CellText, DocumentPath, FieldName, SourceLine},
    },
    Kind,
    placeholders::{has_revision_citation, placeholder, unavailable_reason},
};

pub(super) type RegisterRows = BTreeMap<String, Row>;

pub(super) const NOTE_MARKER: &str = "<!-- macro-cost-note -->";
const REAL_STRESS_EXCEPTION: &str = "Not applicable: real comparison";

pub(super) fn marker_count(source: &str, marker: &str) -> usize {
    source.lines().filter(|line| line.trim() == marker).count()
}

pub(super) fn row<'a>(
    path: DocumentPath<'_>,
    rows: &'a RegisterRows,
    field: FieldName<'_>,
) -> Result<&'a Row, String> {
    rows.get(field.0)
        .ok_or_else(|| format!("{path}: required field `{field}` is missing"))
}

pub(super) struct Row {
    pub(super) value: String,
    pub(super) evidence: String,
}

/// One parsed `Field | Value | Evidence` register row, still borrowed.
#[derive(Clone, Copy)]
struct RegisterCells<'a> {
    field: CellText<'a>,
    value: CellText<'a>,
    evidence: CellText<'a>,
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
        let Some(cells) = table_cells(file, SourceLine(line))? else {
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
        let [field, value, evidence] = cells;
        insert_row(
            file,
            RegisterCells {
                field: CellText(field),
                value: CellText(value),
                evidence: CellText(evidence),
            },
            &mut register,
        )?;
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

fn insert_row(
    file: &MarkdownFile,
    cells: RegisterCells<'_>,
    register: &mut Register,
) -> Result<(), String> {
    let RegisterCells {
        field,
        value,
        evidence,
    } = cells;
    let field_name = field.0.to_owned();
    if register
        .rows
        .insert(
            field.0.to_owned(),
            Row {
                value: value.0.to_owned(),
                evidence: evidence.0.to_owned(),
            },
        )
        .is_some()
    {
        return Err(format!(
            "{}: duplicate macro-cost field `{field_name}`",
            file.path
        ));
    }
    register.fields.push(field_name);
    Ok(())
}

pub(super) fn table_cells<'a>(
    file: &MarkdownFile,
    raw_line: SourceLine<'a>,
) -> Result<Option<[&'a str; 3]>, String> {
    let line = raw_line.0.trim();
    if !line.starts_with('|') || !line.ends_with('|') {
        return Ok(None);
    }
    let Some(inner) = line.get(1..line.len().saturating_sub(1)) else {
        return Ok(None);
    };
    let cells = inner.split('|').map(str::trim).collect::<Vec<_>>();
    let [field, value, evidence] = cells.as_slice() else {
        return Err(format!(
            concat!(
                "{}: malformed register row `{}` has {} cells; expected 3; ",
                "`|` inside cells is unsupported"
            ),
            file.path,
            line,
            cells.len()
        ));
    };
    Ok(Some([*field, *value, *evidence]))
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
        let field_row = row(
            DocumentPath(file.path.as_str()),
            &register.rows,
            FieldName(field),
        )?;
        let template_row = row(
            DocumentPath(file.path.as_str()),
            &template.rows,
            FieldName(field),
        )?;
        if field_row.value.trim() == template_row.value.trim() {
            return Err(format!(
                "{}: `{field}` still matches the template value",
                file.path
            ));
        }
        check_required_field(file, kind, FieldName(field), field_row)?;
    }
    if register
        .rows
        .values()
        .any(|field_row| unavailable_reason(CellText(field_row.value.as_str())).is_some())
        && !row(
            DocumentPath(file.path.as_str()),
            &register.rows,
            FieldName("budget-verdict"),
        )?
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
    field: FieldName<'_>,
    field_row: &Row,
) -> Result<(), String> {
    if required_value_is_placeholder(field, field_row, kind) {
        return Err(format!(
            "{}: `{field}` is empty or still a placeholder",
            file.path
        ));
    }
    if placeholder(CellText(field_row.evidence.as_str()))
        || !has_revision_citation(CellText(field_row.evidence.as_str()))
    {
        return Err(format!(
            "{}: `{field}` needs revision-bound evidence",
            file.path
        ));
    }
    Ok(())
}

fn required_value_is_placeholder(field: FieldName<'_>, row: &Row, kind: Kind) -> bool {
    let value = row.value.trim();
    match field.0 {
        "stress-fixture-results" if kind == Kind::Mdtablefix => {
            value != REAL_STRESS_EXCEPTION && placeholder(CellText(value))
        }
        "review" => {
            value != "Pending: G3 decision not yet recorded" && placeholder(CellText(value))
        }
        _ => placeholder(CellText(value)),
    }
}

pub(super) fn check_measurement_values(
    file: &MarkdownFile,
    register: &Register,
    kind: Kind,
) -> Result<(), String> {
    let count = row(
        DocumentPath(file.path.as_str()),
        &register.rows,
        FieldName("annotation-count"),
    )?
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
    let stress_results = row(
        DocumentPath(file.path.as_str()),
        &register.rows,
        FieldName("stress-fixture-results"),
    )?;
    match (kind, stress_results.value.trim() == REAL_STRESS_EXCEPTION) {
        (Kind::Mdtablefix, false) => Err(format!(
            "{}: real comparison must mark stress results not applicable",
            file.path
        )),
        (Kind::Synthetic20, true) => Err(format!(
            "{}: synthetic-20 must record its 20-transition results",
            file.path
        )),
        _ => Ok(()),
    }
}
