"""Read and validate Statelet's binding Whitaker consumer contracts."""

import re
from pathlib import Path

from suite_provisioning import load_workflows

ROOT = Path(__file__).resolve().parents[2]
INSTALL_ACTION = "leynos/shared-actions/.github/actions/install-whitaker"
INSTALL_PIN = "5bc2b2611f5921ef2c56e4ee3fde2b879a319361"


def _ci() -> dict[object, object]:
    workflows = load_workflows()
    assert "ci.yml" in workflows, (
        "_ci contract failed"
    )
    return workflows["ci.yml"]


def _is_lint(step: dict[object, object]) -> bool:
    run = step.get("run")
    return isinstance(run, str) and run.strip() == "make lint"


def _key(value: object) -> str:
    return str(value).strip().lower().replace("_", "-")


def _workflow_problems(workflow: dict[object, object]) -> list[str]:
    """Check action provenance, job ordering, reachable inputs, and failure binding."""
    jobs = workflow.get("jobs")
    if not isinstance(jobs, dict):
        return ["CI workflow has no readable jobs"]
    candidates = _lint_jobs(jobs)
    if len(candidates) != 1:
        return [f"expected one CI job running direct make lint, found {len(candidates)}"]
    job_name, job, steps = candidates[0]
    return sorted(set(_lint_job_findings(job_name, job, steps)))


def _lint_jobs(
    jobs: dict[object, object],
) -> list[tuple[str, dict[object, object], list[dict[object, object]]]]:
    """Find jobs with readable step mappings and one direct Make lint route."""
    candidates = []
    for name, job in jobs.items():
        if not isinstance(job, dict) or not isinstance(job.get("steps"), list):
            continue
        steps = job["steps"]
        if all(isinstance(step, dict) for step in steps) and any(map(_is_lint, steps)):
            candidates.append((str(name), job, steps))
    return candidates


def _lint_job_findings(
    job_name: str,
    job: dict[object, object],
    steps: list[dict[object, object]],
) -> list[str]:
    """Validate the job-level binding, action sequence, and direct lint step."""
    problems: list[str] = []
    if "if" in job or "continue-on-error" in job:
        problems.append(f"CI job {job_name!r} can skip or soften lint")
    installs = _installer_indices(steps)
    lints = [index for index, step in enumerate(steps) if _is_lint(step)]
    if len(installs) != 1:
        problems.append(f"expected one shared Whitaker installer action, found {len(installs)}")
    if len(lints) != 1:
        problems.append(f"expected one direct make lint step, found {len(lints)}")
    if len(installs) == 1:
        problems.extend(_installer_findings(steps, installs[0], lints))
    problems.extend(_lint_step_skip_findings(steps, lints))
    problems.extend(_provisioning_step_findings(steps))
    return problems


def _lint_step_skip_findings(
    steps: list[dict[object, object]], lints: list[int]
) -> list[str]:
    """Reject a single direct lint step that can be skipped or softened."""
    if len(lints) != 1:
        return []
    if not _step_can_skip(steps[lints[0]]):
        return []
    return ["direct make lint can be skipped or allowed to fail"]


def _installer_indices(steps: list[dict[object, object]]) -> list[int]:
    """Return steps invoking the selected shared installer action."""
    return [
        index
        for index, step in enumerate(steps)
        if (uses := step.get("uses"))
        and isinstance(uses, str)
        and uses.split("@", 1)[0].lower() == INSTALL_ACTION
    ]


def _installer_findings(
    steps: list[dict[object, object]], installer_index: int, lint_indices: list[int]
) -> list[str]:
    """Validate one installer pin, input set, environment, and ordering."""
    installer = steps[installer_index]
    findings = []
    if installer.get("uses") != f"{INSTALL_ACTION}@{INSTALL_PIN}":
        findings.append("Whitaker installer is not pinned to the approved full SHA")
    if len(lint_indices) == 1 and installer_index >= lint_indices[0]:
        findings.append("Whitaker installation does not precede direct make lint")
    if _step_can_skip(installer):
        findings.append("Whitaker installer can be skipped or allowed to fail")
    findings.extend(_installer_input_findings(installer))
    findings.extend(_installer_environment_findings(installer))
    return findings


