//! Checks design bindings, roadmap claims and candidate-bound cost notes.

use std::collections::BTreeMap;

use evidence::{
    NOTE_MARKER,
    Register,
    check_fields,
    check_measurement_values,
    check_required_values,
    marker_count,
    parse_register,
    row,
};

use super::{MarkdownFile, Task, candidate_identity, is_heading, linked_note_paths, task_block};

#[path = "evidence.rs"]
pub(super) mod evidence;

pub(super) fn full_revision(revision: &str) -> bool {
    revision.len() == 40
        && revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
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

    pub(super) const fn label(self) -> &'static str {
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

type RoadmapTasks = BTreeMap<String, Task>;
type ParsedNotes = BTreeMap<String, Option<Note>>;

/// Check the live template, design and roadmap bindings, and supplied notes.
pub(crate) fn check_contract(
    template: &MarkdownFile,
    design: &MarkdownFile,
    roadmap: &MarkdownFile,
    notes: &[MarkdownFile],
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

fn check_template(template: &MarkdownFile) -> Result<(), String> {
    if marker_count(&template.text, NOTE_MARKER) != 1 {
        return Err(format!(
            "{} needs exactly one `{NOTE_MARKER}` marker",
            template.path
        ));
    }
    let register = parse_register(template)?;
    check_fields(template, &register.fields)?;
    let kind_row = row(&template.path, &register.rows, "comparison-kind")?;
    if !kind_row.value.contains("mdtablefix") || !kind_row.value.contains("synthetic-20") {
        return Err(format!(
            "{}: template must document both comparison kinds",
            template.path
        ));
    }
    let normalized = normalize(&template.text);
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
                "{} is missing calculation guidance `{phrase}`",
                template.path
            ));
        }
    }
    Ok(())
}

fn check_design(design: &MarkdownFile) -> Result<(), String> {
    let section = section_body(&design.text, "### 11.7 Compile-time and binary-size budget")
        .ok_or_else(|| format!("{} section 11.7 is missing", design.path))?;
    let normalized = normalize(&section);
    for phrase in [
        "macro-cost-benchmark-note-template.md",
        "evidence-completeness contract",
        "4.2.2",
        "4.3.1",
    ] {
        if !normalized.contains(phrase) {
            return Err(format!(
                "{} section 11.7 is missing `{phrase}`",
                design.path
            ));
        }
    }
    Ok(())
}

fn check_roadmap(roadmap: &MarkdownFile) -> Result<RoadmapTasks, String> {
    let mut tasks = BTreeMap::new();
    for (id, evidence) in [
        ("4.2.2", "synthetic macro cost note"),
        ("4.3.1", "both real and synthetic macro cost notes"),
    ] {
        let task = task_block(roadmap, id)?;
        let body = normalize(&task.text.to_ascii_lowercase());
        for phrase in [evidence, "macro-cost-candidate", "design revision"] {
            if !body.contains(phrase) {
                return Err(format!(
                    "{} task {id} must require `{phrase}`",
                    roadmap.path
                ));
            }
        }
        tasks.insert(id.to_owned(), task);
    }
    Ok(tasks)
}

fn check_claim(
    id: &str,
    expected: &[Kind],
    tasks: &RoadmapTasks,
    notes: &ParsedNotes,
) -> Result<(), String> {
    let task = tasks
        .get(id)
        .ok_or_else(|| format!("roadmap task {id} was not checked"))?;
    if !task.complete {
        return Ok(());
    }
    candidate_identity(task)?;
    let paths = linked_note_paths(task);
    if paths.len() != expected.len() {
        return Err(format!(
            "completed task {id} must link exactly {} cost note(s)",
            expected.len()
        ));
    }
    let selected = paths
        .iter()
        .map(|path| resolve_selected_note(task, expected, path, notes))
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(kind) = expected.iter().find(|kind| {
        selected
            .iter()
            .filter(|selected_kind| *selected_kind == *kind)
            .count()
            != 1
    }) {
        return Err(format!(
            "completed task {id} needs one `{}` note",
            kind.label()
        ));
    }
    Ok(())
}

fn resolve_selected_note(
    task: &Task,
    expected: &[Kind],
    note_path: &str,
    notes: &ParsedNotes,
) -> Result<Kind, String> {
    let candidate = candidate_identity(task)?;
    let qualified_path = format!("docs/{note_path}");
    let note = notes
        .get(&qualified_path)
        .ok_or_else(|| {
            format!(
                "completed task {} links missing note `{qualified_path}`",
                task.id
            )
        })?
        .as_ref()
        .ok_or_else(|| format!("selected note `{qualified_path}` must declare `{NOTE_MARKER}`"))?;
    if !expected.contains(&note.kind) {
        return Err(format!(
            "task {} selects unexpected `{}` note",
            task.id,
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
    Ok(note.kind)
}

fn parse_note(file: &MarkdownFile) -> Result<Option<Note>, String> {
    match marker_count(&file.text, NOTE_MARKER) {
        0 => return Ok(None),
        1 => (),
        _ => {
            return Err(format!(
                "{} needs exactly one `{NOTE_MARKER}` marker",
                file.path
            ));
        }
    }
    let register = parse_register(file)?;
    check_fields(file, &register.fields)?;
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

fn parse_kind(file: &MarkdownFile, register: &Register) -> Result<Kind, String> {
    row(&file.path, &register.rows, "comparison-kind").and_then(|kind_row| {
        Kind::parse(&kind_row.value).ok_or_else(|| {
            format!(
                "{}: comparison-kind must be mdtablefix or synthetic-20",
                file.path
            )
        })
    })
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

fn section_body(source: &str, heading: &str) -> Option<String> {
    let mut lines = source.lines();
    lines.position(|line| line.trim() == heading)?;
    Some(
        lines
            .take_while(|line| !is_heading(line))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn normalize(value: &str) -> String { value.split_whitespace().collect::<Vec<_>>().join(" ") }
