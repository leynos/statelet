"""Contract for the dependency-topology gate's routine wiring.

The policy suites prove what the check decides. Nothing in them proves the
check *runs*: a Make target that no entrypoint calls, or a CI step that runs
only on some events, passes every policy case while leaving the boundary
unguarded. This module holds that missing contract.

The evidence and the finding functions live in
``topology_wiring_contract.py``; this module supplies the live evidence, one
gold case, and one negative control per clause. The clauses are:

- ``make all`` runs the pinned-input ``check-topology`` form, and the
  standalone target pins the repository manifest even when a healthy fixture
  path is supplied as ``TOPOLOGY_MANIFEST_PATH`` on the command line.
- CI runs ``check-topology`` once, unconditionally, after the Rust toolchain
  and nextest are installed.
- ``make test`` runs the command-boundary suite after the ordinary nextest and
  doctest routes, from a cleared manifest environment, and hands the harness
  the caller's ``RUSTFLAGS`` so the nested recipe composes them only once.
- CI runs that suite after the coverage action, with none of the coverage
  environment inherited.
- The policy selector fails closed: an absent or ignored match fails through
  ``--no-tests fail``, and the success line prints only after the guard.
- Both composite targets stay in ``.NOTPARALLEL``, so ``make -j`` cannot
  interleave them with another gate.

Run via ``make test-workflow-contracts``.
"""

import copy
from collections.abc import Callable
from dataclasses import replace

import pytest

import workflow_reading as reading
from mock_wiring import recipe_lines
from topology_wiring_contract import (
    CHECK_TARGET,
    COMMAND_TARGET,
    REPOSITORY_MANIFEST,
    Wiring,
    ci_steps,
    evaluate_live_wiring,
    findings,
    step_command,
)


def _ci_step(workflow: reading.Workflow, target: str) -> reading.Step | None:
    """Return one workflow's ``make <target>`` step, or ``None`` when absent."""
    return next(
        (
            item for item in ci_steps(workflow)
            if step_command(item) == f"make {target}"
        ),
        None,
    )


def _drop_ci_step(wiring: Wiring, target: str) -> Wiring:
    """Return a copy whose workflow runs no ``make <target>`` step."""
    workflow = copy.deepcopy(wiring.workflow)
    steps = ci_steps(workflow)
    steps[:] = [step for step in steps if step_command(step) != f"make {target}"]
    return replace(wiring, workflow=workflow)


def _unpin_all(wiring: Wiring) -> Wiring:
    """Return ``make all`` evidence with the manifest pin replaced by empty."""
    return replace(
        wiring,
        all_recipe=wiring.all_recipe.replace(
            REPOSITORY_MANIFEST, 'STATELET_TOPOLOGY_MANIFEST_PATH=""'
        ),
    )


def _move_ci_check(wiring: Wiring, before: str) -> Wiring:
    """Return a workflow whose topology step runs before ``before``."""
    workflow = copy.deepcopy(wiring.workflow)
    steps = ci_steps(workflow)
    step = _ci_step(workflow, CHECK_TARGET)
    if step is None:
        return replace(wiring, workflow=workflow)
    steps.remove(step)
    anchor = next(
        (
            index for index, item in enumerate(steps)
            if before in step_command(item)
        ),
        0,
    )
    steps.insert(anchor, step)
    return replace(wiring, workflow=workflow)


def _conditional_ci_check(wiring: Wiring) -> Wiring:
    """Return a workflow whose topology step runs only on one event."""
    workflow = copy.deepcopy(wiring.workflow)
    step = _ci_step(workflow, CHECK_TARGET)
    if step is not None:
        step["if"] = "github.event_name == 'workflow_dispatch'"
    return replace(wiring, workflow=workflow)


def _drop_test_gate(wiring: Wiring) -> Wiring:
    """Return ``make test`` evidence without the command-boundary suite."""
    return replace(
        wiring,
        test_recipe="\n".join(
            line for line in recipe_lines(wiring.test_recipe)
            if "--test topology_command" not in line
        ),
    )


