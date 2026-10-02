"""Contracts for dev flags, held-out Cargo routes, and CI tool provisioning.

Evaluated Make recipes must preserve Cargo configuration defaults when they
assign ``RUSTFLAGS``. Coverage, release, and Whitaker remain isolated. The
workflow check requires pinned tools before Linux Rust gates.
"""

from __future__ import annotations

import os
import re
import shlex
import subprocess
import tomllib
from pathlib import Path

import pytest

from suite_provisioning import (
    load_workflows,
    suite_findings,
)

ROOT = Path(__file__).resolve().parents[2]
THREADS_FLAG = "-Zthreads=8"
LINKER_FLAG = "-Clink-arg=-fuse-ld=mold"
LINUX_TABLES = {"x86_64-unknown-linux-gnu", 'cfg(target_os = "linux")'}
NIGHTLY = True
RUSTFLAGS_RE = re.compile(r'RUSTFLAGS="([^"]*)"')
INHERITED = "--cfg inherited_from_caller"
COMPILE_COMMANDS = {
    "build", "check", "clippy", "doc", "test", "llvm-cov", "nextest-run",
}
DEVELOPMENT_TARGETS = ["test", "typecheck", "lint", "build"]
ASSIGNING_TARGETS = ["test", "typecheck", "lint", "build"]
HELD_OUT_TARGETS = ["coverage", "release"]


def _normalized(flags: list[str]) -> list[str]:
    """Join ``-C value`` pairs into ``-Cvalue`` so spellings compare equal."""
    joined: list[str] = []
    for flag in flags:
        if joined and joined[-1] == "-C":
            joined[-1] = f"-C{flag}"
        else:
            joined.append(flag)
    return joined


def _sources() -> dict[str, list[str]]:
    """Return every ``rustflags`` source in the configuration, by table."""
    config = tomllib.loads((ROOT / ".cargo" / "config.toml").read_text("utf-8"))
    sources = {"build": config.get("build", {}).get("rustflags")}
    for key, table in config.get("target", {}).items():
        sources[key] = table.get("rustflags")
    # A declared empty list is still a source Cargo would select, so only an
    # absent key is dropped.
    return {
        key: _normalized(flags) for key, flags in sources.items() if flags is not None
    }


def _expanded(value: str, inherited: str | None) -> list[str]:
    """Expand an assigned value as the recipe's shell would, then split it."""
    env = {key: val for key, val in os.environ.items() if key != "RUSTFLAGS"}
    if inherited is not None:
        env["RUSTFLAGS"] = inherited
    result = subprocess.run(
        ["bash", "-c", f'printf "%s" "{value}"'],
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )
    return _normalized(shlex.split(result.stdout))


def _recipe_lines(stdout: str) -> list[str]:
    """Join Make's continued recipe lines without losing command order."""
    return [
        line.strip()
        for line in stdout.replace("\\\n", " ").splitlines()
        if line.strip()
    ]


def _cargo_commands(lines: list[str]) -> list[tuple[str, str]]:
    """Classify Cargo calls, distinguishing Nextest's version probe."""
    commands = []
    for line in lines:
        try:
            words = shlex.split(line)
        except ValueError:
            continue
        for index, word in enumerate(words[:-1]):
            if Path(word).name not in {"cargo", "probe-cargo"}:
                continue
            subcommand = words[index + 1]
            if subcommand == "nextest":
                following = words[index + 2] if index + 2 < len(words) else ""
                if following == "--version":
                    kind = "nextest-version"
                elif following == "run":
                    kind = "nextest-run"
                else:
                    kind = "nextest-other"
            else:
                kind = (
                    subcommand
                    if subcommand in COMPILE_COMMANDS
                    else f"cargo-{subcommand}"
                )
            commands.append((kind, line))
            break
    return commands


