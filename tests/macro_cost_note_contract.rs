//! Guard the macro cost evidence form and the roadmap claims that consume it.

#[path = "macro_cost_note_contract/contract.rs"]
mod contract;
#[path = "macro_cost_note_contract/fixtures.rs"]
mod fixtures;

use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use camino::Utf8Path;
use cap_std::{ambient_authority, fs::Dir};
use contract::check_contract;

const REQUIRED_FIELDS: [&str; 18] = [
    "comparison-kind",
    "candidate-id",
    "revisions",
    "environment",
    "clean-build-protocol",
    "clean-debug-baseline-seconds",
    "clean-debug-macro-seconds",
    "annotation-count",
    "marginal-seconds",
    "marginal-percent",
    "release-baseline-bytes",
    "release-macro-bytes",
    "release-growth-percent",
    "stress-fixture-results",
    "fallible-expansion",
    "infallible-expansion",
    "budget-verdict",
    "review",
];

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

struct Row {
    value: String,
    evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NoteFile {
    pub(crate) path: String,
    pub(crate) text: String,
}

struct Task {
    complete: bool,
    text: String,
}

struct Candidate {
    macro_revision: String,
    design_revision: String,
}

fn candidate_identity(block: &str, id: &str) -> Result<Candidate, String> {
    const PREFIX: &str = "<!-- macro-cost-candidate:";
    if block.matches(PREFIX).count() != 1 {
        return Err(format!(
            "completed task {id} needs one macro-cost-candidate comment"
        ));
    }
    let tail = block
        .split_once(PREFIX)
        .map(|(_, tail)| tail)
        .ok_or_else(|| format!("completed task {id} has no candidate comment"))?;
    let content = tail
        .split_once("-->")
        .map(|(content, _)| content.trim())
        .ok_or_else(|| format!("completed task {id} has an unterminated candidate comment"))?;
    let (macro_revision_raw, design_revision_raw) = content
        .split_once("; design-revision:")
        .ok_or_else(|| format!("completed task {id} comment must name a design revision"))?;
    let macro_revision = macro_revision_raw.trim();
    let design_revision = design_revision_raw.trim();
    if !contract::full_revision(macro_revision) || !contract::full_revision(design_revision) {
        return Err(format!(
            "completed task {id} comment requires full lowercase revisions"
        ));
    }
    Ok(Candidate {
        macro_revision: macro_revision.into(),
        design_revision: design_revision.into(),
    })
}

fn linked_note_paths(block: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = block;
    while let Some((_, tail)) = rest.split_once("](") {
        let Some((target, remaining)) = tail.split_once(')') else {
            break;
        };
        if target.starts_with("validation-notes/") {
            found.push(target.into());
        }
        rest = remaining;
    }
    found
}

fn task_block(roadmap: &str, id: &str) -> Result<Task, String> {
    let lines = roadmap.lines().collect::<Vec<_>>();
    let matches = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| task_status(line, id).map(|complete| (index, complete)))
        .collect::<Vec<_>>();
    let [(start, complete)] = matches.as_slice() else {
        return Err(format!(
            "roadmap must contain exactly one checklist task {id}"
        ));
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (line.starts_with("- [") || is_heading(line)).then_some(index))
        .unwrap_or(lines.len());
    Ok(Task {
        complete: *complete,
        text: lines
            .get(*start..end)
            .ok_or_else(|| format!("roadmap task {id} has an invalid source range"))?
            .join("\n"),
    })
}

fn task_status(line: &str, id: &str) -> Option<bool> {
    let (complete, rest) = if let Some(rest) = line.strip_prefix("- [x] ") {
        (true, rest)
    } else {
        (false, line.strip_prefix("- [ ] ")?)
    };
    rest.strip_prefix(id)
        .is_some_and(|suffix| suffix.starts_with(". "))
        .then_some(complete)
}

struct Register {
    rows: BTreeMap<String, Row>,
    fields: Vec<String>,
}

