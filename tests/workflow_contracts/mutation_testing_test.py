"""Contract tests for the mutation-testing caller workflow.

The executable logic lives in the ``leynos/shared-actions`` reusable
workflow, which carries its own unit and integration tests; statelet's
caller is declarative configuration. These tests parse the caller strictly
and pin the contract it must uphold, so drift (repointing the pin at a
branch, widening permissions, or losing its pre-suite tool setup and
feature configuration) fails CI on the pull request rather than
surfacing in a scheduled or manual run. The caller must reference the
correct reusable workflow at a commit SHA; Dependabot owns the SHA
value, so these tests assert the shape of the pin rather than its exact
value.

Run via ``make test-workflow-contracts``.
"""

import re
from copy import deepcopy
from pathlib import Path

import pytest

import codescene_reading as reading
from suite_provisioning import (
    INSTALL_COMMAND,
    load_workflows,
    runner_platforms,
)
from suite_discovery import suite_findings

WORKFLOW_PATH = (
    Path(__file__).resolve().parents[2] / ".github" / "workflows" / "mutation-testing.yml"
)

pytestmark = pytest.mark.skipif(
    not WORKFLOW_PATH.exists(),
    reason="workflow file not present in this working copy (e.g. "
    "inside mutmut's mutants/ sandbox, which does not copy .github/)",
)

USES_RE = re.compile(
    r"^leynos/shared-actions/\.github/workflows/mutation-cargo\.yml@[0-9a-f]{40}$"
)

EXPECTED_SETUP_COMMANDS = (
    "set -euo pipefail\n"
    "export DEBIAN_FRONTEND=noninteractive\n"
    "sudo apt-get update\n"
    "sudo apt-get install --yes --no-install-recommends clang lld\n"
    "make install-build-tools\n"
    'echo "$HOME/.local/bin" >> "$GITHUB_PATH"\n'
)

#: The exact caller configuration: --all-features mirrors the CI test
#: baseline (CARGO_FLAGS = --all-targets --all-features), and the setup
#: setup commands provision Linux linkers and pinned build tools before the
#: shared reusable workflow starts cargo-mutants.
EXPECTED_WITH_BLOCK = {
    "extra-args": "--all-features",
    "setup-commands": EXPECTED_SETUP_COMMANDS,
}


def _load() -> dict[str, object]:
    """Parse the workflow strictly, refusing duplicate or ambiguous keys."""
    return reading.parse(
        WORKFLOW_PATH.name, WORKFLOW_PATH.read_text(encoding="utf-8")
    )


def _triggers(workflow: dict[str, object]) -> dict[str, object]:
    """Return the ``on:`` mapping (PyYAML parses the bare key as True)."""
    triggers = workflow.get("on", workflow.get(True))
    assert isinstance(triggers, dict), "the workflow must declare an on: mapping"
    return triggers


def _mutation_job(workflow: dict[str, object]) -> dict[str, object]:
    """Return the single calling job."""
    jobs = workflow.get("jobs")
    assert isinstance(jobs, dict), "the workflow must declare a jobs mapping"
    assert jobs, "the workflow must declare at least one job"
    assert list(jobs) == ["mutation"], (
        f"expected a single job named 'mutation', found {sorted(jobs)}"
    )
    return jobs["mutation"]


def test_uses_reference_is_pinned_to_a_commit_sha() -> None:
    """The job must call mutation-cargo.yml pinned to a full commit SHA.

    Dependabot owns the SHA value, so this only asserts the shape of the
    pin (the correct reusable workflow path, pinned to a 40-character
    lowercase hex commit SHA rather than a mutable branch or tag) and not
    which SHA is currently pinned.
    """
    uses = _mutation_job(_load()).get("uses")
    assert isinstance(uses, str), "jobs.mutation.uses must be a string"
    assert USES_RE.match(uses), (
        "jobs.mutation.uses must reference mutation-cargo.yml pinned to a "
        f"full 40-character lowercase hex commit SHA, got {uses!r}"
    )


def test_job_permissions_are_exactly_least_privilege() -> None:
    """The job grants contents: read and id-token: write, nothing broader."""
    permissions = _mutation_job(_load()).get("permissions")
    assert permissions == {"contents": "read", "id-token": "write"}, (
        "jobs.mutation.permissions must be exactly "
        f"{{'contents': 'read', 'id-token': 'write'}}, got {permissions!r}"
    )


