//! Controls on what an evidence cell *says*: which word is a citation, and
//! whether the text beside it names a consumer or a required property.
//!
//! These are the controls on `claims.rs`'s predicates. They live apart from
//! `note_scenarios.rs` for the reason the suite splits anywhere — that module
//! had reached the 400-line cap — and the split is the natural one: every
//! scenario here reads *one cell's words*, while `note_scenarios.rs` asks what
//! the whole note obliges.

use pretty_assertions::assert_eq;
use rstest::rstest;

use super::{
    claims::{is_citation_shaped, names_a_consumer},
    fixtures::{
        IDENTIFIER_NEED_ROW,
        METRICS_CARDINALITY_ROW,
        STATE_DISPLAY_NAME_ROW,
        TRACING_USE_ROW,
        note_with_rows,
    },
    live_status,
    parse::note_rows,
    policy::{check_note_cells, resolve_note},
    types::Resolution,
};

/// Accepts a citation wearing the punctuation that closed its clause.
///
/// A citation ends the clause that cites it, so it arrives with whatever mark
/// ended that clause. ADR 004's own worked example writes
/// "`mdtablefix@abc1234:src/process.rs`, `LineMode`", which is the comma case
/// below: an engineer copying the shape the document teaches writes one, and a
/// citation that ends its cell wears a full stop or a semicolon instead. All
/// three name the same revision, so a check that refused them would refuse the
/// shape the document teaches.
#[rstest]
#[case::comma("`mdtablefix@abc1234:src/process.rs`, LineMode")]
#[case::period("`mdtablefix@abc1234:src/process.rs`. Observed in ProcessBuffer.")]
#[case::semicolon("`mdtablefix@abc1234:src/process.rs`; three names")]
#[case::upper_case("`mdtablefix@ABC1234:src/process.rs`. Observed in ProcessBuffer.")]
#[case::full_sha(
    "`mdtablefix@abc1234def5678901234567890abcdef5678901:src/process.rs`. Observed in \
     ProcessBuffer."
)]
fn punctuated_citations_are_accepted(#[case] evidence: &str) -> Result<(), String> {
    let rows = live_status()?;
    let note = note_with_rows(&[
        &format!("| state-display-name | Enumerated | {evidence} |"),
        IDENTIFIER_NEED_ROW,
        METRICS_CARDINALITY_ROW,
        TRACING_USE_ROW,
    ]);
    let cells = note_rows(&note).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &cells)?;
    assert_eq!(resolve_note(&rows, &cells)?, Resolution::Sufficient);
    Ok(())
}

/// Rejects a citation whose revision names a ref, and keeps its path out of the
/// keyword scans.
///
/// The coupling the two citation rules must not have. A cell citing
/// `mdtablefix@main:src/tracing.rs` must be rejected, because `main` is a ref:
/// it moves, and once it has, the cell no longer pins the tree its status was
/// observed in. The obvious way to implement that is to narrow the predicate
/// that also decides what `narrative_text` strips — and everything left in the
/// scanned text is read as a claim, so the rejected citation's *path* would
/// supply the consumer token. `names_a_consumer` would then report a consumer
/// the note never named, and the cell would be admissible on evidence it does
/// not carry the moment the two checks were ordered the other way.
///
/// The `is_citation_shaped` assertions are what keep the rejection from being
/// vacuous: the `main` revision is refused, and the same citation with a commit
/// revision is accepted, so a predicate that simply refused every citation would
/// fail here. The `names_a_consumer` assertion is the leak control proper: it
/// reads the predicate directly rather than through `check_note_cells`, so it
/// cannot depend on which obligation happens to be checked first.
#[test]
fn a_ref_citing_cell_is_rejected_without_leaking_its_path() -> Result<(), String> {
    let ref_citing = "`mdtablefix@main:src/tracing.rs`";
    let commit_citing = "`mdtablefix@abc1234:src/tracing.rs`";
    assert_eq!(
        is_citation_shaped(ref_citing),
        false,
        "{ref_citing} cites a ref, which moves, so it must not be accepted as a citation"
    );
    assert_eq!(
        is_citation_shaped(commit_citing),
        true,
        "{commit_citing} cites a commit and must be accepted, or the check above passes because \
         every citation is refused"
    );
    assert_eq!(
        names_a_consumer(ref_citing),
        false,
        "{ref_citing} names no consumer outside its citation, but the path inside the citation \
         supplied one — so the citation was left in the text the keyword scan reads. The path of \
         a cell whose revision is refused must leave the scan with the rest of the citation."
    );
    assert_eq!(
        names_a_consumer("the tracing subscriber was considered"),
        true,
        "the consumer scan reads no prose at all, so the assertion above would pass whether or \
         not the citation was stripped — it is the *contrast* that shows the path was hidden \
         rather than the scan being dead"
    );
    let rows = live_status()?;
    let cells = note_rows(&note_with_rows(&[
        STATE_DISPLAY_NAME_ROW,
        &format!("| identifier-need | None | {ref_citing} |"),
        METRICS_CARDINALITY_ROW,
        TRACING_USE_ROW,
    ]))
    .map_err(|error| error.to_string())?;
    assert_eq!(
        check_note_cells(&rows, &cells),
        Err(
            "a StateName note records evidence for field \"identifier-need\" that is not a \
             citation of the shape <repo>@<sha>:<path>. Repair: cite the revision the observation \
             was made against."
                .to_owned()
        ),
        "the note must be rejected for the revision it cites, not accepted on the consumer its \
         citation's path happens to name"
    );
    Ok(())
}

