"""Hold the Python lint and typecheck gateways to the repository inventory."""

import os
import re
import tomllib
from pathlib import Path
from typing import Any

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = ROOT / "Makefile"
PYPROJECT = ROOT / "pyproject.toml"
SOURCE_ROOTS = {".github", "tests", "scripts", "benches", "benchmarks"}
DF12_MESSAGES = {
    "R9101",
    "C9102",
    "R9103",
    "R9104",
    "C9105",
    "C9106",
    "C9107",
    "R9108",
    "R9109",
    "R9110",
    "R9111",
    "R9112",
    "C9112",
}
PRUNED_DIRECTORIES = {
    ".git",
    ".venv",
    "venv",
    ".uv-cache",
    ".uv-tools",
    "target",
    "vendor",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
}


def _makefile_lines() -> list[str]:
    """Return Makefile lines with continuations preserved for simple parsing."""
    return MAKEFILE.read_text(encoding="utf-8").splitlines()


def _make_variable(name: str) -> str:
    """Read one Make assignment, joining any continued physical lines."""
    assignment = re.compile(rf"{re.escape(name)}\s*[:?]?=\s*(?P<value>.+?)\s*")
    continued = ""
    for physical_line in _makefile_lines():
        line = continued + physical_line.lstrip()
        if line.endswith("\\"):
            continued = line[:-1] + " "
            continue
        continued = ""
        match = assignment.fullmatch(line)
        if match is not None:
            return match["value"]
    return pytest.fail(f"Makefile does not assign {name}", pytrace=False)


def _recipe(target: str) -> str:
    """Return one target's recipe for gateway-contract assertions."""
    lines = _makefile_lines()
    start = next(
        (index for index, line in enumerate(lines) if line.startswith(f"{target}:")),
        None,
    )
    if start is None:
        pytest.fail(f"Makefile does not define {target}", pytrace=False)
    recipe: list[str] = []
    for line in lines[start + 1 :]:
        if line.strip() and not line.startswith("\t"):
            break
        recipe.append(line)
    return "\n".join(recipe)


def _python_files() -> list[Path]:
    """List repository Python files, pruning generated and vendor trees."""
    found: list[Path] = []
    for directory, subdirectories, files in os.walk(ROOT):
        subdirectories[:] = sorted(
            name for name in subdirectories if name not in PRUNED_DIRECTORIES
        )
        found.extend(
            (Path(directory) / name).relative_to(ROOT)
            for name in files
            if name.endswith(".py")
        )
    return sorted(found)


def _workflow_steps(filename: str) -> list[dict[str, Any]]:
    """Read workflow steps from a GitHub Actions workflow file."""
    document = yaml.safe_load(
        (ROOT / ".github" / "workflows" / filename).read_text(encoding="utf-8")
    )
    jobs = document.get("jobs")
    assert isinstance(jobs, dict), f"{filename} must define workflow jobs"
    found: list[dict[str, Any]] = []
    for job in jobs.values():
        if not isinstance(job, dict):
            continue
        steps = job.get("steps")
        if isinstance(steps, list):
            found.extend(step for step in steps if isinstance(step, dict))
    return found


def test_python_baseline_matches_pylint_configuration() -> None:
    """Make and Pylint must agree on the managed CPython language baseline."""
    baseline = _make_variable("PYTHON_BASELINE")
    pylint_config = tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))["tool"][
        "pylint"
    ]["main"]
    assert baseline == "3.14", (
        "test_python_baseline_matches_pylint_configuration contract failed"
    )
    assert pylint_config["py-version"] == baseline, (
        "test_python_baseline_matches_pylint_configuration contract failed"
    )
    assert (ROOT / ".python-version").read_text(encoding="utf-8").strip() == baseline, (
        "root uv interpreter selection must match Make and Pylint"
    )


def test_every_python_file_is_inside_a_linted_source_root() -> None:
    """Workflow, action, test, script, and benchmark files share the inventory."""
    roots = set(_make_variable("PYTHON_SOURCE_ROOTS").split())
    assert SOURCE_ROOTS <= roots, (
        "test_every_python_file_is_inside_a_linted_source_root contract failed"
    )
    assert "find $(PYTHON_EXISTING_SOURCE_ROOTS)" in _make_variable("PYTHON_SOURCES"), (
        "test_every_python_file_is_inside_a_linted_source_root contract failed"
    )
    escaped = [
        str(path)
        for path in _python_files()
        if not any(path.is_relative_to(Path(root)) for root in roots)
    ]
    assert not escaped, f"Python files escape the lint roots: {escaped}"


