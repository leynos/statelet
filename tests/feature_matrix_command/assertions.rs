//! Interpret the command-boundary suite's recorded Cargo invocations.

use std::process::Output;

use googletest::prelude::*;

use super::output_text;

/// Return the individual Cargo commands reported by cargo-hack.
pub(super) fn hack_rows(output: &Output) -> Vec<Vec<String>> {
    output_text(output)
        .lines()
        .filter_map(|line| line.split_once("info: running `"))
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
