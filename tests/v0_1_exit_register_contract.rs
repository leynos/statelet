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

fn live_rows() -> Vec<Row> {
    match parse_register(ADR) {
        Ok(rows) => rows,
        Err(error) => panic!("{error}"),
    }
}

#[rstest]
#[case::b1_and_b2_falsified(Verdict::Falsified, Verdict::Falsified)]
#[case::b1_falsified_b2_held(Verdict::Falsified, Verdict::Held)]
#[case::b1_held_b2_falsified(Verdict::Held, Verdict::Falsified)]
#[case::b1_and_b2_held(Verdict::Held, Verdict::Held)]
fn totality_holds(#[case] b1: Verdict, #[case] b2: Verdict) {
    let rows = live_rows();
    check_totality(&rows)
        .expect("ADR 003 must provide one row for every final verdict combination");
    assert_that!(
        rows.iter()
            .filter(|row| row.b1 == b1 && row.b2 == b2)
            .count(),
        eq(1)
    );
}

#[test]
fn dominance_holds() {
    let rows = live_rows();
    check_dominance(&rows).expect("a falsified B1 must choose the off-ramp");
    let invalid = parse_register(&valid_register().replace("E1 ship nothing", "E3 ship macro"))
        .expect("the dominance control must remain syntactically valid");
    assert!(
        check_dominance(&invalid).is_err(),
        "the policy check must reject a syntactically valid E3 row"
    );
}

#[test]
fn quoted_passages_still_resolve() {
    check_quoted_clauses(ADR, DESIGN, TERMS, CONTEXT).expect("ADR 003 citations must resolve");
    let rewritten = DESIGN.replace(
        "the macro crate does not ship in v0.1",
        "the macro crate ships",
    );
    assert!(check_quoted_clauses(ADR, &rewritten, TERMS, CONTEXT).is_err());
}

#[test]
fn gate_bindings_resolve() {
    let rows = live_rows();
    check_gate_bindings(&rows, ADR, ROADMAP)
        .expect("ADR 003 gates must resolve to live roadmap tasks");
}

#[test]
fn gate_bindings_reject_wrong_adr_task() {
    let rows = live_rows();
    let changed_gate = ADR.replace("3.1.3", "3.1.4");
    assert_eq!(
        check_gate_bindings(&rows, &changed_gate, ROADMAP),
        Err(
            "docs/adr-003-v0-1-exit-register.md: G2 must bind roadmap task 3.1.3. Repair: restore \
             the gate table binding."
                .to_owned()
        )
    );
}

#[test]
fn gate_bindings_reject_ticked_roadmap_task() {
    let rows = live_rows();
    let ticked_task = ROADMAP.replace("- [ ] 3.1.3.", "- [x] 3.1.3.");
    assert_eq!(
        check_gate_bindings(&rows, ADR, &ticked_task),
        Err(
            "docs/roadmap.md: task 3.1.3 is absent or already ticked. Repair: retain the live, \
             unticked gate named by ADR 003."
                .to_owned()
        )
    );
}

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

#[rstest]
#[case(Verdict::Falsified, Verdict::Falsified, Exit::E1)]
#[case(Verdict::Falsified, Verdict::Held, Exit::E1)]
#[case(Verdict::Held, Verdict::Falsified, Exit::E2)]
#[case(Verdict::Held, Verdict::Held, Exit::E3)]
fn hand_written_cases_match_register(#[case] b1: Verdict, #[case] b2: Verdict, #[case] exit: Exit) {
    let rows = live_rows();
    let actual = rows
        .iter()
        .find(|row| row.b1 == b1 && row.b2 == b2)
        .map(|row| row.exit);
    assert_eq!(actual, Some(exit));
}

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
    assert!(
        check_totality(&parse_register(&missing).expect("missing-row control parses")).is_err()
    );
    assert!(
        check_totality(&parse_register(&duplicate).expect("duplicate control parses")).is_err()
    );
    assert!(check_totality(&parse_register(&extra).expect("extra-row control parses")).is_err());
}