def _make_output(
    target: str,
    host: str = "Linux",
    inherited: str | None = None,
    overrides: tuple[str, ...] = (),
) -> str:
    """Return evaluated Make recipes with an injectable Cargo executable."""
    env = {key: val for key, val in os.environ.items() if key != "RUSTFLAGS"}
    if inherited is not None:
        env["RUSTFLAGS"] = inherited
    result = subprocess.run(
        [
            "make", "-n", "-B", "--no-print-directory", "CARGO=probe-cargo",
            f"BUILD_HOST_OS={host}", *overrides, target,
        ],
        cwd=ROOT,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout


def _make_rustflags(
    target: str,
    host: str = "Linux",
    inherited: str | None = None,
    overrides: tuple[str, ...] = (),
) -> list[list[str] | None]:
    """Return RUSTFLAGS for each Cargo compile/test route, excluding tools."""
    lines = _recipe_lines(_make_output(target, host, inherited, overrides))
    observed = _cargo_commands(lines)
    unknown = [
        (kind, line)
        for kind, line in observed
        if kind not in COMPILE_COMMANDS | {"nextest-version"}
    ]
    assert unknown == [], f"unclassified Cargo calls in `make {target}`: {unknown}"
    assert all("probe-cargo" in shlex.split(line) for _, line in observed), observed
    commands = [line for kind, line in observed if kind in COMPILE_COMMANDS]
    assert commands, f"`make -n {target}` runs no cargo command"
    if target in DEVELOPMENT_TARGETS:
        checks = [i for i, line in enumerate(lines) if "scripts/check-build-tools.sh" in line]
        first_compile = min(lines.index(line) for line in commands)
        assert checks and min(checks) < first_compile, (
            f"`make {target}` compiles before check-build-tools"
        )
    assigned: list[list[str] | None] = []
    for line in commands:
        match = RUSTFLAGS_RE.search(line)
        # Any other spelling still replaces the configuration's sources, so a
        # form this reader cannot parse fails rather than passing.
        assert match or "RUSTFLAGS=" not in line, (
            f"unreadable RUSTFLAGS assignment in {line!r}"
        )
        assigned.append(_expanded(match.group(1), inherited) if match else None)
    return assigned


def _contains(flags: list[str], wanted: list[str]) -> bool:
    """Return whether ``wanted`` appears in ``flags`` as a contiguous run."""
    return any(
        flags[start : start + len(wanted)] == wanted
        for start in range(len(flags) - len(wanted) + 1)
    )


def _flag_problems(
    where: str, flags: list[str], *, expects_linker: bool, inherited: str | None
) -> list[str]:
    """Check one assigned ``RUSTFLAGS`` value against the standard.

    ``where`` names the command and host, so a finding says which recipe broke.
    """
    problems = []
    if (THREADS_FLAG in flags) != NIGHTLY:
        problems.append(f"{where} gets {THREADS_FLAG} wrong: {flags}")
    if (LINKER_FLAG in flags) != expects_linker:
        problems.append(f"{where} gets `mold` wrong: {flags}")
    if inherited is not None and not _contains(flags, shlex.split(inherited)):
        problems.append(f"{where} drops the caller's RUSTFLAGS: {flags}")
    return problems


def _unassigned_problems(target: str, inherited: str | None) -> list[str]:
    """Report a command with no assignment that would take only the caller's flags.

    Without an assignment the command takes the configuration's flags, unless
    the caller exports RUSTFLAGS, which displaces them; setup-rust does exactly
    that in CI.
    """
    if inherited is None:
        return []
    return [f"`make {target}` runs a command that takes only the caller's RUSTFLAGS"]


def _development_problems(
    host: str,
    *,
    expects_linker: bool,
    inherited: str | None = None,
    overrides: tuple[str, ...] = (),
) -> list[str]:
    """Check every development target on one host: an assigned ``RUSTFLAGS``,
    empty or not, carries the frontend flag, and carries `mold` exactly when on
    Linux."""
    problems = []
    for target in DEVELOPMENT_TARGETS:
        for flags in _make_rustflags(target, host, inherited, overrides):
            problems += (
                _unassigned_problems(target, inherited)
                if flags is None
                else _flag_problems(
                    f"`make {target}` on {host}",
                    flags,
                    expects_linker=expects_linker,
                    inherited=inherited,
                )
            )
    return problems


def test_every_rustflags_source_carries_the_parallel_frontend() -> None:
    sources = _sources()
    if not NIGHTLY:
        carrying = [key for key, flags in sources.items() if THREADS_FLAG in flags]
        assert carrying == [], f"{THREADS_FLAG} on a stable pin in {carrying}"
        return
    assert "build" in sources, "no [build] rustflags for non-Linux hosts"
    missing = [key for key, flags in sources.items() if THREADS_FLAG not in flags]
    assert missing == [], f"{THREADS_FLAG} missing from {missing}"


def test_linker_is_confined_to_linux() -> None:
    sources = _sources()
    linux = [key for key in sources if key in LINUX_TABLES]
    assert linux, "no Linux target table carries rustflags"
    assert all(LINKER_FLAG in sources[key] for key in linux), "Linux lost `mold`"
    wider = [
        key
        for key, flags in sources.items()
        if key not in LINUX_TABLES and LINKER_FLAG in flags
    ]
    assert wider == [], f"`mold` named beyond Linux in {wider}"


def test_sources_differ_only_by_the_linker() -> None:
    stripped = {
        tuple(f for f in flags if f != LINKER_FLAG) for flags in _sources().values()
    }
    assert len(stripped) == 1, f"rustflags sources disagree: {stripped}"


def test_rust_formatter_uses_the_repository_toolchain() -> None:
    """Keep rustfmt aligned with the toolchain used by check-fmt and CI."""
    commands = [
        line for line in _recipe_lines(_make_output("fmt"))
        if line.startswith("probe-cargo ")
    ]
    assert commands == ["probe-cargo fmt --all"], commands


def test_development_targets_restate_both_flags_on_linux() -> None:
    problems = _development_problems("Linux", expects_linker=True)
    assert problems == [], problems
    for target in ASSIGNING_TARGETS:
        assert any(flags is not None for flags in _make_rustflags(target)), (
            f"`make {target}` assigns no RUSTFLAGS"
        )


def test_development_targets_keep_the_standard_under_inherited_rustflags() -> None:
    problems = _development_problems("Linux", expects_linker=True, inherited=INHERITED)
    assert problems == [], problems


def test_development_targets_keep_the_frontend_but_not_the_linker_elsewhere() -> None:
    problems = _development_problems("Darwin", expects_linker=False)
    assert problems == [], problems


def test_development_targets_leave_the_linker_off_a_non_linux_target() -> None:
    problems = _development_problems(
        "Linux",
        expects_linker=False,
        overrides=("CARGO_BUILD_TARGET=aarch64-apple-darwin",),
    )
    problems += _development_problems(
        "Linux",
        expects_linker=True,
        overrides=("CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu",),
    )
    # Cargo resolves `host-tuple` to the host's own triple.
    problems += _development_problems(
        "Linux", expects_linker=True, overrides=("CARGO_BUILD_TARGET=host-tuple",)
    )
    assert problems == [], problems


def test_make_test_checks_nextest_then_runs_each_test_route() -> None:
    """Separate Nextest's version probe from both compiled test commands."""
    commands = _cargo_commands(_recipe_lines(_make_output("test")))
    assert [kind for kind, _ in commands] == [
        "nextest-version", "nextest-run", "test",
    ], commands
    assert all("probe-cargo" in shlex.split(line) for _, line in commands)
    assert len(_make_rustflags("test")) == 2
    assert "--doc" in shlex.split(commands[-1][1])
    lines = _recipe_lines(_make_output("test"))
    assert lines.index("scripts/check-build-tools.sh") < lines.index(commands[0][1])


def test_make_lint_keeps_whitaker_outside_development_rustflags() -> None:
    """Only repository-toolchain Cargo gates receive development flags."""
    lines = _recipe_lines(_make_output("lint"))
    commands = _cargo_commands(lines)
    assert [kind for kind, _ in commands] == ["doc", "clippy"], commands
    whitaker = [line for line in lines if line.startswith("RUSTFLAGS= whitaker ")]
    assert len(whitaker) == 1, whitaker
    assert whitaker[0] == (
        "RUSTFLAGS= whitaker --all --workspace -- --all-targets --all-features"
    )
    assert lines.index(commands[-1][1]) < lines.index(whitaker[0])
    assert all("probe-cargo" in shlex.split(line) for _, line in commands)


@pytest.mark.parametrize("target", HELD_OUT_TARGETS)
def test_coverage_and_release_take_neither_flag(target: str) -> None:
    for flags in _make_rustflags(target):
        assert flags is not None, (
            f"`make {target}` runs a command that takes the configuration's flags"
        )
        assert THREADS_FLAG not in flags, f"`make {target}` takes {THREADS_FLAG}"
        assert LINKER_FLAG not in flags, f"`make {target}` takes {LINKER_FLAG}"


def test_ci_installs_build_tools_before_every_suite_route() -> None:
    """Every discovered Linux suite path has an unconditional local preflight."""
    problems = suite_findings(load_workflows())
    assert problems == [], problems
