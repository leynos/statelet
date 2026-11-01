"""Keep Act's Python workflow contracts provisioned before the test gate."""

import copy
from pathlib import Path
from typing import Any

import pytest

import workflow_reading as reading

WORKFLOW = Path(__file__).resolve().parents[2] / ".github/workflows/act-validation.yml"
MDTABLEFIX_ACTION = (
    "leynos/shared-actions/.github/actions/install-mdtablefix@"
    "5bc2b2611f5921ef2c56e4ee3fde2b879a319361"
)
UV_ACTION = "astral-sh/setup-uv@b06acff4b6a41bdd9cdac56507ead0bd7734e757"
RUST_TOOLCHAIN_COMMAND = "make install-rust-toolchain"
CARGO_HACK_COMMAND = "make install-cargo-hack"


def _steps() -> list[dict[str, Any]]:
    """Read Act job steps through the shared strict workflow parser."""
    workflow = reading.parse(WORKFLOW.name, WORKFLOW.read_text(encoding="utf-8"))
    return workflow["jobs"]["act-validation"]["steps"]


def _findings(steps: list[dict[str, Any]]) -> list[str]:
    """Report missing, late, or non-binding runtime prerequisites."""
    names = (
        "Install repository Rust components",
        "Install cargo-hack",
        "Install mdtablefix",
        "Setup uv",
        "Run tests with act validation",
    )
    matching = {name: [step for step in steps if step.get("name") == name] for name in names}
    if any(len(found) != 1 for found in matching.values()):
        return ["Act provisioning and test steps must each occur exactly once"]
    selected = {name: matching[name][0] for name in names}
    return _pin_findings(selected) + _binding_findings(steps, selected)


def _pin_findings(selected: dict[str, dict[str, Any]]) -> list[str]:
    """Hold installer provenance, versions, and the exact test invocation."""
    rust = selected["Install repository Rust components"]
    cargo_hack = selected["Install cargo-hack"]
    formatter = selected["Install mdtablefix"]
    uv = selected["Setup uv"]
    test = selected["Run tests with act validation"]
    findings = []
    if rust.get("run") != RUST_TOOLCHAIN_COMMAND:
        findings.append("Rust toolchain installation command changed")
    if cargo_hack.get("run") != CARGO_HACK_COMMAND:
        findings.append("cargo-hack installation command changed")
    if formatter.get("uses") != MDTABLEFIX_ACTION:
        findings.append("formatter action pin changed")
    if formatter.get("with", {}).get("version") != "0.6.1":
        findings.append("formatter version changed")
    if uv.get("uses") != UV_ACTION:
        findings.append("uv action pin changed")
    if uv.get("with", {}).get("python-version") != "3.14":
        findings.append("managed Python version changed")
    if test.get("run") != "make test WITH_ACT=1":
        findings.append("Act test invocation changed")
    return findings


def _binding_findings(
    steps: list[dict[str, Any]], selected: dict[str, dict[str, Any]]
) -> list[str]:
    """Keep cargo-hack after Rust setup and all prerequisites before Make."""
    rust = selected["Install repository Rust components"]
    cargo_hack = selected["Install cargo-hack"]
    formatter = selected["Install mdtablefix"]
    uv = selected["Setup uv"]
    test = selected["Run tests with act validation"]
    findings = []
    if steps.index(formatter) >= steps.index(test) or steps.index(uv) >= steps.index(test):
        findings.append("runtime provisioning follows the test gate")
    if steps.index(cargo_hack) <= steps.index(rust):
        findings.append("cargo-hack installation precedes Rust provisioning")
    if steps.index(cargo_hack) >= steps.index(test):
        findings.append("cargo-hack installation follows the test gate")
    if any(
        "if" in step or "continue-on-error" in step
        for step in (rust, cargo_hack, formatter, uv, test)
    ):
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


def test_act_rejects_missing_cargo_hack_installation() -> None:
    """The Act test gate requires its feature-matrix tool to be installed."""
    steps = copy.deepcopy(_steps())
    steps[:] = [step for step in steps if step.get("name") != "Install cargo-hack"]
    assert _findings(steps), "missing cargo-hack installation was accepted"


def test_act_rejects_duplicate_cargo_hack_installation() -> None:
    """The cargo-hack installer must have one unambiguous workflow owner."""
    steps = copy.deepcopy(_steps())
    installer = next(step for step in steps if step.get("name") == "Install cargo-hack")
    steps.append(copy.deepcopy(installer))
    assert _findings(steps), "duplicate cargo-hack installation was accepted"


@pytest.mark.parametrize("position", ["before Rust provisioning", "after Act test"])
def test_act_rejects_misordered_cargo_hack_installation(position: str) -> None:
    """Cargo-hack installation follows Rust setup and precedes Make."""
    steps = copy.deepcopy(_steps())
    installer = next(step for step in steps if step.get("name") == "Install cargo-hack")
    steps.remove(installer)
    if position == "before Rust provisioning":
        rust = next(
            index
            for index, step in enumerate(steps)
            if step.get("name") == "Install repository Rust components"
        )
        steps.insert(rust, installer)
    else:
        steps.append(installer)
    assert _findings(steps), f"cargo-hack installation {position} was accepted"


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
        ("Install cargo-hack", "run", "make check-cargo-hack"),
        ("Install cargo-hack", "if", "always()"),
        ("Install cargo-hack", "continue-on-error", True),
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
