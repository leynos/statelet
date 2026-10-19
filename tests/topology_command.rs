//! Command-boundary cases for `make topology-check`.
//!
//! The unit, property, and behavioural suites feed the policy a document in
//! memory. This target is the only place the *command* is exercised: it writes
//! isolated Cargo fixtures to a scratch directory, runs the real Make target
//! against each one, and asserts the process boundary — exit status, the
//! absence of success text on failure, and the presence of a repair hint.
//!
//! Every case here is `#[ignore]`d, and the reason is the gate rather than the
//! test. A routine or coverage run discovers these tests too, and each one
//! starts a nested Cargo process; the nested run would compile while the outer
//! binary holds the build directory, and under coverage instrumentation the
//! child would inherit instrumentation it was never meant to have. The
//! dedicated `make test-topology-command` gate runs them after the ordinary
//! tests and after coverage, one thread at a time, with
//! `--run-ignored only --no-tests fail`.
//!
//! Two of the cases are the plan's V5 controls rather than fixture verdicts:
//! the pinned `check-topology` target run against a forbidden repository
//! manifest while a healthy path is supplied as `TOPOLOGY_MANIFEST_PATH`, and
//! the policy selection run with the guard's name ignored. Both need a real
//! process, so they cannot live in the pure wiring contract.
//!
//! The fixture path reaches the command through `TOPOLOGY_MANIFEST_PATH`, the
//! Make variable the recipe forwards, not through the environment: the recipe
//! assigns `STATELET_TOPOLOGY_MANIFEST_PATH` itself from that variable, so an
//! environment value would be overwritten and every case would quietly check
//! the repository manifest instead. A per-case make argument is also the only
//! way to vary the input without mutating process-global state.
//!
//! The helpers and the fixture table live beside this entrypoint so each file
//! stays under the repository's length limit. Neither sibling decides a
//! verdict; they build fixtures, invoke the command, and assert what it
//! printed.

#[path = "topology_command/fixtures.rs"]
mod fixtures;
#[path = "topology_command/harness.rs"]
mod harness;
#[path = "topology_command/manifests.rs"]
mod manifests;

use std::{path::Path, process::Command};

use fixtures::fixtures;
use harness::{CARGO, CaseName, check_verdict, remove_scratch, run_case, unique_scratch_dir};
use manifests::{PackageName, plain_package};

/// The pin wins over a supplied override, so the refusal names the pinned tree.
///
/// This is the executable half of the V5 control. A dry run can show that the
/// recipe assigns `STATELET_TOPOLOGY_MANIFEST_PATH` from its own pin, but only
/// a real invocation shows which tree the policy actually refused. The fixture
/// is therefore a genuine two-member workspace built under its own root: a
/// manifest naming a member Cargo cannot load would fail during metadata
/// acquisition, and the test would never reach the policy rule it exists to
/// observe.
#[test]
#[ignore = "run by `make test-topology-command`, which nests a Cargo invocation; routine and \
            coverage runs must not start it"]
fn the_pin_overrides_a_healthy_fixture_override() -> Result<(), String> {
    let root = unique_scratch_dir(CaseName("v5-pin"));
    let forbidden = format!(
        "{}\n[workspace]\nmembers = [\"extra\"]\n",
        plain_package(PackageName("statelet"), "")
    );
    let supplied = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let outcome = harness::write_pinned_fixture(&root, &forbidden).and_then(|()| {
        let pinned = harness::invoke_pinned(&root, &supplied)?;
        check_verdict("the pinned run", &pinned, false, "remove the extra member")
    });
    let cleanup = remove_scratch(&root);
    outcome.and(cleanup)
}

/// A policy selection that matches nothing fails through `--no-tests fail`.
///
/// Two selector spellings select zero tests: a filter naming a test that does
/// not exist, and the guard's own name under `--run-ignored only`, which
/// excludes it as if it were ignored. `cargo test --exact` exits zero in both
/// shapes — the defect D5 recorded — so each must instead exit non-zero and
/// print no success text. The success line belongs to the Make recipe rather
/// than to nextest, so this case runs nextest directly and asserts the absence
/// of success text in its own output.
#[test]
#[ignore = "run by `make test-topology-command`, which nests a Cargo invocation; routine and \
            coverage runs must not start it"]
fn a_zero_selection_policy_selector_fails_closed() -> Result<(), String> {
    let spellings = [
        ("absent", "test(=topology_guard_that_does_not_exist)"),
        ("ignored", "test(=repository_obeys_topology)"),
    ];
    for (case, selector) in spellings {
        let run_ignored = if case == "ignored" { "only" } else { "default" };
        let output = Command::new(CARGO)
            .args([
                "nextest",
                "run",
                "--test",
                "topology_check",
                "--no-tests",
                "fail",
                "--run-ignored",
                run_ignored,
                "-E",
                selector,
            ])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .map_err(|error| format!("cannot run cargo nextest: {error}"))?;
        check_verdict(
            &format!("the {case} selector"),
            &output,
            false,
            "no tests to run",
        )?;
    }
    Ok(())
}

/// Every fixture produces the verdict and text the contract promises.
///
/// Failures are collected rather than short-circuited so one run reports every
/// disagreeing fixture, which keeps a diagnosis from needing one gate run per
/// case.
#[test]
#[ignore = "run by `make test-topology-command`, which nests a Cargo invocation; routine and \
            coverage runs must not start it"]
fn every_fixture_produces_its_documented_verdict() -> Result<(), String> {
    let failures: Vec<String> = fixtures()
        .iter()
        .filter_map(|fixture| run_case(fixture).err())
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
