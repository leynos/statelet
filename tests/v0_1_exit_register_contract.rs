//! Contract tests for the v0.1 exit register.
//!
//! The tests treat the decision record as the source of truth. They verify its
//! syntax separately from the policies it records, so a malformed table and a
//! mistaken exit rule report different repairs.

#[path = "v0_1_exit_register_contract/support.rs"]
mod support;

use googletest::prelude::*;
use pretty_assertions::assert_eq;
use rstest::rstest;
use support::{
    Exit,
    ParseError,
    Row,
    Verdict,
    check_dominance,
    check_gate_bindings,
    check_quoted_clauses,
    check_totality,
    parse_register,
    valid_register,
};

const ADR: &str = include_str!("../docs/adr-003-v0-1-exit-register.md");
const DESIGN: &str = include_str!("../docs/design.md");
const TERMS: &str = include_str!("../docs/terms-of-reference.md");
const CONTEXT: &str = include_str!("../docs/context.md");
const ROADMAP: &str = include_str!("../docs/roadmap.md");

/// Parses the live ADR; for example, its complete register yields four rows.
fn live_rows() -> Result<Vec<Row>, String> {
    parse_register(ADR).map_err(|error| error.to_string())
}
/// Produces a syntactically valid register whose dominated row selects E3.
fn dominance_invalid_rows() -> Result<Vec<Row>, String> {
    let source = valid_register().replace(
        "| Falsified  | Held       | E1 ship nothing          | G2   | no        |",
        "| Falsified  | Held       | E3 ship macro            | G2   | no        |",
    );
    parse_register(&source).map_err(|error| error.to_string())
}
/// Covers each final B1/B2 verdict combination exactly once.
#[rstest]
#[case::b1_and_b2_falsified(Verdict::Falsified, Verdict::Falsified)]
#[case::b1_falsified_b2_held(Verdict::Falsified, Verdict::Held)]
#[case::b1_held_b2_falsified(Verdict::Held, Verdict::Falsified)]
#[case::b1_and_b2_held(Verdict::Held, Verdict::Held)]
fn totality_holds(#[case] b1: Verdict, #[case] b2: Verdict) -> Result<(), String> {
    let rows = live_rows()?;
    check_totality(&rows)
        .expect("ADR 003 must provide one row for every final verdict combination");
    assert_that!(
        rows.iter()
            .filter(|row| row.b1 == b1 && row.b2 == b2)
            .count(),
        eq(1)
    );
    Ok(())
}

/// Enforces B1 dominance and checks its prescribed repair message.
#[test]
fn dominance_holds() -> Result<(), String> {
    let rows = live_rows()?;
    check_dominance(&rows).expect("a falsified B1 must choose the off-ramp");
    let invalid = dominance_invalid_rows()?;
    assert_eq!(
        check_dominance(&invalid),
        Err(
            "docs/adr-003-v0-1-exit-register.md: Falsified/Held selects E3. Repair: a falsified \
             B1 must select E1 ship nothing."
                .to_owned()
        )
    );
    Ok(())
}

/// Rejects a row that marks the dominated Falsified/Held combination reachable.
#[test]
fn dominance_rejects_a_reachable_dead_row() -> Result<(), String> {
    let source = valid_register().replace(
        "| Falsified  | Held       | E1 ship nothing          | G2   | no        |",
        "| Falsified  | Held       | E1 ship nothing          | G2   | yes       |",
    );
    let invalid = parse_register(&source).map_err(|error| error.to_string())?;
    assert_eq!(
        check_dominance(&invalid),
        Err(
            "docs/adr-003-v0-1-exit-register.md: Falsified/Held must be marked unreachable. \
             Repair: set its Reachable cell to no."
                .to_owned()
        )
    );
    Ok(())
}

/// Resolves ADR evidence and rejects rewritten, relocated, and fabricated clauses.
#[test]
fn quoted_passages_still_resolve() {
    check_quoted_clauses(ADR, DESIGN, TERMS, CONTEXT).expect("ADR 003 citations must resolve");
    let rewritten = DESIGN.replace(
        "the macro crate does not ship in v0.1",
        "the macro crate ships",
    );
    assert_eq!(
        check_quoted_clauses(ADR, &rewritten, TERMS, CONTEXT),
        Err(
            "docs/design.md no longer contains \"the macro crate does not ship in v0.1\". Repair: \
             update ADR 003 and its contract together."
                .to_owned()
        )
    );
    let b1_row_deleted = format!(
        "{}\nB1 still says both improve without framework adoption outside the table.",
        DESIGN.replace(
            "| B1  | A real segment prefers handwritten state machines and wants shared \
             convention | Low-medium | `mdtablefix` plus one second non-toy example both improve \
             without framework adoption          |\n",
            "",
        )
    );
    assert_eq!(
        check_quoted_clauses(ADR, &b1_row_deleted, TERMS, CONTEXT),
        Err(
            "docs/design.md is missing B1 or B2 from the bet table. Repair: restore the bet row \
             or revise ADR 003."
                .to_owned()
        )
    );
    let split_case_rewritten = DESIGN
        .replace(
            "in either validation\nexample",
            "in both validation\nexamples",
        )
        .replace(
            "## 14. Deferred decisions",
            "## 14. Deferred decisions\n\nin either validation example",
        );
    assert_eq!(
        check_quoted_clauses(ADR, &split_case_rewritten, TERMS, CONTEXT),
        Err(
            "docs/design.md §13.7 does not record the R1 split-case rule. Repair: amend section \
             13.7 to say either validation example."
                .to_owned()
        )
    );
    let invented_citation = ADR.replace(
        "both improve without framework adoption",
        "a made-up validation clause",
    );
    assert_eq!(
        check_quoted_clauses(&invented_citation, DESIGN, TERMS, CONTEXT),
        Err(
            "docs/design.md no longer contains \"a made-up validation clause\". Repair: update \
             ADR 003 and its contract together."
                .to_owned()
        )
    );
}

