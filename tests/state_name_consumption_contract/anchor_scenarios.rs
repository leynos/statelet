//! Anchor scenarios: the template, the quoted clauses, and the roadmap bindings.
//!
//! Each of these checks a *link* between two documents: the blank form against
//! the register it instantiates, the clauses ADR 004 quotes against the
//! sections they came from, and the gate table and success criterion against
//! the live roadmap. A drift on either side of any pair fails here.

use pretty_assertions::assert_eq;
use rstest::rstest;

use super::{
    ADR,
    ADR_002,
    DESIGN,
    EMPTY_CLAUSE_LIST,
    ROADMAP,
    STRONGER,
    clauses,
    fixtures::{TEMPLATE, gate_table, status_register_without},
    live_status,
    mutated,
    parse::{gate_rows, note_rows, status_rows},
    registers::{check_deferred_clause, check_gate_titles, check_success_criterion},
    types::{Register, field_order},
};

/// Checks the blank form field-by-field against the register.
///
/// The form is read from `TEMPLATE`, which Step 6 replaces with the
/// `include_str!` of `docs/phase-2-validation-note-template.md`.
#[test]
fn template_matches_the_status_register() -> Result<(), String> {
    let rows = live_status()?;
    let template = note_rows(TEMPLATE).map_err(|error| error.to_string())?;
    assert_eq!(
        template
            .iter()
            .map(|row| row.field.clone())
            .collect::<Vec<String>>(),
        field_order(&rows)
    );
    for row in &template {
        assert_eq!(row.status, "TBD", "field {} must ship blank", row.field);
        assert_eq!(row.evidence, "TBD", "field {} must ship blank", row.field);
    }
    Ok(())
}

/// Resolves the three quoted clauses, and rejects a rewritten clause, a
/// relocated clause, a fabricated one, a mis-attributed one, and an emptied
/// evidence section.
#[test]
fn quoted_passages_still_resolve() -> Result<(), String> {
    clauses::check_quoted_clauses(ADR, DESIGN, ROADMAP, ADR_002)?;
    // The check reports the file a reader must open, not the attribution word
    // the ADR uses for it.
    let drifted = |path: &str, quoted: &str| {
        Err(format!(
            "{path} no longer contains the quoted clause {quoted:?} under \"6.1 State naming\". \
             Repair: update ADR 004 and its contract together."
        ))
    };
    // The source no longer carries the clause the ADR quotes. The reported
    // text is the ADR's quotation, because that is what failed to resolve.
    let rewritten = mutated(DESIGN, "stronger", "a stronger type");
    assert_eq!(
        clauses::check_quoted_clauses(ADR, &rewritten, ROADMAP, ADR_002),
        drifted("docs/design.md", STRONGER)
    );
    // The clause is still in `docs/design.md`, word for word, but a heading now
    // ends §6.1 before it. A resolver that searched the whole document rather
    // than the named section would accept it.
    let relocated = mutated(
        DESIGN,
        "The `mdtablefix` baseline",
        "### 6.1.1 Superseded\n\nThe `mdtablefix` baseline",
    );
    assert_eq!(
        clauses::check_quoted_clauses(ADR, &relocated, ROADMAP, ADR_002),
        drifted("docs/design.md", STRONGER)
    );
    // The ADR quotes a clause its source never carried. The needle is a single
    // word: `mdtablefix --wrap` breaks the quoted clause between "something"
    // and "stronger", so any longer needle would be split by the formatter.
    let fabricated = mutated(ADR, "stronger", "weaker");
    assert_eq!(
        clauses::check_quoted_clauses(&fabricated, DESIGN, ROADMAP, ADR_002),
        drifted(
            "docs/design.md",
            "The default remains `&'static str` until a real example consumes something weaker"
        )
    );
    // The clause is real and unmoved, but the ADR now credits it to a document
    // this contract does not read.
    let misattributed = mutated(ADR, "— design", "— context");
    assert_eq!(
        clauses::check_quoted_clauses(&misattributed, DESIGN, ROADMAP, ADR_002),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md attributes a clause to \"context\", \
             which this contract does not read. Repair: use design, roadmap or adr-002."
                .to_owned()
        )
    );
    // The evidence section still holds its delimiters but no clause at all.
    let emptied = empty_evidence_block(ADR);
    assert_eq!(
        clauses::check_quoted_clauses(&emptied, DESIGN, ROADMAP, ADR_002),
        Err(EMPTY_CLAUSE_LIST.to_owned())
    );
    Ok(())
}

