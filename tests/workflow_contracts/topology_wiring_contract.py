"""Evaluate the dependency-topology gate's routine wiring.

The policy suites prove what the check decides. This module supplies the
evidence and the clauses that prove the check *runs*: it evaluates ``make -n``
output for the affected targets, parses ``ci.yml`` with the strict workflow
reader, and judges both with the same pure finding functions.

Reading the Makefile as text would accept a comment, a different target, or a
variable that never expands, so every recipe clause is judged on what Make
would actually print. The negative controls that show each clause can fail
live beside the tests in ``topology_wiring_test.py``.

This is test-only support, run via ``make test-workflow-contracts``. Runtime
code and unrelated document contracts must not depend on it.
"""

import re
from dataclasses import dataclass
from pathlib import Path

import workflow_reading as reading
from mock_wiring import make_n, recipe_lines

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
MAKEFILE = ROOT / "Makefile"
CHECK_TARGET = "check-topology"
UNPINNED_TARGET = "topology-check"
COMMAND_TARGET = "test-topology-command"
COVERAGE_ACTION = "leynos/shared-actions/.github/actions/generate-coverage"
POLICY_TEST = "repository_obeys_topology"
REPOSITORY_MANIFEST = f'STATELET_TOPOLOGY_MANIFEST_PATH="{ROOT / "Cargo.toml"}"'
# A path the policy would accept if the check honoured it. The clause reads
# the printed assignment rather than running the check, so the path need not
# exist; a real substitution would be caught by the pin.
HEALTHY_FIXTURE = "/tmp/statelet-healthy-fixture/Cargo.toml"
# The environment the coverage action sets, which the command suite must not
# inherit.
COVERAGE_ENVIRONMENT = frozenset({
    "RUSTFLAGS",
    "CFLAGS",
    "LDFLAGS",
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
})


@dataclass(frozen=True, slots=True)
class Wiring:
    """The evaluated evidence the routine-wiring clauses judge."""

    workflow: reading.Workflow
    makefile: str
    all_recipe: str
    test_recipe: str
    check_recipe: str


def evaluate_live_wiring() -> Wiring:
    """Evaluate the real recipes and workflow, with a fixture override supplied."""
    return Wiring(
        workflow=reading.parse(
            WORKFLOW.name, WORKFLOW.read_text(encoding="utf-8")
        ),
        makefile=MAKEFILE.read_text(encoding="utf-8"),
        all_recipe=make_n("all"),
        test_recipe=make_n("test"),
        check_recipe=make_n(
            CHECK_TARGET, overrides=(f"TOPOLOGY_MANIFEST_PATH={HEALTHY_FIXTURE}",)
        ),
    )


def step_command(step: reading.Step) -> str:
    """Return one step's run command, stripped, or the empty string."""
    value = step.get("run")
    return value.strip() if isinstance(value, str) else ""


def _is_binding(step: reading.Step) -> bool:
    """Return whether a step runs unconditionally and cannot soften failure."""
    softened = step.get("continue-on-error")
    return "if" not in step and softened in {None, False, "false"}


def ci_steps(workflow: reading.Workflow) -> list[reading.Step]:
    """Return the CI job's own step list, or none when its shape is unreadable.

    The job's list is returned rather than a filtered copy: the negative
    controls mutate it in place, and a copy would accept every removal.
    """
    jobs = reading.jobs(workflow)
    if len(jobs) != 1:
        return []
    steps = jobs[0][1].get("steps")
    if not isinstance(steps, list) or any(not isinstance(step, dict) for step in steps):
        return []
    return steps


def _index_running(steps: list[reading.Step], command: str) -> int | None:
    """Return the index of the step whose run command is exactly ``command``."""
    return next(
        (index for index, step in enumerate(steps) if step_command(step) == command),
        None,
    )


def _index_matching(steps: list[reading.Step], needle: str) -> int | None:
    """Return the index of the first step whose run command contains ``needle``."""
    return next(
        (index for index, step in enumerate(steps) if needle in step_command(step)),
        None,
    )


def _uses_coverage(step: reading.Step) -> bool:
    """Return whether a step invokes the pinned coverage action."""
    return reading.uses(step).split("@", maxsplit=1)[0] == COVERAGE_ACTION


def _counted_index(
    steps: list[reading.Step], command: str
) -> tuple[int | None, list[str]]:
    """Require exactly one step running ``command`` and return its index.

    A missing step and a duplicate both raise the same finding: a duplicated
    gate can be re-ordered around the one that matters, so "exactly once" is
    checked before the position is read. ``None`` tells the caller to stop:
    every later clause would otherwise report the same absence.
    """
    if sum(step_command(step) == command for step in steps) != 1:
        return None, [f"ci.yml must run `{command}` exactly once"]
    return _index_running(steps, command), []


def _order_problems(
    found: int, wanted: str, before: int | None, anchor: str
) -> list[str]:
    """Report an anchor that is missing or runs after the step it must precede."""
    if before is None or before > found:
        return [f"`{wanted}` must run after {anchor}"]
    return []


def _findings_check_step(steps: list[reading.Step]) -> list[str]:
    """Check the CI topology step and the provisioning that must precede it."""
    wanted = f"make {CHECK_TARGET}"
    index, problems = _counted_index(steps, wanted)
    if index is None:
        return problems
    if not _is_binding(steps[index]):
        problems.append(f"`{wanted}` must run unconditionally")
    problems += _order_problems(
        index,
        wanted,
        _index_running(steps, "make install-rust-toolchain"),
        "`make install-rust-toolchain`",
    )
    problems += _order_problems(
        index,
        wanted,
        _index_matching(steps, "cargo-nextest"),
        "the nextest installation",
    )
    return problems