def _installer_input_findings(installer: dict[object, object]) -> list[str]:
    """Require only Cranelift while retaining the rolling suite and pinned installer."""
    inputs = installer.get("with", {})
    if not isinstance(inputs, dict):
        return ["Whitaker installer inputs are not a mapping"]
    names = {_key(name) for name in inputs}
    missing_cranelift = inputs.get("cranelift") not in (True, "true")
    unsupported_inputs = names != {"cranelift"}
    rolling_pin = bool(names & {"suite-version", "allow-suite-pin"})
    installer_override = any("installer" in name and "version" in name for name in names)
    checks = (
        (missing_cranelift, "Whitaker installer must receive cranelift: 'true'"),
        (unsupported_inputs, "Whitaker installer has an unsupported input or version override"),
        (rolling_pin, "rolling Whitaker suite pinning is forbidden"),
        (installer_override, "Whitaker installer version override is forbidden"),
    )
    return [message for failed, message in checks if failed]


def _installer_environment_findings(installer: dict[object, object]) -> list[str]:
    """Reject installer-specific RUSTFLAGS that bypass its managed toolchain."""
    installer_env = installer.get("env", {})
    if not isinstance(installer_env, dict):
        return ["Whitaker installer environment must be a mapping"]
    if installer_env.get("RUSTFLAGS", "") != "":
        return ["non-empty repository RUSTFLAGS are injected into the Whitaker installer"]
    return []


def _step_can_skip(step: dict[object, object]) -> bool:
    """Return whether a workflow step is conditional or configured to fail softly."""
    return "if" in step or "continue-on-error" in step


def _provisioning_step_findings(steps: list[dict[object, object]]) -> list[str]:
    """Reject duplicate caches, shims, and ad hoc Whitaker installers."""
    findings = []
    for step in steps:
        findings.extend(_step_provisioning_findings(step))
    return findings


def _step_provisioning_findings(step: dict[object, object]) -> list[str]:
    """Check one workflow step for competing Whitaker provisioning."""
    uses = str(step.get("uses", "")).lower()
    run = step.get("run")
    findings = []
    if _uses_competing_provisioner(uses):
        findings.append("Whitaker provisioning uses a second action or shim")
    if _runs_manual_provisioner(run):
        findings.append("Whitaker has an ad hoc installer, cache, or PATH shim")
    findings.extend(_duplicate_cache_findings(step, uses))
    return findings


def _uses_competing_provisioner(uses: str) -> bool:
    """Detect a Whitaker-related action that is not the approved installer."""
    is_whitaker_action = "whitaker" in uses or "dylint" in uses
    return is_whitaker_action and uses.split("@", 1)[0] != INSTALL_ACTION


def _runs_manual_provisioner(run: object) -> bool:
    """Detect a Whitaker command outside the direct lint target."""
    if not isinstance(run, str):
        return False
    if re.search(r"whitaker|dylint", run, re.IGNORECASE) is None:
        return False
    return run.strip() != "make lint"


def _duplicate_cache_findings(step: dict[object, object], uses: str) -> list[str]:
    """Reject a separate cache that duplicates the install action's cache."""
    inputs = step.get("with")
    if not isinstance(inputs, dict):
        return []
    cache = " ".join(str(inputs.get(key, "")) for key in ("key", "path", "restore-keys"))
    name = str(step.get("name", "")).lower()
    if _is_duplicate_whitaker_cache(uses, name, cache):
        return ["consumer duplicates the Whitaker action's cache"]
    return []