fn parse_register(source_name: &str, source: &str) -> Result<Register, String> {
    let begin = "<!-- macro-cost-register:begin -->";
    let end_marker = "<!-- macro-cost-register:end -->";
    if contract::marker_count(source, begin) != 1 || contract::marker_count(source, end_marker) != 1
    {
        return Err(format!(
            "{source_name}: register needs one ordered begin/end marker pair"
        ));
    }
    let start = source
        .find(begin)
        .ok_or_else(|| format!("{source_name}: register begin marker missing"))?
        + begin.len();
    let tail = source
        .get(start..)
        .ok_or_else(|| format!("{source_name}: register begin marker is invalid"))?;
    let end = tail
        .find(end_marker)
        .map(|offset| start + offset)
        .ok_or_else(|| format!("{source_name}: register end marker is missing or out of order"))?;
    let mut rows = BTreeMap::new();
    let mut fields = Vec::new();
    let mut header_seen = false;
    let register_body = source
        .get(start..end)
        .ok_or_else(|| format!("{source_name}: register marker range is invalid"))?;
    for line in register_body.lines() {
        let Some(cells) = table_cells(line) else {
            continue;
        };
        if cells == ["Field", "Value", "Evidence"] {
            if header_seen {
                return Err(format!("{source_name}: register repeats its header"));
            }
            header_seen = true;
            continue;
        }
        if cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }
        if cells.len() != 3 || !header_seen {
            return Err(format!("{source_name}: malformed register row"));
        }
        let [field_cell, value_cell, evidence_cell] = cells.as_slice() else {
            return Err(format!("{source_name}: malformed register row"));
        };
        let field = (*field_cell).to_owned();
        if rows
            .insert(
                field.clone(),
                Row {
                    value: (*value_cell).into(),
                    evidence: (*evidence_cell).into(),
                },
            )
            .is_some()
        {
            return Err(format!(
                "{source_name}: duplicate macro-cost field `{field}`"
            ));
        }
        fields.push(field);
    }
    if !header_seen {
        return Err(format!("{source_name}: register has no table header"));
    }
    Ok(Register { rows, fields })
}

fn table_cells(raw_line: &str) -> Option<Vec<&str>> {
    let line = raw_line.trim();
    if !line.starts_with('|') || !line.ends_with('|') {
        return None;
    }
    line.get(1..line.len().checked_sub(1)?)
        .map(|cells| cells.split('|').map(str::trim).collect())
}

fn check_fields(source: &str, fields: &[String]) -> Result<(), String> {
    if fields.iter().map(String::as_str).eq(REQUIRED_FIELDS) {
        return Ok(());
    }
    if let Some(field) = REQUIRED_FIELDS
        .iter()
        .find(|field| !fields.iter().any(|actual| actual == **field))
    {
        return Err(format!("{source}: missing required field `{field}`"));
    }
    if let Some(field) = fields
        .iter()
        .find(|field| !REQUIRED_FIELDS.contains(&field.as_str()))
    {
        return Err(format!("{source}: unexpected field `{field}`"));
    }
    Err(format!(
        "{source}: fields must appear in the template order"
    ))
}

fn is_heading(line: &str) -> bool { line.starts_with("## ") || line.starts_with("### ") }

fn read_document(root: &Utf8Path, path: &str) -> Result<String, String> {
    let directory = Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| format!("{root}: cannot open repository root: {error}"))?;
    directory
        .read_to_string(path)
        .map_err(|error| format!("{root}/{path}: {error}"))
}

fn read_notes(root: &Utf8Path) -> Result<Vec<NoteFile>, String> {
    let repository = Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| format!("{root}: cannot open repository root: {error}"))?;
    let directory = match repository.open_dir("docs/validation-notes") {
        Ok(directory) => directory,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{root}/docs/validation-notes: {error}")),
    };
    let mut notes = Vec::new();
    for entry_result in directory
        .entries()
        .map_err(|error| format!("{root}/docs/validation-notes: {error}"))?
    {
        let note_entry =
            entry_result.map_err(|error| format!("{root}/docs/validation-notes: {error}"))?;
        let name = note_entry.file_name().into_string().map_err(|name| {
            format!(
                "{root}/docs/validation-notes/{}: entry name is not UTF-8",
                name.display()
            )
        })?;
        if !name.to_ascii_lowercase().ends_with(".md") {
            continue;
        }
        let text = directory
            .read_to_string(&name)
            .map_err(|error| format!("{root}/docs/validation-notes/{name}: {error}"))?;
        notes.push(NoteFile {
            path: format!("docs/validation-notes/{name}"),
            text,
        });
    }
    notes.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(notes)
}