/// Empties ADR 004's evidence block while leaving both delimiters in place.
///
/// Replacing the quoted text would leave the italic markers and so leave an
/// (empty-bodied) clause behind; the empty-list failure needs a block that
/// genuinely holds none.
fn empty_evidence_block(adr: &str) -> String {
    let begin = Register::Evidence.begin();
    let end = Register::Evidence.end();
    let Some((head, rest)) = adr.split_once(begin) else {
        return adr.to_owned();
    };
    let Some((_, tail)) = rest.split_once(end) else {
        return adr.to_owned();
    };
    format!("{head}{begin}\n\n{end}{tail}")
}

/// Resolves each gate's title fragment to exactly one live roadmap task, and
/// rejects a fragment matching none and one matching many.
#[rstest]
#[case::records_process_buffer("S1", "Annotate `mdtablefix` `ProcessBuffer`", None)]
#[case::records_continuation("S2", "Annotate `mdtablefix` continuation", None)]
#[case::records_baseline("S3", "Apply the conventions-only baseline", None)]
#[case::decides("S4", "Finalize the `StateName` return shape", None)]
#[case::unresolved(
    "S4",
    "Do the thing",
    Some(
        "docs/roadmap.md: gate S4 names task fragment \"Do the thing\", which matches no task. \
         Repair: restore that task's title, or revise ADR 004's gate table."
    )
)]
#[case::ambiguous(
    "S3",
    "baseline",
    Some(
        "docs/roadmap.md: gate S3 names task fragment \"baseline\", which matches 15 tasks. \
         Repair: use a fragment specific to one task."
    )
)]
fn gate_titles_resolve(
    #[case] gate: &str,
    #[case] fragment: &str,
    #[case] failure: Option<&str>,
) -> Result<(), String> {
    let mutated = gate_table().replace(&gate_table_fragment(gate)?, fragment);
    let row = gate_rows(&mutated)
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|row| row.gate == gate)
        .expect("the fixture gate table must name every gate");
    assert_eq!(row.fragment, fragment);
    match failure {
        None => assert_eq!(check_gate_titles(&mutated, ROADMAP), Ok(())),
        Some(expected) => assert_eq!(
            check_gate_titles(&mutated, ROADMAP),
            Err(expected.to_owned())
        ),
    }
    Ok(())
}

/// The fragment the fixture gate table ships for one gate, so that a case can
/// replace it without the table being written out twice.
///
/// Fallible rather than `expect`ing, for the reason given on `blocked_by`. An
/// absent gate is not a fixture defect either: `gate_titles_resolve` asserts the
/// fragment it receives after calling this, so returning an empty string for an
/// unnamed gate keeps the case's own assertion as the place the failure lands.
fn gate_table_fragment(gate: &str) -> Result<String, String> {
    Ok(gate_rows(&gate_table())
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|row| row.gate == gate)
        .map_or_else(String::new, |row| row.fragment))
}

/// Checks the acceptance criterion the task is graded on still resolves.
#[test]
fn success_criterion_still_maps() -> Result<(), String> {
    check_success_criterion(&live_status()?, ROADMAP)?;
    let missing = status_register_without("tracing-use");
    let rows = status_rows(&missing).expect("the mutated register still parses");
    assert_eq!(
        check_success_criterion(&rows, ROADMAP),
        Err(
            "docs/adr-004-state-name-consumption-evidence.md: no status-register field maps the \
             criterion noun \"tracing use\". Repair: add the tracing-use field, or reword the \
             roadmap task."
                .to_owned()
        )
    );
    let reworded = mutated(ROADMAP, "and tracing use", "and tracing coverage");
    assert_eq!(
        check_success_criterion(&live_status()?, &reworded),
        Err(
            "docs/roadmap.md no longer contains the 1.1.3 success criterion. Repair: restore \
             \"the Phase 2 validation note template has fields for state display name, optional \
             identifier need, metrics cardinality, and tracing use\", or revise this contract \
             together with the task."
                .to_owned()
        )
    );
    Ok(())
}

/// Checks that `docs/design.md` still defers the default to evidence.
#[test]
fn deferred_clause_still_resolves() -> Result<(), String> {
    check_deferred_clause(DESIGN)?;
    let reworded = mutated(DESIGN, "stronger", "a stronger type");
    assert_eq!(
        check_deferred_clause(&reworded),
        Err(format!(
            "docs/design.md §6.1 no longer contains {STRONGER:?}. Repair: restore the default's \
             deferral, or revise ADR 004 and its contract together."
        ))
    );
    Ok(())
}
