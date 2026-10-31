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


def _flag_value(words: list[str], flag: str) -> str | None:
    """Return the value following a flag when both operands are present."""
    try:
        return words[words.index(flag) + 1]
    except (ValueError, IndexError):
        return None


def _has_repository_binding(words: list[str]) -> bool:
    """Return whether the library check pins this repository and package."""
    return (
        "--lib" in words
        and _flag_value(words, "--manifest-path") == str(ROOT / "Cargo.toml")
        and _flag_value(words, "-p") == "statelet"
    )


def _has_development_flags(words: list[str]) -> bool:
    """Read the assigned flags as arguments, ignoring inherited shell expansion."""
    assignment = next((word for word in words if word.startswith("RUSTFLAGS=")), "")
    value = re.sub(r"\$\{[^}]*\}", "", assignment.removeprefix("RUSTFLAGS="))
    flags = shlex.split(value)
    return "-Zthreads=8" in flags and any(
        left == "-D" and right == "warnings" for left, right in zip(flags, flags[1:])
    )


def _binding_findings(name: str, words: list[str]) -> list[str]:
    """Check one row's package, manifest, library, and development flags."""
    problems = []
    if not _has_repository_binding(words):
        problems.append(f"{name} row must bind repository manifest and package")
    # build_standard_test covers platform-dependent linker selection.
    if not _has_development_flags(words):
        problems.append(f"{name} row must use warning and development flags")
    return problems


def _activates_default(words: list[str]) -> bool:
    """Detect explicit Cargo default selection in repeated or combined lists."""
    values = []
    for index, word in enumerate(words):
        if word in {"--features", "-F"} and index + 1 < len(words):
            values.append(words[index + 1])
        elif word.startswith("--features="):
            values.append(word.removeprefix("--features="))
        elif word.startswith("-F") and word != "-F":
            values.append(word[2:])
    return any(
        {"default", "statelet/default"}.intersection(re.split(r"[,\s]+", value))
        for value in values
    )


def _feature_findings(name: str, words: list[str]) -> list[str]:
    """Check required and forbidden feature selectors for one matrix row."""
    policies = {
        "no-default": (
            {"--no-default-features"},
            {"--all-features"},
            "no-default row must disable default features",
        ),
        "default": (
            set(),
            {"--no-default-features", "--all-features", "--each-feature"},
            "default row must use default features",
        ),
        "individual": (
            {"hack", "--each-feature", "--optional-deps", "--exclude-all-features"},
            {"--all-features"},
            "individual row must isolate each feature and optional dependency",
        ),
        "all": ({"--all-features"}, set(), "all row must enable all features"),
    }
    required, forbidden, message = policies[name]
    present = set(words)
    if name == "no-default" and _activates_default(words):
        return [message]
    return [] if required <= present and not forbidden.intersection(present) else [message]


def _matrix_findings(recipe: str) -> list[str]:
    """Judge feature-matrix rows and their bindings."""
    lines = matrix_lines(recipe)
    if len(lines) != len(ROWS):
        return ["matrix must run four Cargo checks"]
    problems: list[str] = []
    for name, line in zip(ROWS, lines, strict=True):
        words = shlex.split(line)
        problems.extend(_binding_findings(name, words))
        problems.extend(_feature_findings(name, words))
    return problems


def _all_findings(recipe: str) -> list[str]:
    """Check the order and uniqueness of the direct composite gates."""
    lines = recipe_lines(recipe)
    gates = ["make check-topology", "make feature-matrix", "make spelling"]
    if any(lines.count(gate) != 1 for gate in gates):
        return ["make all must run matrix exactly once"]
    if not lines.index(gates[0]) < lines.index(gates[1]) < lines.index(gates[2]):
        return ["make all must order matrix after topology and before spelling"]
    return []


def _suite_ordered(suite: list[int], topology: list[int]) -> bool:
    """Return whether the sole matrix suite follows the sole topology suite."""
    return len(suite) == len(topology) == 1 and topology[0] < suite[0]


def _has_suite_flags(words: list[str]) -> bool:
    """Return whether Nextest selects ignored tests, fails closed, and serializes."""
    return (
        _flag_value(words, "--run-ignored") == "only"
        and _flag_value(words, "--no-tests") == "fail"
        and "--test-threads=1" in words
    )


def _test_target(line: str) -> str | None:
    """Return the exact Nextest test target in an evaluated recipe line."""
    return _flag_value(shlex.split(line), "--test")


def _test_findings(recipe: str) -> list[str]:
    """Check that the ignored boundary suite follows topology and fails closed."""
    lines = recipe_lines(recipe)
    topology = [i for i, line in enumerate(lines) if _test_target(line) == "topology_command"]
    suite = [
        i for i, line in enumerate(lines) if _test_target(line) == "feature_matrix_command"
    ]
    if not _suite_ordered(suite, topology):
        return ["make test must run the matrix command suite after topology"]
    words = shlex.split(lines[suite[0]])
    if not _has_suite_flags(words):
        return ["matrix command suite must run ignored tests serially and fail closed"]
    return []


def _preflight_findings(wiring: Wiring) -> list[str]:
    """Check serialization declarations and the exact cargo-hack version."""
    problems = []
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


def _make_findings(wiring: Wiring) -> list[str]:
    """Judge the evaluated Make gates and their ordering."""
    return (
        _matrix_findings(wiring.matrix_recipe)
        + _all_findings(wiring.all_recipe)
        + _test_findings(wiring.test_recipe)
        + _preflight_findings(wiring)
    )


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
    positions, problems = _ci_positions(steps, commands)
    chain = commands[:3]
    if all(c in positions for c in chain):
        if not positions[chain[0]] < positions[chain[1]] < positions[chain[2]]:
            problems.append("CI must install and run the matrix after topology")
    return problems + _boundary_findings(workflow, steps, positions)


def _ci_positions(
    steps: list[reading.Step], commands: list[str]
) -> tuple[dict[str, int], list[str]]:
    """Collect unique step positions and report conditional or missing gates."""
    positions: dict[str, int] = {}
    problems = []
    for command in commands:
        matches = [i for i, step in enumerate(steps) if step.get("run") == command]
        if len(matches) != 1:
            problems.append(f"CI must run {command} exactly once")
            continue
        positions[command] = matches[0]
        step = steps[matches[0]]
        if "if" in step or step.get("continue-on-error") not in (None, False, "false"):
            problems.append(f"CI must run {command} unconditionally")
    return positions, problems


def _boundary_ordered(index: int, coverage: int, topology: int | None) -> bool:
    """Return whether the boundary follows coverage and the topology suite."""
    return 0 <= coverage < index and (topology is None or index > topology)


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
    if not _boundary_ordered(index, coverage, positions.get(topology)):
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
