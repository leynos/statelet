//! Protect the ordered fields and markers in the copyable template.

use super::{check, open_roadmap, template};

#[test]
fn template_requires_all_fields_once_and_both_comparison_kinds() {
    let form = template();
    let missing = form.replace("| annotation-count | <value> | <repo>@<sha>:<path> |\n", "");
    assert!(
        check(&missing, &open_roadmap(), &[])
            .expect_err("invalid test input must be rejected")
            .contains("annotation-count")
    );

    let duplicate = form.replace(
        "<!-- macro-cost-register:end -->",
        "| environment | <value> | <repo>@<sha>:<path> |\n<!-- macro-cost-register:end -->",
    );
    assert!(
        check(&duplicate, &open_roadmap(), &[])
            .expect_err("invalid test input must be rejected")
            .contains("duplicate")
    );

    let wrong_marker = form.replace("<!-- macro-cost-note -->", "<!-- another-note -->");
    assert!(
        check(&wrong_marker, &open_roadmap(), &[])
            .expect_err("invalid test input must be rejected")
            .contains("marker")
    );

    let missing_end = form.replace("<!-- macro-cost-register:end -->", "<!-- removed -->");
    assert!(
        check(&missing_end, &open_roadmap(), &[])
            .expect_err("invalid test input must be rejected")
            .contains("marker pair")
    );
}
