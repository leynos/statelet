"""Hold Statelet's Markdown formatter and CI routes to the selected baseline."""

from __future__ import annotations

import copy
import json
import os
import re
import shutil
import subprocess
import typing as typ
from pathlib import Path

import pytest

import codescene_reading as workflows

ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = ROOT / "Makefile"
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
INSTALL_ACTION = "leynos/shared-actions/.github/actions/install-mdtablefix"
INSTALL_PIN = "6dea5677a84fec60ca51b07202570e3af12ffdb4"
MARKDOWNLINT_ACTION = "DavidAnson/markdownlint-cli2-action"
SHA = re.compile(r"[0-9a-f]{40}\Z")

Job = dict[typ.Any, typ.Any]
Workflow = dict[typ.Any, typ.Any]


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
        return problems

    install_index = install_indices[0]
    format_index = format_indices[0]
    action_index = action_indices[0]
    installer = steps[install_index]
    formatter = steps[format_index]
    markdownlint = steps[action_index]

    installer_inputs = installer.get("with", {})
    action_inputs = markdownlint.get("with", {})
    if not isinstance(installer_inputs, dict) or not isinstance(action_inputs, dict):
        return ["Markdown installer or action inputs are not mappings"]

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


def test_ci_uses_binding_markdown_tools_in_order() -> None:
    """CI provisions mdtablefix before the check and runs the pinned linter."""
    problems = _markdown_ci_problems(_current_ci())
    assert not problems, f"unexpected Markdown CI route: {problems}"


def test_local_markdown_tools_match_the_pinned_ci_versions() -> None:
    """Make pins local Markdown tools to the versions used by CI."""
    makefile = MAKEFILE.read_text(encoding="utf-8")
    assert re.search(r"(?m)^MARKDOWNLINT_VERSION \?= 0\.23\.2$", makefile)
    assert re.search(r"(?m)^MDLINT \?= markdownlint-cli2$", makefile)
    assert re.search(
        r'(?ms)^install-markdownlint:.*?\n\tnpm install --global --prefix '
        r'"\$\(BUILD_TOOLS_PREFIX\)" "markdownlint-cli2@'
        r'\$\(MARKDOWNLINT_VERSION\)"$',
        makefile,
    )
    assert re.search(r"(?m)^MDTABLEFIX_VERSION \?= 0\.6\.0$", makefile)
    assert re.search(
        r"(?ms)^install-mdtablefix:.*?\n\t\$\(CARGO\) install --locked "
        r"--version \$\(MDTABLEFIX_VERSION\) mdtablefix$",
        makefile,
    )
    markdown_config = json.loads(
        (ROOT / ".markdownlint-cli2.jsonc").read_text("utf-8")
    )
    assert "**/target/**" in markdown_config["ignores"]
    assert "-not -path './target/*'" in makefile


@pytest.mark.parametrize(
    ("mutation", "expected"),
    [
        ("remove installer", "expected one"),
        ("move installer after check", "does not precede"),
        ("condition installer", "can be skipped"),
        ("soft-fail formatter", "can be skipped"),
        ("string soft-fail formatter", "can be skipped"),
        ("narrow globs", "globs are not exactly"),
        ("use mutable action tag", "full commit SHA"),
        ("skip markdown action", "can be skipped"),
    ],
    ids=[
        "installer-required",
        "installer-before-check",
        "installer-unconditional",
        "check-binding",
        "string-valued-soft-failure",
        "all-markdown-globs",
        "full-action-sha",
        "markdown-action-binding",
    ],
)
def test_ci_mutations_are_rejected(mutation: str, expected: str) -> None:
    """Representative CI bypasses cannot make the contract pass."""
    document = copy.deepcopy(_current_ci())
    job, _ = _ci_route(document)
    assert job is not None
    steps = job["steps"]

    install_index = _step_index(
        steps,
        lambda step: str(step.get("uses", "")).startswith(f"{INSTALL_ACTION}@"),
    )
    format_index = _step_index(
        steps, lambda step: step.get("run", "").strip() == "make check-fmt"
    )
    action_index = _step_index(
        steps,
        lambda step: str(step.get("uses", "")).startswith(f"{MARKDOWNLINT_ACTION}@"),
    )

    if mutation == "remove installer":
        steps.pop(install_index)
    elif mutation == "move installer after check":
        steps.insert(format_index + 1, steps.pop(install_index))
    elif mutation == "condition installer":
        steps[install_index]["if"] = "false"
    elif mutation == "soft-fail formatter":
        steps[format_index]["continue-on-error"] = True
    elif mutation == "string soft-fail formatter":
        steps[format_index]["continue-on-error"] = "true"
    elif mutation == "narrow globs":
        steps[action_index]["with"]["globs"] = "docs/*.md"
    elif mutation == "use mutable action tag":
        steps[action_index]["uses"] = f"{MARKDOWNLINT_ACTION}@v24.2.0"
    elif mutation == "skip markdown action":
        steps[action_index]["if"] = "false"
    else:
        raise AssertionError(f"unrecognized test mutation {mutation!r}")

    problems = _markdown_ci_problems(document)
    assert any(expected in problem for problem in problems), (
        f"mutation {mutation!r} escaped its intended check: {problems}"
    )


