"""Exercise every feature-matrix wiring clause against live and altered evidence."""

import copy
from collections.abc import Callable
from dataclasses import replace

import pytest

import workflow_reading as reading
from mock_wiring import recipe_lines
from feature_matrix_wiring_contract import (
    ROWS,
    Wiring,
    ci_steps,
    evaluate_live_wiring,
    findings,
    matrix_lines,
)


def _complete(wiring: Wiring) -> Wiring:
    """Supply planned EP-M2 steps so controls are independent of implementation."""
    workflow = copy.deepcopy(wiring.workflow)
    steps = ci_steps(workflow)
    topology = next(
        i for i, step in enumerate(steps) if step.get("run") == "make check-topology"
    )
    for offset, command in enumerate(("make install-cargo-hack", "make feature-matrix"), 1):
        if not any(step.get("run") == command for step in steps):
            steps.insert(topology + offset, {"run": command})
    if not any(step.get("run") == "make test-feature-matrix-command" for step in steps):
        steps.append({"run": "make test-feature-matrix-command"})
    all_recipe = wiring.all_recipe
    if "make feature-matrix\n" not in all_recipe:
        all_recipe = all_recipe.replace(
            "make check-topology\n", "make check-topology\nmake feature-matrix\n", 1
        )
    return replace(wiring, workflow=workflow, all_recipe=all_recipe)


def _drop_step(wiring: Wiring, command: str) -> Wiring:
    workflow = copy.deepcopy(wiring.workflow)
    steps = ci_steps(workflow)
    steps[:] = [step for step in steps if step.get("run") != command]
    return replace(wiring, workflow=workflow)


def _step(wiring: Wiring, command: str) -> tuple[reading.Workflow, reading.Step]:
    workflow = copy.deepcopy(wiring.workflow)
    step = next(step for step in ci_steps(workflow) if step.get("run") == command)
    return workflow, step


def _conditional(wiring: Wiring, command: str) -> Wiring:
    workflow, step = _step(wiring, command)
    step["if"] = "false"
    return replace(wiring, workflow=workflow)


def _move_before(wiring: Wiring, command: str, anchor: str) -> Wiring:
    workflow, step = _step(wiring, command)
    steps = ci_steps(workflow)
    steps.remove(step)
    index = next(i for i, item in enumerate(steps) if item.get("run") == anchor)
    steps.insert(index, step)
    return replace(wiring, workflow=workflow)


def _leak(wiring: Wiring, scope: str) -> Wiring:
    workflow, step = _step(wiring, "make test-feature-matrix-command")
    destination = {"step": step, "job": reading.jobs(workflow)[0][1], "workflow": workflow}[
        scope
    ]
    destination["env"] = {**(destination.get("env") or {}), "RUSTFLAGS": "coverage"}
    return replace(wiring, workflow=workflow)


def _row_replace(wiring: Wiring, row: int, old: str, new: str) -> Wiring:
    line = matrix_lines(wiring.matrix_recipe)[row]
    assert old in line, f"Missing {old!r} in row {row}: {line}"
    return replace(
        wiring,
        matrix_recipe="\n".join(recipe_lines(wiring.matrix_recipe)).replace(
            line, line.replace(old, new), 1
        ),
    )


def _drop_row(wiring: Wiring, row: int) -> Wiring:
    line = matrix_lines(wiring.matrix_recipe)[row]
    return replace(
        wiring, matrix_recipe="\n".join(recipe_lines(wiring.matrix_recipe)).replace(line, "", 1)
    )


