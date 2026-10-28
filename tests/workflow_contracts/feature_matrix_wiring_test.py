"""Exercise every feature-matrix wiring clause against live and altered evidence."""

import copy
from collections.abc import Callable
from dataclasses import replace

import pytest

import workflow_reading as reading
from mock_wiring import make_n, recipe_lines
from feature_matrix_wiring_contract import (
    ROWS,
    Wiring,
    ci_steps,
    evaluate_live_wiring,
    findings,
    matrix_lines,
)


def _drop_step(wiring: Wiring, command: str) -> Wiring:
    """Remove one CI command from copied evidence."""
    workflow = copy.deepcopy(wiring.workflow)
    steps = ci_steps(workflow)
    steps[:] = [step for step in steps if step.get("run") != command]
    return replace(wiring, workflow=workflow)


def _step(wiring: Wiring, command: str) -> tuple[reading.Workflow, reading.Step]:
    """Find one CI command in copied evidence."""
    workflow = copy.deepcopy(wiring.workflow)
    step = next(step for step in ci_steps(workflow) if step.get("run") == command)
    return workflow, step


def _conditional(wiring: Wiring, command: str) -> Wiring:
    """Make one CI command conditional."""
    workflow, step = _step(wiring, command)
    step["if"] = "false"
    return replace(wiring, workflow=workflow)


def _move_before(wiring: Wiring, command: str, anchor: str) -> Wiring:
    """Move one CI command before its anchor."""
    workflow, step = _step(wiring, command)
    steps = ci_steps(workflow)
    steps.remove(step)
    index = next(i for i, item in enumerate(steps) if item.get("run") == anchor)
    steps.insert(index, step)
    return replace(wiring, workflow=workflow)


def _leak(wiring: Wiring, scope: str) -> Wiring:
    """Add coverage flags at one CI environment scope."""
    workflow, step = _step(wiring, "make test-feature-matrix-command")
    destination = {"step": step, "job": reading.jobs(workflow)[0][1], "workflow": workflow}[
        scope
    ]
    destination["env"] = {**(destination.get("env") or {}), "RUSTFLAGS": "coverage"}
    return replace(wiring, workflow=workflow)


def _row_replace(wiring: Wiring, row: int, old: str, new: str) -> Wiring:
    """Replace text in one evaluated matrix row."""
    line = matrix_lines(wiring.matrix_recipe)[row]
    assert old in line, f"Missing {old!r} in row {row}: {line}"
    return replace(
        wiring,
        matrix_recipe="\n".join(recipe_lines(wiring.matrix_recipe)).replace(
            line, line.replace(old, new), 1
        ),
    )


def _drop_row(wiring: Wiring, row: int) -> Wiring:
    """Remove one evaluated matrix row."""
    line = matrix_lines(wiring.matrix_recipe)[row]
    return replace(
        wiring, matrix_recipe="\n".join(recipe_lines(wiring.matrix_recipe)).replace(line, "", 1)
    )


def _matrix_after_spelling(wiring: Wiring) -> Wiring:
    """Move the existing matrix invocation after spelling without duplication."""
    lines = recipe_lines(wiring.all_recipe)
    lines.remove("make feature-matrix")
    lines.insert(lines.index("make spelling") + 1, "make feature-matrix")
    return replace(wiring, all_recipe="\n".join(lines))


type Mutator = Callable[[Wiring], Wiring]
MUTATIONS: list[tuple[str, Mutator, str]] = [
    (
        "all-matrix",
        lambda w: replace(w, all_recipe=w.all_recipe.replace("make feature-matrix\n", "")),
        "make all must run matrix exactly once",
    ),
    (
        "all-order",
        _matrix_after_spelling,
        "make all must order matrix after topology and before spelling",
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
        "threads-near-miss",
        lambda w: replace(
            w, test_recipe=w.test_recipe.replace("--test-threads=1", "--test-threads=10")
        ),
        "matrix command suite must run ignored",
    ),
    (
        "no-default-all",
        lambda w: _row_replace(
            w, 0, "--no-default-features", "--no-default-features --all-features"
        ),
        "no-default row must disable",
    ),
    (
        "serialization",
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
                f"warnings-near-miss-{row_name}",
                lambda w, i=row_index: _row_replace(w, i, "-D warnings", "-D warnings_extra"),
                f"{row_name} row must use warning",
            ),
            (
                f"threads-near-miss-{row_name}",
                lambda w, i=row_index: _row_replace(w, i, "-Zthreads=8", "-Zthreads=80"),
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
MUTATIONS.extend(
    (
        f"coverage-{env_scope}",
        lambda w, s=env_scope: _leak(w, s),
        "CI matrix boundary must not inherit coverage",
    )
    for env_scope in ("step", "job", "workflow")
)


@pytest.fixture(scope="module", name="live")
def _live() -> Wiring:
    """Evaluate the live wiring once for the test module."""
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
    assert not findings(live), findings(live)
    problems = findings(mutate(live))
    assert any(expected in problem for problem in problems), f"{name}: {problems}"


@pytest.mark.parametrize(
    ("host", "target", "linker_expected"),
    [
        ("Linux", "", True),
        ("Linux", "aarch64-unknown-linux-gnu", True),
        ("Linux", "aarch64-apple-darwin", False),
        ("Darwin", "", False),
    ],
)
def test_matrix_accepts_platform_specific_development_flags(
    live: Wiring, host: str, target: str, linker_expected: bool
) -> None:
    """Platform-specific linker flags coexist with mandatory warning checks."""
    recipe = make_n(
        "feature-matrix", overrides=(f"BUILD_HOST_OS={host}", f"CARGO_BUILD_TARGET={target}")
    )
    lines = matrix_lines(recipe)
    assert len(lines) == 4, lines
    assert all(("-Clink-arg=-fuse-ld=mold" in line) == linker_expected for line in lines), lines
    platform = replace(live, matrix_recipe=recipe)
    assert not findings(platform), findings(platform)
    for row, tested_row in enumerate(ROWS):
        problems = findings(_row_replace(platform, row, "-D warnings", ""))
        assert f"{tested_row} row must use warning and development flags" in problems, problems
