//! In-memory forms and candidate-bound roadmap claims for contract tests.

use rstest::rstest;

use super::{NoteFile, check, check_contract};

const FIELDS: [&str; 18] = [
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

pub(crate) const MACRO_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(crate) const DESIGN_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(crate) fn template() -> String {
    let mut text = String::from(
        "<!-- macro-cost-note -->\n\n<!-- macro-cost-register:begin -->\n| Field | Value | \
         Evidence |\n| --- | --- | --- |\n",
    );
    for field in FIELDS {
        let value = if field == "comparison-kind" {
            "mdtablefix or synthetic-20"
        } else {
            "<value>"
        };
        text.push_str("| ");
        text.push_str(field);
        text.push_str(" | ");
        text.push_str(value);
        text.push_str(" | <repo>@<sha>:<path> |\n");
    }
    text.push_str("<!-- macro-cost-register:end -->\n\n");
    text.push_str("marginal seconds = (T1 - T0) / N\n");
    text.push_str("marginal percent = 100 * (T1 - T0) / (N * T0)\n");
    text.push_str("release growth percent = 100 * (S1 - S0) / S0\n");
    text.push_str("T1 = 109.8 gives 0.49%; T1 = 110 gives 0.5%.\n");
    text.push_str("S1 = 102000 gives 2%; S1 = 102001 exceeds 2%.\n");
    text
}

pub(crate) fn design() -> String {
    "### 11.7 Compile-time and binary-size budget\n\nThe macro-cost-benchmark-note-template.md \
     uses an evidence-completeness contract for tasks 4.2.2 and 4.3.1.\n\n### 12 Next\n"
        .to_owned()
}

pub(crate) fn open_roadmap() -> String {
    String::from(
        "- [ ] 4.2.2. Require a synthetic macro cost note and macro-cost-candidate with the \
         design revision.\n- [ ] 4.3.1. Require both real and synthetic macro cost notes, \
         macro-cost-candidate and the design revision.\n",
    )
}

pub(crate) fn roadmap_with_claims(claim_422: bool, claim_431: bool) -> String {
    let mut roadmap = String::new();
    let status_422 = if claim_422 { "x" } else { " " };
    roadmap.push_str("- [");
    roadmap.push_str(status_422);
    roadmap.push_str(
        "] 4.2.2. Require a synthetic macro cost note and macro-cost-candidate with the design \
         revision.\n",
    );
    if claim_422 {
        roadmap.push_str(&candidate_comment());
        roadmap
            .push_str("  Evidence: [synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)\n");
    }
    let status_431 = if claim_431 { "x" } else { " " };
    roadmap.push_str("- [");
    roadmap.push_str(status_431);
    roadmap.push_str(
        "] 4.3.1. Require both real and synthetic macro cost notes, macro-cost-candidate and the \
         design revision.\n",
    );
    if claim_431 {
        roadmap.push_str(&candidate_comment());
        roadmap.push_str(
            "  Evidence: [real](validation-notes/4.2.1-mdtablefix-macro-cost.md) and \
             [synthetic](validation-notes/4.2.2-synthetic-macro-cost.md)\n",
        );
    }
    roadmap
}

pub(crate) fn note(kind: &str, count: u32) -> NoteFile {
    let path = if kind == "mdtablefix" {
        "docs/validation-notes/4.2.1-mdtablefix-macro-cost.md"
    } else {
        "docs/validation-notes/4.2.2-synthetic-macro-cost.md"
    };
    let mut text = String::from(
        "<!-- macro-cost-note -->\n\n<!-- macro-cost-register:begin -->\n| Field | Value | \
         Evidence |\n| --- | --- | --- |\n",
    );
    for field in FIELDS {
        let value = match field {
            "comparison-kind" => kind.to_owned(),
            "candidate-id" => format!("macro={MACRO_REVISION}; design={DESIGN_REVISION}"),
            "annotation-count" => count.to_string(),
            "stress-fixture-results" if kind == "mdtablefix" => {
                "Not applicable: real comparison".into()
            }
            "review" => "Pending: G3 decision not yet recorded".into(),
            "budget-verdict" => "Inconclusive; evidence fixture".into(),
            _ => format!("recorded {field}"),
        };
        text.push_str("| ");
        text.push_str(field);
        text.push_str(" | ");
        text.push_str(&value);
        text.push_str(" | statelet@cccccccccccccccccccccccccccccccccccccccc:artifacts/");
        text.push_str(field);
        text.push_str(".txt |\n");
    }
    text.push_str("<!-- macro-cost-register:end -->\n");
    NoteFile {
        path: path.into(),
        text,
    }
}

pub(crate) fn replace_value(note: &mut NoteFile, field: &str, value: &str) -> Result<(), String> {
    let prefix = format!("| {field} |");
    let line = note
        .text
        .lines()
        .find(|line| line.starts_with(&prefix))
        .ok_or_else(|| format!("fixture is missing field `{field}`"))?;
    let evidence = line
        .rsplit('|')
        .nth(1)
        .ok_or_else(|| format!("fixture field `{field}` has no evidence cell"))?
        .trim();
    let replacement = format!("| {field} | {value} | {evidence} |");
    note.text = note.text.replace(line, &replacement);
    Ok(())
}

pub(crate) fn replace_evidence(
    note: &mut NoteFile,
    field: &str,
    evidence: &str,
) -> Result<(), String> {
    let prefix = format!("| {field} |");
    let line = note
        .text
        .lines()
        .find(|line| line.starts_with(&prefix))
        .ok_or_else(|| format!("fixture is missing field `{field}`"))?;
    let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
    let field_value = cells
        .get(1)
        .ok_or_else(|| format!("fixture field `{field}` has no name cell"))?;
    let value = cells
        .get(2)
        .ok_or_else(|| format!("fixture field `{field}` has no value cell"))?;
    let replacement = format!("| {field_value} | {value} | {evidence} |");
    note.text = note.text.replace(line, &replacement);
    Ok(())
}

fn candidate_comment() -> String {
    format!(
        "  <!-- macro-cost-candidate: {MACRO_REVISION}; design-revision: {DESIGN_REVISION} -->\n"
    )
}

#[test]
fn open_tasks_accept_empty_notes_and_both_complete_comparison_forms() {
    let form = template();
    check(&form, &open_roadmap(), &[]).expect("open roadmap accepts an empty note set");
    check(&form, &open_roadmap(), &[note("mdtablefix", 3)])
        .expect("real evidence can be recorded independently");
    check(&form, &open_roadmap(), &[note("synthetic-20", 20)])
        .expect("synthetic evidence can be recorded independently");
    let notes = [note("mdtablefix", 3), note("synthetic-20", 20)];
    check(&form, &open_roadmap(), &notes).expect("complete real and synthetic fixtures are valid");

    let unrelated = NoteFile {
        path: "docs/validation-notes/state-name.md".into(),
        text: "<!-- state-name-note -->\nnot a macro cost register".into(),
    };
    check(&form, &open_roadmap(), &[unrelated]).expect("unrelated StateName notes are ignored");
}

#[rstest]
#[case("tbd")]
#[case("TODO")]
#[case("pending")]
#[case("N/A")]
#[case("<fill this>")]
fn marked_note_rejects_placeholder_values(#[case] value: &str) {
    let form = template();
    let mut evidence = note("synthetic-20", 20);
    replace_value(&mut evidence, "environment", value).expect("fixture field exists");
    let error = check(&form, &open_roadmap(), &[evidence])
        .expect_err("invalid test input must be rejected");
    assert!(error.contains("environment"), "{error}");
}

#[test]
fn live_design_and_roadmap_bindings_are_required() {
    let form = template();
    let design = design().replace("evidence-completeness contract", "evidence contract");
    assert!(
        check_contract(&form, &design, &open_roadmap(), &[])
            .expect_err("invalid test input must be rejected")
            .contains("evidence-completeness contract")
    );

    let roadmap = open_roadmap().replace("synthetic macro cost note", "macro note");
    assert!(
        check(&form, &roadmap, &[])
            .expect_err("invalid test input must be rejected")
            .contains("synthetic macro cost note")
    );
}

#[rstest]
#[case(
    "| annotation-count | <value> | <repo>@<sha>:<path> |\n",
    "",
    "annotation-count"
)]
#[case(
    "<!-- macro-cost-register:end -->",
    "| environment | <value> | <repo>@<sha>:<path> |\n<!-- macro-cost-register:end -->",
    "duplicate"
)]
#[case("<!-- macro-cost-note -->", "<!-- another-note -->", "marker")]
#[case("<!-- macro-cost-register:end -->", "<!-- removed -->", "marker pair")]
fn template_rejects_schema_mutations(
    #[case] needle: &str,
    #[case] replacement: &str,
    #[case] expected: &str,
) {
    let malformed = template().replace(needle, replacement);
    let error =
        check(&malformed, &open_roadmap(), &[]).expect_err("invalid template must be rejected");
    assert!(error.contains(expected), "{error}");
}