def _findings_command_step(steps: list[reading.Step]) -> list[str]:
    """Check the CI command suite and the coverage boundary it must follow."""
    wanted = f"make {COMMAND_TARGET}"
    index, problems = _counted_index(steps, wanted)
    if index is None:
        return problems
    step = steps[index]
    if not _is_binding(step):
        problems.append(f"`{wanted}` must run unconditionally")
    coverage = next(
        (index for index, item in enumerate(steps) if _uses_coverage(item)), None
    )
    problems += _order_problems(index, wanted, coverage, "the coverage action")
    environment = step.get("env")
    inherited = (
        sorted(set(environment) & COVERAGE_ENVIRONMENT)
        if isinstance(environment, dict)
        else []
    )
    if inherited:
        problems.append(f"`{wanted}` must not inherit coverage environment {inherited}")
    return problems


def _ci_findings(workflow: reading.Workflow) -> list[str]:
    """Check both CI steps, or fail when the workflow shape is unreadable."""
    steps = ci_steps(workflow)
    if not steps:
        return ["ci.yml must define exactly one readable job with steps"]
    return _findings_check_step(steps) + _findings_command_step(steps)


def _findings_make_all(recipe: str) -> list[str]:
    """Check that ``make all`` runs the pinned-input form."""
    lines = recipe_lines(recipe)
    problems = []
    if REPOSITORY_MANIFEST not in recipe:
        problems.append("`make all` must run the pinned `check-topology` target")
    if any(line == f"make {UNPINNED_TARGET}" for line in lines):
        problems.append("`make all` must not run the unpinned `topology-check` target")
    return problems


def _findings_make_test(recipe: str) -> list[str]:
    """Check that ``make test`` ends with the cleared-environment command suite."""
    lines = recipe_lines(recipe)
    joined = "\n".join(lines)
    nextest = joined.find("cargo nextest run")
    doctest = joined.find("cargo test --workspace --doc")
    command = joined.find("--test topology_command")
    if min(nextest, doctest, command) < 0:
        return ["`make test` must run nextest, doctests, and the topology command suite"]
    problems = []
    if not nextest < doctest < command:
        problems.append(
            "`make test` must run the topology command suite after nextest and doctests"
        )
    command_line = next(line for line in lines if "--test topology_command" in line)
    problems += _findings_command_line(command_line)
    return problems


def _findings_command_line(line: str) -> list[str]:
    """Check one evaluated command-suite invocation for its isolation flags."""
    problems = []
    if not line.startswith("env -u STATELET_TOPOLOGY_MANIFEST_PATH"):
        problems.append("the topology command suite must clear the manifest override")
    for flag in ("--run-ignored only", "--no-tests fail", "--test-threads=1"):
        if flag not in line:
            problems.append(f"the topology command suite must pass `{flag}`")
    return problems


def _findings_policy_selector(command_line: str) -> list[str]:
    """Check the policy selector's closed failure and full test-set flags."""
    problems = []
    if "--no-tests fail" not in command_line:
        problems.append("the policy selector must pass `--no-tests fail`")
    if "--run-ignored default" not in command_line:
        problems.append("the policy selector must pass `--run-ignored default`")
    if f"-E 'test(={POLICY_TEST})'" not in command_line:
        problems.append(f"the policy selector must name `{POLICY_TEST}`")
    return problems


def _findings_success_order(recipe: str) -> list[str]:
    """Check that the success line is emitted only after the guarded run."""
    run_index = recipe.find("--test topology_check")
    success_index = recipe.find("Topology check passed")
    if success_index < 0:
        return ["a passing run must print the success line"]
    if success_index < run_index:
        return ["the success line must print only after the guard"]
    return []


def _findings_check_recipe(recipe: str) -> list[str]:
    """Check the pinned manifest and the closed failure of the policy selector."""
    lines = recipe_lines(recipe)
    command_line = next((line for line in lines if "--test topology_check" in line), "")
    if not command_line:
        return ["`check-topology` must run the policy test"]
    problems = []
    if REPOSITORY_MANIFEST not in recipe or HEALTHY_FIXTURE in recipe:
        problems.append("`check-topology` must pin the repository manifest")
    problems += _findings_policy_selector(command_line)
    problems += _findings_success_order(recipe)
    return problems


def _findings_not_parallel(makefile: str) -> list[str]:
    """Check that both composite targets stay out of parallel execution."""
    match = re.search(r"^\.NOTPARALLEL:(.*)$", makefile, re.MULTILINE)
    declared = match[1].split() if match else []
    missing = [
        target for target in (CHECK_TARGET, COMMAND_TARGET) if target not in declared
    ]
    if missing:
        return [f"`.NOTPARALLEL` must name {missing}"]
    return []


def findings(wiring: Wiring) -> list[str]:
    """Return every routine-wiring clause the evaluated evidence fails."""
    return [
        *_findings_make_all(wiring.all_recipe),
        *_ci_findings(wiring.workflow),
        *_findings_make_test(wiring.test_recipe),
        *_findings_check_recipe(wiring.check_recipe),
        *_findings_not_parallel(wiring.makefile),
    ]