type Mutator = Callable[[Wiring], Wiring]
MUTATIONS: list[tuple[str, Mutator, str]] = [
    (
        "all-matrix",
        lambda w: replace(w, all_recipe=w.all_recipe.replace("make feature-matrix\n", "")),
        "make all must run matrix",
    ),
    (
        "all-order",
        lambda w: replace(
            w,
            all_recipe=w.all_recipe.replace(
                "make check-topology\n", "make feature-matrix\nmake check-topology\n", 1
            ).replace("make feature-matrix\nmake spelling", "make spelling", 1),
        ),
        "make all must run matrix",
    ),
    (
        "test-suite",
        lambda w: replace(
            w,
            test_recipe=w.test_recipe.replace("--test feature_matrix_command", "--test other"),
        ),
        "make test must run the matrix command suite",
    ),
    (
        "ignored-suite",
        lambda w: replace(
            w, test_recipe=w.test_recipe.replace("--run-ignored only", "--run-ignored default")
        ),
        "matrix command suite must run ignored",
    ),
    (
        "serialisation",
        lambda w: replace(
            w, makefile=w.makefile.replace("feature-matrix test-feature-matrix-command", "")
        ),
        "NOTPARALLEL must name",
    ),
    (
        "version",
        lambda w: replace(
            w,
            version_recipe=w.version_recipe.replace(
                '[ "$actual" != "$expected" ]', '[ -z "$actual" ]'
            ),
        ),
        "preflight must enforce",
    ),
    (
        "isolated-all",
        lambda w: _row_replace(w, 2, "--each-feature", "--each-feature --all-features"),
        "individual row must isolate",
    ),
    (
        "no-default",
        lambda w: _row_replace(w, 0, "--no-default-features", ""),
        "no-default row must disable",
    ),
    (
        "default",
        lambda w: _row_replace(w, 1, "cargo check", "cargo check --all-features"),
        "default row must use default",
    ),
    ("all-features", lambda w: _row_replace(w, 3, "--all-features", ""), "all row must enable"),
]
for row_index, row_name in enumerate(ROWS):
    MUTATIONS.extend(
        [
            (
                f"missing-{row_name}",
                lambda w, i=row_index: _drop_row(w, i),
                "matrix must run four",
            ),
            (
                f"package-{row_name}",
                lambda w, i=row_index: _row_replace(w, i, "-p statelet", "-p other"),
                f"{row_name} row must bind",
            ),
            (
                f"manifest-{row_name}",
                lambda w, i=row_index: _row_replace(w, i, '/Cargo.toml"', '/other.toml"'),
                f"{row_name} row must bind",
            ),
            (
                f"warnings-{row_name}",
                lambda w, i=row_index: _row_replace(w, i, "-D warnings", ""),
                f"{row_name} row must use warning",
            ),
            (
                f"build-{row_name}",
                lambda w, i=row_index: _row_replace(w, i, "-Zthreads=8", ""),
                f"{row_name} row must use warning",
            ),
        ]
    )
for ci_command in (
    "make install-cargo-hack",
    "make feature-matrix",
    "make test-feature-matrix-command",
):
    MUTATIONS.extend(
        [
            (
                f"missing-{ci_command}",
                lambda w, c=ci_command: _drop_step(w, c),
                f"CI must run {ci_command} exactly once",
            ),
            (
                f"conditional-{ci_command}",
                lambda w, c=ci_command: _conditional(w, c),
                f"CI must run {ci_command} unconditionally",
            ),
        ]
    )
MUTATIONS.extend(
    [
        (
            "early-install",
            lambda w: _move_before(w, "make install-cargo-hack", "make check-topology"),
            "CI must install and run the matrix after topology",
        ),
        (
            "early-matrix",
            lambda w: _move_before(w, "make feature-matrix", "make install-cargo-hack"),
            "CI must install and run the matrix after topology",
        ),
        (
            "early-boundary",
            lambda w: _move_before(
                w, "make test-feature-matrix-command", "make test-topology-command"
            ),
            "CI matrix boundary must follow",
        ),
    ]
)
for env_scope in ("step", "job", "workflow"):
    MUTATIONS.append(
        (
            f"coverage-{env_scope}",
            lambda w, s=env_scope: _leak(w, s),
            "CI matrix boundary must not inherit coverage",
        )
    )


@pytest.fixture(scope="module", name="live")
def _live() -> Wiring:
    return evaluate_live_wiring()


def test_live_feature_matrix_wiring(live: Wiring) -> None:
    """The actual evaluated commands and workflow meet the complete contract."""
    assert not (problems := findings(live)), problems


@pytest.mark.parametrize(
    ("name", "mutate", "expected"), MUTATIONS, ids=[m[0] for m in MUTATIONS]
)
def test_mutations_reject_their_own_clause(
    live: Wiring, name: str, mutate: Mutator, expected: str
) -> None:
    """Each negative control must fail for its intended reason."""
    baseline = _complete(live)
    assert not findings(baseline), findings(baseline)
    problems = findings(mutate(baseline))
    assert any(expected in problem for problem in problems), f"{name}: {problems}"
