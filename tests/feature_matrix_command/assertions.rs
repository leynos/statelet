//! Interpret the command-boundary suite's recorded Cargo invocations.

use std::process::Output;

use googletest::prelude::*;

use super::output_text;

/// Return the individual Cargo commands reported by cargo-hack.
pub(super) fn hack_rows(output: &Output) -> Vec<Vec<String>> {
    hack_rows_from_text(&output_text(output))
}

/// Interpret cargo-hack status lines regardless of status-label colouring.
fn hack_rows_from_text(text: &str) -> Vec<Vec<String>> {
    text.lines()
        .filter_map(|line| line.split_once("running `"))
        .filter(|(prefix, _)| prefix.contains("info"))
        .filter_map(|(_, rest)| rest.split_once('`'))
        .map(|(command, _)| command.split_whitespace().map(str::to_owned).collect())
        .collect()
}

/// Check for a whole argument without matching substrings.
pub(super) fn has_arg(command: &[String], arg: &str) -> bool {
    command.iter().any(|word| word == arg)
}

/// Check for a feature selected by an explicit Cargo argument.
pub(super) fn has_feature(command: &[String], feature: &str) -> bool {
    command
        .windows(2)
        .any(|pair| matches!(pair, [flag, value] if flag == "--features" && value == feature))
}

/// Check each direct Cargo mode even when two modes select the same features.
pub(super) fn assert_direct_modes(invocations: &[Vec<String>]) {
    let checks: Vec<_> = invocations
        .iter()
        .filter(|args| args.first().is_some_and(|arg| arg == "check"))
        .collect();
    assert_that!(checks.len(), eq(3));
    assert_that!(
        checks
            .iter()
            .filter(|args| has_arg(args, "--no-default-features"))
            .count(),
        eq(1)
    );
    assert_that!(
        checks
            .iter()
            .filter(|args| has_arg(args, "--all-features"))
            .count(),
        eq(1)
    );
    assert_that!(
        checks
            .iter()
            .filter(|args| !has_arg(args, "--no-default-features")
                && !has_arg(args, "--all-features")
                && !has_arg(args, "--features"))
            .count(),
        eq(1)
    );
}

/// Assert the selected modes, package binding, and isolated feature rows.
pub(super) fn assert_invocation_contract(
    invocations: &[Vec<String>],
    output: &Output,
    label: &str,
    features: &[&str],
) {
    let commands: Vec<_> = invocations
        .iter()
        .filter(|args| is_matrix_command(args))
        .collect();
    assert_that!(commands.len(), eq(4));
    for command in commands {
        assert_command_binding(command);
    }
    assert_direct_modes(invocations);
    assert_isolated_rows(output, label, features);
}

/// Identify the direct Cargo commands used by the matrix recipe.
fn is_matrix_command(args: &[String]) -> bool {
    args.first().is_some_and(|arg| arg == "check")
        || (args.first().is_some_and(|arg| arg == "hack")
            && args.get(1).is_some_and(|arg| arg == "check"))
}

/// Every matrix command names the fixture library and its pinned manifest.
fn assert_command_binding(command: &[String]) {
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

/// Check independently selected features and the relevant default row.
fn assert_isolated_rows(output: &Output, label: &str, features: &[&str]) {
    let rows = hack_rows(output);
    assert_that!(rows.is_empty(), eq(false), "{}", output_text(output));
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
}

#[cfg(test)]
mod tests {
    //! Cover the status formats emitted by cargo-hack.

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::hack_rows_from_text;

    #[rstest]
    #[case("info: running `cargo check --features renamed`")]
    #[case("\x1b[1minfo\x1b[1m:\x1b[0m running `cargo check --features renamed`")]
    fn parses_uncoloured_and_coloured_status(#[case] line: &str) {
        assert_eq!(
            hack_rows_from_text(line),
            vec![vec!["cargo", "check", "--features", "renamed"]]
        );
    }
}