fn live_documents(root: &Utf8Path) -> Result<(String, String, String, Vec<NoteFile>), String> {
    Ok((
        read_document(root, "docs/macro-cost-benchmark-note-template.md")?,
        read_document(root, "docs/design.md")?,
        read_document(root, "docs/roadmap.md")?,
        read_notes(root)?,
    ))
}

fn check(template: &str, roadmap: &str, notes: &[NoteFile]) -> Result<(), String> {
    check_contract(template, &fixtures::design(), roadmap, notes)
}

fn fixture_root() -> Result<String, String> {
    let root = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
        .map_err(|error| format!("cannot open manifest directory: {error}"))?;
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let relative = format!(
        "target/macro-cost-note-contract-{}-{fixture_id}",
        std::process::id()
    );
    root.create_dir_all(format!("{relative}/docs/validation-notes"))
        .map_err(|error| format!("{relative}: cannot create fixture tree: {error}"))?;
    Ok(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative))
}

fn remove_fixture_root(path: &str) -> Result<(), String> {
    let root = Dir::open_ambient_dir(env!("CARGO_MANIFEST_DIR"), ambient_authority())
        .map_err(|error| format!("cannot open manifest directory: {error}"))?;
    let relative = path
        .strip_prefix(&format!("{}/", env!("CARGO_MANIFEST_DIR")))
        .ok_or_else(|| format!("fixture path is outside the manifest directory: {path}"))?;
    root.remove_dir_all(relative)
        .map_err(|error| format!("{path}: cannot remove fixture tree: {error}"))
}

#[test]
fn live_template_exists_and_open_tasks_accept_no_cost_notes() {
    let root = Utf8Path::new(env!("CARGO_MANIFEST_DIR"));
    let (template, design, roadmap, notes) =
        live_documents(root).expect("live repository documents are readable");
    check_contract(&template, &design, &roadmap, &notes)
        .expect("live documents satisfy the macro cost contract");
}

#[test]
fn missing_live_template_reports_its_path() {
    let root = fixture_root().expect("test fixture tree can be created");
    let error = read_document(
        Utf8Path::new(&root),
        "docs/macro-cost-benchmark-note-template.md",
    )
    .expect_err("missing template must be reported");
    remove_fixture_root(&root).expect("test fixture tree can be removed");
    assert!(
        error.contains("docs/macro-cost-benchmark-note-template.md"),
        "{error}"
    );
}

#[test]
fn directory_named_like_a_note_is_a_read_error() {
    let root = fixture_root().expect("test fixture tree can be created");
    let repository =
        Dir::open_ambient_dir(&root, ambient_authority()).expect("test fixture root can be opened");
    repository
        .open_dir("docs/validation-notes")
        .expect("fixture notes directory can be opened")
        .create_dir("unreadable.md")
        .expect("directory-shaped Markdown entry can be created");
    let error = read_notes(Utf8Path::new(&root))
        .expect_err("a directory-shaped Markdown note must be reported");
    remove_fixture_root(&root).expect("test fixture tree can be removed");
    assert!(error.contains("unreadable.md"), "{error}");
}

#[test]
fn absent_notes_directory_is_an_empty_collection() {
    let root = fixture_root().expect("test fixture tree can be created");
    let repository =
        Dir::open_ambient_dir(&root, ambient_authority()).expect("test fixture root can be opened");
    repository
        .open_dir("docs")
        .expect("fixture docs directory can be opened")
        .remove_dir("validation-notes")
        .expect("empty fixture notes directory can be removed");
    let notes = read_notes(Utf8Path::new(&root)).expect("missing notes directory means no notes");
    remove_fixture_root(&root).expect("test fixture tree can be removed");
    assert_eq!(notes, Vec::<NoteFile>::new());
}
