"""Mutation probes for CV-005's two coverage lanes and publisher preflight."""

from __future__ import annotations

import codescene_baseline as baseline
import codescene_calls as calls
import codescene_integrity as integrity
import codescene_reading as reading
import codescene_rules as rules
import pytest


@pytest.fixture
def every() -> dict[str, reading.Workflow]:
    """Read isolated workflow values so each mutation has its own subject."""
    return reading.workflows(reading.WORKFLOW_DIR)


def _publisher_steps(every: dict[str, reading.Workflow]) -> list[reading.Step]:
    """Return the publisher's step list, which each probe mutates locally."""
    return every["coverage-main.yml"]["jobs"]["coverage-upload"]["steps"]


def _coverage(every: dict[str, reading.Workflow], name: str) -> reading.Step:
    """Return the single coverage step in a named workflow."""
    return next(step for step in reading.steps(every[name]) if rules.is_coverage(step))


@pytest.mark.parametrize(
    ("change", "reason"),
    [
        ("pr_action_sha", "unapproved coverage action SHA"),
        ("publisher_action_sha", "unapproved coverage action SHA"),
        ("setup_sha", "setup-rust action has an unapproved SHA"),
        ("upload_sha", "uploader has an unapproved action SHA"),
        ("pr_features", "mismatched coverage inputs"),
        ("publisher_targets", "mismatched coverage inputs"),
        ("publisher_report", "mismatched coverage inputs"),
        ("pr_linker", "mismatched coverage toolchain/linker"),
        ("publisher_profile", "mismatched build profile"),
        ("pr_soft_failure", "coverage step may skip or fail softly"),
        ("no_install", "build-tool install is absent"),
        ("wrong_install_command", "build-tool install is absent"),
        ("install_soft_failure", "build-tool install is absent"),
        ("late_install", "preflight must precede coverage"),
        ("no_linker", "clang/lld installation is absent"),
        ("late_linker", "preflight must precede coverage"),
        ("linker_soft_failure", "clang/lld installation is absent"),
        ("ci_no_install", "ci.yml build-tool preflight"),
        ("ci_late_install", "ci.yml build-tool preflight"),
        ("ci_install_soft_failure", "ci.yml build-tool preflight"),
        ("extra_linux_suite", "exactly one coverage-upload job"),
        ("extra_trigger", "main push and dispatch only"),
        ("checkout_credentials", "disable persisted credentials"),
        ("duplicate_check", "token check command is absent or altered"),
    ],
)
def test_integrity_mutations_are_rejected(
    every: dict[str, reading.Workflow], change: str, reason: str
) -> None:
    """A single wrong pin, scope, preflight or job shape breaks the contract."""
    publisher = every["coverage-main.yml"]
    steps = _publisher_steps(every)
    pr_coverage = _coverage(every, "ci.yml")
    main_coverage = _coverage(every, "coverage-main.yml")
    if change == "pr_action_sha":
        pr_coverage["uses"] = pr_coverage["uses"].replace("@6dea5677", "@4fb8eb7")
    elif change == "publisher_action_sha":
        main_coverage["uses"] = main_coverage["uses"].replace("@6dea5677", "@4fb8eb7")
    elif change == "setup_sha":
        next(step for step in steps if "setup-rust" in reading.uses(step))["uses"] += (
            "x"
        )
    elif change == "upload_sha":
        next(step for step in steps if rules.is_upload_action(step))["uses"] += "x"
    elif change == "pr_features":
        pr_coverage["with"]["all-features"] = "true"
    elif change == "publisher_targets":
        main_coverage["with"]["all-targets"] = "true"
    elif change == "publisher_report":
        main_coverage["with"]["output-path"] = "other.info"
    elif change == "pr_linker":
        pr_coverage["env"]["RUSTFLAGS"] = ""
    elif change == "publisher_profile":
        publisher["jobs"]["coverage-upload"]["env"]["BUILD_PROFILE"] = "release"
    elif change == "pr_soft_failure":
        pr_coverage["continue-on-error"] = True
    elif change in {"no_install", "late_install"}:
        install = next(
            step for step in steps if step.get("name") == "Install build tools"
        )
        steps.remove(install)
        if change == "late_install":
            steps.append(install)
    elif change == "wrong_install_command":
        next(step for step in steps if step.get("name") == "Install build tools")[
            "run"
        ] = "true"
    elif change == "install_soft_failure":
        next(step for step in steps if step.get("name") == "Install build tools")[
            "continue-on-error"
        ] = True
    elif change in {"no_linker", "late_linker", "linker_soft_failure"}:
        linker = next(
            step
            for step in steps
            if step.get("name") == "Install coverage linker tools"
        )
        if change == "linker_soft_failure":
            linker["continue-on-error"] = True
        else:
            steps.remove(linker)
            if change == "late_linker":
                steps.append(linker)
    elif change in {"ci_no_install", "ci_late_install", "ci_install_soft_failure"}:
        ci_steps = every["ci.yml"]["jobs"]["build-test"]["steps"]
        install = next(
            step for step in ci_steps if step.get("name") == "Install build tools"
        )
        if change == "ci_install_soft_failure":
            install["continue-on-error"] = True
        else:
            ci_steps.remove(install)
            if change == "ci_late_install":
                ci_steps.append(install)
    elif change == "extra_linux_suite":
        publisher["jobs"]["other-linux-suite"] = {
            "runs-on": "ubuntu-latest",
            "steps": [{"run": "make test"}],
        }
    elif change == "extra_trigger":
        publisher["on"]["schedule"] = [{"cron": "0 0 * * *"}]
    elif change == "checkout_credentials":
        steps[0]["with"]["persist-credentials"] = True
    elif change == "duplicate_check":
        steps.insert(-1, {"id": "other-check", "run": rules.CHECK_COMMAND})
    findings = integrity.coverage_contract_findings(every)
    assert any(reason in finding for finding in findings), (change, findings)


