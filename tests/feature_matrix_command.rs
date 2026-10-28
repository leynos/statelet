//! Command-boundary tests for the current-crate feature matrix.
//!
//! These ignored cases run the real Make target against isolated packages.
//! The dedicated target runs them serially after ordinary tests and coverage.

#[path = "feature_matrix_command/harness.rs"]
mod harness;

use std::process::{Command, Output};

use camino::Utf8Path;
use cap_std::{ambient_authority, fs::Dir};
use googletest::prelude::*;
use harness::Fixture;
use pretty_assertions::assert_eq;
use rstest::rstest;

const PACKAGE: &str = "[package]\nname = \"statelet\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
const OPTIONAL_MANIFEST: &str =
    "[package]\nname = \"matrix-optional\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";
const EMPTY_LIB: &str = "//! Fixture library.\n";
const OPTIONAL_LIB: &str = "//! Optional fixture dependency.\n";
const DISTINCT_FEATURES: &str = concat!(
    "[features]\n",
    "default = [\"baseline\"]\n",
    "baseline = []\ntracing = []\nserde = []\n",
);

fn fixture(label: &str, extra: &str, source: &str) -> Result<Fixture, String> {
    let manifest = format!("{PACKAGE}\n{extra}");
    Fixture::new(
        label,
        &[
            ("Cargo.toml", &manifest),
            ("src/lib.rs", source),
            ("optional/Cargo.toml", OPTIONAL_MANIFEST),
            ("optional/src/lib.rs", OPTIONAL_LIB),
        ],
    )
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_verdict(output: &Output, success: bool, message: &str) {
    let text = output_text(output);
    assert_that!(output.status.success(), eq(success), "{message}: {text}");
    if !success {
        assert_that!(text.as_str(), contains_substring(message));
        assert_that!(text.contains("feature matrix passed"), eq(false));
    }
}

/// Return the individual Cargo commands reported by cargo-hack.
fn hack_rows(output: &Output) -> Vec<Vec<String>> {
    output_text(output)
        .lines()
        .filter_map(|line| line.split_once("info: running `"))
        .filter_map(|(_, rest)| rest.split_once('`'))
        .map(|(command, _)| command.split_whitespace().map(str::to_owned).collect())
        .collect()
}

fn has_arg(command: &[String], arg: &str) -> bool { command.iter().any(|word| word == arg) }

fn has_feature(command: &[String], feature: &str) -> bool {
    command
        .windows(2)
        .any(|pair| matches!(pair, [flag, value] if flag == "--features" && value == feature))
}

/// Inspect a fixture's parent after the fixture has been dropped.
fn root_exists(root: &Utf8Path) -> Result<bool, String> {
    let parent = root
        .parent()
        .ok_or_else(|| format!("{root} has no parent"))?;
    let name = root
        .file_name()
        .ok_or_else(|| format!("{root} has no name"))?;
    let directory = Dir::open_ambient_dir(parent, ambient_authority())
        .map_err(|error| format!("cannot inspect fixture parent {parent}: {error}"))?;
    Ok(directory.exists(name))
}

/// Fixture-owned directories are removed on normal return.
#[rstest]
fn fixture_cleanup_removes_the_owned_root() -> Result<(), String> {
    let fixture = fixture("cleanup", "", EMPTY_LIB)?;
    let root = fixture.root().to_owned();
    drop(fixture);
    assert_that!(root_exists(&root)?, eq(false));
    Ok(())
}

/// Assertion failures happen after the fixture has already been removed.
#[rstest]
#[should_panic(expected = "post-cleanup assertion")]
fn fixture_cleanup_precedes_a_failing_assertion() {
    let fixture = fixture("cleanup-failure", "", EMPTY_LIB).expect("fixture setup should succeed");
    let root = fixture.root().to_owned();
    drop(fixture);
    assert_that!(
        root_exists(&root).expect("fixture parent should be readable"),
        eq(false)
    );
    assert_that!(true, eq(false), "post-cleanup assertion");
}

/// Required rows are witnessed by real Cargo and cargo-hack invocations.
#[rstest]
#[case::no_features("none", "", &[])]
#[case::tracing("tracing", "[features]\ntracing = []\n", &["tracing"])]
#[case::serde("serde", "[features]\nserde = []\n", &["serde"])]
#[case::both("both", "[features]\ntracing = []\nserde = []\n", &["tracing", "serde"])]
#[case::nonempty_default("default", DISTINCT_FEATURES, &["baseline", "tracing", "serde"])]
#[case::optional_dependency(
    "optional",
    "[dependencies]\nimplicit_optional = { package = \"matrix-optional\", path = \"optional\", optional = true }\n",
    &["implicit_optional"]
)]
#[ignore = "run by `make test-feature-matrix-command`; these tests start nested Cargo"]
fn required_rows_run(
    #[case] label: &str,
    #[case] extra: &str,
    #[case] features: &[&str],
) -> Result<(), String> {
    let source = if label == "optional" {
        "//! Fixture library.\n#[cfg(feature = \"implicit_optional\")] use implicit_optional as \
         _;\n"
    } else {
        EMPTY_LIB
    };
    let fixture = fixture(label, extra, source)?;
    let before = [
        fixture.read("Cargo.toml")?,
        fixture.read("optional/Cargo.toml")?,
    ];
    let output = fixture.run(None)?;
    let after = [
        fixture.read("Cargo.toml")?,
        fixture.read("optional/Cargo.toml")?,
    ];
    let invocations = fixture.invocations()?;
    drop(fixture);
    assert_verdict(&output, true, "healthy fixture should pass");
    assert_eq!(after, before);
    let matrix_commands: Vec<_> = invocations
        .iter()
        .filter(|args| {
            args.first().is_some_and(|arg| arg == "check")
                || (args.first().is_some_and(|arg| arg == "hack")
                    && args.get(1).is_some_and(|arg| arg == "check"))
        })
        .collect();
    assert_that!(matrix_commands.len(), eq(3));
    for command in matrix_commands {
        assert_that!(has_arg(command, "--lib"), eq(true));
        assert_that!(
            command
                .windows(2)
                .any(|pair| matches!(pair, [flag, value] if flag == "-p" && value == "statelet")),
            eq(true)
        );
        assert_that!(
            command.windows(2).any(|pair| matches!(pair, [flag, value]
                if flag == "--manifest-path" && value.ends_with("fixture with spaces/Cargo.toml"))),
            eq(true)
        );
    }
    let no_default = invocations.iter().filter(|args| {
        args.first().is_some_and(|arg| arg == "check") && has_arg(args, "--no-default-features")
    });
    assert_that!(no_default.count(), eq(1));
    let all = invocations.iter().filter(|args| {
        args.first().is_some_and(|arg| arg == "check") && has_arg(args, "--all-features")
    });
    assert_that!(all.count(), eq(1));
    let rows = hack_rows(&output);
    assert_that!(rows.is_empty(), eq(false), "{}", output_text(&output));
    for feature in features {
        assert_that!(
            rows.iter()
                .any(|row| has_arg(row, "--no-default-features") && has_feature(row, feature)),
            eq(true),
            "missing isolated {feature} row: {rows:?}"
        );
    }
    if label == "default" {
        assert_that!(rows.iter().any(|row| has_feature(row, "default")), eq(true));
    } else if features.is_empty() {
        assert_that!(
            rows.iter()
                .any(|row| !has_arg(row, "--no-default-features")),
            eq(true)
        );
    }
    Ok(())
}

