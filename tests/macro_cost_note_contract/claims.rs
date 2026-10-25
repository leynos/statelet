//! Candidate-bound roadmap claims for macro cost validation notes.

use super::full_revision;

pub(super) struct Task {
    pub(super) complete: bool,
    pub(super) text: String,
}

pub(super) struct Candidate {
    pub(super) macro_revision: String,
    pub(super) design_revision: String,
}

pub(super) fn candidate_identity(block: &str, id: &str) -> Result<Candidate, String> {
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
    if !full_revision(macro_revision) || !full_revision(design_revision) {
        return Err(format!(
            "completed task {id} comment requires full lowercase revisions"
        ));
    }
    Ok(Candidate {
        macro_revision: macro_revision.into(),
        design_revision: design_revision.into(),
    })
}

pub(super) fn linked_note_paths(block: &str) -> Vec<String> {
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

pub(super) fn task_block(roadmap: &str, id: &str) -> Result<Task, String> {
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

fn is_heading(line: &str) -> bool { line.starts_with("## ") || line.starts_with("### ") }
