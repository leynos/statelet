"""Check local build-tool provisioning before CI routes enter a Rust suite.

This helper owns only Statelet's workflow-to-build-tools contract. It uses
the shared strict workflow reader so the provisioning check cannot see a
different trigger or workflow graph from the CV-005 contracts.
"""

from __future__ import annotations

import re
import shlex
from pathlib import Path
from typing import Any

import codescene_reading as reading

WORKFLOW_DIR = Path(__file__).resolve().parents[2] / ".github" / "workflows"
INSTALL_COMMAND = "make install-build-tools"
INSTALL_STEP_RE = re.compile(r"make[ \t]+install-build-tools")
COVERAGE_ACTION = "leynos/shared-actions/.github/actions/generate-coverage"
MUTATION_WORKFLOW = "leynos/shared-actions/.github/workflows/mutation-cargo.yml"
MUTATION_CALL_RE = re.compile(rf"^{re.escape(MUTATION_WORKFLOW)}@[0-9a-f]{{40}}$")
MUTATION_CALLER = ".github/workflows/dependabot-automerge.yml"
EXPECTED_COVERAGE_WORKFLOWS = {"ci.yml", "coverage-main.yml"}
MAKE_SUITE_TARGETS = {"all", "build", "coverage", "lint", "test", "typecheck"}
MAKE_NON_SUITE_TARGETS = {
    "audit", "check-build-tools", "check-fmt", "check-nextest", "fmt",
    "install-build-tools", "install-mdtablefix", "markdownlint", "nixie",
    "rust-audit", "spelling",
    "test-workflow-contracts",
}
CARGO_SUITE_COMMANDS = {
    "bench", "build", "check", "clippy", "doc", "llvm-cov", "mutants",
    "run", "test",
}
CARGO_NON_SUITE_COMMANDS = {
    "audit", "binstall", "clean", "fmt", "install", "metadata", "version",
}
CARGO_OPTIONS_WITH_VALUES = {
    "--color", "--config", "--exclude", "--features", "--manifest-path",
    "--package", "--profile", "--target", "-Z",
}


class ContractError(ValueError):
    """A workflow route cannot be classified safely by this contract."""


def load_workflows(directory: Path = WORKFLOW_DIR) -> dict[str, reading.Workflow]:
    """Load all workflow files with duplicate-key and trigger checks enabled."""
    return reading.workflows(directory)


