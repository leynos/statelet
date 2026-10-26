//! Guard the macro cost evidence form and the roadmap claims that consume it.

#[path = "macro_cost_note_contract/contract.rs"]
mod contract;
#[path = "macro_cost_note_contract/fixtures.rs"]
mod fixtures;

use std::sync::atomic::{AtomicUsize, Ordering};

use camino::{Utf8Path, Utf8PathBuf};
use cap_std::{ambient_authority, fs::Dir};
use contract::check_contract;
use rstest::rstest;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MarkdownFile {
    pub(crate) path: String,
    pub(crate) text: String,
}

struct Task {
    id: String,
    complete: bool,
    text: String,
}

struct Candidate {
    macro_revision: String,
    design_revision: String,
}

fn candidate_identity(task: &Task) -> Result<Candidate, String> {
    const PREFIX: &str = "<!-- macro-cost-candidate:";
    let Task {
        id, text: block, ..
    } = task;
    let comments = block
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with(PREFIX))
        .collect::<Vec<_>>();
    let [comment] = comments.as_slice() else {
        return Err(format!(
            "completed task {id} needs one macro-cost-candidate comment"
        ));
    };
    let tail = comment
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

fn linked_note_paths(task: &Task) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = task.text.as_str();
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

fn task_block(roadmap: &MarkdownFile, id: &str) -> Result<Task, String> {
    let lines = roadmap.text.lines().collect::<Vec<_>>();
    let matches = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| task_status(line, id).map(|complete| (index, complete)))
        .collect::<Vec<_>>();
    let [(start, complete)] = matches.as_slice() else {
        return Err(format!(
            "{} must contain exactly one checklist task {id}",
            roadmap.path
        ));
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| (line.starts_with("- [") || is_heading(line)).then_some(index))
        .unwrap_or(lines.len());
    Ok(Task {
        id: id.to_owned(),
        complete: *complete,
        text: lines
            .get(*start..end)
            .ok_or_else(|| format!("{} task {id} has an invalid source range", roadmap.path))?
            .join("\n"),
    })
}

fn task_status(line: &str, id: &str) -> Option<bool> {
    let (status, rest) = line.strip_prefix("- [")?.split_once(']')?;
    let complete = match status {
        "x" => true,
        " " => false,
        _ => return None,
    };
    if !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    rest.trim_start()
        .strip_prefix(id)
        .is_some_and(|suffix| suffix.starts_with(". "))
        .then_some(complete)
}

#[rstest]
#[case("- [x] 4.2.2. Require a synthetic note.", Some(true))]
#[case("- [x]  4.2.2. Require a synthetic note.", Some(true))]
#[case("- [ ]\t4.2.2. Require a synthetic note.", Some(false))]
#[case("- [x]4.2.2. Require a synthetic note.", None)]
#[case("- [y] 4.2.2. Require a synthetic note.", None)]
fn task_status_tolerates_checkbox_spacing(#[case] line: &str, #[case] expected: Option<bool>) {
    assert_eq!(task_status(line, "4.2.2"), expected);
}

fn is_heading(line: &str) -> bool { line.starts_with("## ") || line.starts_with("### ") }

fn read_document(root: &Utf8Path, path: &str) -> Result<String, String> {
    let directory = Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| format!("{root}: cannot open repository root: {error}"))?;
    directory
        .read_to_string(path)
        .map_err(|error| format!("{root}/{path}: {error}"))
}

fn read_notes(root: &Utf8Path) -> Result<Vec<MarkdownFile>, String> {
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
        notes.push(MarkdownFile {
            path: format!("docs/validation-notes/{name}"),
            text,
        });
    }
    notes.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(notes)
}

fn live_documents(
    root: &Utf8Path,
) -> Result<(MarkdownFile, MarkdownFile, MarkdownFile, Vec<MarkdownFile>), String> {
    let read = |path: &str| {
        read_document(root, path).map(|text| MarkdownFile {
            path: path.to_owned(),
            text,
        })
    };
    Ok((
        read("docs/macro-cost-benchmark-note-template.md")?,
        read("docs/design.md")?,
        read("docs/roadmap.md")?,
        read_notes(root)?,
    ))
}