/// Resolves every ADR gate to its live, unticked roadmap task.
#[test]
fn gate_bindings_resolve() -> Result<(), String> {
    let rows = live_rows()?;
    check_gate_bindings(&rows, ADR, ROADMAP)
        .expect("ADR 003 gates must resolve to live roadmap tasks");
    Ok(())
}

/// Rejects a gate table entry whose task identifier differs from its binding.
#[test]
fn gate_bindings_reject_wrong_adr_task() -> Result<(), String> {
    let rows = live_rows()?;
    let changed_gate = ADR.replace("3.1.3", "3.1.4");
    assert_eq!(
        check_gate_bindings(&rows, &changed_gate, ROADMAP),
        Err(
            "docs/adr-003-v0-1-exit-register.md: G2 must bind roadmap task 3.1.3. Repair: restore \
             the gate table binding."
                .to_owned()
        )
    );
    Ok(())
}

/// Rejects an ADR gate table that omits the required G2 identifier.
#[test]
fn gate_bindings_reject_unknown_adr_gate_table_identifier() -> Result<(), String> {
    let rows = live_rows()?;
    let changed_gate = ADR.replace(
        "| G2   | 3.1.3        | B1 across both validation examples                             |",
        "| G9   | 3.1.3        | B1 across both validation examples                             |",
    );
    assert_eq!(
        check_gate_bindings(&rows, &changed_gate, ROADMAP),
        Err(
            "docs/adr-003-v0-1-exit-register.md: G2 must bind roadmap task 3.1.3. Repair: restore \
             the gate table binding."
                .to_owned()
        )
    );
    Ok(())
}

/// Rejects a roadmap task that has already been marked complete.
#[test]
fn gate_bindings_reject_ticked_roadmap_task() -> Result<(), String> {
    let rows = live_rows()?;
    let ticked_task = ROADMAP.replace("- [ ] 3.1.3.", "- [x] 3.1.3.");
    assert_eq!(
        check_gate_bindings(&rows, ADR, &ticked_task),
        Err(
            "docs/roadmap.md: task 3.1.3 is absent or already ticked. Repair: retain the live, \
             unticked gate named by ADR 003."
                .to_owned()
        )
    );
    Ok(())
}

/// Rejects a prefix-only task match after the required task is renumbered.
#[test]
fn gate_bindings_reject_a_roadmap_task_prefix() -> Result<(), String> {
    let rows = live_rows()?;
    let renumbered_task = ROADMAP.replace("- [ ] 3.1.3.", "- [ ] 3.1.30.");
    assert_eq!(
        check_gate_bindings(&rows, ADR, &renumbered_task),
        Err(
            "docs/roadmap.md: task 3.1.3 is absent or already ticked. Repair: retain the live, \
             unticked gate named by ADR 003."
                .to_owned()
        )
    );
    Ok(())
}

/// Rejects a register row that uses an identifier outside the gate table.
#[test]
fn gate_bindings_reject_unknown_row_gate() {
    let rows = parse_register(&valid_register().replace("G2", "G9"))
        .expect("the unknown-gate control must remain syntactically valid");
    assert_eq!(
        check_gate_bindings(&rows, ADR, ROADMAP),
        Err(
            "docs/adr-003-v0-1-exit-register.md: row uses unknown gate G9. Repair: use G1, G2, or \
             G3."
            .to_owned()
        )
    );
}

/// Rejects a row that selects a known gate for the wrong exit path.
#[test]
fn gate_bindings_reject_wrong_known_row_gate() {
    let rows = parse_register(&valid_register().replace(
        "| Held       | Held       | E3 ship macro            | G3   | yes       |",
        "| Held       | Held       | E3 ship macro            | G1   | yes       |",
    ))
    .expect("the wrong-known-gate control must remain syntactically valid");
    assert_eq!(
        check_gate_bindings(&rows, ADR, ROADMAP),
        Err(
            "docs/adr-003-v0-1-exit-register.md: Held/Held with E3 must use gate G3, not G1. \
             Repair: bind this exit row to G3."
                .to_owned()
        )
    );
}