def suite_findings(every: dict[str, reading.Workflow]) -> list[str]:
    """Return missing or unprovisioned Rust-suite routes in parsed workflows."""
    if not every:
        return ["no workflows were available to inspect"]

    findings = reading.missing_callees(every, set(every))
    coverage_workflows: set[str] = set()
    direct_commands: set[tuple[str, str, str]] = set()
    mutation_calls: list[tuple[str, str, dict[str, Any]]] = []
    suite_routes: list[tuple[str, str, dict[str, Any], int]] = []

    for workflow_name, workflow in sorted(every.items()):
        for job_id, job in reading.jobs(workflow):
            job_uses = job.get("uses")
            if isinstance(job_uses, str):
                kind, local_name = reading.classify_call(job_uses)
                if kind == reading.REFUSED:
                    findings.append(f"{workflow_name}:{job_id} has an unprovable local call")
                elif kind == reading.LOCAL and local_name not in every:
                    findings.append(
                        f"{workflow_name}:{job_id} calls missing workflow {local_name!r}"
                    )
                elif MUTATION_CALL_RE.fullmatch(job_uses):
                    mutation_calls.append((workflow_name, job_id, job))
                elif kind == reading.REMOTE and _workflow_reference(job_uses):
                    if _workflow_reference(job_uses) != MUTATION_CALLER:
                        findings.append(
                            f"{workflow_name}:{job_id} has an unresolved reusable call "
                            f"{job_uses!r}"
                        )
                elif kind == reading.REMOTE:
                    findings.append(
                        f"{workflow_name}:{job_id} has an unclassified reusable call "
                        f"{job_uses!r}"
                    )
                continue

            steps = reading.job_steps(job)
            for step_index, step in enumerate(steps):
                action = reading.uses(step)
                if action.split("@", maxsplit=1)[0].lower() == COVERAGE_ACTION:
                    coverage_workflows.add(workflow_name)
                    if not re.fullmatch(rf"{re.escape(COVERAGE_ACTION)}@[0-9a-f]{{40}}", action):
                        findings.append(
                            f"{workflow_name}:{job_id} has an unresolved coverage action pin"
                        )
                    suite_routes.append((workflow_name, job_id, job, step_index))
                run = step.get("run")
                if isinstance(run, str):
                    commands, errors = _run_routes(run)
                    findings.extend(
                        f"{workflow_name}:{job_id} {error}" for error in errors
                    )
                    direct_commands.update(
                        (workflow_name, job_id, command) for command, _ in commands
                    )
                    suite_routes.extend(
                        (workflow_name, job_id, job, step_index)
                        for _command, _position in commands
                    )

    for required in sorted(EXPECTED_COVERAGE_WORKFLOWS - coverage_workflows):
        findings.append(f"{required} has no recognized pinned coverage suite action")
    if not any(
        workflow == "ci.yml" and command == "make lint"
        for workflow, _, command in direct_commands
    ):
        findings.append("ci.yml has no direct Make lint route")
    if not any(
        workflow == "act-validation.yml" and command == "make test"
        for workflow, _, command in direct_commands
    ):
        findings.append("act-validation.yml has no direct Make test route")
    if len(mutation_calls) != 1:
        findings.append(
            "expected exactly one pinned mutation suite caller; "
            f"found {len(mutation_calls)}"
        )
    for workflow_name, job_id, job in mutation_calls:
        findings.extend(_mutation_setup_findings(workflow_name, job_id, job))
        suite_routes.append((workflow_name, job_id, job, -1))

    if not suite_routes:
        findings.append("no Rust suite routes were recognized")
    for workflow_name, job_id, job, step_index in suite_routes:
        if step_index < 0:  # The remote mutation caller supplies setup-commands.
            continue
        try:
            platforms = runner_platforms(job)
        except ContractError as error:
            findings.append(f"{workflow_name}:{job_id} {error}")
            continue
        if "linux" not in platforms:
            continue
        if not _has_prior_install(job, step_index):
            findings.append(
                f"{workflow_name}:{job_id} reaches a Linux suite before an "
                f"unconditional {INSTALL_COMMAND}"
            )

    return sorted(set(findings))


def runner_platforms(job: dict[str, Any]) -> set[str]:
    """Classify scalar, label-list, group/labels, and simple matrix runners.

    Unknown labels, unsupported expressions, and matrix include/exclude forms
    fail closed: a suite job must not pass on an unproved runner platform.
    """
    if "runs-on" not in job:
        raise ContractError("has no determinate runs-on value")
    value = job["runs-on"]
    if isinstance(value, str) and "${{" in value:
        match = re.fullmatch(r"\$\{\{\s*matrix\.([A-Za-z_][\w]*)\s*}}", value)
        if not match:
            raise ContractError(f"has unsupported runner expression {value!r}")
        strategy = job.get("strategy")
        matrix = strategy.get("matrix") if isinstance(strategy, dict) else None
        if not isinstance(matrix, dict) or any(matrix.get(key) for key in ("include", "exclude")):
            raise ContractError("has an unsupported matrix runner shape")
        values = matrix.get(match.group(1))
        if isinstance(values, list) and values:
            return {_runner_platform(item) for item in values}
        if isinstance(values, (str, dict)):
            return {_runner_platform(values)}
        raise ContractError("has an unresolved matrix runner axis")
    return {_runner_platform(value)}


def _runner_platform(value: object) -> str:
    if isinstance(value, dict):
        if set(value) - {"group", "labels"} or not isinstance(value.get("group"), str):
            raise ContractError(f"has unsupported runner mapping {value!r}")
        labels = value.get("labels")
        if not isinstance(labels, (str, list)):
            raise ContractError(f"has indeterminate runner labels {labels!r}")
        value = labels
    labels = [value] if isinstance(value, str) else value
    if not isinstance(labels, list) or not labels or not all(
        isinstance(label, str) for label in labels
    ):
        raise ContractError(f"has indeterminate runner labels {value!r}")
    text = " ".join(labels).lower()
    platforms = {
        platform
        for platform, pattern in (
            ("linux", r"linux|ubuntu"),
            ("windows", r"windows"),
            ("macos", r"macos|mac-os|darwin"),
        )
        if re.search(pattern, text)
    }
    if len(platforms) != 1:
        raise ContractError(f"has unknown or mixed runner labels {value!r}")
    return next(iter(platforms))


def _workflow_reference(reference: str) -> str:
    match = re.fullmatch(r"[^/]+/[^/]+/(\.github/workflows/[^@]+)@[^@]+", reference)
    return match.group(1) if match else ""