/// A fault confined to each required configuration must reach the compiler.
#[rstest]
#[case::no_default(
    "no-default",
    "not(any(feature = \"baseline\", feature = \"tracing\", feature = \"serde\", feature = \
     \"default\"))"
)]
#[case::default(
    "default",
    "all(feature = \"default\", feature = \"baseline\", not(any(feature = \"tracing\", feature = \
     \"serde\")))"
)]
#[case::tracing(
    "tracing",
    "all(feature = \"tracing\", not(any(feature = \"baseline\", feature = \"serde\", feature = \
     \"default\")))"
)]
#[case::serde(
    "serde",
    "all(feature = \"serde\", not(any(feature = \"baseline\", feature = \"tracing\", feature = \
     \"default\")))"
)]
#[case::all(
    "all-features",
    "all(feature = \"default\", feature = \"baseline\", feature = \"tracing\", feature = \
     \"serde\")"
)]
#[ignore = "run by `make test-feature-matrix-command`; these tests start nested Cargo"]
fn each_configuration_propagates_its_own_failure(
    #[case] label: &str,
    #[case] condition: &str,
) -> Result<(), String> {
    let healthy = fixture(&format!("{label}-healthy"), DISTINCT_FEATURES, EMPTY_LIB)?;
    let healthy_manifest = [
        healthy.read("Cargo.toml")?,
        healthy.read("optional/Cargo.toml")?,
    ];
    let healthy_output = healthy.run(None)?;
    let healthy_after = [
        healthy.read("Cargo.toml")?,
        healthy.read("optional/Cargo.toml")?,
    ];
    drop(healthy);
    assert_verdict(&healthy_output, true, "healthy counterpart should pass");
    assert_eq!(healthy_after, healthy_manifest);
    let message = format!("matrix poison {label}");
    let source =
        format!("//! Poison fixture.\n#[cfg({condition})]\ncompile_error!(\"{message}\");\n");
    let poisoned = fixture(&format!("{label}-poison"), DISTINCT_FEATURES, &source)?;
    let before = [
        poisoned.read("Cargo.toml")?,
        poisoned.read("optional/Cargo.toml")?,
    ];
    let output = poisoned.run(None)?;
    let after = [
        poisoned.read("Cargo.toml")?,
        poisoned.read("optional/Cargo.toml")?,
    ];
    drop(poisoned);
    assert_verdict(&output, false, &message);
    assert_eq!(after, before);
    Ok(())
}

