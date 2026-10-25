//! Private, marker-scoped checks for macro cost notes and roadmap claims.

use std::collections::BTreeMap;

use super::{
    NoteFile,
    REQUIRED_FIELDS,
    check_fields,
    claims::{Task, candidate_identity, linked_note_paths, task_block},
    full_revision,
    has_revision_citation,
    marker_count,
    normalize,
    parse_register,
    placeholder,
    row,
    section_body,
};

const NOTE_MARKER: &str = "<!-- macro-cost-note -->";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Mdtablefix,
    Synthetic20,
}

impl Kind {
    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "mdtablefix" => Some(Self::Mdtablefix),
            "synthetic-20" => Some(Self::Synthetic20),
            _ => None,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Mdtablefix => "mdtablefix",
            Self::Synthetic20 => "synthetic-20",
        }
    }
}

struct Note {
    kind: Kind,
    macro_revision: String,
    design_revision: String,
}

/// Check the live template, design and roadmap bindings, and supplied notes.
pub(crate) fn check_contract(
    template: &str,
    design: &str,
    roadmap: &str,
    notes: &[NoteFile],
) -> Result<(), String> {
    check_template(template)?;
    check_design(design)?;
    let tasks = check_roadmap(roadmap)?;
    let mut parsed = BTreeMap::new();
    for note in notes {
        if parsed.contains_key(&note.path) {
            return Err(format!("duplicate macro-cost note path `{}`", note.path));
        }
        parsed.insert(note.path.clone(), parse_note(note)?);
    }
    check_claim("4.2.2", &[Kind::Synthetic20], &tasks, &parsed)?;
    check_claim(
        "4.3.1",
        &[Kind::Mdtablefix, Kind::Synthetic20],
        &tasks,
        &parsed,
    )
}

fn check_template(template: &str) -> Result<(), String> {
    if marker_count(template, NOTE_MARKER) != 1 {
        return Err(format!("template needs exactly one `{NOTE_MARKER}` marker"));
    }
    let register = parse_register("macro-cost template", template)?;
    check_fields("macro-cost template", &register.fields)?;
    let kind_row = row("macro-cost template", &register.rows, "comparison-kind")?;
    let kind_value = kind_row.value.as_str();
    if !kind_value.contains("mdtablefix") || !kind_value.contains("synthetic-20") {
        return Err("template must document both comparison kinds".into());
    }
    let normalized = normalize(template);
    for phrase in [
        "marginal seconds = (T1 - T0) / N",
        "marginal percent = 100 * (T1 - T0) / (N * T0)",
        "release growth percent = 100 * (S1 - S0) / S0",
        "T1 = 109.8 gives 0.49%",
        "T1 = 110 gives 0.5%",
        "S1 = 102000 gives 2%",
        "S1 = 102001 exceeds 2%",
    ] {
        if !normalized.contains(phrase) {
            return Err(format!(
                "template is missing calculation guidance `{phrase}`"
            ));
        }
    }
    Ok(())
}

fn check_design(design: &str) -> Result<(), String> {
    let section = section_body(design, "### 11.7 Compile-time and binary-size budget")
        .ok_or_else(|| "design section 11.7 is missing".to_owned())?;
    let normalized_section = normalize(&section);
    for phrase in [
        "macro-cost-benchmark-note-template.md",
        "evidence-completeness contract",
        "4.2.2",
        "4.3.1",
    ] {
        if !normalized_section.contains(phrase) {
            return Err(format!("design section 11.7 is missing `{phrase}`"));
        }
    }
    Ok(())
}

fn check_roadmap(roadmap: &str) -> Result<BTreeMap<String, Task>, String> {
    let mut tasks = BTreeMap::new();
    for (id, evidence) in [
        ("4.2.2", "synthetic macro cost note"),
        ("4.3.1", "both real and synthetic macro cost notes"),
    ] {
        let task = task_block(roadmap, id)?;
        let body = normalize(&task.text.to_ascii_lowercase());
        for phrase in [evidence, "macro-cost-candidate", "design revision"] {
            if !body.contains(phrase) {
                return Err(format!("roadmap task {id} must require `{phrase}`"));
            }
        }
        tasks.insert(id.to_owned(), task);
    }
    Ok(tasks)
}

