"""Check local build-tool provisioning before CI routes enter a Rust suite.

This helper owns only Statelet's workflow-to-build-tools contract. It uses
a generic strict workflow reader to preserve the complete job and step
inventory without duplicating the shared CV-005 coverage or token policy.
"""

import re
import shlex
from pathlib import Path
from typing import Any

import workflow_reading as reading

WORKFLOW_DIR = Path(__file__).resolve().parents[2] / ".github" / "workflows"
INSTALL_COMMAND = "make install-rust-toolchain"
INSTALL_STEP_RE = re.compile(r"make[ \t]+install-rust-toolchain")
COVERAGE_ACTION = "leynos/shared-actions/.github/actions/generate-coverage"
MUTATION_WORKFLOW = "leynos/shared-actions/.github/workflows/mutation-cargo.yml"
MUTATION_CALL_RE = re.compile(rf"^{re.escape(MUTATION_WORKFLOW)}@[0-9a-f]{{40}}$")
EXPECTED_COVERAGE_WORKFLOWS = {"ci.yml", "coverage-main.yml"}
MAKE_SUITE_TARGETS = {"all", "build", "coverage", "lint", "test", "typecheck"}
MAKE_NON_SUITE_TARGETS = {
    "audit", "check-build-tools", "check-fmt", "check-nextest", "fmt",
    "install-build-tools", "install-rust-toolchain", "install-mdtablefix", "markdownlint", "nixie",
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
        return _matrix_runner_platforms(job, value)
    return {_runner_platform(value)}


def _matrix_runner_platforms(job: dict[str, Any], expression: str) -> set[str]:
    """Resolve one supported matrix expression to its runner platforms."""
    match = re.fullmatch(r"\$\{\{\s*matrix\.([A-Za-z_][\w]*)\s*}}", expression)
    if not match:
        raise ContractError(f"has unsupported runner expression {expression!r}")
    strategy = job.get("strategy")
    matrix = strategy.get("matrix") if isinstance(strategy, dict) else None
    if not isinstance(matrix, dict) or any(
        matrix.get(key) for key in ("include", "exclude")
    ):
        raise ContractError("has an unsupported matrix runner shape")
    return _matrix_axis_platforms(matrix.get(match.group(1)))


def _matrix_axis_platforms(values: object) -> set[str]:
    """Classify scalar, mapping, or list values from a simple matrix axis."""
    match values:
        case list() if values:
            return {_runner_platform(item) for item in values}
        case str() | dict():
            return {_runner_platform(values)}
        case _:
            raise ContractError("has an unresolved matrix runner axis")


def _runner_platform(value: object) -> str:
    """Classify supported runner labels, rejecting unknown and mixed hosts."""
    labels = _runner_labels(value)
    text = " ".join(labels).lower()
    platforms = _matching_platforms(text)
    if len(platforms) != 1:
        raise ContractError(f"has unknown or mixed runner labels {value!r}")
    return next(iter(platforms))


def _runner_labels(value: object) -> list[str]:
    """Normalize GitHub's string, list, and group/labels runner forms."""
    if isinstance(value, dict):
        value = _runner_mapping_labels(value)
    return _validated_runner_labels(value)


def _runner_mapping_labels(value: dict[Any, Any]) -> object:
    """Extract valid labels from GitHub's group/labels runner mapping."""
    if set(value) - {"group", "labels"}:
        raise ContractError(f"has unsupported runner mapping {value!r}")
    if not isinstance(value.get("group"), str):
        raise ContractError(f"has unsupported runner mapping {value!r}")
    labels = value.get("labels")
    if not isinstance(labels, (str, list)):
        raise ContractError(f"has indeterminate runner labels {labels!r}")
    return labels


def _validated_runner_labels(value: object) -> list[str]:
    """Require a non-empty runner label string or string list."""
    labels = [value] if isinstance(value, str) else value
    if not isinstance(labels, list):
        raise ContractError(f"has indeterminate runner labels {value!r}")
    if not labels:
        raise ContractError(f"has indeterminate runner labels {value!r}")
    if not all(isinstance(label, str) for label in labels):
        raise ContractError(f"has indeterminate runner labels {value!r}")
    return labels


def _matching_platforms(text: str) -> set[str]:
    """Return platform names identified by recognized runner-label patterns."""
    return {
        platform
        for platform, pattern in (
            ("linux", r"linux|ubuntu"),
            ("windows", r"windows"),
            ("macos", r"macos|mac-os|darwin"),
        )
        if re.search(pattern, text)
    }



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
    """Reject duplicate linker ownership before checking provisioning order."""
    steps = reading.job_steps(job)
    if _has_full_build_tool_install(steps):
        return False
    return _has_ordered_component_install(steps, route_step)


def _has_full_build_tool_install(steps: list[dict[str, Any]]) -> bool:
    """Reject full local installation anywhere the shared action owns linkers."""
    return any(
        re.search(r"make[ \t]+install-build-tools\b", str(step.get("run", "")))
        for step in steps
    )


def _has_ordered_component_install(
    steps: list[dict[str, Any]], route_step: int
) -> bool:
    """Find binding component installation after linker setup and before use."""
    setup_seen = False
    for index, step in enumerate(steps):
        if index >= route_step:
            break
        if not _is_unconditional(step):
            continue
        if _is_shared_linker_setup(step):
            setup_seen = True
        if not setup_seen:
            continue
        if _is_component_install(step):
            return True
    return False


def _is_shared_linker_setup(step: dict[str, Any]) -> bool:
    """Recognize the pinned action only when both linker inputs are enabled."""
    pinned_action = re.fullmatch(
        r"leynos/shared-actions/\.github/actions/setup-rust@[0-9a-f]{40}",
        reading.uses(step),
    )
    if pinned_action is None:
        return False
    return _shared_linker_inputs(step)


def _is_component_install(step: dict[str, Any]) -> bool:
    """Recognize only a standalone component-only installation command."""
    run = step.get("run")
    if not isinstance(run, str):
        return False
    return INSTALL_STEP_RE.fullmatch(run.strip()) is not None


def _shared_linker_inputs(step: dict[str, Any]) -> bool:
    """Require the shared owner to provision both pinned linker inputs."""
    inputs = step.get("with")
    return isinstance(inputs, dict) and all(
        inputs.get(name) == "true" for name in ("install-mold", "install-clang-lld")
    )


def _mutation_setup_findings(
    workflow_name: str, job_id: str, job: dict[str, Any]
) -> list[str]:
    findings = []
    if "if" in job or job.get("continue-on-error") is True:
        findings.append(f"{workflow_name}:{job_id} may skip or soften its suite call")
    if not _shared_linker_inputs(job):
        findings.append(f"{workflow_name}:{job_id} must forward shared linker inputs")
    setup = _mutation_setup_commands(job)
    if not isinstance(setup, str):
        return findings + [f"{workflow_name}:{job_id} has no determinate caller setup"]
    findings.extend(_mutation_setup_order_findings(workflow_name, job_id, setup))
    if 'echo "$HOME/.local/bin" >> "$GITHUB_PATH"' not in setup:
        findings.append(f"{workflow_name}:{job_id} does not expose installed tools to the suite")
    return findings


def _mutation_setup_commands(job: dict[str, Any]) -> object:
    """Read the setup-commands input only from a mapping-shaped caller."""
    with_block = job.get("with")
    return with_block.get("setup-commands") if isinstance(with_block, dict) else None


def _mutation_setup_order_findings(
    workflow_name: str, job_id: str, setup: str
) -> list[str]:
    """Require only component provisioning after the shared linker setup."""
    expected = INSTALL_COMMAND + '\necho "$HOME/.local/bin" >> "$GITHUB_PATH"'
    if setup.strip() != expected:
        return [f"{workflow_name}:{job_id} must pass only binding component setup"]
    return []