def test_python_discovery_prunes_generated_and_vendor_trees() -> None:
    """The Make inventory excludes generated, cached, and vendored Python."""
    exclusions = _make_variable("PYTHON_PRUNED_DIRECTORIES")
    pruned = set(re.findall(r"-name\s+([^\s]+)\s+-prune", exclusions))
    assert pruned == PRUNED_DIRECTORIES, (
        "test_python_discovery_prunes_generated_and_vendor_trees contract failed"
    )
    discovery = _make_variable("PYTHON_SOURCES")
    assert "-type f -name '*.py' -print" in discovery and "| sort" in discovery, (
        "test_python_discovery_prunes_generated_and_vendor_trees contract failed"
    )


def test_make_lint_runs_pylint_and_typecheck_runs_ty() -> None:
    """The lint and typecheck leaves must use the same discovered sources."""
    pylint = _recipe("lint-python")
    typecheck = _recipe("typecheck-python")
    assert "$(PYLINT) $(PYTHON_SOURCES)" in pylint, (
        "test_make_lint_runs_pylint_and_typecheck_runs_ty contract failed"
    )
    assert "$(TY) check --python-version $(PYTHON_BASELINE)" in typecheck, (
        "test_make_lint_runs_pylint_and_typecheck_runs_ty contract failed"
    )
    assert "$(PYTHON_IMPORT_ROOTS) $(PYTHON_SOURCES)" in typecheck, (
        "test_make_lint_runs_pylint_and_typecheck_runs_ty contract failed"
    )
    assert "--from ty==$(TY_VERSION)" in _make_variable("TY"), (
        "test_make_lint_runs_pylint_and_typecheck_runs_ty contract failed"
    )
    for recipe in (pylint, typecheck):
        assert not re.search(r"(?m)^\s*[-+]\s*\$\(", recipe), (
            "test_make_lint_runs_pylint_and_typecheck_runs_ty contract failed"
        )
        assert "|| true" not in recipe, (
            "test_make_lint_runs_pylint_and_typecheck_runs_ty contract failed"
        )


def test_pylint_defaults_remain_active_and_df12_messages_are_complete() -> None:
    """Load the house plugin without disabling Pylint's default diagnostics."""
    config = tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))
    messages = config.get("tool", {}).get("pylint", {}).get("messages control", {})
    assert "disable" not in messages, (
        "test_pylint_defaults_remain_active_and_df12_messages_are_complete contract failed"
    )
    pylint = _make_variable("PYLINT")
    assert "--load-plugins=df12_python_lints" in pylint, (
        "test_pylint_defaults_remain_active_and_df12_messages_are_complete contract failed"
    )
    assert "--enable=$(DF12_PYLINT_MESSAGES)" in pylint, (
        "test_pylint_defaults_remain_active_and_df12_messages_are_complete contract failed"
    )
    assert "--disable" not in pylint, (
        "test_pylint_defaults_remain_active_and_df12_messages_are_complete contract failed"
    )
    assert set(_make_variable("DF12_PYLINT_MESSAGES").split(",")) == DF12_MESSAGES, (
        "test_pylint_defaults_remain_active_and_df12_messages_are_complete contract failed"
    )
    assert re.fullmatch(r"[0-9a-f]{40}", _make_variable("DF12_PYTHON_LINTS_REF")), (
        "test_pylint_defaults_remain_active_and_df12_messages_are_complete contract failed"
    )


def test_composite_gates_retain_sequential_python_leaves() -> None:
    """Parallel Make cannot skip or race either Python gateway."""
    lines = _makefile_lines()
    notparallel = next(line for line in lines if line.startswith(".NOTPARALLEL:"))
    assert "lint" in notparallel and "typecheck" in notparallel, (
        "test_composite_gates_retain_sequential_python_leaves contract failed"
    )
    lint_recipe = _recipe("lint")
    typecheck_recipe = _recipe("typecheck")
    assert lint_recipe.index("lint-whitaker") < lint_recipe.index("lint-python"), (
        "test_composite_gates_retain_sequential_python_leaves contract failed"
    )
    assert "+$(MAKE) lint-python" in lint_recipe, (
        "test_composite_gates_retain_sequential_python_leaves contract failed"
    )
    assert typecheck_recipe.index("typecheck-python") < typecheck_recipe.index(
        "typecheck-rust"
    ), (
        "test_composite_gates_retain_sequential_python_leaves contract failed"
    )
    assert "+$(MAKE) typecheck-python" in typecheck_recipe, (
        "test_composite_gates_retain_sequential_python_leaves contract failed"
    )
    assert "+$(MAKE) typecheck-rust" in typecheck_recipe, (
        "test_composite_gates_retain_sequential_python_leaves contract failed"
    )


