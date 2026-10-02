"""Read and validate Statelet's binding Whitaker consumer contracts."""

import re
from pathlib import Path

from suite_provisioning import load_workflows

ROOT = Path(__file__).resolve().parents[2]
INSTALL_ACTION = "leynos/shared-actions/.github/actions/install-whitaker"
INSTALL_PIN = "6dea5677a84fec60ca51b07202570e3af12ffdb4"


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
    if len(lints) == 1 and _step_can_skip(steps[lints[0]]):
        problems.append("direct make lint can be skipped or allowed to fail")
    problems.extend(_provisioning_step_findings(steps))
    return problems


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
    findings = []
    if inputs.get("cranelift") not in (True, "true"):
        findings.append("Whitaker installer must receive cranelift: 'true'")
    if names != {"cranelift"}:
        findings.append("Whitaker installer has an unsupported input or version override")
    if names & {"suite-version", "allow-suite-pin"}:
        findings.append("rolling Whitaker suite pinning is forbidden")
    if any("installer" in name and "version" in name for name in names):
        findings.append("Whitaker installer version override is forbidden")
    return findings


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
    if ("whitaker" in uses or "dylint" in uses) and uses.split("@", 1)[0] != INSTALL_ACTION:
        findings.append("Whitaker provisioning uses a second action or shim")
    if isinstance(run, str) and re.search(r"whitaker|dylint", run, re.IGNORECASE):
        if run.strip() != "make lint":
            findings.append("Whitaker has an ad hoc installer, cache, or PATH shim")
    findings.extend(_duplicate_cache_findings(step, uses))
    return findings


def _duplicate_cache_findings(step: dict[object, object], uses: str) -> list[str]:
    """Reject a separate cache that duplicates the install action's cache."""
    inputs = step.get("with")
    if not isinstance(inputs, dict):
        return []
    cache = " ".join(str(inputs.get(key, "")) for key in ("key", "path", "restore-keys"))
    name = str(step.get("name", "")).lower()
    if ("cache" in uses or "cache" in name) and re.search(r"whitaker|dylint", cache, re.I):
        return ["consumer duplicates the Whitaker action's cache"]
    return []


def _rules(text: str) -> dict[str, tuple[list[str], list[str]]]:
    """Collect target prerequisites and simple continued Make recipes."""
    lines, result, index = text.splitlines(), {}, 0
    while index < len(lines):
        line = lines[index]
        if line.startswith(("\t", " ")) or ":" not in line or line.startswith("."):
            index += 1
            continue
        header, dependencies = line.split(":", 1)
        names = header.split()
        if not names or any("=" in name for name in names):
            index += 1
            continue
        prerequisites = dependencies.split("#", 1)[0].split("|", 1)[0].split()
        recipes = []
        index += 1
        while index < len(lines) and lines[index].startswith("\t"):
            command = lines[index].strip()
            while command.endswith("\\") and index + 1 < len(lines):
                index += 1
                command = f"{command[:-1]} {lines[index].strip()}"
            recipes.append(command)
            index += 1
        for name in names:
            old = result.get(name, ([], []))
            result[name] = (old[0] + prerequisites, old[1] + recipes)
    return result


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
        clippy = [
            i for i, line in enumerate(recipes)
            if re.fullmatch(r"\+\$\(MAKE\)(?:\s+--\S+)*\s+lint-clippy", line)
        ]
        whitaker = [
            i for i, line in enumerate(recipes)
            if re.fullmatch(r"\+\$\(MAKE\)(?:\s+--\S+)*\s+lint-whitaker", line)
        ]
        if len(clippy) != 1 or len(whitaker) != 1:
            problems.append("lint must invoke Clippy then Whitaker as sequential Make recipes")
        elif clippy[0] >= whitaker[0]:
            problems.append("composite lint does not run Clippy before Whitaker")

    return problems


def _whitaker_leaf_findings(
    targets: dict[str, tuple[list[str], list[str]]],
) -> list[str]:
    """Validate the direct leaf command and ensure its failure reaches Make."""
    problems = []
    leaf = targets.get("lint-whitaker")
    if leaf is None:
        problems.append("Make has no lint-whitaker leaf target")
    else:
        _, recipes = leaf
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
        if len(calls) != 1:
            problems.append("lint-whitaker must run exactly one direct Whitaker command")
        else:
            ignored, command = calls[0]
            clears_rustflags = command.startswith("RUSTFLAGS= $(WHITAKER) ")
            if "RUSTFLAGS" in command and not clears_rustflags:
                problems.append("lint-whitaker injects repository RUSTFLAGS")
            if not clears_rustflags:
                problems.append("lint-whitaker does not clear inherited RUSTFLAGS")
            words = command.removeprefix("RUSTFLAGS= ").split()
            if ignored or re.search(r"\|\||\||;|&&", command):
                problems.append("lint-whitaker can mask or ignore a Whitaker failure")
            if (
                not words or words[0] != "$(WHITAKER)"
                or "$(WHITAKER_PACKAGES)" not in words or "--" not in words
            ):
                problems.append("lint-whitaker does not pass the workspace package scope")
            if "$(CARGO_FLAGS)" not in words:
                problems.append("lint-whitaker does not preserve the configured Cargo flags")
    return problems


def _makefile() -> str:
    return (ROOT / "Makefile").read_text(encoding="utf-8")
