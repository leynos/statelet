//! Parsing and policy checks for the v0.1 exit-register contract tests.

use std::fmt::{self, Display, Formatter};

const BEGIN: &str = "<!-- exit-register:begin -->";
const END: &str = "<!-- exit-register:end -->";
const GATES: [(&str, &str); 3] = [("G1", "2.2.3"), ("G2", "3.1.3"), ("G3", "4.3.1")];
const CLAUSES: [(&str, &str); 5] = [
    ("design.md", "both improve without framework adoption"),
    ("design.md", "states the concrete value added by the macro"),
    ("design.md", "the macro crate does not ship in v0.1"),
    (
        "design.md",
        "the project ships the conventions/runtime crate and defers",
    ),
    (
        "design.md",
        "the project should ship nothing and keep the pattern local",
    ),
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum Verdict {
    Falsified,
    Held,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Exit {
    E1,
    E2,
    E3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Row {
    pub(super) b1: Verdict,
    pub(super) b2: Verdict,
    pub(super) exit: Exit,
    gate: String,
    reachable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ParseError {
    MissingDelimiters,
    MalformedRow { line: usize },
    UnknownVerdict { found: String },
    UnknownExit { found: String },
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDelimiters => write!(
                formatter,
                "docs/adr-003-v0-1-exit-register.md: no exit register found between {BEGIN} and \
                 {END}. Repair: add the register block to the Exit register section."
            ),
            Self::MalformedRow { line } => write!(
                formatter,
                "docs/adr-003-v0-1-exit-register.md:{line}: malformed exit-register row. Repair: \
                 supply B1, B2, Exit, Gate and Reachable cells."
            ),
            Self::UnknownVerdict { found } => write!(
                formatter,
                "docs/adr-003-v0-1-exit-register.md: unknown verdict {found:?}. Repair: use \
                 Falsified or Held."
            ),
            Self::UnknownExit { found } => write!(
                formatter,
                "docs/adr-003-v0-1-exit-register.md: unknown exit {found:?}. Repair: use E1 ship \
                 nothing, E2 ship conventions only, or E3 ship macro."
            ),
        }
    }
}

pub(super) fn parse_register(adr: &str) -> Result<Vec<Row>, ParseError> {
    let Some((_, after_begin)) = adr.split_once(BEGIN) else {
        return Err(ParseError::MissingDelimiters);
    };
    let Some((register, _)) = after_begin.split_once(END) else {
        return Err(ParseError::MissingDelimiters);
    };
    let rows = register
        .lines()
        .enumerate()
        .filter_map(|(offset, line)| row_from_line(line, offset + 1).transpose())
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        Err(ParseError::MissingDelimiters)
    } else {
        Ok(rows)
    }
}

fn row_from_line(line: &str, line_number: usize) -> Result<Option<Row>, ParseError> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return Ok(None);
    }
    if is_register_header_or_divider(trimmed) {
        return Ok(None);
    }
    let cells = trimmed
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect::<Vec<_>>();
    let [b1, b2, exit, gate, reachable] = cells.as_slice() else {
        return Err(ParseError::MalformedRow { line: line_number });
    };
    Ok(Some(Row {
        b1: parse_verdict(b1)?,
        b2: parse_verdict(b2)?,
        exit: parse_exit(exit)?,
        gate: (*gate).to_owned(),
        reachable: parse_reachability(reachable, line_number)?,
    }))
}

fn is_register_header_or_divider(row: &str) -> bool {
    row.contains("B1 verdict") || row.contains("---")
}

fn parse_verdict(value: &str) -> Result<Verdict, ParseError> {
    match value {
        "Falsified" => Ok(Verdict::Falsified),
        "Held" => Ok(Verdict::Held),
        _ => Err(ParseError::UnknownVerdict {
            found: value.to_owned(),
        }),
    }
}

fn parse_exit(value: &str) -> Result<Exit, ParseError> {
    match value {
        "E1 ship nothing" => Ok(Exit::E1),
        "E2 ship conventions only" => Ok(Exit::E2),
        "E3 ship macro" => Ok(Exit::E3),
        _ => Err(ParseError::UnknownExit {
            found: value.to_owned(),
        }),
    }
}

fn parse_reachability(value: &str, line: usize) -> Result<bool, ParseError> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ => Err(ParseError::MalformedRow { line }),
    }
}

pub(super) fn check_totality(rows: &[Row]) -> Result<(), String> {
    if rows.len() != 4 {
        return Err(
            "docs/adr-003-v0-1-exit-register.md: register must contain exactly four rows. Repair: \
             provide one row for each B1/B2 verdict combination."
                .to_owned(),
        );
    }
    for (b1, b2) in verdict_combinations() {
        let matches = rows
            .iter()
            .filter(|row| row.b1 == b1 && row.b2 == b2)
            .count();
        if matches == 0 {
            return Err(format!(
                "docs/adr-003-v0-1-exit-register.md: missing {b1:?}/{b2:?}. Repair: add that \
                 verdict combination."
            ));
        }
        if matches > 1 {
            return Err(format!(
                "docs/adr-003-v0-1-exit-register.md: ambiguous {b1:?}/{b2:?}. Repair: keep \
                 exactly one row for that combination."
            ));
        }
    }
    Ok(())
}