def _run_make(
    target: str,
    checkout: Path,
    mdtablefix: str,
    md_lint: Path,
    md_lint_status: int = 0,
) -> subprocess.CompletedProcess[str]:
    """Run one Make Markdown target in a temporary Git checkout."""
    make = shutil.which("make")
    assert make is not None, "GNU Make is required to test the Markdown recipes"
    cargo = shutil.which("true")
    assert cargo is not None, "the platform true command is required to stub Cargo"
    environment = os.environ.copy()
    environment.update(
        {
            "CARGO": cargo,
            "MDTABLEFIX": mdtablefix,
            "MDLINT": str(md_lint),
            "MDLINT_STATUS": str(md_lint_status),
            "MDLINT_CAPTURE": str(checkout.parent / f"{target}.markdownlint.args"),
            "UV": cargo,
        }
    )
    return subprocess.run(
        [make, "--no-print-directory", "-f", str(MAKEFILE), target],
        cwd=checkout,
        env=environment,
        check=False,
        capture_output=True,
        text=True,
    )


def test_make_targets_select_untracked_and_skip_ignored_markdown(tmp_path: Path) -> None:
    """An untracked numbering defect fails check-fmt, fmt repairs it, and the
    generated target directory stays excluded.
    """
    mdtablefix = shutil.which("mdtablefix")
    assert mdtablefix is not None, "install pinned mdtablefix before this contract"
    version = subprocess.run(
        [mdtablefix, "--version"], check=True, capture_output=True, text=True
    )
    assert version.stdout.strip() == "mdtablefix 0.6.0", version.stdout
    markdownlint_stub = tmp_path / "markdownlint-cli2"
    markdownlint_stub.write_text(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$MDLINT_CAPTURE\"\n"
        "exit \"${MDLINT_STATUS:-0}\"\n",
        encoding="utf-8",
    )
    markdownlint_stub.chmod(0o755)

    checkout = tmp_path / "checkout"
    checkout.mkdir()
    shutil.copy2(ROOT / ".markdownlint-cli2.jsonc", checkout / ".markdownlint-cli2.jsonc")
    (checkout / ".gitignore").write_text("/target/\n", encoding="utf-8")
    subprocess.run(["git", "init", "--quiet", str(checkout)], check=True)
    subprocess.run(
        ["git", "-C", str(checkout), "config", "user.name", "Markdown contract"],
        check=True,
    )
    subprocess.run(
        ["git", "-C", str(checkout), "config", "user.email", "markdown-contract@example.invalid"],
        check=True,
    )
    subprocess.run(
        ["git", "-C", str(checkout), "add", ".gitignore", ".markdownlint-cli2.jsonc"],
        check=True,
    )
    subprocess.run(
        ["git", "-C", str(checkout), "commit", "--quiet", "-m", "Seed Markdown contract"],
        check=True,
    )

    defect = "1. First item\n3. Third item\n"
    untracked = checkout / "untracked.md"
    untracked.write_text(defect, encoding="utf-8")
    generated = checkout / "target" / "generated.md"
    generated.parent.mkdir()
    generated.write_text(defect, encoding="utf-8")
    ignored_check = subprocess.run(
        ["git", "-C", str(checkout), "check-ignore", "--quiet", "target/generated.md"],
        check=False,
    )
    assert ignored_check.returncode == 0, "generated Markdown fixture must be Git-ignored"

    before = _run_make("check-fmt", checkout, mdtablefix, markdownlint_stub)
    assert before.returncode != 0, (
        "check-fmt accepted a formatting defect in a non-ignored untracked Markdown file"
    )
    assert "untracked.md" in before.stdout + before.stderr, (
        "check-fmt failed without identifying the untracked Markdown defect"
    )
    assert untracked.read_text(encoding="utf-8") == defect, "check-fmt rewrote the untracked file"

    formatted = _run_make("fmt", checkout, mdtablefix, markdownlint_stub)
    assert formatted.returncode == 0, formatted.stdout + formatted.stderr
    assert untracked.read_text(encoding="utf-8") != defect, "fmt did not repair the untracked file"
    assert generated.read_text(encoding="utf-8") == defect, "fmt rewrote ignored generated Markdown"

    after = _run_make("check-fmt", checkout, mdtablefix, markdownlint_stub)
    assert after.returncode == 0, after.stdout + after.stderr

    markdownlint_failure = _run_make(
        "markdownlint", checkout, mdtablefix, markdownlint_stub, md_lint_status=23
    )
    assert markdownlint_failure.returncode != 0 and "Error 123" in (
        markdownlint_failure.stdout + markdownlint_failure.stderr
    ), "xargs masked the markdownlint-cli2 failure before Make"
    lint_arguments = (checkout.parent / "markdownlint.markdownlint.args").read_text(
        encoding="utf-8"
    )
    assert "generated.md" not in lint_arguments, (
        "make markdownlint passed ignored generated Markdown to markdownlint-cli2"
    )

    markdownlint_failure = _run_make(
        "fmt", checkout, mdtablefix, markdownlint_stub, md_lint_status=23
    )
    assert markdownlint_failure.returncode != 0 and "Error 23" in (
        markdownlint_failure.stdout + markdownlint_failure.stderr
    ), (
        "fmt masked the markdownlint-cli2 failure: "
        f"{markdownlint_failure.stdout}{markdownlint_failure.stderr}"
    )