def _named_step(steps: list[dict[str, Any]], name: str) -> dict[str, Any]:
    return next(step for step in steps if step.get("name") == name)


def _ci_python_gateway_steps(steps: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        step
        for step in steps
        if isinstance(step.get("run"), str)
        and step["run"].strip() in {"make lint", "make typecheck"}
    ]


def _assert_python_gates_are_required(
    steps: list[dict[str, Any]],
    setup: dict[str, Any],
    gateways: list[dict[str, Any]],
) -> None:
    assert {step["run"].strip() for step in gateways} == {
        "make lint", "make typecheck"
    }, (
        "test_ci_uses_cpython_314_and_requires_both_python_gateways contract failed"
    )
    assert all(
        "if" not in step and "continue-on-error" not in step for step in gateways
    ), (
        "test_ci_uses_cpython_314_and_requires_both_python_gateways contract failed"
    )
    setup_index = next(i for i, step in enumerate(steps) if step is setup)
    assert setup_index < min(steps.index(step) for step in gateways), (
        "test_ci_uses_cpython_314_and_requires_both_python_gateways contract failed"
    )


def test_ci_uses_cpython_314_and_requires_both_python_gateways() -> None:
    """The protected CI job runs binding lint and typecheck under the baseline."""
    steps = _workflow_steps("ci.yml")
    uv_setup = _named_step(steps, "Setup uv")
    assert uv_setup.get("with", {}).get("python-version") == "3.14", (
        "test_ci_uses_cpython_314_and_requires_both_python_gateways contract failed"
    )
    _assert_python_gates_are_required(steps, uv_setup, _ci_python_gateway_steps(steps))


def test_audit_workflow_uses_the_python_baseline() -> None:
    """Workflow-owned Python commands use the same explicit interpreter."""
    steps = _workflow_steps("audit.yml")
    setup_python = next(
        step
        for step in steps
        if isinstance(step.get("uses"), str)
        and step["uses"].startswith("actions/setup-python@")
    )
    assert setup_python.get("with", {}).get("python-version") == "3.14", (
        "test_audit_workflow_uses_the_python_baseline contract failed"
    )
    setup_uv = _named_step(steps, "Setup uv")
    assert setup_uv.get("uses") == (
        "astral-sh/setup-uv@b06acff4b6a41bdd9cdac56507ead0bd7734e757"
    ), "scheduled audit must use the approved pinned uv action"
    assert setup_uv.get("with", {}).get("python-version") == "3.14", (
        "scheduled audit uv setup must select CPython 3.14"
    )
    assert steps.index(setup_python) < steps.index(setup_uv) < steps.index(
        _named_step(steps, "Audit dependencies")
    ), "scheduled audit must provision Python and uv before invoking Make"


def test_audit_metadata_uses_managed_python_and_pipefail() -> None:
    """Cargo metadata parsing cannot fall back to an ambient interpreter."""
    recipe = _recipe("rust-audit")
    assert "set -eo pipefail" in recipe, "audit must propagate pipeline failures"
    assert "$(CARGO) metadata --no-deps --format-version 1 |" in recipe, (
        "audit must derive the actual Cargo workspace"
    )
    managed_invocation = (
        "$(UV) run --no-project --managed-python "
        "--python $(PYTHON_BASELINE) python -c"
    )
    assert managed_invocation in recipe, (
        "audit metadata must use managed CPython 3.14"
    )
    assert not re.search(r"\bpython3\b", recipe), (
        "audit must not use an ambient Python interpreter"
    )


def test_workflow_contract_tests_use_pinned_managed_python() -> None:
    """Pytest itself runs under the same managed interpreter and pinned version."""
    recipe = _recipe("test-workflow-contracts")
    assert all(
        required in recipe
        for required in ("--managed-python --python $(PYTHON_BASELINE)", "--no-project")
    ), "workflow contract tests must use isolated managed Python"
    dependencies = _make_variable("PYTHON_DEPENDENCIES")
    assert "pytest==$(PYTEST_VERSION)" in dependencies, (
        "workflow contract tests must pin their pytest dependency"
    )
