"""Hold Statelet's Markdown formatter and CI routes to the selected baseline."""

import copy
import json
import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest

from markdown_ci_contract import (
    INSTALL_ACTION,
    MARKDOWNLINT_ACTION,
    MAKEFILE,
    ROOT,
    _ci_route,
    _current_ci,
    _markdown_ci_problems,
    _step_index,
)


@dataclass(frozen=True, slots=True)
class MarkdownTools:
    """Executables supplied to a Make formatter contract run."""

    mdtablefix: str
    markdownlint: Path


def test_ci_uses_binding_markdown_tools_in_order() -> None:
    """CI provisions mdtablefix before the check and runs the pinned linter."""
    problems = _markdown_ci_problems(_current_ci())
    assert not problems, f"unexpected Markdown CI route: {problems}"


def test_local_markdown_tools_match_the_pinned_ci_versions() -> None:
    """Make pins local Markdown tools to the versions used by CI."""
    makefile = MAKEFILE.read_text(encoding="utf-8")
    assert re.search(r"(?m)^MARKDOWNLINT_VERSION \?= 0\.23\.2$", makefile), (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )
    assert re.search(r"(?m)^MDLINT \?= markdownlint-cli2$", makefile), (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )
    assert re.search(
        r'(?ms)^install-markdownlint:.*?\n\tnpm install --global --prefix '
        r'"\$\(BUILD_TOOLS_PREFIX\)" "markdownlint-cli2@'
        r'\$\(MARKDOWNLINT_VERSION\)"$',
        makefile,
    ), (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )
    assert re.search(r"(?m)^MDTABLEFIX_VERSION \?= 0\.6\.0$", makefile), (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )
    assert re.search(
        r"(?ms)^install-mdtablefix:.*?\n\t\$\(CARGO\) install --locked "
        r"--version \$\(MDTABLEFIX_VERSION\) mdtablefix$",
        makefile,
    ), (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )
    markdown_config = json.loads(
        (ROOT / ".markdownlint-cli2.jsonc").read_text("utf-8")
    )
    assert "**/target/**" in markdown_config["ignores"], (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )
    assert "**/.pytest_cache/**" in markdown_config["ignores"], (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )
    assert "git ls-files --cached --others --exclude-standard -z" in makefile, (
        "test_local_markdown_tools_match_the_pinned_ci_versions contract failed"
    )


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
    assert job is not None, (
        "test_ci_mutations_are_rejected contract failed"
    )
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

    match mutation:
        case "remove installer":
            steps.pop(install_index)
        case "move installer after check":
            steps.insert(format_index + 1, steps.pop(install_index))
        case "condition installer":
            steps[install_index]["if"] = "false"
        case "soft-fail formatter":
            steps[format_index]["continue-on-error"] = True
        case "string soft-fail formatter":
            steps[format_index]["continue-on-error"] = "true"
        case "narrow globs":
            steps[action_index]["with"]["globs"] = "docs/*.md"
        case "use mutable action tag":
            steps[action_index]["uses"] = f"{MARKDOWNLINT_ACTION}@v24.2.0"
        case "skip markdown action":
            steps[action_index]["if"] = "false"
        case _:
            raise AssertionError(f"unrecognized test mutation {mutation!r}")

    problems = _markdown_ci_problems(document)
    assert any(expected in problem for problem in problems), (
        f"mutation {mutation!r} escaped its intended check: {problems}"
    )


def _run_make(
    target: str,
    checkout: Path,
    tools: MarkdownTools,
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
            "MDTABLEFIX": tools.mdtablefix,
            "MDLINT": str(tools.markdownlint),
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
    tools = MarkdownTools(mdtablefix, markdownlint_stub)

    checkout = tmp_path / "checkout"
    checkout.mkdir()
    shutil.copy2(ROOT / ".markdownlint-cli2.jsonc", checkout / ".markdownlint-cli2.jsonc")
    (checkout / ".gitignore").write_text(
        "/target/\n/.pytest_cache/\n", encoding="utf-8"
    )
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
    pytest_cache = checkout / ".pytest_cache" / "README.md"
    pytest_cache.parent.mkdir()
    pytest_cache.write_text(defect, encoding="utf-8")
    for relative_path in ("target/generated.md", ".pytest_cache/README.md"):
        ignored_check = subprocess.run(
            ["git", "-C", str(checkout), "check-ignore", "--quiet", relative_path],
            check=False,
        )
        assert ignored_check.returncode == 0, (
            f"generated Markdown fixture must be Git-ignored: {relative_path}"
        )

    before = _run_make("check-fmt", checkout, tools)
    assert before.returncode != 0, (
        "check-fmt accepted a formatting defect in a non-ignored untracked Markdown file"
    )
    assert "untracked.md" in before.stdout + before.stderr, (
        "check-fmt failed without identifying the untracked Markdown defect"
    )
    assert untracked.read_text(encoding="utf-8") == defect, "check-fmt rewrote the untracked file"

    formatted = _run_make("fmt", checkout, tools)
    assert formatted.returncode == 0, formatted.stdout + formatted.stderr
    assert untracked.read_text(encoding="utf-8") != defect, "fmt did not repair the untracked file"
    assert generated.read_text(encoding="utf-8") == defect, "fmt rewrote ignored generated Markdown"
    assert pytest_cache.read_text(encoding="utf-8") == defect, (
        "fmt rewrote ignored pytest-cache Markdown"
    )

    after = _run_make("check-fmt", checkout, tools)
    assert after.returncode == 0, after.stdout + after.stderr

    markdownlint_failure = _run_make(
        "markdownlint", checkout, tools, md_lint_status=23
    )
    assert markdownlint_failure.returncode != 0 and "Error 123" in (
        markdownlint_failure.stdout + markdownlint_failure.stderr
    ), "xargs masked the markdownlint-cli2 failure before Make"
    lint_arguments = (checkout.parent / "markdownlint.markdownlint.args").read_text(
        encoding="utf-8"
    )
    assert "generated.md" not in lint_arguments and "pytest_cache" not in lint_arguments, (
        "make markdownlint passed ignored generated Markdown to markdownlint-cli2"
    )

    markdownlint_failure = _run_make(
        "fmt", checkout, tools, md_lint_status=23
    )
    assert markdownlint_failure.returncode != 0 and "Error 23" in (
        markdownlint_failure.stdout + markdownlint_failure.stderr
    ), (
        "fmt masked the markdownlint-cli2 failure: "
        f"{markdownlint_failure.stdout}{markdownlint_failure.stderr}"
    )
