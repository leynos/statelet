"""Read the Markdown workflow route for focused consumer contracts."""

import re
import typing as typ
from pathlib import Path

import codescene_reading as workflows

ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = ROOT / "Makefile"
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
INSTALL_ACTION = "leynos/shared-actions/.github/actions/install-mdtablefix"
INSTALL_PIN = "6dea5677a84fec60ca51b07202570e3af12ffdb4"
MARKDOWNLINT_ACTION = "DavidAnson/markdownlint-cli2-action"
SHA = re.compile(r"[0-9a-f]{40}\Z")

type Job = dict[typ.Any, typ.Any]
type Workflow = dict[typ.Any, typ.Any]


def _ci_route(document: Workflow) -> tuple[Job | None, list[str]]:
    """Return the unique job with Markdown checks and any contract failures."""
    jobs = document.get("jobs")
    if not isinstance(jobs, dict) or not jobs:
        return None, ["CI workflow has no readable jobs"]

    candidates: list[tuple[str, Job]] = []
    for name, job in jobs.items():
        if not isinstance(job, dict):
            return None, [f"CI job {name!r} is not a mapping"]
        steps = job.get("steps")
        if not isinstance(steps, list) or any(not isinstance(step, dict) for step in steps):
            return None, [f"CI job {name!r} has an unreadable steps list"]
        if any(
            isinstance(step.get("run"), str)
            and step["run"].strip() == "make check-fmt"
            for step in steps
        ):
            candidates.append((str(name), job))

    if len(candidates) != 1:
        return None, [f"expected one CI job running make check-fmt, found {len(candidates)}"]

    job_name, job = candidates[0]
    problems: list[str] = []
    if "if" in job or "continue-on-error" in job:
        problems.append(f"CI job {job_name!r} can skip or soften its Markdown gates")
    return job, problems


def _markdown_ci_problems(document: Workflow) -> list[str]:
    """Prove the pinned installer and Markdown gates run bindingly in order."""
    job, problems = _ci_route(document)
    if job is None:
        return problems
    steps = job["steps"]
    indices, missing = _markdown_step_indices(steps)
    problems.extend(missing)
    if missing:
        return problems
    install_index, format_index, action_index = indices
    installer = steps[install_index]
    formatter = steps[format_index]
    markdownlint = steps[action_index]
    problems.extend(
        _markdown_tool_findings(installer, formatter, markdownlint, indices)
    )
    problems.extend(_markdown_action_findings(formatter, markdownlint))
    return problems


def _markdown_step_indices(
    steps: list[dict[typ.Any, typ.Any]],
) -> tuple[tuple[int, int, int], list[str]]:
    """Find the one installer, formatter, and lint action in a CI job."""
    problems = []
    installer_prefix = f"{INSTALL_ACTION}@"
    install_indices = [
        index
        for index, step in enumerate(steps)
        if isinstance(step.get("uses"), str)
        and step["uses"].startswith(installer_prefix)
    ]
    format_indices = [
        index
        for index, step in enumerate(steps)
        if isinstance(step.get("run"), str)
        and step["run"].strip() == "make check-fmt"
    ]
    action_indices = [
        index
        for index, step in enumerate(steps)
        if isinstance(step.get("uses"), str)
        and step["uses"].startswith(f"{MARKDOWNLINT_ACTION}@")
    ]

    if len(install_indices) != 1:
        problems.append(f"expected one {INSTALL_ACTION} step, found {len(install_indices)}")
    if len(format_indices) != 1:
        problems.append(f"expected one make check-fmt step, found {len(format_indices)}")
    if len(action_indices) != 1:
        problems.append(
            f"expected one {MARKDOWNLINT_ACTION} step, found {len(action_indices)}"
        )
    if problems:
        return (0, 0, 0), problems
    return (install_indices[0], format_indices[0], action_indices[0]), problems


def _markdown_tool_findings(
    installer: dict[typ.Any, typ.Any],
    formatter: dict[typ.Any, typ.Any],
    markdownlint: dict[typ.Any, typ.Any],
    indices: tuple[int, int, int],
) -> list[str]:
    """Verify the local table formatter version and its gate ordering."""
    problems = []
    installer_inputs = installer.get("with", {})
    if not isinstance(installer_inputs, dict):
        return ["Markdown installer inputs are not a mapping"]
    install_index, format_index, action_index = indices

    if installer["uses"] != f"{INSTALL_ACTION}@{INSTALL_PIN}":
        problems.append("mdtablefix installer is not pinned to the approved shared action")
    if installer_inputs.get("version") != "0.6.0":
        problems.append("mdtablefix installer version is not 0.6.0")
    if not install_index < format_index:
        problems.append("mdtablefix installation does not precede make check-fmt")
    if not install_index < action_index:
        problems.append("mdtablefix installation does not precede Markdown CI lint")

    for label, step in (
        ("installer", installer),
        ("format check", formatter),
        ("Markdown lint", markdownlint),
    ):
        if "if" in step or "continue-on-error" in step:
            problems.append(f"Markdown {label} can be skipped or allowed to fail")

    return problems


def _markdown_action_findings(
    formatter: dict[typ.Any, typ.Any], markdownlint: dict[typ.Any, typ.Any]
) -> list[str]:
    """Verify the pinned markdownlint action and its tracked-file glob."""
    problems = []
    action_inputs = markdownlint.get("with", {})
    if not isinstance(action_inputs, dict):
        return ["Markdown action inputs are not mappings"]
    uses = markdownlint.get("uses")
    if not isinstance(uses, str) or not uses.startswith(f"{MARKDOWNLINT_ACTION}@"):
        problems.append("Markdown CI lint does not use markdownlint-cli2-action")
    else:
        revision = uses.removeprefix(f"{MARKDOWNLINT_ACTION}@")
        if SHA.fullmatch(revision) is None:
            problems.append("markdownlint-cli2-action is not pinned to a full commit SHA")
    if action_inputs.get("globs") != "**/*.md":
        problems.append("markdownlint-cli2-action globs are not exactly **/*.md")
    if formatter.get("run", "").strip() != "make check-fmt":
        problems.append("CI does not run make check-fmt directly")
    return problems


def _current_ci() -> Workflow:
    """Read CI through the strict YAML reader shared with CV-005 contracts."""
    return workflows.parse(WORKFLOW.name, WORKFLOW.read_text(encoding="utf-8"))


def _step_index(steps: list[dict[typ.Any, typ.Any]], predicate: typ.Callable) -> int:
    """Find exactly one step satisfying ``predicate`` for a focused mutation."""
    matches = [index for index, step in enumerate(steps) if predicate(step)]
    assert len(matches) == 1, f"expected one matching step, found {matches}"
    return matches[0]
