//! The roadmap's task-record grammar and the two checks that bind ADR 004 to
//! `docs/roadmap.md`.
//!
//! Both checks read the roadmap as *task records* rather than as lines, because
//! both claim to name a task, and `task_records` is the reader they share. The gate table binds
//! each gate to the title of the validation note's task, and the success criterion names the
//! sentence the task is graded on. A line scan cannot tell either from the prose around it:
//! `docs/roadmap.md` carries a phase heading called "Foundational contracts and
//! kill gates", a task whose link text is a filename, and dozens of sub-bullets,
//! and every one of them would satisfy a check that matched raw lines. The
//! grammar that separates them is `task_records`, below: it is the roadmap's
//! own and no other document's.
//!
//! The binding is by title rather than by task number, so that completing a
//! bound task or renumbering the roadmap does not break the build. The cost is
//! that a reworded title does break it, deliberately: the fragment is the link
//! between the two documents, and a link that no longer resolves must fail
//! loudly rather than quietly bind the wrong task.

use super::{
    fold_whitespace,
    parse::gate_rows,
    policy::IDENTIFIER_NEED,
    types::{StatusRow, TaskRecord},
};

/// The task records of a roadmap, in document order.
///
/// A record is a checklist line — `- [ ] 2.2.1. Title` — followed by every
/// indented line beneath it, which is where the roadmap writes a task's
/// sub-bullets and the continuation of its wrapped title. The first unindented
/// line ends the record, so prose a task is *not* part of can never be read as
/// one.
///
/// The title and the record's body are tracked separately, because they are
/// different claims. A *title* is what the gate table binds: a fragment there
/// must name the task. The body is where a task's sub-bullets live, including
/// the success criterion `check_success_criterion` resolves. A wrapped title
/// continues onto the indented line after it, and is folded back into the title;
/// the first sub-bullet — an indented line that opens with `-` — ends the title
/// and starts the body. Both folds are needed because `mdtablefix --wrap`
/// rewraps a long title at the column the surrounding bullet needs, so a title
/// fragment can be split across two lines.
///
/// A line that is not a checklist line simply is not a task and is skipped. The
/// recognition is exact rather than shape-based, for the reason
/// `parse::row_from_line` gives: a phase heading such as `## 1. Foundational
/// contracts and kill gates` carries a number and a following space, so a laxer
/// reader would take it for a task. There is no failure to report — a roadmap
/// with no tasks yields no records, and the checks consuming them say what that
/// means for them.
pub(crate) fn task_records(roadmap: &str) -> Vec<TaskRecord> {
    let mut records: Vec<TaskRecord> = Vec::new();
    let mut open = false;
    let mut in_title = false;
    for line in roadmap.lines() {
        if let Some(record) = record_from_line(line) {
            records.push(record);
            (open, in_title) = (true, true);
        } else if !(open && line.starts_with(' ')) {
            // The line is unindented, so it ends the open record. It must not be
            // absorbed into it: a line *between* tasks is prose, and absorbing
            // it would let a fragment naming a task resolve against prose.
            open = false;
        } else if let Some(record) = records.last_mut() {
            extend(record, line, &mut in_title);
        }
    }
    for record in &mut records {
        record.title = fold_whitespace(&record.title);
        record.text = fold_whitespace(&record.text);
    }
    records
}

/// Appends an indented line to the record it continues.
///
/// The line is either a sub-bullet or the continuation of a wrapped one, and a
/// sub-bullet is the one that carries a `- `. Whichever it is, the line belongs
/// to the record's text; it belongs to the *title* only while no sub-bullet has
/// been seen, because a title ends where its first sub-bullet begins.
fn extend(record: &mut TaskRecord, line: &str, in_title: &mut bool) {
    let rest = line.trim_start();
    let bullet = rest.strip_prefix("- ");
    let text = bullet.unwrap_or(rest);
    record.text.push(' ');
    record.text.push_str(text);
    if *in_title && bullet.is_none() {
        record.title.push(' ');
        record.title.push_str(text);
    } else if bullet.is_some() {
        *in_title = false;
    }
}

/// Reads one checklist line as the head of a task record.
///
/// The grammar the roadmap uses is `- [<mark>] <number>. <title>`, where the
/// mark is a space or an `x`. The title carries no other requirement.
fn record_from_line(line: &str) -> Option<TaskRecord> {
    let (mark, rest) = line.strip_prefix("- [")?.split_once("] ")?;
    if mark.len() != 1 || !matches!(mark, " " | "x" | "X") {
        return None;
    }
    let (number, title) = rest.split_once(". ")?;
    if number.is_empty() || !number.split('.').all(|part| part.parse::<u32>().is_ok()) {
        return None;
    }
    Some(TaskRecord {
        number: number.to_owned(),
        title: title.to_owned(),
        text: line.to_owned(),
    })
}

/// The four roadmap nouns of task 1.1.3's success criterion, and the register
/// field each must map to.
const CRITERION_NOUNS: [(&str, &str); 4] = [
    ("state display name", "state-display-name"),
    ("optional identifier need", IDENTIFIER_NEED),
    ("metrics cardinality", "metrics-cardinality"),
    ("tracing use", "tracing-use"),
];

