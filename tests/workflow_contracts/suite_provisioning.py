"""Check local build-tool provisioning before CI routes enter a Rust suite.

This helper owns only Statelet's workflow-to-build-tools contract. It uses
the shared strict workflow reader so the provisioning check cannot see a
different trigger or workflow graph from the CV-005 contracts.
"""

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
        if not isinstance(matrix, dict) or any(
            matrix.get(key) for key in ("include", "exclude")
        ):
            raise ContractError("has an unsupported matrix runner shape")
        values = matrix.get(match.group(1))
        match values:
            case list() if values:
                return {_runner_platform(item) for item in values}
            case str() | dict():
                return {_runner_platform(values)}
            case _:
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
        route, error = _route_for_word(name, words, index)
        if route is not None:
            routes.append((route, index))
        if error is not None:
            errors.append(error)
    return routes, errors


def _route_for_word(
    name: str, words: list[str], index: int
) -> tuple[str | None, str | None]:
    """Classify one shell word when it names a supported suite command."""
    match name:
        case "make":
            return _make_route(words, index)
        case "cargo":
            return _cargo_route(words, index)
        case "cargo-mutants":
            return "cargo-mutants", None
        case _:
            return None, None


def _make_route(words: list[str], index: int) -> tuple[str | None, str | None]:
    target_index = _make_target_index(words, index)
    target = words[target_index] if target_index < len(words) else ""
    if target in MAKE_SUITE_TARGETS:
        return f"make {target}", None
    if target not in MAKE_NON_SUITE_TARGETS:
        return None, f"has an unresolved Make target {target!r}"
    return None, None


def _cargo_route(words: list[str], index: int) -> tuple[str | None, str | None]:
    """Classify a Cargo command and report unknown subcommands."""
    command_index = _cargo_command_index(words, index)
    command = words[command_index] if command_index < len(words) else ""
    match command:
        case "nextest":
            return _nextest_route(words, command_index)
        case suite_command if suite_command in CARGO_SUITE_COMMANDS:
            return f"cargo {suite_command}", None
        case known_non_suite if known_non_suite in CARGO_NON_SUITE_COMMANDS:
            return None, None
        case _:
            return None, f"has an unresolved Cargo command {command!r}"


def _cargo_command_index(words: list[str], index: int) -> int:
    command_index = index + 1
    if command_index < len(words) and words[command_index].startswith("+"):
        command_index += 1
    while command_index < len(words) and words[command_index].startswith("-"):
        command_index += (
            2 if words[command_index] in CARGO_OPTIONS_WITH_VALUES else 1
        )
    return command_index


def _nextest_route(
    words: list[str], command_index: int
) -> tuple[str | None, str | None]:
    """Count Nextest's run command but ignore its version probe."""
    next_index = command_index + 1
    next_command = words[next_index] if next_index < len(words) else ""
    match next_command:
        case "run":
            return "cargo nextest run", None
        case "--version":
            return None, None
        case _:
            return None, f"has an unresolved cargo nextest command {next_command!r}"


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