@pytest.mark.parametrize(
    "reference",
    [
        "leynos/statelet/.github/workflows/called.yml@6dea5677a84fec60ca51b07202570e3af12ffdb4",
        "leynos/other/.github/workflows/called.yml@6dea5677a84fec60ca51b07202570e3af12ffdb4",
        "./.github/workflows/called.yml@main",
    ],
    ids=["qualified_self", "remote", "local_with_ref"],
)
def test_unprovable_pull_request_call_is_rejected(reference: str) -> None:
    """A qualified or remote call cannot silently escape the local closure."""
    caller = reading.parse(
        "caller.yml",
        "on: pull_request\njobs:\n  call:\n    uses: " + reference + "\n",
    )
    every = {"caller.yml": caller}
    findings = calls.unprovable_callees(every, reading.pull_request_closure(every))
    assert any("unprovable call" in finding for finding in findings), findings


def test_reviewed_remote_call_is_tied_to_its_immutable_pin() -> None:
    """The audited Dependabot route has one exact workflow, job and pin."""
    caller = reading.parse(
        "dependabot-automerge.yml",
        "on: pull_request_target\njobs:\n  automerge:\n    uses: "
        + calls.REVIEWED_AUTOMERGE_CALL
        + "\n    with:\n"
        "      pull-request-number: ${{ inputs.pull-request-number || github.event.pull_request.number }}\n",
    )
    every = {"dependabot-automerge.yml": caller}
    assert calls.unprovable_callees(every, reading.pull_request_closure(every)) == []
    caller["jobs"]["automerge"]["secrets"] = "inherit"
    assert any("inherit" in finding for finding in rules.pull_request_findings(caller))


@pytest.mark.parametrize(
    "prefix", ["./", "$/"], ids=["dot_prefixed", "dollar_prefixed"]
)
def test_a_called_baseline_writer_is_counted(prefix: str) -> None:
    """A push reaching a second ratcheted step through a call is two writers."""
    caller = reading.parse(
        "caller",
        "on:\n  push:\n    branches: ['**']\njobs:\n  call:\n"
        f"    uses: {prefix}.github/workflows/called.yml\n",
    )
    called = reading.parse(
        "called",
        "on: workflow_call\njobs:\n  measure:\n    steps:\n"
        "      - uses: leynos/shared-actions/.github/actions/generate-coverage@abc\n"
        "        with:\n          with-ratchet: 'true'\n",
    )
    every = {
        "coverage-main.yml": reading.workflows(reading.WORKFLOW_DIR)[
            "coverage-main.yml"
        ],
        "caller.yml": caller,
        "called.yml": called,
    }
    assert baseline.baseline_writers(every) == ["called.yml", "coverage-main.yml"]