/// The clause `docs/roadmap.md` must still carry for the criterion to resolve.
const SUCCESS_CRITERION_CLAUSE: &str = "the Phase 2 validation note template has fields for state \
                                        display name, optional identifier need, metrics \
                                        cardinality, and tracing use";

/// The fragment naming the task the criterion grades, in that task's *title*.
///
/// The check binds by title rather than by number, for the reason the gate table
/// does: completing a task or renumbering the roadmap must not break the build.
/// The cost is the same one the gate table pays — a reworded title breaks it —
/// and it is paid deliberately, because the alternative is a check that follows
/// the clause to whichever task happens to carry it.
pub(crate) const CRITERION_TASK_FRAGMENT: &str = "consumption question";

/// Checks that roadmap task 1.1.3's success bullet still resolves and that each
/// of its four nouns maps to exactly one register field.
///
/// The clause is looked for inside the task record *whose title names the task*,
/// not in the document at large and not in any task record that happens to carry
/// it. Both halves are load-bearing. A copy of the sentence in a phase
/// introduction, a step's framing prose or a task's rationale is not the
/// criterion, and a check over the whole document would report the criterion as
/// intact while the task it grades had lost it. A copy in *another task's*
/// record is not the criterion either: the criterion is what task 1.1.3 is
/// graded on, so moving it to 3.2.2 or to any neighbouring task leaves this one
/// ungraded, and a check accepting it from anywhere would call that a pass.
///
/// The title is matched against the record's *title* and the clause against its
/// *text*, because the two live in different places: the title is the checklist
/// line, and the clause is a sub-bullet in the body beneath it.
pub(crate) fn check_success_criterion(rows: &[StatusRow], roadmap: &str) -> Result<(), String> {
    let clause = fold_whitespace(SUCCESS_CRITERION_CLAUSE);
    let graded = task_records(roadmap)
        .into_iter()
        .filter(|record| record.title.contains(CRITERION_TASK_FRAGMENT))
        .collect::<Vec<TaskRecord>>();
    match graded.as_slice() {
        [] => {
            return Err(format!(
                "docs/roadmap.md no longer carries a task title naming \
                 {CRITERION_TASK_FRAGMENT:?}, which is how this check finds the task the success \
                 criterion grades. Repair: restore that task's title, or revise this contract \
                 together with the task."
            ));
        }
        [record] => {
            if !record.text.contains(&clause) {
                return Err(format!(
                    "docs/roadmap.md no longer contains the 1.1.3 success criterion. Repair: \
                     restore {SUCCESS_CRITERION_CLAUSE:?}, or revise this contract together with \
                     the task."
                ));
            }
        }
        records => {
            let titles = records
                .iter()
                .map(|record| format!("{:?}", record.title))
                .collect::<Vec<String>>()
                .join(", ");
            return Err(format!(
                "docs/roadmap.md: the title fragment {CRITERION_TASK_FRAGMENT:?} names {} tasks \
                 ({titles}), so the success criterion resolves against more than one. Repair: use \
                 a fragment specific to the task this contract grades.",
                records.len()
            ));
        }
    }
    for (noun, field) in CRITERION_NOUNS {
        if !rows.iter().any(|row| row.field == field) {
            return Err(format!(
                "docs/adr-004-state-name-consumption-evidence.md: no status-register field maps \
                 the criterion noun {noun:?}. Repair: add the {field} field, or reword the \
                 roadmap task."
            ));
        }
    }
    Ok(())
}

/// Checks each gate's title fragment against `docs/roadmap.md`.
///
/// A fragment binds a gate when it names exactly one task's *title*. It is
/// matched against the title alone, not against the whole record, because that
/// is what the gate table claims: ADR 004 binds each gate "by task title rather
/// than task number", and Table 4 says the gates are "bound to roadmap tasks by
/// title". Matching the record would also accept a fragment naming one of the
/// task's sub-bullets or its link text, which names no task at all — the repair
/// message offers to "restore that task's title", and a check that accepted a
/// sub-bullet would be promising something it never verified.
pub(crate) fn check_gate_titles(adr: &str, roadmap: &str) -> Result<(), String> {
    let records = task_records(roadmap);
    for row in gate_rows(adr).map_err(|error| error.to_string())? {
        let matches = records
            .iter()
            .filter(|record| record.title.contains(&row.fragment))
            .count();
        match matches {
            1 => {}
            0 => {
                return Err(format!(
                    "docs/roadmap.md: gate {} names task fragment {:?}, which matches no task. \
                     Repair: restore that task's title, or revise ADR 004's gate table.",
                    row.gate, row.fragment
                ));
            }
            _ => {
                return Err(format!(
                    "docs/roadmap.md: gate {} names task fragment {:?}, which matches {matches} \
                     tasks. Repair: use a fragment specific to one task.",
                    row.gate, row.fragment
                ));
            }
        }
    }
    Ok(())
}
