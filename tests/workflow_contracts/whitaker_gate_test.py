"""Consumer contracts for Statelet's binding Whitaker installation and gate."""

from __future__ import annotations

import copy
import os
import re
import subprocess
import tomllib
from pathlib import Path

import pytest

from suite_provisioning import load_workflows

ROOT = Path(__file__).resolve().parents[2]
INSTALL_ACTION = "leynos/shared-actions/.github/actions/install-whitaker"
INSTALL_PIN = "6dea5677a84fec60ca51b07202570e3af12ffdb4"


def _ci() -> dict[object, object]:
    workflows = load_workflows()
    assert "ci.yml" in workflows
    return workflows["ci.yml"]


def _is_lint(step: dict[object, object]) -> bool:
    return isinstance(step.get("run"), str) and step["run"].strip() == "make lint"


def _key(value: object) -> str:
    return str(value).strip().lower().replace("_", "-")


def _workflow_problems(workflow: dict[object, object]) -> list[str]:
    """Check action provenance, job ordering, reachable inputs, and failure binding."""
    jobs = workflow.get("jobs")
    if not isinstance(jobs, dict):
        return ["CI workflow has no readable jobs"]
    candidates = []
    for name, job in jobs.items():
        if not isinstance(job, dict) or not isinstance(job.get("steps"), list):
            continue
        steps = job["steps"]
        if all(isinstance(step, dict) for step in steps) and any(map(_is_lint, steps)):
            candidates.append((str(name), job, steps))
    if len(candidates) != 1:
        return [f"expected one CI job running direct make lint, found {len(candidates)}"]

    job_name, job, steps = candidates[0]
    problems: list[str] = []
    if "if" in job or "continue-on-error" in job:
        problems.append(f"CI job {job_name!r} can skip or soften lint")
    installs = [
        i for i, step in enumerate(steps)
        if isinstance(step.get("uses"), str)
        and step["uses"].split("@", 1)[0].lower() == INSTALL_ACTION
    ]
    lints = [i for i, step in enumerate(steps) if _is_lint(step)]
    if len(installs) != 1:
        problems.append(f"expected one shared Whitaker installer action, found {len(installs)}")
    if len(lints) != 1:
        problems.append(f"expected one direct make lint step, found {len(lints)}")

    if len(installs) == 1:
        index = installs[0]
        installer = steps[index]
        if installer.get("uses") != f"{INSTALL_ACTION}@{INSTALL_PIN}":
            problems.append("Whitaker installer is not pinned to the approved full SHA")
        if len(lints) == 1 and index >= lints[0]:
            problems.append("Whitaker installation does not precede direct make lint")
        if "if" in installer or "continue-on-error" in installer:
            problems.append("Whitaker installer can be skipped or allowed to fail")
        inputs = installer.get("with", {})
        if not isinstance(inputs, dict):
            problems.append("Whitaker installer inputs are not a mapping")
        else:
            names = {_key(name) for name in inputs}
            if inputs.get("cranelift") not in (True, "true"):
                problems.append("Whitaker installer must receive cranelift: 'true'")
            if names != {"cranelift"}:
                problems.append("Whitaker installer has an unsupported input or version override")
            if names & {"suite-version", "allow-suite-pin"}:
                problems.append("rolling Whitaker suite pinning is forbidden")
            if any("installer" in name and "version" in name for name in names):
                problems.append("Whitaker installer version override is forbidden")
        installer_env = installer.get("env", {})
        if not isinstance(installer_env, dict):
            problems.append("Whitaker installer environment must be a mapping")
        elif installer_env.get("RUSTFLAGS", "") != "":
            problems.append(
                "non-empty repository RUSTFLAGS are injected into the Whitaker installer"
            )

    if len(lints) == 1 and (
        "if" in steps[lints[0]] or "continue-on-error" in steps[lints[0]]
    ):
        problems.append("direct make lint can be skipped or allowed to fail")
    for step in steps:
        uses = str(step.get("uses", "")).lower()
        if ("whitaker" in uses or "dylint" in uses) and uses.split("@", 1)[0] != INSTALL_ACTION:
            problems.append("Whitaker provisioning uses a second action or shim")
        run = step.get("run")
        if isinstance(run, str) and re.search(r"whitaker|dylint", run, re.IGNORECASE):
            if run.strip() != "make lint":
                problems.append("Whitaker has an ad hoc installer, cache, or PATH shim")
        inputs = step.get("with")
        if isinstance(inputs, dict):
            cache = " ".join(str(inputs.get(k, "")) for k in ("key", "path", "restore-keys"))
            name = str(step.get("name", "")).lower()
            if ("cache" in uses or "cache" in name) and re.search(r"whitaker|dylint", cache, re.I):
                problems.append("consumer duplicates the Whitaker action's cache")
    return sorted(set(problems))


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
    assignments: dict[str, list[str]] = {}
    for line in text.splitlines():
        match = re.match(r"^\s*(WHITAKER(?:_[A-Z0-9_]+)?)\s*(?:\?=|:=|=)\s*(.*?)\s*$", line)
        if match:
            assignments.setdefault(match[1], []).append(match[2])
    problems = []
    if assignments.get("WHITAKER") != ["whitaker"]:
        problems.append("Make must default WHITAKER to the whitaker executable")
    package_values = assignments.get("WHITAKER_PACKAGES", [])
    if not any("--workspace" in value.split() for value in package_values):
        problems.append("WHITAKER_PACKAGES does not select every workspace package")

    targets = _rules(text)
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
    return sorted(set(problems))


def _makefile() -> str:
    return (ROOT / "Makefile").read_text(encoding="utf-8")