/// Rejects a B1 row copied outside section 11.1 of the design document.
#[test]
fn quoted_passages_reject_a_bet_outside_its_table() {
    let b1_row = "| B1  | A real segment prefers handwritten state machines and wants shared \
                  convention | Low-medium | `mdtablefix` plus one second non-toy example both \
                  improve without framework adoption          |\n";
    let b1_outside_its_table = DESIGN.replace(b1_row, "").replace(
        "### 11.2 Baseline comparison",
        &format!("### 11.2 Baseline comparison\n\n{b1_row}"),
    );
    assert_eq!(
        check_quoted_clauses(ADR, &b1_outside_its_table, TERMS, CONTEXT),
        Err(
            "docs/design.md is missing B1 or B2 from the bet table. Repair: restore the bet row \
             or revise ADR 003."
                .to_owned()
        )
    );
}

/// Matches each handwritten exit expectation against the parsed register.
#[rstest]
#[case(Verdict::Falsified, Verdict::Falsified, Exit::E1)]
#[case(Verdict::Falsified, Verdict::Held, Exit::E1)]
#[case(Verdict::Held, Verdict::Falsified, Exit::E2)]
#[case(Verdict::Held, Verdict::Held, Exit::E3)]
fn hand_written_cases_match_register(
    #[case] b1: Verdict,
    #[case] b2: Verdict,
    #[case] exit: Exit,
) -> Result<(), String> {
    check_hand_written_case(&live_rows()?, b1, b2, exit)
        .expect("the handwritten expectation must match the live register");
    Ok(())
}

/// Rejects handwritten expectations when the dominated row selects E3.
#[test]
fn hand_written_cases_reject_a_dominance_invalid_register() -> Result<(), String> {
    assert_eq!(
        check_hand_written_case(
            &dominance_invalid_rows()?,
            Verdict::Falsified,
            Verdict::Held,
            Exit::E1,
        ),
        Err(
            "docs/adr-003-v0-1-exit-register.md: Falsified/Held selects Some(E3). Repair: restore \
             its handwritten exit expectation to E1."
                .to_owned()
        )
    );
    Ok(())
}

/// Compares one parsed row with its expected exit; for example, Held/Held maps to E3.
fn check_hand_written_case(
    rows: &[Row],
    b1: Verdict,
    b2: Verdict,
    expected_exit: Exit,
) -> Result<(), String> {
    let actual = rows
        .iter()
        .find(|row| row.b1 == b1 && row.b2 == b2)
        .map(|row| row.exit);
    if actual == Some(expected_exit) {
        Ok(())
    } else {
        Err(format!(
            "docs/adr-003-v0-1-exit-register.md: {b1:?}/{b2:?} selects {actual:?}. Repair: \
             restore its handwritten exit expectation to {expected_exit:?}."
        ))
    }
}

/// Rejects empty, malformed, and unknown-value register fixtures.
#[rstest]
#[case::empty("", ParseError::MissingDelimiters)]
#[case::empty_block(
    "<!-- exit-register:begin -->\n<!-- exit-register:end -->",
    ParseError::MissingDelimiters
)]
#[case::unknown_verdict("<!-- exit-register:begin -->\n| Maybe | Held | E1 ship nothing | G2 | yes |\n<!-- exit-register:end -->", ParseError::UnknownVerdict { found: "Maybe".to_owned() })]
#[case::malformed_row("<!-- exit-register:begin -->\n| Held | E3 ship macro | G3 | yes |\n<!-- exit-register:end -->", ParseError::MalformedRow { line: 2 })]
fn parser_rejects_malformed_registers(#[case] source: &str, #[case] expected: ParseError) {
    assert_eq!(parse_register(source), Err(expected));
}

/// Distinguishes missing, duplicate, and extra-row totality failures.
#[test]
fn totality_rejects_missing_duplicate_and_extra_rows() {
    let missing = valid_register().replace(
        "| Held       | Held       | E3 ship macro            | G3   | yes       |\n",
        "",
    );
    let duplicate = valid_register().replace(
        "| Held       | Held       | E3 ship macro            | G3   | yes       |",
        "| Falsified  | Held       | E1 ship nothing          | G2   | no        |",
    );
    let extra = valid_register().replace(
        "<!-- exit-register:end -->",
        "| Held | Held | E3 ship macro | G3 | yes |\n<!-- exit-register:end -->",
    );
    assert_eq!(
        check_totality(&parse_register(&missing).expect("missing-row control parses")),
        Err(
            "docs/adr-003-v0-1-exit-register.md: missing Held/Held. Repair: add that verdict \
             combination."
                .to_owned()
        )
    );
    assert_eq!(
        check_totality(&parse_register(&duplicate).expect("duplicate control parses")),
        Err(
            "docs/adr-003-v0-1-exit-register.md: ambiguous Falsified/Held. Repair: keep exactly \
             one row for that combination."
                .to_owned()
        )
    );
    assert_eq!(
        check_totality(&parse_register(&extra).expect("extra-row control parses")),
        Err(
            "docs/adr-003-v0-1-exit-register.md: register must contain exactly four rows. Repair: \
             provide one row for each B1/B2 verdict combination."
                .to_owned()
        )
    );
}
