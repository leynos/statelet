//! Exercises the macro-cost validation contract through realistic fixtures.

#[path = "tests/placeholder_tests.rs"]
mod placeholder_tests;

use rstest::rstest;

use super::{
    super::contract::check_contract,
    DESIGN_REVISION,
    FixtureField,
    Kind,
    MACRO_REVISION,
    MarkdownFile,
    SchemaMutation,
    check,
    design,
    note,
    open_roadmap,
    replace_evidence,
    replace_value,
    roadmap_with_claims,
    template,
};

#[test]
fn open_tasks_accept_empty_notes_and_both_complete_comparison_forms() {
    let form = template();
    check(&form, &open_roadmap(), &[]).expect("open roadmap accepts an empty note set");
    check(&form, &open_roadmap(), &[note(Kind::Mdtablefix, 3)])
        .expect("real evidence can be recorded independently");
    check(&form, &open_roadmap(), &[note(Kind::Synthetic20, 20)])
        .expect("synthetic evidence can be recorded independently");
    let notes = [note(Kind::Mdtablefix, 3), note(Kind::Synthetic20, 20)];
    check(&form, &open_roadmap(), &notes).expect("complete real and synthetic fixtures are valid");

    let unrelated = MarkdownFile {
        path: "docs/validation-notes/state-name.md".into(),
        text: "<!-- state-name-note -->\nnot a macro cost register".into(),
    };
    check(&form, &open_roadmap(), &[unrelated]).expect("unrelated StateName notes are ignored");
}

#[rstest]
#[case("Unavailable:")]
#[case("Unavailable: <reason>")]
#[case("unavailable:")]
#[case("unavailable: TODO later")]
fn marked_note_rejects_unfinished_unavailable_measurements(#[case] value: &str) {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::CLEAN_DEBUG_BASELINE_SECONDS,
        value,
    )
    .expect("fixture field exists");

    let error = check(&template(), &open_roadmap(), &[evidence])
        .expect_err("unavailable measurements need a concrete reason");
    assert!(
        error.contains("clean-debug-baseline-seconds")
            && error.contains("is empty or still a placeholder"),
        "{error}"
    );
}

#[rstest]
#[case("Unavailable: compiler failed before producing a sample")]
#[case("unavailable: compiler failed before producing a sample")]
fn marked_note_accepts_a_cited_unavailable_measurement_with_a_reason(#[case] value: &str) {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::CLEAN_DEBUG_BASELINE_SECONDS,
        value,
    )
    .expect("fixture field exists");

    check(&template(), &open_roadmap(), &[evidence])
        .expect("a cited failure reason and inconclusive verdict are complete evidence");
}

#[test]
fn marked_note_rejects_descriptive_values_copied_from_the_template() {
    let template = template().replace(
        "| environment | <value> |",
        "| environment | CPU/RAM; OS; target; compiler/tools |",
    );
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::ENVIRONMENT,
        "CPU/RAM; OS; target; compiler/tools",
    )
    .expect("fixture field exists");

    let error = check(&template, &open_roadmap(), &[evidence])
        .expect_err("unchanged template prose must be rejected");
    assert!(
        error.contains("environment") && error.contains("matches the template value"),
        "{error}"
    );
}

#[test]
fn synthetic_note_rejects_the_real_comparison_stress_exception() {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::STRESS_FIXTURE_RESULTS,
        "Not applicable: real comparison",
    )
    .expect("fixture field exists");

    let error = check(&template(), &open_roadmap(), &[evidence])
        .expect_err("synthetic evidence must report its stress results");
    assert!(
        error.contains("synthetic-20 must record its 20-transition results"),
        "{error}"
    );
}

#[test]
fn live_design_and_roadmap_bindings_are_required() {
    let form = template();
    let template = MarkdownFile {
        path: "macro-cost template fixture".into(),
        text: form,
    };
    let design = MarkdownFile {
        path: "design fixture".into(),
        text: design().replace("evidence-completeness contract", "evidence contract"),
    };
    let roadmap = MarkdownFile {
        path: "roadmap fixture".into(),
        text: open_roadmap(),
    };
    assert!(
        check_contract(&template, &design, &roadmap, &[])
            .expect_err("invalid test input must be rejected")
            .contains("evidence-completeness contract")
    );

    let invalid_roadmap = open_roadmap().replace("synthetic macro cost note", "macro note");
    assert!(
        check(&template.text, &invalid_roadmap, &[])
            .expect_err("invalid test input must be rejected")
            .contains("synthetic macro cost note")
    );
}

#[rstest]
#[case(SchemaMutation { needle: "| annotation-count | <value> | <repo>@<sha>:<path> |\n", replacement: "", expected_error: "annotation-count" })]
#[case(SchemaMutation { needle: "<!-- macro-cost-register:end -->", replacement: "| environment | <value> | <repo>@<sha>:<path> |\n<!-- macro-cost-register:end -->", expected_error: "duplicate" })]
#[case(SchemaMutation { needle: "<!-- macro-cost-note -->", replacement: "<!-- another-note -->", expected_error: "marker" })]
#[case(SchemaMutation { needle: "<!-- macro-cost-register:end -->", replacement: "<!-- removed -->", expected_error: "marker pair" })]
fn template_rejects_schema_mutations(#[case] mutation: SchemaMutation) {
    let malformed = template().replace(mutation.needle, mutation.replacement);
    let error =
        check(&malformed, &open_roadmap(), &[]).expect_err("invalid template must be rejected");
    assert!(error.contains(mutation.expected_error), "{error}");
}

