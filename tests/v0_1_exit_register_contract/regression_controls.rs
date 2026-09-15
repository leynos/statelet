//! Regression scenarios for Markdown structure at the contract boundary.

use pretty_assertions::assert_eq;

use super::{ADR, CONTEXT, DESIGN, ParseError, TERMS, check_quoted_clauses, parse_register};

/// Distinguishes indented headings from four-space code content in section 13.7.
#[test]
fn indented_headings_end_sections_but_code_lines_do_not() {
    let relocated_phrase = DESIGN
        .replace(
            "in either validation\nexample",
            "in both validation\nexamples",
        )
        .replace(
            "## 14. Deferred decisions",
            "   ## 14. Deferred decisions\n\nin either validation example",
        );
    assert_eq!(
        check_quoted_clauses(ADR, &relocated_phrase, TERMS, CONTEXT),
        Err(
            "docs/design.md §13.7 does not record the R1 split-case rule. Repair: amend section \
             13.7 to say either validation example."
                .to_owned()
        )
    );
    let code_line = DESIGN.replace(
        "in either validation\nexample",
        "    # in either validation example",
    );
    check_quoted_clauses(ADR, &code_line, TERMS, CONTEXT)
        .expect("four-space code content remains inside section 13.7");
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