/// Accepts a cell that denies a consumer exists, in either of the two phrasings
/// the obligation admits.
///
/// ADR 004 words the second obligation as naming a consumer "or stat[ing] that
/// none of them exist", and the template repeats that sentence verbatim, so a
/// note that follows the document has to write those words. The consumer
/// predicate scans a token list, and the list once carried only the shorter
/// "none exist" — which is not a substring of the longer phrase, so the wording
/// the document prescribed was the one the check refused.
///
/// Each case carries the negative phrase *and no consumer token*, because the
/// predicate is a list scan and either kind of match satisfies it. A case that
/// named a consumer as well would pass whether or not its own phrase was in the
/// list — the first draft of this control did exactly that, with "the tracing
/// subscriber ... none of them exist", and passed with the phrase's token
/// removed. The absence of every positive token is what makes each case bear on
/// the wording it is named for, so the two can be checked independently rather
/// than one covering for the other.
#[rstest]
#[case::adr_wording(
    "the consumers of ADR 004's search set were considered; none of them exist; \
     `mdtablefix@abc1234:src/process.rs`"
)]
#[case::short_wording("no consumer exists in the workspace; `mdtablefix@abc1234:src/process.rs`")]
fn a_stated_absence_of_consumers_is_usable(#[case] evidence: &str) -> Result<(), String> {
    let rows = live_status()?;
    let note = note_with_rows(&[
        STATE_DISPLAY_NAME_ROW,
        &format!("| identifier-need | None | {evidence} |"),
        METRICS_CARDINALITY_ROW,
        TRACING_USE_ROW,
    ]);
    let cells = note_rows(&note).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &cells)?;
    assert_eq!(resolve_note(&rows, &cells)?, Resolution::Sufficient);
    Ok(())
}

/// Accepts a `None` cell that names a property in order to *deny* it.
///
/// The agreement rule reads a status against the properties its evidence
/// names, and a cell recording `None` says so by naming the property nobody
/// asked for. A substring test cannot tell naming-to-assert from
/// naming-to-deny, so it rejects every negative phrased as anything other than
/// silence — leaving an engineer no way to record one except by not mentioning
/// the property at all. The rule would then be demanding a euphemism rather
/// than agreement, and punishing the more informative note.
#[rstest]
#[case::negated_stability(
    "the subscriber and the metrics recorder were both considered; no stability requirement was \
     observed; `mdtablefix@abc1234:src/process.rs`"
)]
#[case::negated_ordering(
    "subscriber, metrics and documentation considered; none of them need ordering; \
     `mdtablefix@abc1234:src/process.rs`"
)]
#[case::negated_equality(
    "no consumer needs equality of identifiers; `mdtablefix@abc1234:src/process.rs`"
)]
fn negated_property_claims_do_not_disagree_with_none(#[case] evidence: &str) -> Result<(), String> {
    let rows = live_status()?;
    let note = note_with_rows(&[
        STATE_DISPLAY_NAME_ROW,
        &format!("| identifier-need | None | {evidence} |"),
        METRICS_CARDINALITY_ROW,
        TRACING_USE_ROW,
    ]);
    let cells = note_rows(&note).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &cells)?;
    assert_eq!(resolve_note(&rows, &cells)?, Resolution::Sufficient);
    Ok(())
}

/// Accepts a `Property required` cell whose evidence records the property as
/// *unmet*, which is the observation ADR 004 exists to collect.
///
/// The counterpart of `negated_property_claims_do_not_disagree_with_none`, and
/// the reason the two cannot share one rule. Both cells negate a property word,
/// and they fall on opposite sides of the register: "no stability requirement"
/// denies that anything was asked for and must read as `None`; "labels that are
/// not stable across releases" records that stability *was* required and is not
/// supplied, and must read as `Property required`. A rule counting any nearby
/// "not" as a denial rejects the second — the central rename-stability scenario
/// — while accepting the first, so the suite would refuse the note the design's
/// whole deferral is waiting on.
///
/// The two phrasings are told apart by what the negation scopes: a determiner
/// ("no", "none") takes the requirement away, and the adverb "not" denies the
/// requirement only where it governs a requirement verb. Each case below is an
/// assertion, so a rule collapsing them fails on the status it read.
#[rstest]
#[case::unmet_stability(
    "the subscriber requires labels that are not stable across releases; \
     `mdtablefix@abc1234:src/process.rs`"
)]
#[case::unmet_ordering(
    "the metrics recorder needs an ordering that is not stable across releases; \
     `mdtablefix@abc1234:src/process.rs`"
)]
#[case::unmet_contracted(
    "the subscriber requires labels that aren't stable across releases; \
     `mdtablefix@abc1234:src/process.rs`"
)]
fn a_required_property_recorded_as_unmet_is_not_a_denial(
    #[case] evidence: &str,
) -> Result<(), String> {
    let rows = live_status()?;
    let note = note_with_rows(&[
        STATE_DISPLAY_NAME_ROW,
        &format!("| identifier-need | Property required | {evidence} |"),
        METRICS_CARDINALITY_ROW,
        TRACING_USE_ROW,
    ]);
    let cells = note_rows(&note).map_err(|error| error.to_string())?;
    check_note_cells(&rows, &cells).map_err(|error| {
        format!(
            "the evidence {evidence:?} records a required property the current value fails to \
             supply, which is the observation the register's decisive status is for, but it was \
             read as a denial of the requirement: {error}"
        )
    })?;
    assert_eq!(
        resolve_note(&rows, &cells)?,
        Resolution::Insufficient,
        "a note recording a required property must reach the register's decisive status; the \
         evidence {evidence:?} did not"
    );
    Ok(())
}