def test_workflow_default_permissions_are_empty() -> None:
    """The workflow-level default token scope is empty."""
    workflow = _load()
    assert workflow.get("permissions") == {}, (
        f"top-level permissions must be an empty mapping, got "
        f"{workflow.get('permissions')!r}"
    )


def test_concurrency_serializes_per_ref_without_cancelling() -> None:
    """Runs queue per ref instead of cancelling one another."""
    concurrency = _load().get("concurrency")
    assert isinstance(concurrency, dict), "the workflow must declare concurrency"
    assert concurrency.get("group") == "mutation-testing-${{ github.ref }}", (
        f"concurrency.group must key on the triggering ref, got "
        f"{concurrency.get('group')!r}"
    )
    assert concurrency.get("cancel-in-progress") is False, (
        f"concurrency.cancel-in-progress must be false, got "
        f"{concurrency.get('cancel-in-progress')!r}"
    )


def test_triggers_keep_schedule_and_plain_dispatch() -> None:
    """The daily schedule stays; dispatch has no legacy branch input."""
    triggers = _triggers(_load())
    schedule = triggers.get("schedule")
    assert schedule == [{"cron": "20 11 * * *"}], (
        f"on.schedule must be the daily 11:20 UTC cron, got {schedule!r}"
    )
    assert "workflow_dispatch" in triggers, "on.workflow_dispatch is missing"
    dispatch = triggers.get("workflow_dispatch")
    inputs: dict[str, object] = {}
    if dispatch is not None:
        assert isinstance(dispatch, dict), "workflow_dispatch must be a mapping"
        declared_inputs = dispatch.get("inputs")
        if declared_inputs is not None:
            assert isinstance(declared_inputs, dict), (
                "workflow_dispatch.inputs must be a mapping"
            )
            inputs = declared_inputs
    assert "branch" not in inputs, (
        "on.workflow_dispatch must not declare a branch input; the Actions "
        "run-workflow control selects the ref"
    )


def test_with_block_carries_the_caller_configuration() -> None:
    """The caller passes test features and pre-suite tool provisioning."""
    with_block = _mutation_job(_load()).get("with")
    assert with_block == EXPECTED_WITH_BLOCK, (
        "jobs.mutation.with must configure exactly --all-features (the CI "
        "test baseline) and clang/lld plus pinned build-tool setup, "
        f"got {with_block!r}"
    )


def test_workflow_reader_rejects_duplicate_and_ambiguous_trigger_keys() -> None:
    """A lossy YAML parse cannot hide a second route from contracts."""
    duplicate = "jobs:\n  run:\n    runs-on: ubuntu-latest\n    runs-on: macos-latest\n"
    with pytest.raises(reading.ContractError, match="duplicate key"):
        reading.parse("duplicate.yml", duplicate)
    both_trigger_keys = (
        "'on': push\ntrue: workflow_dispatch\n"
        "jobs: {run: {runs-on: ubuntu-latest, steps: []}}\n"
    )
    with pytest.raises(reading.ContractError, match="both 'on' and true"):
        reading.parse("ambiguous.yml", both_trigger_keys)

    for trigger, expected in (
        ("on: push", ["push"]),
        ("on: [push, pull_request]", ["push", "pull_request"]),
        ("on:\n  push:\n  pull_request:\n", ["push", "pull_request"]),
    ):
        workflow = reading.parse(
            "trigger-shape.yml",
            f"{trigger}\njobs: {{run: {{runs-on: ubuntu-latest, "
            "steps: []}}\n",
        )
        assert reading.trigger_names(workflow) == expected, (
            "test_workflow_reader_rejects_duplicate_and_ambiguous_trigger_keys contract failed"
        )