def test_ci_provisions_the_binding_whitaker_gate() -> None:
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
    workflow = copy.deepcopy(_ci())
    jobs = workflow["jobs"]
    assert isinstance(jobs, dict)
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
    if mutation == "wrong pin":
        steps[installer]["uses"] = f"{INSTALL_ACTION}@v0.2.9"
    elif mutation == "remove Cranelift input":
        steps[installer]["with"].pop("cranelift")
    elif mutation == "move installer after lint":
        steps.insert(lint + 1, steps.pop(installer))
    elif mutation == "condition installer":
        steps[installer]["if"] = "false"
    elif mutation == "soft-fail lint":
        steps[lint]["continue-on-error"] = "true"
    elif mutation == "pin rolling suite":
        steps[installer]["with"]["suite-version"] = "0.2.9"
    elif mutation == "remove lint gate":
        steps.pop(lint)
    elif mutation == "inject RUSTFLAGS":
        steps[installer]["env"] = {"RUSTFLAGS": "-Ctarget-cpu=native"}
    elif mutation == "add installer shim":
        steps.insert(
            installer + 1,
            {"name": "Whitaker shim", "run": "whitaker-installer --cranelift"},
        )
    problems = _workflow_problems(workflow)
    assert any(expected in problem for problem in problems), f"{mutation}: {problems}"


def test_make_runs_all_packages_and_sequences_clippy_before_whitaker() -> None:
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
    text = _makefile()
    if mutation == "remove gate":
        text = re.sub(r"(?m)^lint-whitaker:.*(?:\n\t[^\n]*)*\n?", "", text)
    elif mutation == "narrow packages":
        text = re.sub(
            r"(?m)^(WHITAKER_PACKAGES\s*\?=\s*).*--workspace.*$",
            r"\1--package statelet",
            text,
        )
    elif mutation == "ignore failure":
        text = text.replace("\tRUSTFLAGS= $(WHITAKER)", "\t-$(WHITAKER)")
    elif mutation == "mask failure":
        text = re.sub(r"(?m)^(\tRUSTFLAGS= \$\(WHITAKER\)[^\n]*)$", r"\1 || true", text)
    elif mutation == "inject RUSTFLAGS":
        text = text.replace(
            "\tRUSTFLAGS= $(WHITAKER)", '\tRUSTFLAGS="-D warnings" $(WHITAKER)'
        )
    problems = _make_problems(text)
    assert any(expected in problem for problem in problems), f"{mutation}: {problems}"


def _executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(0o755)


def test_parallel_composite_propagates_whitaker_failure(tmp_path: Path) -> None:
    tool_bin = tmp_path / "bin"
    tool_bin.mkdir()
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text("utf-8"))["toolchain"]
    channel = toolchain["channel"]
    components = [
        f"{name.removesuffix('-preview')}-x86_64-unknown-linux-gnu"
        for name in toolchain.get("components", [])
    ]
    linker_version = (ROOT / "tools" / "mold" / "VERSION").read_text("utf-8").strip()
    events = tmp_path / "events.log"
    _executable(
        tool_bin / "mold",
        f"#!/bin/sh\n[ \"${{1:-}}\" = --version ] || exit 2\n"
        f"printf 'mold {linker_version} (fixture)\\n'\n",
    )
    _executable(tool_bin / "clang", "#!/bin/sh\n[ \"${1:-}\" = --version ] && exit 0\nexit 2\n")
    _executable(
        tool_bin / "rustup",
        "#!/bin/sh\ncase \"$*\" in\n"
        " 'toolchain list') echo \"$RUSTUP_TEST_TOOLCHAIN\" ;;\n"
        " 'component list'*) echo \"$RUSTUP_TEST_COMPONENTS\" ;;\n"
        " *) exit 2 ;;\nesac\n",
    )
    cargo = tool_bin / "fake-cargo"
    whitaker = tool_bin / "fake-whitaker"
    _executable(
        cargo,
        "#!/bin/sh\nprintf 'cargo:%s\\n' \"$*\" >> \"$WHITAKER_TEST_EVENT_LOG\"\n"
        "exit 0\n",
    )
    _executable(
        whitaker,
        "#!/bin/sh\nprintf 'whitaker:%s:RUSTFLAGS=%s\\n' \"$*\" "
        "\"${RUSTFLAGS-unset}\" >> \"$WHITAKER_TEST_EVENT_LOG\"\n"
        "exit 23\n",
    )
    env = {
        **os.environ,
        "BUILD_TOOLS_PREFIX": str(tmp_path),
        "RUSTUP_TEST_TOOLCHAIN": f"{channel}-x86_64-unknown-linux-gnu",
        "RUSTUP_TEST_COMPONENTS": "\n".join(components),
        "WHITAKER_TEST_EVENT_LOG": str(events),
    }
    result = subprocess.run(
        ["make", "--no-print-directory", "-j", "lint", f"CARGO={cargo}", f"WHITAKER={whitaker}"],
        cwd=ROOT, env=env, capture_output=True, text=True, check=False,
    )
    observed = events.read_text(encoding="utf-8").splitlines()
    kinds = [event.split(":", 1)[0] for event in observed]
    assert result.returncode != 0, (
        f"Whitaker failure was swallowed: {result.stdout}\n"
        f"{result.stderr}\n{observed}"
    )
    assert kinds[-1:] == ["whitaker"] and kinds.count("cargo") >= 2
    assert observed[-1].endswith("RUSTFLAGS="), observed
    cargo_calls = [
        (i, event.partition(":")[2].split()[0])
        for i, event in enumerate(observed)
        if event.startswith("cargo:")
    ]
    doc = next(i for i, subcommand in cargo_calls if subcommand == "doc")
    clippy = next(i for i, subcommand in cargo_calls if subcommand == "clippy")
    suite = next(i for i, kind in enumerate(kinds) if kind == "whitaker")
    assert doc < clippy < suite, observed