def _run_routes(run: str) -> tuple[list[tuple[str, int]], list[str]]:
    """Find recognized Make/Cargo compile routes and refuse unknown commands."""
    flattened = run.replace("\\\n", " ")
    routes: list[tuple[str, int]] = []
    errors: list[str] = []
    try:
        words = _shell_words(flattened)
    except ValueError:
        return [], ["has an unreadable shell command"]
    for index, word in enumerate(words):
        name = Path(word).name
        if name == "make":
            target_index = _make_target_index(words, index)
            target = words[target_index] if target_index < len(words) else ""
            if target in MAKE_SUITE_TARGETS:
                routes.append((f"make {target}", index))
            elif target not in MAKE_NON_SUITE_TARGETS:
                errors.append(f"has an unresolved Make target {target!r}")
        elif name == "cargo":
            command_index = index + 1
            if command_index < len(words) and words[command_index].startswith("+"):
                command_index += 1
            while command_index < len(words) and words[command_index].startswith("-"):
                command_index += (
                    2 if words[command_index] in CARGO_OPTIONS_WITH_VALUES else 1
                )
            command = words[command_index] if command_index < len(words) else ""
            if command == "nextest":
                next_command = words[command_index + 1] if command_index + 1 < len(words) else ""
                if next_command == "run":
                    routes.append(("cargo nextest run", index))
                elif next_command != "--version":
                    errors.append(f"has an unresolved cargo nextest command {next_command!r}")
            elif command in CARGO_SUITE_COMMANDS:
                routes.append((f"cargo {command}", index))
            elif command not in CARGO_NON_SUITE_COMMANDS:
                errors.append(f"has an unresolved Cargo command {command!r}")
        elif name == "cargo-mutants":
            routes.append(("cargo-mutants", index))
    return routes, errors


def _shell_words(command: str) -> list[str]:
    """Tokenize a complete shell block before inspecting commands.

    Splitting a script at semicolons first breaks quoted strings in compound
    ``if`` blocks and can conceal a Cargo invocation. Let shlex preserve those
    strings while exposing command separators as tokens.
    """
    lexer = shlex.shlex(command, posix=True, punctuation_chars=";&|")
    lexer.whitespace_split = True
    lexer.commenters = "#"
    return list(lexer)


def _make_target_index(words: list[str], command_index: int) -> int:
    target_index = command_index + 1
    while target_index < len(words):
        item = words[target_index]
        if item in {"-C", "-f", "-j", "--directory", "--file", "--jobs"}:
            target_index += 2
        elif item.startswith("-") or "=" in item:
            target_index += 1
        else:
            break
    return target_index


def _is_unconditional(step: dict[str, Any]) -> bool:
    if "if" in step:
        return False
    soft_fail = step.get("continue-on-error", False)
    return soft_fail is False or (isinstance(soft_fail, str) and soft_fail.lower() == "false")


def _has_prior_install(job: dict[str, Any], route_step: int) -> bool:
    for index, step in enumerate(reading.job_steps(job)):
        run = step.get("run")
        if not isinstance(run, str) or not _is_unconditional(step):
            continue
        if not INSTALL_STEP_RE.fullmatch(run.strip()):
            continue
        if index < route_step:
            return True
    return False


def _mutation_setup_findings(
    workflow_name: str, job_id: str, job: dict[str, Any]
) -> list[str]:
    findings = []
    if "if" in job or job.get("continue-on-error") is True:
        findings.append(f"{workflow_name}:{job_id} may skip or soften its suite call")
    with_block = job.get("with")
    setup = with_block.get("setup-commands") if isinstance(with_block, dict) else None
    if not isinstance(setup, str):
        return findings + [f"{workflow_name}:{job_id} has no determinate caller setup"]
    install = setup.find(INSTALL_COMMAND)
    if setup.count(INSTALL_COMMAND) != 1 or install < 0:
        findings.append(f"{workflow_name}:{job_id} must pass one {INSTALL_COMMAND}")
    compiler = setup.find("apt-get install")
    if compiler < 0 or compiler > install:
        findings.append(
            f"{workflow_name}:{job_id} must install Linux compiler tools before build tools"
        )
    if 'echo "$HOME/.local/bin" >> "$GITHUB_PATH"' not in setup:
        findings.append(f"{workflow_name}:{job_id} does not expose installed tools to the suite")
    return findings