fn check_claim(
    id: &str,
    expected: &[Kind],
    tasks: &BTreeMap<String, Task>,
    notes: &BTreeMap<String, Option<Note>>,
) -> Result<(), String> {
    let task = tasks
        .get(id)
        .ok_or_else(|| format!("roadmap task {id} was not checked"))?;
    if !task.complete {
        return Ok(());
    }
    let candidate = candidate_identity(&task.text, id)?;
    let paths = linked_note_paths(&task.text);
    if paths.len() != expected.len() {
        return Err(format!(
            "completed task {id} must link exactly {} cost note(s)",
            expected.len()
        ));
    }
    let mut selected = Vec::new();
    for note_path in paths {
        let qualified_path = format!("docs/{note_path}");
        let note = notes
            .get(&qualified_path)
            .ok_or_else(|| format!("completed task {id} links missing note `{qualified_path}`"))?
            .as_ref()
            .ok_or_else(|| {
                format!("selected note `{qualified_path}` must declare `{NOTE_MARKER}`")
            })?;
        if !expected.contains(&note.kind) {
            return Err(format!(
                "task {id} selects unexpected `{}` note",
                note.kind.label()
            ));
        }
        if note.macro_revision != candidate.macro_revision {
            return Err(format!(
                "selected note `{qualified_path}` evaluates a different macro commit"
            ));
        }
        if note.design_revision != candidate.design_revision {
            return Err(format!(
                "selected note `{qualified_path}` uses a different design revision"
            ));
        }
        selected.push(note.kind);
    }
    for kind in expected {
        if selected
            .iter()
            .filter(|selected_kind| *selected_kind == kind)
            .count()
            != 1
        {
            return Err(format!(
                "completed task {id} needs one `{}` note",
                kind.label()
            ));
        }
    }
    Ok(())
}

fn parse_note(file: &NoteFile) -> Result<Option<Note>, String> {
    match marker_count(&file.text, NOTE_MARKER) {
        0 => return Ok(None),
        1 => {}
        _ => {
            return Err(format!(
                "{} needs exactly one `{NOTE_MARKER}` marker",
                file.path
            ));
        }
    }
    let register = parse_register(&file.path, &file.text)?;
    check_fields(&file.path, &register.fields)?;
    let kind = parse_kind(file, &register)?;
    check_required_values(file, &register, kind)?;
    check_measurement_values(file, &register, kind)?;
    let (macro_revision, design_revision) = candidate_id(
        &file.path,
        &row(&file.path, &register.rows, "candidate-id")?.value,
    )?;
    Ok(Some(Note {
        kind,
        macro_revision,
        design_revision,
    }))
}

fn parse_kind(file: &NoteFile, register: &super::Register) -> Result<Kind, String> {
    row(&file.path, &register.rows, "comparison-kind").and_then(|kind_row| {
        Kind::parse(&kind_row.value).ok_or_else(|| {
            format!(
                "{}: comparison-kind must be mdtablefix or synthetic-20",
                file.path
            )
        })
    })
}

fn check_required_values(
    file: &NoteFile,
    register: &super::Register,
    kind: Kind,
) -> Result<(), String> {
    for field in REQUIRED_FIELDS {
        let field_row = row(&file.path, &register.rows, field)?;
        let allowed = (field == "stress-fixture-results"
            && kind == Kind::Mdtablefix
            && field_row.value.trim() == "Not applicable: real comparison")
            || (field == "review"
                && field_row.value.trim() == "Pending: G3 decision not yet recorded");
        if !allowed && placeholder(&field_row.value) {
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
    }
    if register
        .rows
        .values()
        .any(|field_row| field_row.value.trim().starts_with("Unavailable:"))
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

fn check_measurement_values(
    file: &NoteFile,
    register: &super::Register,
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
    }
    Ok(())
}

fn candidate_id(path: &str, candidate_value: &str) -> Result<(String, String), String> {
    let value = candidate_value
        .trim()
        .strip_prefix("macro=")
        .ok_or_else(|| format!("{path}: candidate-id must name macro and design revisions"))?;
    let (macro_revision, design_revision) = value
        .split_once("; design=")
        .ok_or_else(|| format!("{path}: candidate-id must name macro and design revisions"))?;
    if !full_revision(macro_revision) || !full_revision(design_revision) {
        return Err(format!(
            "{path}: candidate-id requires full lowercase 40-character revisions"
        ));
    }
    Ok((macro_revision.to_owned(), design_revision.to_owned()))
}