def _unguarded_test_gate(wiring: Wiring) -> Wiring:
    """Return command-suite evidence that inherits the manifest override."""
    return replace(
        wiring,
        test_recipe=wiring.test_recipe.replace(
            "env -u STATELET_TOPOLOGY_MANIFEST_PATH", ""
        ),
    )


def _drop_caller_flags(wiring: Wiring) -> Wiring:
    """Return command-suite evidence with no caller-flags copy for the harness."""
    return replace(
        wiring,
        test_recipe=wiring.test_recipe.replace(
            'STATELET_TOPOLOGY_CALLER_FLAGS="${RUSTFLAGS-}" \\\n', ""
        ),
    )


def _inherit_coverage_env(wiring: Wiring) -> Wiring:
    """Return a workflow whose command suite inherits coverage RUSTFLAGS."""
    workflow = copy.deepcopy(wiring.workflow)
    step = _ci_step(workflow, COMMAND_TARGET)
    if step is not None:
        step["env"] = {"RUSTFLAGS": "-C link-arg=-fuse-ld=lld"}
    return replace(wiring, workflow=workflow)


def _drop_fail_closed(wiring: Wiring) -> Wiring:
    """Return policy-selector evidence without ``--no-tests fail``."""
    return replace(
        wiring,
        check_recipe=wiring.check_recipe.replace("--no-tests fail", ""),
    )


def _early_success(wiring: Wiring) -> Wiring:
    """Return policy evidence that prints success before the guard runs."""
    lines = recipe_lines(wiring.check_recipe)
    success = next((line for line in lines if "Topology check passed" in line), "")
    guard = next((line for line in lines if "--test topology_check" in line), "")
    if not success or not guard:
        return wiring
    body = [line for line in lines if line not in {success, guard}]
    guard_index = lines.index(guard)
    return replace(
        wiring,
        check_recipe="\n".join(
            [success, *body[:guard_index], guard, *body[guard_index:]]
        ),
    )


def _drop_not_parallel(wiring: Wiring) -> Wiring:
    """Return Makefile evidence that no longer names both composite targets."""
    return replace(
        wiring,
        makefile=wiring.makefile.replace(f"{CHECK_TARGET} {COMMAND_TARGET}", ""),
    )


MUTATIONS: list[tuple[str, Callable[[Wiring], Wiring]]] = [
    ("unpinned-all", _unpin_all),
    ("missing-ci-check", lambda wiring: _drop_ci_step(wiring, CHECK_TARGET)),
    ("conditional-ci-check", _conditional_ci_check),
    (
        "check-before-toolchain",
        lambda wiring: _move_ci_check(wiring, "install-rust-toolchain"),
    ),
    ("check-before-nextest", lambda wiring: _move_ci_check(wiring, "cargo-nextest")),
    ("missing-test-gate", _drop_test_gate),
    ("unguarded-test-gate", _unguarded_test_gate),
    ("missing-caller-flags", _drop_caller_flags),
    ("missing-ci-command", lambda wiring: _drop_ci_step(wiring, COMMAND_TARGET)),
    ("inherited-coverage-env", _inherit_coverage_env),
    ("open-policy-selector", _drop_fail_closed),
    ("early-success-line", _early_success),
    ("parallel-composite", _drop_not_parallel),
]


@pytest.fixture(scope="module", name="live_wiring")
def _live_wiring_fixture() -> Wiring:
    """Evaluate the live evidence once; every mutation works on a copy."""
    return evaluate_live_wiring()


def test_every_routine_wiring_clause_holds(live_wiring: Wiring) -> None:
    """The repository's own recipes and workflow satisfy every clause."""
    assert not (problems := findings(live_wiring)), problems


@pytest.mark.parametrize(
    ("mutation", "mutate"), MUTATIONS, ids=[name for name, _ in MUTATIONS]
)
def test_each_removed_clause_is_reported(
    live_wiring: Wiring, mutation: str, mutate: Callable[[Wiring], Wiring]
) -> None:
    """Removing one clause from the evidence must fail this contract."""
    assert findings(mutate(live_wiring)), f"{mutation} was accepted"
