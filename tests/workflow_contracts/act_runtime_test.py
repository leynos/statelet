"""Keep Act's Python workflow contracts provisioned before the test gate."""

import copy
from pathlib import Path
from typing import Any

import pytest

import codescene_reading as reading

WORKFLOW = Path(__file__).resolve().parents[2] / ".github/workflows/act-validation.yml"
MDTABLEFIX_ACTION = (
    "leynos/shared-actions/.github/actions/install-mdtablefix@"
    "6dea5677a84fec60ca51b07202570e3af12ffdb4"
)
UV_ACTION = "astral-sh/setup-uv@a96208bed1fb5efb8da349c9bcc6cc58af9e7d74"


def _steps() -> list[dict[str, Any]]:
    """Read Act job steps through the shared strict workflow parser."""
    workflow = reading.parse(WORKFLOW.name, WORKFLOW.read_text(encoding="utf-8"))
    return workflow["jobs"]["act-validation"]["steps"]


def _findings(steps: list[dict[str, Any]]) -> list[str]:
    """Report missing, late, or non-binding runtime prerequisites."""
    names = ("Install mdtablefix", "Setup uv", "Run tests with act validation")
    matching = {name: [step for step in steps if step.get("name") == name] for name in names}
    if any(len(found) != 1 for found in matching.values()):
        return ["Act runtime steps must each occur exactly once"]
    formatter, uv, test = (matching[name][0] for name in names)
    return _pin_findings(formatter, uv, test) + _binding_findings(
        steps, formatter, uv, test
    )


def _pin_findings(
    formatter: dict[str, Any], uv: dict[str, Any], test: dict[str, Any]
) -> list[str]:
    """Hold installer provenance, versions, and the exact test invocation."""
    findings = []
    if formatter.get("uses") != MDTABLEFIX_ACTION:
        findings.append("formatter action pin changed")
    if formatter.get("with", {}).get("version") != "0.6.0":
        findings.append("formatter version changed")
    if uv.get("uses") != UV_ACTION:
        findings.append("uv action pin changed")
    if uv.get("with", {}).get("python-version") != "3.14":
        findings.append("managed Python version changed")
    if test.get("run") != "make test WITH_ACT=1":
        findings.append("Act test invocation changed")
    return findings


def _binding_findings(
    steps: list[dict[str, Any]],
    formatter: dict[str, Any],
    uv: dict[str, Any],
    test: dict[str, Any],
) -> list[str]:
    """Keep both installers before a non-skippable Make test step."""
    findings = []
    if steps.index(formatter) >= steps.index(test) or steps.index(uv) >= steps.index(test):
        findings.append("runtime provisioning follows the test gate")
    if any("if" in step or "continue-on-error" in step for step in (formatter, uv, test)):
        findings.append("runtime provisioning or test gate can be skipped")
    return findings


def test_act_provisions_runtime_before_binding_make_test() -> None:
    """The Act job installs the tested formatter and Python before Make."""
    assert not (findings := _findings(_steps())), findings


@pytest.mark.parametrize("step_name", ["Install mdtablefix", "Setup uv"])
def test_act_rejects_late_runtime_provisioning(step_name: str) -> None:
    """Moving either required installer after Make invalidates the route."""
    steps = copy.deepcopy(_steps())
    installer = next(step for step in steps if step.get("name") == step_name)
    steps.remove(installer)
    steps.append(installer)
    assert _findings(steps), f"late {step_name} was accepted"


@pytest.mark.parametrize(
    ("step_name", "field", "invalid"),
    [
        (
            "Install mdtablefix",
            "uses",
            "leynos/shared-actions/.github/actions/install-mdtablefix@main",
        ),
        ("Install mdtablefix", "with", {"version": "0.5.0"}),
        ("Setup uv", "uses", "astral-sh/setup-uv@main"),
        ("Setup uv", "with", {"python-version": "3.13"}),
        ("Run tests with act validation", "run", "echo make test WITH_ACT=1"),
        ("Install mdtablefix", "if", "false"),
        ("Setup uv", "continue-on-error", True),
        ("Run tests with act validation", "continue-on-error", True),
    ],
)
def test_act_rejects_unpinned_or_nonbinding_runtime(
    step_name: str, field: str, invalid: Any
) -> None:
    """A changed pin, interpreter, or failure path cannot pass the contract."""
    steps = copy.deepcopy(_steps())
    step = next(item for item in steps if item.get("name") == step_name)
    step[field] = invalid
    assert _findings(steps), f"{step_name} accepted invalid {field}"