def _is_duplicate_whitaker_cache(uses: str, name: str, cache: str) -> bool:
    """Recognize cache actions whose key or path duplicates Whitaker storage."""
    is_cache_step = "cache" in uses or "cache" in name
    has_whitaker_cache_path = re.search(r"whitaker|dylint", cache, re.I)
    return bool(is_cache_step and has_whitaker_cache_path)


def _rules(text: str) -> dict[str, tuple[list[str], list[str]]]:
    """Collect target prerequisites and simple continued Make recipes."""
    lines, result, index = text.splitlines(), {}, 0
    while index < len(lines):
        parsed = _rule_header(lines[index])
        if parsed is None:
            index += 1
            continue
        names, prerequisites = parsed
        recipes, index = _recipe_lines(lines, index + 1)
        for name in names:
            old = result.get(name, ([], []))
            result[name] = (old[0] + prerequisites, old[1] + recipes)
    return result


def _rule_header(line: str) -> tuple[list[str], list[str]] | None:
    """Return target names and prerequisites for a simple Make rule line."""
    if line.startswith(("\t", " ")):
        return None
    if ":" not in line:
        return None
    if line.startswith("."):
        return None
    header, dependencies = line.split(":", 1)
    names = header.split()
    if not names:
        return None
    if any("=" in name for name in names):
        return None
    prerequisites = dependencies.split("#", 1)[0].split("|", 1)[0].split()
    return names, prerequisites


def _recipe_lines(lines: list[str], index: int) -> tuple[list[str], int]:
    """Collect tab-indented recipes, preserving the parser's continuation rule."""
    recipes = []
    while index < len(lines) and lines[index].startswith("\t"):
        command, index = _continued_recipe(lines, index)
        recipes.append(command)
    return recipes, index


def _continued_recipe(lines: list[str], index: int) -> tuple[str, int]:
    """Join continuation lines consumed by one Make recipe."""
    command = lines[index].strip()
    while command.endswith("\\") and index + 1 < len(lines):
        index += 1
        command = f"{command[:-1]} {lines[index].strip()}"
    return command, index + 1


def _make_problems(text: str) -> list[str]:
    """Check package scope, the Make -j sequence, and direct failure propagation."""
    assignments = _whitaker_assignments(text)
    targets = _rules(text)
    problems = _whitaker_variable_findings(assignments)
    problems.extend(_lint_order_findings(targets))
    problems.extend(_whitaker_leaf_findings(targets))
    return sorted(set(problems))


def _whitaker_assignments(text: str) -> dict[str, list[str]]:
    """Read literal Make assignments for the Whitaker executable and scope."""
    assignments: dict[str, list[str]] = {}
    for line in text.splitlines():
        match = re.match(r"^\s*(WHITAKER(?:_[A-Z0-9_]+)?)\s*(?:\?=|:=|=)\s*(.*?)\s*$", line)
        if match:
            assignments.setdefault(match[1], []).append(match[2])
    return assignments


def _whitaker_variable_findings(assignments: dict[str, list[str]]) -> list[str]:
    """Require the real Whitaker executable and all workspace packages."""
    problems = []
    if assignments.get("WHITAKER") != ["whitaker"]:
        problems.append("Make must default WHITAKER to the whitaker executable")
    package_values = assignments.get("WHITAKER_PACKAGES", [])
    if not any("--workspace" in value.split() for value in package_values):
        problems.append("WHITAKER_PACKAGES does not select every workspace package")

    return problems


def _lint_order_findings(
    targets: dict[str, tuple[list[str], list[str]]],
) -> list[str]:
    """Require the composite lint recipe to run Clippy before Whitaker."""
    problems = []
    if "lint" not in targets:
        problems.append("Make has no lint composite target")
    else:
        prerequisites, recipes = targets["lint"]
        if {"lint-clippy", "lint-whitaker"} & set(prerequisites):
            problems.append("lint gates are unordered prerequisites under make -j")
        problems.extend(_sequential_lint_recipe_findings(recipes))

    return problems


