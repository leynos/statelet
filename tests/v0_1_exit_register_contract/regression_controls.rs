//! Regression scenarios for Markdown structure at the contract boundary.

use pretty_assertions::assert_eq;

use super::{
    ADR,
    ParseError,
    parse_register,
    support::{QUOTED_CLAUSE_DOCUMENTS, QuotedClauseDocuments, check_quoted_clauses},
};

/// Distinguishes indented headings from four-space code content in section 13.7.
#[test]
fn indented_headings_end_sections_but_code_lines_do_not() {
    let relocated_phrase = QUOTED_CLAUSE_DOCUMENTS
        .design
        .replace(
            "in either validation\nexample",
            "in both validation\nexamples",
        )
        .replace(
            "## 14. Deferred decisions",
            "   ## 14. Deferred decisions\n\nin either validation example",
        );
    assert_eq!(
        check_quoted_clauses(
            ADR,
            QuotedClauseDocuments {
                design: &relocated_phrase,
                ..QUOTED_CLAUSE_DOCUMENTS
            },
        ),
        Err(
            "docs/design.md §13.7 does not record the R1 split-case rule. Repair: amend section \
             13.7 to say either validation example."
                .to_owned()
        )
    );
    let code_line = QUOTED_CLAUSE_DOCUMENTS.design.replace(
        "in either validation\nexample",
        "    # in either validation example",
    );
    check_quoted_clauses(
        ADR,
        QuotedClauseDocuments {
            design: &code_line,
            ..QUOTED_CLAUSE_DOCUMENTS
        },
    )
    .expect("four-space code content remains inside section 13.7");
}

/// Resolves every quoted clause against the live source documents.
#[test]
fn quoted_passages_still_resolve() {
    check_quoted_clauses(ADR, QUOTED_CLAUSE_DOCUMENTS).expect("ADR 003 citations must resolve");
}

/// Refuses a rewritten design claim even when the ADR quote is unchanged.
#[test]
fn quoted_passages_reject_a_rewritten_design_claim() {
    let rewritten = QUOTED_CLAUSE_DOCUMENTS.design.replace(
        "the macro crate does not ship in v0.1",
        "the macro crate ships",
    );
    assert_eq!(
        check_quoted_clauses(
            ADR,
            QuotedClauseDocuments {
                design: &rewritten,
                ..QUOTED_CLAUSE_DOCUMENTS
            },
        ),
        Err(
            "docs/design.md no longer contains \"the macro crate does not ship in v0.1\". Repair: \
             update ADR 003 and its contract together."
                .to_owned()
        )
    );
}

/// Refuses a missing B1 bet row even if the words survive outside the table.
#[test]
fn quoted_passages_reject_a_missing_bet_row() {
    let b1_row_deleted = format!(
        "{}\nB1 still says both improve without framework adoption outside the table.",
        QUOTED_CLAUSE_DOCUMENTS.design.replace(
            "| B1  | A real segment prefers handwritten state machines and wants shared \
             convention | Low-medium | `mdtablefix` plus one second non-toy example both improve \
             without framework adoption          |\n",
            "",
        )
    );
    assert_eq!(
        check_quoted_clauses(
            ADR,
            QuotedClauseDocuments {
                design: &b1_row_deleted,
                ..QUOTED_CLAUSE_DOCUMENTS
            },
        ),
        Err(
            "docs/design.md is missing B1 or B2 from the bet table. Repair: restore the bet row \
             or revise ADR 003."
                .to_owned()
        )
    );
}

/// Refuses the R1 rule when its phrase is relocated outside section 13.7.
#[test]
fn quoted_passages_reject_a_relocated_split_case_rule() {
    let split_case_rewritten = QUOTED_CLAUSE_DOCUMENTS
        .design
        .replace(
            "in either validation\nexample",
            "in both validation\nexamples",
        )
        .replace(
            "## 14. Deferred decisions",
            "## 14. Deferred decisions\n\nin either validation example",
        );
    assert_eq!(
        check_quoted_clauses(
            ADR,
            QuotedClauseDocuments {
                design: &split_case_rewritten,
                ..QUOTED_CLAUSE_DOCUMENTS
            },
        ),
        Err(
            "docs/design.md §13.7 does not record the R1 split-case rule. Repair: amend section \
             13.7 to say either validation example."
                .to_owned()
        )
    );
}

/// Refuses a fabricated ADR citation absent from the source document.
#[test]
fn quoted_passages_reject_an_invented_citation() {
    let invented_citation = ADR.replace(
        "both improve without framework adoption",
        "a made-up validation clause",
    );
    assert_eq!(
        check_quoted_clauses(&invented_citation, QUOTED_CLAUSE_DOCUMENTS),
        Err(
            "docs/design.md no longer contains \"a made-up validation clause\". Repair: update \
             ADR 003 and its contract together."
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
    let b1_outside_its_table = QUOTED_CLAUSE_DOCUMENTS.design.replace(b1_row, "").replace(
        "### 11.2 Baseline comparison",
        &format!("### 11.2 Baseline comparison\n\n{b1_row}"),
    );
    assert_eq!(
        check_quoted_clauses(
            ADR,
            QuotedClauseDocuments {
                design: &b1_outside_its_table,
                ..QUOTED_CLAUSE_DOCUMENTS
            },
        ),
        Err(
            "docs/design.md is missing B1 or B2 from the bet table. Repair: restore the bet row \
             or revise ADR 003."
                .to_owned()
        )
    );
}

/// Parses marker text in ordinary cells and rejects malformed marker rows.
#[test]
fn marker_text_does_not_hide_register_rows() {
    let ordinary = "<!-- exit-register:begin -->\n| Held | Held | E3 ship macro | B1 verdict --- \
                    | yes |\n<!-- exit-register:end -->";
    assert_eq!(parse_register(ordinary).map(|rows| rows.len()), Ok(1));
    let unknown_verdict = "<!-- exit-register:begin -->\n| B1 verdict | Held | E1 ship nothing | \
                           G2 | yes |\n<!-- exit-register:end -->";
    assert_eq!(
        parse_register(unknown_verdict),
        Err(ParseError::UnknownVerdict {
            found: "B1 verdict".to_owned()
        })
    );
    let malformed = "<!-- exit-register:begin -->\n| Held | --- | E3 ship macro | G3 | yes | \
                     extra |\n<!-- exit-register:end -->";
    assert_eq!(
        parse_register(malformed),
        Err(ParseError::MalformedRow { line: 2 })
    );
}