#[test]
fn synthetic_notes_require_exactly_twenty_annotations() {
    let error = check(&template(), &open_roadmap(), &[note(Kind::Synthetic20, 19)])
        .expect_err("invalid test input must be rejected");
    assert!(error.contains("annotation-count"), "{error}");
}

#[test]
fn annotation_count_must_be_a_bare_positive_integer() {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::ANNOTATION_COUNT,
        "20 at src/example.rs",
    )
    .expect("fixture field exists");

    let error = check(&template(), &open_roadmap(), &[evidence])
        .expect_err("annotation count must be parseable as an integer");
    assert!(
        error.contains("annotation-count must be a positive integer"),
        "{error}"
    );
}

#[test]
fn marked_notes_require_a_supported_kind_and_revision_bound_evidence() {
    let mut unsupported = note(Kind::Synthetic20, 20);
    replace_value(&mut unsupported, FixtureField::COMPARISON_KIND, "other")
        .expect("fixture field exists");
    assert!(
        check(&template(), &open_roadmap(), &[unsupported])
            .expect_err("invalid test input must be rejected")
            .contains("comparison-kind")
    );

    let mut uncited = note(Kind::Synthetic20, 20);
    replace_evidence(
        &mut uncited,
        FixtureField::ENVIRONMENT,
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
    let mut evidence = note(Kind::Mdtablefix, 3);
    replace_value(
        &mut evidence,
        FixtureField::STRESS_FIXTURE_RESULTS,
        "not applicable",
    )
    .expect("fixture field exists");
    assert!(
        check(&template(), &open_roadmap(), &[evidence])
            .expect_err("invalid test input must be rejected")
            .contains("real comparison must mark stress results")
    );
}

#[rstest]
#[case("Unavailable: compiler failed")]
#[case("unavailable: compiler failed")]
#[case("UNAVAILABLE: compiler failed")]
fn unavailable_measurements_require_an_inconclusive_verdict(#[case] value: &str) {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::CLEAN_DEBUG_BASELINE_SECONDS,
        value,
    )
    .expect("fixture field exists");
    replace_value(
        &mut evidence,
        FixtureField::BUDGET_VERDICT,
        "passes all budgets",
    )
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
    check(&form, &roadmap, &[note(Kind::Synthetic20, 20)])
        .expect("completed synthetic claim selects its matching note");
}

#[test]
fn inline_candidate_example_does_not_satisfy_a_completed_claim() {
    let declaration = format!(
        "  <!-- macro-cost-candidate: {MACRO_REVISION}; design-revision: {DESIGN_REVISION} -->\n"
    );
    let roadmap = roadmap_with_claims(true, false).replace(&declaration, "");
    let error = check(&template(), &roadmap, &[note(Kind::Synthetic20, 20)])
        .expect_err("an inline instruction is not a candidate declaration");
    assert!(
        error.contains("needs one macro-cost-candidate comment"),
        "{error}"
    );
}

#[test]
fn completed_claim_rejects_duplicate_candidate_declarations() {
    let declaration = format!(
        "  <!-- macro-cost-candidate: {MACRO_REVISION}; design-revision: {DESIGN_REVISION} -->\n"
    );
    let roadmap = roadmap_with_claims(true, false).replace(
        "  - Success: [synthetic]",
        &format!("{declaration}  - Success: [synthetic]"),
    );
    let error = check(&template(), &roadmap, &[note(Kind::Synthetic20, 20)])
        .expect_err("a completed claim has exactly one candidate declaration");
    assert!(
        error.contains("needs one macro-cost-candidate comment"),
        "{error}"
    );
}

#[test]
fn publication_claim_requires_both_selected_note_kinds() {
    let form = template();
    let roadmap = roadmap_with_claims(false, true);
    assert!(
        check(&form, &roadmap, &[note(Kind::Synthetic20, 20)])
            .expect_err("invalid test input must be rejected")
            .contains("links missing note")
    );
    let notes = [note(Kind::Mdtablefix, 3), note(Kind::Synthetic20, 20)];
    check(&form, &roadmap, &notes).expect("publication claim selects both matching notes");
}

#[derive(Clone, Copy, Debug)]
enum CandidateMismatch {
    Macro,
    Design,
}

impl CandidateMismatch {
    fn replacement(self) -> String {
        match self {
            Self::Macro => format!("macro={}; design={DESIGN_REVISION}", "d".repeat(40)),
            Self::Design => format!("macro={MACRO_REVISION}; design={}", "d".repeat(40)),
        }
    }

    const fn expected_error(self) -> &'static str {
        match self {
            Self::Macro => "different macro commit",
            Self::Design => "different design revision",
        }
    }
}

#[rstest]
#[case(CandidateMismatch::Macro)]
#[case(CandidateMismatch::Design)]
fn completed_claim_rejects_a_different_candidate_revision(#[case] mismatch: CandidateMismatch) {
    let mut evidence = note(Kind::Synthetic20, 20);
    replace_value(
        &mut evidence,
        FixtureField::CANDIDATE_ID,
        &mismatch.replacement(),
    )
    .expect("fixture field exists");
    let error = check(&template(), &roadmap_with_claims(true, false), &[evidence])
        .expect_err("invalid test input must be rejected");
    assert!(error.contains(mismatch.expected_error()), "{error}");
}

#[test]
fn unmarked_notes_do_not_supply_claim_evidence() {
    let mut evidence = note(Kind::Synthetic20, 20);
    evidence.text = evidence
        .text
        .replace("<!-- macro-cost-note -->", "<!-- state-name-note -->");
    assert!(
        check(&template(), &roadmap_with_claims(true, false), &[evidence])
            .expect_err("invalid test input must be rejected")
            .contains("must declare")
    );
}
