//! Command coverage for optional dependencies named through `dep:` features.

use googletest::prelude::*;
use pretty_assertions::assert_eq;
use rstest::rstest;

use super::{assert_verdict, fixture, hack_rows, has_arg, has_feature, output_text};

/// A suppressed implicit feature must not be invented as a matrix row.
#[rstest]
#[ignore = "run by `make test-feature-matrix-command`; this test starts nested Cargo"]
fn explicitly_named_dependency_uses_only_its_declared_feature() -> Result<(), String> {
    let manifest = concat!(
        "[features]\nrenamed = [\"dep:implicit_optional\"]\n",
        "[dependencies]\nimplicit_optional = { package = \"matrix-optional\", ",
        "path = \"optional\", optional = true }\n",
    );
    let source = concat!(
        "//! Fixture library.\n",
        "#[cfg(feature = \"renamed\")] use implicit_optional as _;\n",
    );
    let fixture = fixture("explicit-dependency", manifest, source)?;
    let before = [
        fixture.read("Cargo.toml")?,
        fixture.read("optional/Cargo.toml")?,
    ];
    let output = fixture.run(None)?;
    let after = [
        fixture.read("Cargo.toml")?,
        fixture.read("optional/Cargo.toml")?,
    ];
    drop(fixture);
    assert_verdict(&output, true, "explicit dependency fixture should pass");
    assert_eq!(after, before);
    let rows = hack_rows(&output);
    assert_that!(
        rows.iter()
            .any(|row| has_arg(row, "--no-default-features") && has_feature(row, "renamed")),
        eq(true),
        "declared dependency feature was not checked: {rows:?}\n{}",
        output_text(&output)
    );
    assert_that!(
        rows.iter().any(|row| has_feature(row, "implicit_optional")),
        eq(false)
    );
    Ok(())
}

/// GitHub Actions output still exposes the actual isolated Cargo row.
#[rstest]
#[ignore = "run by `make test-feature-matrix-command`; this test starts nested Cargo"]
fn github_actions_group_output_identifies_feature_rows() -> Result<(), String> {
    let fixture = fixture(
        "github-actions-output",
        "[features]\nrenamed = []\n",
        "//! Fixture library.\n",
    )?;
    let output = fixture.run_with_ci_output(None, true)?;
    drop(fixture);
    assert_verdict(&output, true, "GitHub Actions fixture should pass");
    let rows = hack_rows(&output);
    assert_that!(
        rows.iter()
            .any(|row| has_arg(row, "--no-default-features") && has_feature(row, "renamed")),
        eq(true),
        "missing GitHub Actions feature row: {}",
        output_text(&output)
    );
    Ok(())
}
