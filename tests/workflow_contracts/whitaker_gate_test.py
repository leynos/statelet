"""Check Statelet's Whitaker action and binding Make gate."""

import copy
import re

import pytest

from whitaker_contracts import (
    INSTALL_ACTION,
    _is_lint,
    _ci,
    _make_problems,
    _makefile,
    _workflow_problems,
)


def test_ci_provisions_the_binding_whitaker_gate() -> None:
    """CI uses the approved shared installer before the direct Make gate."""
    problems = _workflow_problems(_ci())
    assert not problems, f"unexpected Whitaker CI route: {problems}"


@pytest.mark.parametrize(
    ("mutation", "expected"),
    [
        ("wrong pin", "approved full SHA"),
        ("remove Cranelift input", "cranelift: 'true'"),
        ("move installer after lint", "does not precede"),
        ("condition installer", "can be skipped"),
        ("soft-fail lint", "can be skipped"),
        ("pin rolling suite", "unsupported input"),
        ("remove lint gate", "direct make lint"),
        ("inject RUSTFLAGS", "non-empty repository RUSTFLAGS"),
        ("add installer shim", "ad hoc installer"),
    ],
    ids=[
        "action-pin",
        "cranelift",
        "order",
        "installer-binding",
        "lint-binding",
        "rolling-suite",
        "gate-required",
        "non-empty-flags",
        "no-shim",
    ],
)
def test_ci_mutations_are_rejected(mutation: str, expected: str) -> None:
    """Reject installer pin, input, ordering, and gate bypasses."""
    workflow = copy.deepcopy(_ci())
    jobs = workflow["jobs"]
    assert isinstance(jobs, dict), (
        "test_ci_mutations_are_rejected contract failed"
    )
    job = next(
        job for job in jobs.values()
        if isinstance(job, dict) and any(map(_is_lint, job["steps"]))
    )
    steps = job["steps"]
    installer = next(
        i for i, step in enumerate(steps)
        if str(step.get("uses", "")).startswith(f"{INSTALL_ACTION}@")
    )
    lint = next(i for i, step in enumerate(steps) if _is_lint(step))
    match mutation:
        case "wrong pin":
            steps[installer]["uses"] = f"{INSTALL_ACTION}@v0.2.9"
        case "remove Cranelift input":
            steps[installer]["with"].pop("cranelift")
        case "move installer after lint":
            steps.insert(lint + 1, steps.pop(installer))
        case "condition installer":
            steps[installer]["if"] = "false"
        case "soft-fail lint":
            steps[lint]["continue-on-error"] = "true"
        case "pin rolling suite":
            steps[installer]["with"]["suite-version"] = "0.2.9"
        case "remove lint gate":
            steps.pop(lint)
        case "inject RUSTFLAGS":
            steps[installer]["env"] = {"RUSTFLAGS": "-Ctarget-cpu=native"}
        case "add installer shim":
            steps.insert(
                installer + 1,
                {"name": "Whitaker shim", "run": "whitaker-installer --cranelift"},
            )
        case _:
            raise AssertionError(f"unrecognized workflow mutation: {mutation}")
    problems = _workflow_problems(workflow)
    assert any(expected in problem for problem in problems), f"{mutation}: {problems}"


def test_make_runs_all_packages_and_sequences_clippy_before_whitaker() -> None:
    """Make covers every package and runs the gates in a defined sequence."""
    problems = _make_problems(_makefile())
    assert not problems, f"unexpected Make Whitaker route: {problems}"


@pytest.mark.parametrize(
    ("mutation", "expected"),
    [
        ("remove gate", "no lint-whitaker leaf"),
        ("narrow packages", "every workspace package"),
        ("ignore failure", "mask or ignore"),
        ("mask failure", "mask or ignore"),
        ("inject RUSTFLAGS", "injects repository RUSTFLAGS"),
    ],
    ids=("gate-required", "all-workspace-packages", "make-ignore", "shell-mask", "flags-isolated"),
)
def test_make_mutations_are_rejected(mutation: str, expected: str) -> None:
    """Reject an incomplete, reordered, or failure-masked Make gate."""
    text = _makefile()
    match mutation:
        case "remove gate":
            text = re.sub(r"(?m)^lint-whitaker:.*(?:\n\t[^\n]*)*\n?", "", text)
        case "narrow packages":
            text = re.sub(
                r"(?m)^(WHITAKER_PACKAGES\s*\?=\s*).*--workspace.*$",
                r"\1--package statelet",
                text,
            )
        case "ignore failure":
            text = text.replace("\tRUSTFLAGS= $(WHITAKER)", "\t-$(WHITAKER)")
        case "mask failure":
            text = re.sub(r"(?m)^(\tRUSTFLAGS= \$\(WHITAKER\)[^\n]*)$", r"\1 || true", text)
        case "inject RUSTFLAGS":
            text = text.replace(
                "\tRUSTFLAGS= $(WHITAKER)", '\tRUSTFLAGS="-D warnings" $(WHITAKER)'
            )
        case _:
            raise AssertionError(f"unrecognized Make mutation: {mutation}")
    problems = _make_problems(text)
    assert any(expected in problem for problem in problems), f"{mutation}: {problems}"