pub(super) fn check_dominance(rows: &[Row]) -> Result<(), String> {
    if let Some(row) = rows
        .iter()
        .find(|row| row.b1 == Verdict::Falsified && row.exit != Exit::E1)
    {
        return Err(format!(
            "docs/adr-003-v0-1-exit-register.md: {:?}/{:?} selects {:?}. Repair: a falsified B1 \
             must select E1 ship nothing.",
            row.b1, row.b2, row.exit
        ));
    }
    Ok(())
}

pub(super) fn check_quoted_clauses(
    adr: &str,
    design: &str,
    terms: &str,
    context: &str,
) -> Result<(), String> {
    let folded_adr = fold_whitespace(adr);
    let folded_design = fold_whitespace(design);
    for (source, clause) in CLAUSES {
        if !folded_adr.contains(clause) {
            return Err(format!(
                "docs/adr-003-v0-1-exit-register.md omits {clause:?} from {source}. Repair: \
                 restore the cited clause."
            ));
        }
        if !folded_design.contains(clause) {
            return Err(format!(
                "docs/design.md no longer contains {clause:?}. Repair: update ADR 003 and its \
                 contract together."
            ));
        }
    }
    if !has_table_bet(design, "B1") || !has_table_bet(design, "B2") {
        return Err(
            "docs/design.md is missing B1 or B2 from the bet table. Repair: restore the bet row \
             or revise ADR 003."
                .to_owned(),
        );
    }
    if !fold_whitespace(terms).contains("If either validation example shows that") {
        return Err(
            "docs/terms-of-reference.md does not record the R1 split-case rule. Repair: amend \
             section 7.1 to say either validation example."
                .to_owned(),
        );
    }
    if !context.contains("### v0.1 exit") {
        return Err(
            "docs/context.md lacks the v0.1 exit glossary entry. Repair: define the term beside \
             the bet register."
                .to_owned(),
        );
    }
    Ok(())
}

pub(super) fn check_gate_bindings(rows: &[Row], adr: &str, roadmap: &str) -> Result<(), String> {
    for (gate, task) in GATES {
        if gate_task(adr, gate).as_deref() != Some(task) {
            return Err(format!(
                "docs/adr-003-v0-1-exit-register.md: {gate} must bind roadmap task {task}. \
                 Repair: restore the gate table binding."
            ));
        }
        if !roadmap
            .lines()
            .any(|line| line.trim_start().starts_with(&format!("- [ ] {task}")))
        {
            return Err(format!(
                "docs/roadmap.md: task {task} is absent or already ticked. Repair: retain the \
                 live, unticked gate named by ADR 003."
            ));
        }
    }
    for row in rows {
        if !GATES.iter().any(|(gate, _)| *gate == row.gate) {
            return Err(format!(
                "docs/adr-003-v0-1-exit-register.md: row uses unknown gate {}. Repair: use G1, \
                 G2, or G3.",
                row.gate
            ));
        }
    }
    Ok(())
}

pub(super) fn valid_register() -> String {
    format!(
        "{BEGIN}\n| B1 verdict | B2 verdict | Exit | Gate | Reachable |\n| --- | --- | --- | --- \
         | --- |\n| Falsified  | Falsified  | E1 ship nothing          | G2   | yes       |\n| \
         Falsified  | Held       | E1 ship nothing          | G2   | no        |\n| Held       | \
         Falsified  | E2 ship conventions only | G3   | yes       |\n| Held       | Held       | \
         E3 ship macro            | G3   | yes       |\n{END}"
    )
}

fn fold_whitespace(text: &str) -> String { text.split_whitespace().collect::<Vec<_>>().join(" ") }

fn gate_task(adr: &str, gate: &str) -> Option<String> {
    adr.lines().find_map(|line| {
        let cells = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect::<Vec<_>>();
        match cells.as_slice() {
            [found_gate, task, _] if *found_gate == gate => Some((*task).to_owned()),
            _ => None,
        }
    })
}

fn has_table_bet(design: &str, bet: &str) -> bool {
    design.lines().any(|line| {
        line.trim()
            .trim_matches('|')
            .split('|')
            .next()
            .is_some_and(|cell| cell.trim() == bet)
    })
}

const fn verdict_combinations() -> [(Verdict, Verdict); 4] {
    [
        (Verdict::Falsified, Verdict::Falsified),
        (Verdict::Falsified, Verdict::Held),
        (Verdict::Held, Verdict::Falsified),
        (Verdict::Held, Verdict::Held),
    ]
}