#[test]
fn synthetic_notes_require_exactly_twenty_annotations() {
    let error = check(&template(), &open_roadmap(), &[note("synthetic-20", 19)])
        .expect_err("invalid test input must be rejected");
    assert!(error.contains("annotation-count"), "{error}");
}

#[test]
fn marked_notes_require_a_supported_kind_and_revision_bound_evidence() {
    let mut unsupported = note("synthetic-20", 20);
    replace_value(&mut unsupported, "comparison-kind", "other").expect("fixture field exists");
    assert!(
        check(&template(), &open_roadmap(), &[unsupported])
            .expect_err("invalid test input must be rejected")
            .contains("comparison-kind")
    );

    let mut uncited = note("synthetic-20", 20);
    replace_evidence(
        &mut uncited,
        "environment",
        "machine notes without a revision citation",
    )
    .expect("fixture field exists");
    assert!(
        check(&template(), &open_roadmap(), &[uncited])
            .expect_err("invalid test input must be rejected")
            .contains("revision-bound evidence")
    );
}

#[test]
fn real_comparison_requires_the_exact_stress_fixture_exception() {
    let mut evidence = note("mdtablefix", 3);
    replace_value(&mut evidence, "stress-fixture-results", "not applicable")
        .expect("fixture field exists");
    assert!(
        check(&template(), &open_roadmap(), &[evidence])
            .expect_err("invalid test input must be rejected")
            .contains("real comparison must mark stress results")
    );
}

