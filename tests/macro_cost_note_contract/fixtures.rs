//! In-memory forms and candidate-bound roadmap claims for contract tests.

use super::{MarkdownFile, REQUIRED_FIELDS, check, contract::Kind};

pub(crate) const MACRO_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(crate) const DESIGN_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[derive(Clone, Copy)]
pub(super) struct FixtureField(&'static str);

impl FixtureField {
    pub(super) const COMPARISON_KIND: Self = Self("comparison-kind");
    pub(super) const CANDIDATE_ID: Self = Self("candidate-id");
    pub(super) const ENVIRONMENT: Self = Self("environment");
    pub(super) const CLEAN_DEBUG_BASELINE_SECONDS: Self = Self("clean-debug-baseline-seconds");
    pub(super) const STRESS_FIXTURE_RESULTS: Self = Self("stress-fixture-results");
    pub(super) const BUDGET_VERDICT: Self = Self("budget-verdict");

    const fn label(self) -> &'static str { self.0 }
}

#[derive(Clone, Copy)]
enum FixtureCell {
    Value,
    Evidence,
}

#[derive(Clone, Copy, Debug)]
struct SchemaMutation {
    needle: &'static str,
    replacement: &'static str,
    expected_error: &'static str,
}

pub(crate) fn template() -> String {
    let mut text = String::from(
        "<!-- macro-cost-note -->\n\n<!-- macro-cost-register:begin -->\n| Field | Value | \
         Evidence |\n| --- | --- | --- |\n",
    );
    for field in REQUIRED_FIELDS {
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

pub(crate) fn note(kind: Kind, count: u32) -> MarkdownFile {
    let path = match kind {
        Kind::Mdtablefix => "docs/validation-notes/4.2.1-mdtablefix-macro-cost.md",
        Kind::Synthetic20 => "docs/validation-notes/4.2.2-synthetic-macro-cost.md",
    };
    let mut text = String::from(
        "<!-- macro-cost-note -->\n\n<!-- macro-cost-register:begin -->\n| Field | Value | \
         Evidence |\n| --- | --- | --- |\n",
    );
    for field in REQUIRED_FIELDS {
        let value = match field {
            "comparison-kind" => kind.label().to_owned(),
            "candidate-id" => format!("macro={MACRO_REVISION}; design={DESIGN_REVISION}"),
            "annotation-count" => count.to_string(),
            "stress-fixture-results" if kind == Kind::Mdtablefix => {
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
    MarkdownFile {
        path: path.into(),
        text,
    }
}

pub(crate) fn replace_value(
    note: &mut MarkdownFile,
    field: FixtureField,
    value: &str,
) -> Result<(), String> {
    replace_cell(note, field, FixtureCell::Value, value)
}

pub(crate) fn replace_evidence(
    note: &mut MarkdownFile,
    field: FixtureField,
    evidence: &str,
) -> Result<(), String> {
    replace_cell(note, field, FixtureCell::Evidence, evidence)
}

fn replace_cell(
    note: &mut MarkdownFile,
    field: FixtureField,
    target: FixtureCell,
    replacement: &str,
) -> Result<(), String> {
    let field_name = field.label();
    let prefix = format!("| {field_name} |");
    let line = note
        .text
        .lines()
        .find(|line| line.starts_with(&prefix))
        .ok_or_else(|| format!("fixture is missing field `{field_name}`"))?;
    let cells = line.split('|').map(str::trim).collect::<Vec<_>>();
    let field_value = cells
        .get(1)
        .ok_or_else(|| format!("fixture field `{field_name}` has no name cell"))?
        .to_owned();
    let old_value = cells
        .get(2)
        .ok_or_else(|| format!("fixture field `{field_name}` has no value cell"))?
        .to_owned();
    let old_evidence = cells
        .get(3)
        .ok_or_else(|| format!("fixture field `{field_name}` has no evidence cell"))?
        .to_owned();
    let (value, evidence) = match target {
        FixtureCell::Value => (replacement, old_evidence),
        FixtureCell::Evidence => (old_value, replacement),
    };
    let updated_line = format!("| {field_value} | {value} | {evidence} |");
    note.text = note.text.replace(line, &updated_line);
    Ok(())
}

fn candidate_comment() -> String {
    format!(
        "  <!-- macro-cost-candidate: {MACRO_REVISION}; design-revision: {DESIGN_REVISION} -->\n"
    )
}

#[path = "fixtures/tests.rs"]
mod tests;
