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
        problem = _ci_job_shape_problem(name, job)
        if problem is not None:
            return None, [problem]
        if _has_format_check(job["steps"]):
            candidates.append((str(name), job))

    if len(candidates) != 1:
        return None, [f"expected one CI job running make check-fmt, found {len(candidates)}"]

    job_name, job = candidates[0]
    return job, _ci_job_binding_findings(job_name, job)


def _ci_job_shape_problem(name: object, job: object) -> str | None:
    """Reject jobs whose step structure cannot be searched safely."""
    if not isinstance(job, dict):
        return f"CI job {name!r} is not a mapping"
    steps = job.get("steps")
    if not isinstance(steps, list) or any(not isinstance(step, dict) for step in steps):
        return f"CI job {name!r} has an unreadable steps list"
    return None


def _has_format_check(steps: list[Job]) -> bool:
    """Detect the direct formatter-check recipe within readable job steps."""
    return any(
        isinstance(step.get("run"), str)
        and step["run"].strip() == "make check-fmt"
        for step in steps
    )


def _ci_job_binding_findings(job_name: str, job: Job) -> list[str]:
    """Require the job containing Markdown gates to run without softening."""
    if "if" in job or "continue-on-error" in job:
        return [f"CI job {job_name!r} can skip or soften its Markdown gates"]
    return []


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
    install_indices = _action_indices(steps, INSTALL_ACTION)
    format_indices = _matching_indices(steps, _is_formatter_check)
    action_indices = _action_indices(steps, MARKDOWNLINT_ACTION)
    return _required_markdown_step_indices(install_indices, format_indices, action_indices)


def _action_indices(steps: list[Job], action: str) -> list[int]:
    """Return workflow positions using one named action at any revision."""
    prefix = f"{action}@"
    return _matching_indices(steps, lambda step: _uses_action(step, prefix))


def _uses_action(step: Job, prefix: str) -> bool:
    """Check an action reference using a fixed repository/name prefix."""
    uses = step.get("uses")
    return isinstance(uses, str) and uses.startswith(prefix)


def _matching_indices(steps: list[Job], predicate: typ.Callable[[Job], bool]) -> list[int]:
    """Return positions of workflow steps satisfying one route predicate."""
    return [index for index, step in enumerate(steps) if predicate(step)]


def _is_formatter_check(step: Job) -> bool:
    """Recognize the direct Make formatting check used by this workflow."""
    run = step.get("run")
    return isinstance(run, str) and run.strip() == "make check-fmt"


def _required_markdown_step_indices(
    install_indices: list[int], format_indices: list[int], action_indices: list[int]
) -> tuple[tuple[int, int, int], list[str]]:
    """Require exactly one installer, formatter check, and lint action."""
    problems = []

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

    problems = _markdown_installer_findings(installer, installer_inputs)
    problems.extend(
        _markdown_install_order_findings(install_index, format_index, action_index)
    )
    problems.extend(
        _markdown_step_binding_findings(installer, formatter, markdownlint)
    )
    return problems


def _markdown_installer_findings(
    installer: Job, installer_inputs: dict[typ.Any, typ.Any]
) -> list[str]:
    """Check approved installer provenance and pinned mdtablefix version."""
    problems = []
    if installer["uses"] != f"{INSTALL_ACTION}@{INSTALL_PIN}":
        problems.append("mdtablefix installer is not pinned to the approved shared action")
    if installer_inputs.get("version") != "0.6.0":
        problems.append("mdtablefix installer version is not 0.6.0")
    return problems


def _markdown_install_order_findings(
    install_index: int, format_index: int, action_index: int
) -> list[str]:
    """Require formatter and Markdown CI checks to follow provisioning."""
    problems = []
    if install_index >= format_index:
        problems.append("mdtablefix installation does not precede make check-fmt")
    if install_index >= action_index:
        problems.append("mdtablefix installation does not precede Markdown CI lint")
    return problems


def _markdown_step_binding_findings(
    installer: Job, formatter: Job, markdownlint: Job
) -> list[str]:
    """Reject conditional or soft-failing Markdown consumer steps."""
    problems = []
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