fn check(template_text: &str, roadmap_text: &str, notes: &[MarkdownFile]) -> Result<(), String> {
    let template_file = MarkdownFile {
        path: "macro-cost template fixture".into(),
        text: template_text.into(),
    };
    let design = MarkdownFile {
        path: "design fixture".into(),
        text: fixtures::design(),
    };
    let roadmap_file = MarkdownFile {
        path: "roadmap fixture".into(),
        text: roadmap_text.into(),
    };
    check_contract(&template_file, &design, &roadmap_file, notes)
}

/// Owns one isolated filesystem tree for a macro-cost contract test.
struct FixtureRoot {
    path: Utf8PathBuf,
}

impl FixtureRoot {
    fn path(&self) -> &Utf8Path { &self.path }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        if let Err(error) = remove_fixture_root(&self.path) {
            assert!(
                std::thread::panicking(),
                "cannot clean up fixture tree {}: {error}",
                self.path
            );
        }
    }
}

fn fixture_root() -> Result<FixtureRoot, String> {
    let temporary_directory = Utf8Path::new(env!("CARGO_TARGET_TMPDIR"));
    let root = Dir::open_ambient_dir(temporary_directory, ambient_authority())
        .map_err(|error| format!("cannot open Cargo test temporary directory: {error}"))?;
    let fixture_id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
    let relative = format!(
        "macro-cost-note-contract-{}-{fixture_id}",
        std::process::id()
    );
    let fixture = FixtureRoot {
        path: temporary_directory.join(&relative),
    };
    root.create_dir_all(format!("{relative}/docs/validation-notes"))
        .map_err(|error| format!("{relative}: cannot create fixture tree: {error}"))?;
    Ok(fixture)
}

fn remove_fixture_root(path: &Utf8Path) -> Result<(), String> {
    let temporary_directory = Utf8Path::new(env!("CARGO_TARGET_TMPDIR"));
    let root = Dir::open_ambient_dir(temporary_directory, ambient_authority())
        .map_err(|error| format!("cannot open Cargo test temporary directory: {error}"))?;
    let relative = path
        .strip_prefix(temporary_directory)
        .map_err(|_| format!("fixture path is outside Cargo test temporary directory: {path}"))?;
    match root.remove_dir_all(relative) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{path}: cannot remove fixture tree: {error}")),
    }
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
fn fixture_root_is_removed_when_its_guard_drops() {
    let root = fixture_root().expect("test fixture tree can be created");
    let path = root.path().to_owned();
    drop(root);

    assert!(
        Dir::open_ambient_dir(&path, ambient_authority()).is_err(),
        "dropping the fixture guard removes its owned tree"
    );
}

#[test]
fn missing_live_template_reports_its_path() {
    let root = fixture_root().expect("test fixture tree can be created");
    let error = read_document(root.path(), "docs/macro-cost-benchmark-note-template.md")
        .expect_err("missing template must be reported");
    assert!(
        error.contains("docs/macro-cost-benchmark-note-template.md"),
        "{error}"
    );
}

#[test]
fn directory_named_like_a_note_is_a_read_error() {
    let root = fixture_root().expect("test fixture tree can be created");
    let repository = Dir::open_ambient_dir(root.path(), ambient_authority())
        .expect("test fixture root can be opened");
    repository
        .open_dir("docs/validation-notes")
        .expect("fixture notes directory can be opened")
        .create_dir("unreadable.md")
        .expect("directory-shaped Markdown entry can be created");
    let error =
        read_notes(root.path()).expect_err("a directory-shaped Markdown note must be reported");
    assert!(error.contains("unreadable.md"), "{error}");
}

#[test]
fn absent_notes_directory_is_an_empty_collection() {
    let root = fixture_root().expect("test fixture tree can be created");
    let repository = Dir::open_ambient_dir(root.path(), ambient_authority())
        .expect("test fixture root can be opened");
    repository
        .open_dir("docs")
        .expect("fixture docs directory can be opened")
        .remove_dir("validation-notes")
        .expect("empty fixture notes directory can be removed");
    let notes = read_notes(root.path()).expect("missing notes directory means no notes");
    assert_eq!(notes, Vec::<MarkdownFile>::new());
}