def _sequential_lint_recipe_findings(recipes: list[str]) -> list[str]:
    """Require one direct recursive Make call for each ordered lint leaf."""
    clippy = _lint_recipe_indices(recipes, "lint-clippy")
    whitaker = _lint_recipe_indices(recipes, "lint-whitaker")
    if len(clippy) != 1 or len(whitaker) != 1:
        return ["lint must invoke Clippy then Whitaker as sequential Make recipes"]
    if clippy[0] >= whitaker[0]:
        return ["composite lint does not run Clippy before Whitaker"]
    return []


def _lint_recipe_indices(recipes: list[str], target: str) -> list[int]:
    """Locate direct recursive Make recipes for one known lint leaf."""
    pattern = rf"\+\$\(MAKE\)(?:\s+--\S+)*\s+{re.escape(target)}"
    return [index for index, line in enumerate(recipes) if re.fullmatch(pattern, line)]


def _whitaker_leaf_findings(
    targets: dict[str, tuple[list[str], list[str]]],
) -> list[str]:
    """Validate the direct leaf command and ensure its failure reaches Make."""
    leaf = targets.get("lint-whitaker")
    if leaf is None:
        return ["Make has no lint-whitaker leaf target"]
    return _whitaker_recipe_findings(leaf[1])


def _whitaker_recipe_findings(recipes: list[str]) -> list[str]:
    """Require one direct Whitaker command with binding failure semantics."""
    calls = _whitaker_calls(recipes)
    if len(calls) != 1:
        return ["lint-whitaker must run exactly one direct Whitaker command"]
    return _whitaker_command_findings(*calls[0])


def _whitaker_calls(recipes: list[str]) -> list[tuple[bool, str]]:
    """Return Whitaker recipes with Make's ignored-error prefix recorded."""
    calls = []
    for recipe in recipes:
        if "$(WHITAKER)" not in recipe:
            continue
        command = recipe.lstrip()
        ignored = False
        while command and command[0] in "@+-":
            ignored |= command[0] == "-"
            command = command[1:].lstrip()
        calls.append((ignored, command))
    return calls


def _whitaker_command_findings(ignored: bool, command: str) -> list[str]:
    """Check environment, shell failure handling, and workspace scope."""
    clears_rustflags = command.startswith("RUSTFLAGS= $(WHITAKER) ")
    problems = _whitaker_rustflags_findings(command, clears_rustflags)
    if ignored or re.search(r"\|\||\||;|&&", command):
        problems.append("lint-whitaker can mask or ignore a Whitaker failure")
    words = command.removeprefix("RUSTFLAGS= ").split()
    problems.extend(_whitaker_scope_findings(words))
    return problems


def _whitaker_rustflags_findings(command: str, clears_rustflags: bool) -> list[str]:
    """Require the tool to receive empty inherited RUSTFLAGS."""
    problems = []
    if "RUSTFLAGS" in command and not clears_rustflags:
        problems.append("lint-whitaker injects repository RUSTFLAGS")
    if not clears_rustflags:
        problems.append("lint-whitaker does not clear inherited RUSTFLAGS")
    return problems


def _whitaker_scope_findings(words: list[str]) -> list[str]:
    """Require package, argument-boundary, and Cargo-flag forwarding."""
    problems = []
    if not _passes_workspace_scope(words):
        problems.append("lint-whitaker does not pass the workspace package scope")
    if "$(CARGO_FLAGS)" not in words:
        problems.append("lint-whitaker does not preserve the configured Cargo flags")
    return problems


def _passes_workspace_scope(words: list[str]) -> bool:
    """Check the executable, workspace packages, and CLI boundary as a unit."""
    return bool(
        words
        and words[0] == "$(WHITAKER)"
        and "$(WHITAKER_PACKAGES)" in words
        and "--" in words
    )


def _makefile() -> str:
    return (ROOT / "Makefile").read_text(encoding="utf-8")