/// Missing and mismatched tools fail before a compilation and name the repair.
#[rstest]
#[case::missing("missing", "missing")]
#[case::mismatched("cargo-hack 0.0.0", "cargo-hack 0.0.0")]
#[ignore = "run by `make test-feature-matrix-command`; these tests start nested Cargo"]
fn tool_preflight_is_actionable(
    #[case] supplied: &str,
    #[case] observed: &str,
) -> Result<(), String> {
    let fixture = fixture("tool-preflight", "", EMPTY_LIB)?;
    let output = fixture.run(Some(supplied))?;
    let invocations = fixture.invocations()?;
    drop(fixture);
    assert_verdict(&output, false, observed);
    let text = output_text(&output);
    assert_that!(text.as_str(), contains_substring("cargo-hack 0.6.45"));
    assert_that!(text.as_str(), contains_substring("make install-cargo-hack"));
    assert_that!(
        invocations
            .iter()
            .any(|args| args.first().is_some_and(|x| x == "check")),
        eq(false)
    );
    Ok(())
}

/// Invalid Cargo input propagates failure without a success message.
#[rstest]
#[ignore = "run by `make test-feature-matrix-command`; this test starts nested Cargo"]
fn malformed_manifest_fails() -> Result<(), String> {
    let fixture = Fixture::new(
        "malformed",
        &[("Cargo.toml", "[package\n"), ("src/lib.rs", EMPTY_LIB)],
    )?;
    let before = fixture.read("Cargo.toml")?;
    let output = fixture.run(None)?;
    let after = fixture.read("Cargo.toml")?;
    drop(fixture);
    assert_verdict(&output, false, "error");
    assert_eq!(after, before);
    Ok(())
}

/// Development dependency features cannot hide a missing normal dependency feature.
#[rstest]
#[ignore = "run by `make test-feature-matrix-command`; this test starts nested Cargo"]
fn development_features_do_not_mask_missing_normal_features() -> Result<(), String> {
    let manifest = format!(
        "{PACKAGE}\n[dependencies]\nv3_dependency = {{ path = \"dependency\" \
         }}\n[dev-dependencies]\nv3_dependency = {{ path = \"dependency\", features = [\"gated\"] \
         }}\n"
    );
    let dependency = "[package]\nname = \"v3_dependency\"\nversion = \"0.0.0\"\nedition = \
                      \"2024\"\n[features]\ngated = []\n";
    let dependency_source =
        "//! Dependency fixture.\n#[cfg(feature = \"gated\")] pub fn gated() {}\n";
    for (label, source, success) in [
        ("v3-healthy", EMPTY_LIB, true),
        (
            "v3-broken",
            "//! Broken fixture.\npub fn check() { v3_dependency::gated(); }\n",
            false,
        ),
    ] {
        let fixture = Fixture::new(
            label,
            &[
                ("Cargo.toml", &manifest),
                ("src/lib.rs", source),
                ("dependency/Cargo.toml", dependency),
                ("dependency/src/lib.rs", dependency_source),
            ],
        )?;
        let before = [
            fixture.read("Cargo.toml")?,
            fixture.read("dependency/Cargo.toml")?,
        ];
        let output = fixture.run(None)?;
        let after = [
            fixture.read("Cargo.toml")?,
            fixture.read("dependency/Cargo.toml")?,
        ];
        drop(fixture);
        assert_verdict(
            &output,
            success,
            if success {
                "healthy V3 fixture"
            } else {
                "cannot find function `gated`"
            },
        );
        assert_eq!(after, before);
    }
    Ok(())
}

/// The current crate's feature matrix succeeds through the documented command.
#[rstest]
#[ignore = "run by `make test-feature-matrix-command`; this test starts nested Cargo"]
#[expect(
    clippy::disallowed_methods,
    reason = "The nested command must recover the caller's original RUSTFLAGS from its process \
              environment."
)]
fn feature_matrix_succeeds_for_the_current_crate() {
    let inherited = std::env::var_os("STATELET_FEATURE_MATRIX_CALLER_FLAGS")
        .or_else(|| std::env::var_os("RUSTFLAGS"));
    let mut command = Command::new("make");
    command
        .arg("feature-matrix")
        .current_dir(env!("CARGO_MANIFEST_DIR"));
    match inherited {
        Some(value) => command.env("RUSTFLAGS", value),
        None => command.env_remove("RUSTFLAGS"),
    };
    let output = command.output().expect("make feature-matrix should start");
    assert_verdict(&output, true, "current crate matrix should pass");
}
