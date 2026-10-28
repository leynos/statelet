"""Judge the feature matrix's evaluated Make and CI wiring.

This test-only module owns no production interface. Its companion tests exercise
individual clauses with mutations as well as the live repository configuration.
Run them via ``make test-workflow-contracts``; they pass captured evidence to
``findings`` and check its reported clauses.
"""

import re
import shlex
from dataclasses import dataclass
from pathlib import Path

import workflow_reading as reading
from mock_wiring import make_n, recipe_lines

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = ROOT / ".github/workflows/ci.yml"
COVERAGE_ENV = frozenset(
    {
        "RUSTFLAGS",
        "CFLAGS",
        "LDFLAGS",
        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
    }
)
ROWS = ("no-default", "default", "individual", "all")


@dataclass(frozen=True, slots=True)
class Wiring:
    """Evidence captured from evaluated recipes and the strict workflow parser.

    Attributes
    ----------
    matrix_recipe, version_recipe, all_recipe, test_recipe : str
        Evaluated Make recipes for the named targets.
    workflow : reading.Workflow
        Strictly parsed CI workflow.
    makefile : str
        Source text used to check serialization declarations.
    """

    matrix_recipe: str
    version_recipe: str
    workflow: reading.Workflow
    makefile: str
    all_recipe: str
    test_recipe: str


def evaluate_live_wiring() -> Wiring:
    """Capture the repository's recipes, Makefile, and CI workflow.

    Returns
    -------
    Wiring
        Evaluated commands and workflow evidence.
    """
    return Wiring(
        make_n("feature-matrix"),
        make_n("check-cargo-hack"),
        reading.parse(WORKFLOW.name, WORKFLOW.read_text(encoding="utf-8")),
        (ROOT / "Makefile").read_text(encoding="utf-8"),
        make_n("all"),
        make_n("test"),
    )


def matrix_lines(recipe: str) -> list[str]:
    """Return the four Cargo library checks in their evaluated order.

    Parameters
    ----------
    recipe : str
        Evaluated Make recipe.

    Returns
    -------
    list[str]
        Library check command lines in order.
    """
    return [
        line
        for line in recipe_lines(recipe)
        if " cargo check " in line or " cargo hack check " in line
    ]


def _matrix_findings(recipe: str) -> list[str]:
    """Judge feature-matrix rows and their bindings."""
    lines = matrix_lines(recipe)
    if len(lines) != len(ROWS):
        return ["matrix must run four Cargo checks"]
    problems: list[str] = []
    for name, line in zip(ROWS, lines, strict=True):
        words = shlex.split(line)
        required = ["--lib", "--manifest-path", str(ROOT / "Cargo.toml"), "-p", "statelet"]
        if any(word not in words for word in required) or (
            words.index("--manifest-path") + 1 >= len(words)
            or words[words.index("--manifest-path") + 1] != str(ROOT / "Cargo.toml")
            or words.index("-p") + 1 >= len(words)
            or words[words.index("-p") + 1] != "statelet"
        ):
            problems.append(f"{name} row must bind repository manifest and package")
        # build_standard_test covers platform-dependent linker selection.
        if any(flag not in line for flag in ("-D warnings", "-Zthreads=8")):
            problems.append(f"{name} row must use warning and development flags")
        if name == "no-default" and "--no-default-features" not in words:
            problems.append("no-default row must disable default features")
        if name == "default" and any(
            flag in words
            for flag in ("--no-default-features", "--all-features", "--each-feature")
        ):
            problems.append("default row must use default features")
        if name == "individual" and (
            not {
                "hack",
                "--each-feature",
                "--optional-deps",
                "--exclude-all-features",
            }.issubset(words)
            or "--all-features" in words
        ):
            problems.append("individual row must isolate each feature and optional dependency")
        if name == "all" and "--all-features" not in words:
            problems.append("all row must enable all features")
    return problems


