"""Prove Python checker failures bind Make's composite and leaf gates.

These hermetic process tests own temporary executable fixtures. Static Python
inventory and provisioning contracts remain in python_lint_gateway_test.py.
"""

import os
import re
import shlex
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]


def test_make_test_propagates_python_failure_under_parallel_make(tmp_path: Path) -> None:
    """A failing workflow leaf reaches Make after both Rust leaves run."""
    makefile = (ROOT / "Makefile").read_text(encoding="utf-8")
    notparallel = next(
        line for line in makefile.splitlines() if line.startswith(".NOTPARALLEL:")
    )
    assert "test" in notparallel.split(), "make -j must serialize the test target"
    log = tmp_path / "gates.log"
    cargo = tmp_path / "fake-cargo"
    uv = tmp_path / "fake-uv"
    cargo.write_text(
        '#!/bin/sh\nprintf "cargo %s\\n" "$*" >> "$GATE_LOG"\n',
        encoding="utf-8",
    )
    uv.write_text(
        '#!/bin/sh\nprintf "uv %s\\n" "$*" >> "$GATE_LOG"\nexit 23\n',
        encoding="utf-8",
    )
    cargo.chmod(0o755)
    uv.chmod(0o755)
    result = subprocess.run(
        ["make", "-j2", "-o", "check-nextest", "test", f"CARGO={cargo}", f"UV={uv}"],
        cwd=ROOT,
        env={**os.environ, "GATE_LOG": str(log)},
        capture_output=True,
        check=False,
        text=True,
    )
    calls = log.read_text(encoding="utf-8").splitlines()
    assert result.returncode != 0, "make test swallowed the workflow failure"
    commands = [shlex.split(call) for call in calls]
    assert [command[:2] for command in commands] == [
        ["cargo", "nextest"],
        ["cargo", "test"],
        ["uv", "tool"],
    ], f"make test did not run the suites in order: {calls}"
    pin = re.search(r"(?m)^CV005_CONTRACTS_REF \?= ([0-9a-f]{40})$", makefile)
    assert pin is not None, "the shared checker must use a full commit pin"
    source = (
        "git+https://github.com/leynos/shared-actions@"
        f"{pin[1]}"
        "#subdirectory=packages/cv005-contracts"
    )
    expected_checker_command = [
        "uv", "tool", "run", "--python", "3.13", "--from", source,
        "cv005-contracts", "check", "--repository", ".",
    ]
    assert commands[2] == expected_checker_command, (
        f"make test must fail on the pinned shared checker before running pytest: {calls}"
    )


@pytest.mark.parametrize(
    ("target", "override"),
    [
        pytest.param("lint-python", "PYLINT=/bin/false", id="pylint-failure"),
        pytest.param("typecheck-python", "TY=/bin/false", id="ty-failure"),
    ],
)
def test_python_gate_failures_reach_make(
    target: str, override: str
) -> None:
    """A failing checker must fail its Make target instead of being masked."""
    result = subprocess.run(
        ["make", "--no-print-directory", target, override],
        cwd=ROOT,
        capture_output=True,
        check=False,
        text=True,
    )
    assert result.returncode != 0, f"{target} swallowed the checker failure"


@pytest.fixture(name="python_gateway_checkout")
def _python_gateway_checkout(tmp_path: Path) -> Path:
    """Build all source roots beside a recorder that never runs real checkers."""
    checkout = tmp_path / "checkout"
    for source_root in (".github", "tests", "scripts", "benches", "benchmarks"):
        directory = checkout / source_root
        directory.mkdir(parents=True)
        (directory / "source file.py").touch()
        for excluded in (
            ".git", ".venv", "venv", ".uv-cache", ".uv-tools", "target",
            "vendor", "node_modules", "__pycache__", ".pytest_cache",
            ".mypy_cache", ".ruff_cache",
        ):
            generated = directory / excluded
            generated.mkdir()
            (generated / "excluded.py").touch()
    checker = tmp_path / "checker"
    checker.write_text(
        '#!/bin/sh\nprintf "%s\\n" "$@" > "$GATE_LOG"\n', encoding="utf-8"
    )
    checker.chmod(0o755)
    return checkout


def _run_python_gateway(
    checkout: Path, target: str, tool_variable: str
) -> subprocess.CompletedProcess[str]:
    """Invoke one real Make leaf with a fake checker and scratch source roots."""
    scratch = checkout.parent
    return subprocess.run(
        [
            "make", "--no-print-directory", "-f", str(ROOT / "Makefile"), target,
            f"{tool_variable}={scratch / 'checker'}",
            f"BUILD_TOOLS_PREFIX={scratch / 'tools'}",
        ],
        cwd=checkout,
        env={**os.environ, "GATE_LOG": str(scratch / "checker.args")},
        capture_output=True,
        check=False,
        text=True,
    )


@pytest.mark.parametrize(
    ("target", "tool_variable"),
    [("lint-python", "PYLINT"), ("typecheck-python", "TY")],
)
def test_partial_python_discovery_stops_before_checker(
    python_gateway_checkout: Path, target: str, tool_variable: str
) -> None:
    """A traversal failure after one filename cannot masquerade as coverage."""
    scratch = python_gateway_checkout.parent
    binary_dir = scratch / "tools" / "bin"
    binary_dir.mkdir(parents=True)
    failing_find = binary_dir / "find"
    failing_find.write_text(
        '#!/bin/sh\nprintf "%s\\n" "tests/partial.py"\n'
        'printf "%s\\n" "simulated traversal failure" >&2\nexit 23\n',
        encoding="utf-8",
    )
    failing_find.chmod(0o755)
    result = _run_python_gateway(python_gateway_checkout, target, tool_variable)
    assert result.returncode != 0, f"{target} accepted incomplete source discovery"
    assert "simulated traversal failure" in result.stderr, result.stderr
    assert not (scratch / "checker.args").exists(), (
        f"{target} invoked its checker on a partial inventory"
    )


@pytest.mark.parametrize(
    ("target", "tool_variable"),
    [("lint-python", "PYLINT"), ("typecheck-python", "TY")],
)
def test_python_gateway_receives_sorted_complete_inventory(
    python_gateway_checkout: Path, target: str, tool_variable: str
) -> None:
    """Both gateways retain all roots, pruning and filenames containing spaces."""
    result = _run_python_gateway(python_gateway_checkout, target, tool_variable)
    assert result.returncode == 0, result.stdout + result.stderr
    arguments = (python_gateway_checkout.parent / "checker.args").read_text(
        encoding="utf-8"
    ).splitlines()
    sources = sorted(
        f"{source_root}/source file.py"
        for source_root in (".github", "tests", "scripts", "benches", "benchmarks")
    )
    assert [argument for argument in arguments if argument.endswith(".py")] == sources, (
        arguments
    )
    if target == "typecheck-python":
        assert arguments[:3] == ["check", "--python-version", "3.14"], arguments