#[test]
fn unavailable_measurements_require_an_inconclusive_verdict() {
    let mut evidence = note("synthetic-20", 20);
    replace_value(
        &mut evidence,
        "clean-debug-baseline-seconds",
        "Unavailable: compiler failed",
    )
    .expect("fixture field exists");
    replace_value(&mut evidence, "budget-verdict", "passes all budgets")
        .expect("fixture field exists");
    assert!(
        check(&template(), &open_roadmap(), &[evidence])
            .expect_err("invalid test input must be rejected")
            .contains("Inconclusive")
    );
}

#[test]
fn completed_synthetic_task_requires_a_selected_matching_note() {
    let form = template();
    let roadmap = roadmap_with_claims(true, false);
    assert!(
        check(&form, &roadmap, &[])
            .expect_err("invalid test input must be rejected")
            .contains("missing note")
    );
    check(&form, &roadmap, &[note("synthetic-20", 20)])
        .expect("completed synthetic claim selects its matching note");
}

#[test]
fn publication_claim_requires_both_selected_note_kinds() {
    let form = template();
    let roadmap = roadmap_with_claims(false, true);
    assert!(
        check(&form, &roadmap, &[note("synthetic-20", 20)])
            .expect_err("invalid test input must be rejected")
            .contains("links missing note")
    );
    let notes = [note("mdtablefix", 3), note("synthetic-20", 20)];
    check(&form, &roadmap, &notes).expect("publication claim selects both matching notes");
}

#[test]
fn completed_claim_rejects_a_different_candidate_revision() {
    let mut evidence = note("synthetic-20", 20);
    replace_value(
        &mut evidence,
        "candidate-id",
        "macro=dddddddddddddddddddddddddddddddddddddddd; \
         design=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    )
    .expect("fixture field exists");
    assert!(
        check(&template(), &roadmap_with_claims(true, false), &[evidence])
            .expect_err("invalid test input must be rejected")
            .contains("different macro commit")
    );
}

#[test]
fn completed_claim_rejects_a_different_design_revision() {
    let mut evidence = note("synthetic-20", 20);
    replace_value(
        &mut evidence,
        "candidate-id",
        "macro=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; \
         design=dddddddddddddddddddddddddddddddddddddddd",
    )
    .expect("fixture field exists");
    assert!(
        check(&template(), &roadmap_with_claims(true, false), &[evidence])
            .expect_err("invalid test input must be rejected")
            .contains("different design revision")
    );
}

#[test]
fn unmarked_notes_do_not_supply_claim_evidence() {
    let mut evidence = note("synthetic-20", 20);
    evidence.text = evidence
        .text
        .replace("<!-- macro-cost-note -->", "<!-- state-name-note -->");
    assert!(
        check(&template(), &roadmap_with_claims(true, false), &[evidence])
            .expect_err("invalid test input must be rejected")
            .contains("must declare")
    );
}