def _make_findings(wiring: Wiring) -> list[str]:
    """Judge the evaluated Make gates and their ordering."""
    problems = _matrix_findings(wiring.matrix_recipe)
    lines = recipe_lines(wiring.all_recipe)
    gates = ["make check-topology", "make feature-matrix", "make spelling"]
    if any(lines.count(gate) != 1 for gate in gates):
        problems.append("make all must run matrix exactly once")
    elif not lines.index(gates[0]) < lines.index(gates[1]) < lines.index(gates[2]):
        problems.append("make all must order matrix after topology and before spelling")
    test_lines = recipe_lines(wiring.test_recipe)
    topology = next(
        (i for i, line in enumerate(test_lines) if "--test topology_command" in line), -1
    )
    suite = [i for i, line in enumerate(test_lines) if "--test feature_matrix_command" in line]
    if len(suite) != 1 or suite[0] <= topology or topology < 0:
        problems.append("make test must run the matrix command suite after topology")
    elif any(
        flag not in test_lines[suite[0]]
        for flag in ("--run-ignored only", "--no-tests fail", "--test-threads=1")
    ):
        problems.append("matrix command suite must run ignored tests serially and fail closed")
    declarations = re.findall(r"^\.NOTPARALLEL:(.*)$", wiring.makefile, re.MULTILINE)
    named = set(" ".join(declarations).split())
    if not {"feature-matrix", "test-feature-matrix-command"}.issubset(named):
        problems.append("NOTPARALLEL must name both matrix targets")
    if not all(
        part in wiring.version_recipe
        for part in (
            'expected="cargo-hack 0.6.45"',
            '[ "$actual" != "$expected" ]',
            "exit 1",
            "make install-cargo-hack",
        )
    ):
        problems.append("cargo-hack preflight must enforce the exact pinned version")
    return problems


def ci_steps(workflow: reading.Workflow) -> list[reading.Step]:
    """Return the sole CI job's steps, or an empty list for invalid structure.

    Parameters
    ----------
    workflow : reading.Workflow
        Strictly parsed CI workflow.

    Returns
    -------
    list[reading.Step]
        The job's steps, or an empty list when unreadable.
    """
    jobs = reading.jobs(workflow)
    if len(jobs) != 1:
        return []
    steps = jobs[0][1].get("steps")
    return steps if isinstance(steps, list) and all(isinstance(s, dict) for s in steps) else []


def _ci_findings(workflow: reading.Workflow) -> list[str]:
    """Judge CI step presence, ordering, and the coverage boundary."""
    steps = ci_steps(workflow)
    jobs = reading.jobs(workflow)
    if not steps or len(jobs) != 1:
        return ["CI must have one readable job"]
    problems = []
    commands = [
        "make check-topology",
        "make install-cargo-hack",
        "make feature-matrix",
        "make test-topology-command",
        "make test-feature-matrix-command",
    ]
    positions: dict[str, int] = {}
    for command in commands:
        matches = [i for i, step in enumerate(steps) if step.get("run") == command]
        if len(matches) != 1:
            problems.append(f"CI must run {command} exactly once")
            continue
        positions[command] = matches[0]
        step = steps[matches[0]]
        if "if" in step or step.get("continue-on-error") not in (None, False, "false"):
            problems.append(f"CI must run {command} unconditionally")
    chain = commands[:3]
    if (
        all(c in positions for c in chain)
        and not positions[chain[0]] < positions[chain[1]] < positions[chain[2]]
    ):
        problems.append("CI must install and run the matrix after topology")
    return problems + _boundary_findings(workflow, steps, positions)


def _boundary_findings(
    workflow: reading.Workflow, steps: list[reading.Step], positions: dict[str, int]
) -> list[str]:
    """Judge ordering and inherited environment for the CI boundary suite."""
    boundary = "make test-feature-matrix-command"
    topology = "make test-topology-command"
    if boundary not in positions:
        return []
    problems: list[str] = []
    index = positions[boundary]
    coverage = next(
        (
            i
            for i, step in enumerate(steps)
            if reading.uses(step).split("@", maxsplit=1)[0]
            == "leynos/shared-actions/.github/actions/generate-coverage"
        ),
        -1,
    )
    if (
        coverage < 0
        or index <= coverage
        or (topology in positions and index <= positions[topology])
    ):
        problems.append("CI matrix boundary must follow coverage and topology boundary")
    for scope in (workflow, reading.jobs(workflow)[0][1], steps[index]):
        env = scope.get("env")
        if isinstance(env, dict) and COVERAGE_ENV.intersection(env):
            problems.append("CI matrix boundary must not inherit coverage environment")
            break
    return problems


def findings(wiring: Wiring) -> list[str]:
    """Return failed matrix and CI clauses for the supplied evidence.

    Parameters
    ----------
    wiring : Wiring
        Captured Make and workflow evidence.

    Returns
    -------
    list[str]
        Failed clauses, or an empty list when every clause holds.
    """
    return _make_findings(wiring) + _ci_findings(wiring.workflow)
