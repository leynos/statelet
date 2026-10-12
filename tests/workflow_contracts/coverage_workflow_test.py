"""Verify coverage inputs in both workflows with ``make test-workflow-contracts``.

These contracts keep coverage configured for Rust while preventing public
artefact publication in both CI workflows.
"""

from copy import deepcopy
from pathlib import Path

import pytest

import workflow_reading as reading

ROOT = Path(__file__).resolve().parents[2]
COVERAGE_ACTION = "leynos/shared-actions/.github/actions/generate-coverage"
EXPECTED_COVERAGE_WORKFLOWS = ("ci.yml", "coverage-main.yml")


def _load_coverage_workflows() -> dict[str, dict[object, object]]:
    """Parse both coverage workflows with the strict repository reader."""
    workflow_dir = ROOT / ".github" / "workflows"
    return {
        name: reading.parse(name, (workflow_dir / name).read_text(encoding="utf-8"))
        for name in EXPECTED_COVERAGE_WORKFLOWS
    }


def _coverage_inputs(workflow: dict[object, object], name: str) -> dict[str, object]:
    """Return the sole pinned generate-coverage step's input mapping."""
    jobs = workflow.get("jobs")
    assert isinstance(jobs, dict), f"{name} must define jobs"
    steps = [
        step
        for job in jobs.values()
        if isinstance(job, dict)
        for step in job.get("steps", [])
        if isinstance(step, dict)
        and str(step.get("uses", "")).split("@", maxsplit=1)[0] == COVERAGE_ACTION
    ]
    assert len(steps) == 1, f"{name} must have one generate-coverage step, got {steps!r}"
    inputs = steps[0].get("with")
    assert isinstance(inputs, dict), f"{name} generate-coverage step needs inputs"
    return inputs


def _assert_coverage_contract(workflows: dict[str, dict[object, object]]) -> None:
    """Require Rust coverage and keep publication disabled in both workflows."""
    for name in EXPECTED_COVERAGE_WORKFLOWS:
        inputs = _coverage_inputs(workflows[name], name)
        assert inputs.get("language") == "rust", (
            f"{name} generate-coverage language must be 'rust', got {inputs.get('language')!r}"
        )
        assert inputs.get("publish-artefact") == "false", (
            f"{name} must not publish the coverage artefact, "
            f"got {inputs.get('publish-artefact')!r}"
        )


def test_coverage_workflows_keep_the_rust_non_publishing_contract() -> None:
    """Check the action inputs that separate PR coverage from publication."""
    _assert_coverage_contract(_load_coverage_workflows())


@pytest.mark.parametrize(
    ("workflow_name", "input_name", "value"),
    [
        (workflow, "language", None)
        for workflow in EXPECTED_COVERAGE_WORKFLOWS
    ]
    + [
        (workflow, "language", "python")
        for workflow in EXPECTED_COVERAGE_WORKFLOWS
    ]
    + [
        (workflow, "publish-artefact", None)
        for workflow in EXPECTED_COVERAGE_WORKFLOWS
    ]
    + [
        (workflow, "publish-artefact", "true")
        for workflow in EXPECTED_COVERAGE_WORKFLOWS
    ],
)
def test_coverage_contract_rejects_removed_or_changed_inputs(
    workflow_name: str, input_name: str, value: str | None
) -> None:
    """Reject input drift that changes the language or enables publication."""
    workflows = deepcopy(_load_coverage_workflows())
    inputs = _coverage_inputs(workflows[workflow_name], workflow_name)
    if value is None:
        inputs.pop(input_name)
    else:
        inputs[input_name] = value

    with pytest.raises(AssertionError):
        _assert_coverage_contract(workflows)
