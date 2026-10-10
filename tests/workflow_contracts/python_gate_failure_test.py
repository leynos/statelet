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