def test_suite_provisioning_contract_rejects_workflow_mutations() -> None:
    """Representative route, ordering, and softness changes fail closed."""
    baseline = load_workflows()

    def problems_after(change) -> list[str]:
        workflows = deepcopy(baseline)
        change(workflows)
        return suite_findings(workflows)

    def remove_act_installer(workflows) -> None:
        steps = workflows["act-validation.yml"]["jobs"]["act-validation"]["steps"]
        steps[:] = [step for step in steps if step.get("run") != INSTALL_COMMAND]

    assert any(
        "act-validation.yml:act-validation" in finding
        for finding in problems_after(remove_act_installer)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def replace_act_installer_with_echo(workflows) -> None:
        steps = workflows["act-validation.yml"]["jobs"]["act-validation"]["steps"]
        next(step for step in steps if step.get("run") == INSTALL_COMMAND)["run"] = (
            f"echo {INSTALL_COMMAND}"
        )

    assert any(
        "act-validation.yml:act-validation" in finding
        for finding in problems_after(replace_act_installer_with_echo)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def replace_act_installer_with_conditional_noop(workflows) -> None:
        steps = workflows["act-validation.yml"]["jobs"]["act-validation"]["steps"]
        next(step for step in steps if step.get("run") == INSTALL_COMMAND)["run"] = (
            f"if false; then {INSTALL_COMMAND}; fi"
        )

    assert any(
        "act-validation.yml:act-validation" in finding
        for finding in problems_after(replace_act_installer_with_conditional_noop)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def move_ci_installer_after_coverage(workflows) -> None:
        steps = workflows["ci.yml"]["jobs"]["build-test"]["steps"]
        installer = next(step for step in steps if step.get("run") == INSTALL_COMMAND)
        steps.remove(installer)
        steps.append(installer)

    assert any(
        "ci.yml:build-test" in finding
        for finding in problems_after(move_ci_installer_after_coverage)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def make_act_installer_conditional(workflows) -> None:
        steps = workflows["act-validation.yml"]["jobs"]["act-validation"]["steps"]
        next(step for step in steps if step.get("run") == INSTALL_COMMAND)["if"] = "always()"

    assert any(
        "act-validation.yml:act-validation" in finding
        for finding in problems_after(make_act_installer_conditional)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def make_act_installer_soft_fail(workflows) -> None:
        steps = workflows["act-validation.yml"]["jobs"]["act-validation"]["steps"]
        next(step for step in steps if step.get("run") == INSTALL_COMMAND)[
            "continue-on-error"
        ] = True

    assert any(
        "act-validation.yml:act-validation" in finding
        for finding in problems_after(make_act_installer_soft_fail)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def delete_publisher_coverage_action(workflows) -> None:
        steps = workflows["coverage-main.yml"]["jobs"]["coverage-upload"]["steps"]
        steps[:] = [step for step in steps if "generate-coverage" not in step.get("uses", "")]

    assert any(
        "coverage-main.yml" in finding
        for finding in problems_after(delete_publisher_coverage_action)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def add_linux_nextest_job(workflows) -> None:
        jobs = workflows["act-validation.yml"]["jobs"]
        jobs["new-suite"] = {
            "runs-on": "ubuntu-latest",
            "steps": [{"run": "cargo nextest run --workspace"}],
        }

    assert any(
        "act-validation.yml:new-suite" in finding
        for finding in problems_after(add_linux_nextest_job)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )

    def add_unresolved_reusable_suite(workflows) -> None:
        workflows["act-validation.yml"]["jobs"]["external-suite"] = {
            "uses": "example/other/.github/workflows/rust-suite.yml@" + "a" * 40,
            "with": {"setup-commands": INSTALL_COMMAND},
        }

    assert any(
        "unresolved reusable call" in finding
        for finding in problems_after(add_unresolved_reusable_suite)
    ), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )
    assert any("no workflows" in finding for finding in suite_findings({})), (
        "test_suite_provisioning_contract_rejects_workflow_mutations contract failed"
    )


def test_suite_runner_reader_handles_supported_runner_forms() -> None:
    """Linux labels remain visible across scalar, list, group, and matrix forms."""
    assert runner_platforms({"runs-on": "ubuntu-latest"}) == {"linux"}, (
        "test_suite_runner_reader_handles_supported_runner_forms contract failed"
    )
    assert runner_platforms({"runs-on": ["self-hosted", "linux", "x64"]}) == {"linux"}, (
        "test_suite_runner_reader_handles_supported_runner_forms contract failed"
    )
    assert runner_platforms(
        {"runs-on": {"group": "shared", "labels": "ubuntu-24.04"}}
    ) == {"linux"}, (
        "test_suite_runner_reader_handles_supported_runner_forms contract failed"
    )
    assert runner_platforms(
        {
            "runs-on": "${{ matrix.os }}",
            "strategy": {"matrix": {"os": ["ubuntu-latest", "windows-latest"]}},
        }
    ) == {"linux", "windows"}, (
        "test_suite_runner_reader_handles_supported_runner_forms contract failed"
    )
    with pytest.raises(ValueError, match="runner labels"):
        runner_platforms({"runs-on": "self-hosted-special"})
