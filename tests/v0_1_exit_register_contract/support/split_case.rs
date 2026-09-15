//! R1 companion-document policy for the v0.1 exit-register contract.

use super::fold_whitespace;

/// Verifies the R1 split-case wording; e.g. either weak validation selects E1.
pub(super) fn check_split_case_amendments(design: &str, terms: &str) -> Result<(), String> {
    let split_case_section = design
        .split_once("### 13.7 Conventions baseline is also too weak")
        .map_or_else(String::new, |(_, after_heading)| {
            after_heading
                .lines()
                .take_while(|line| {
                    !line.trim_start_matches(' ').starts_with('#') || line.starts_with("    ")
                })
                .collect::<Vec<_>>()
                .join("\n")
        });
    if !fold_whitespace(&split_case_section).contains("in either validation example") {
        return Err(
            "docs/design.md §13.7 does not record the R1 split-case rule. Repair: amend section \
             13.7 to say either validation example."
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
    Ok(())
}
