"""Evaluate Make recipes without running them, for workflow contracts.

Several contracts need the commands a target *would* run rather than a
description of them: the CV-005 pin, the topology wiring, and the build
standard each assert on the evaluated recipe. Reading the Makefile as text
would accept a comment, a different target, or a variable that never expands,
so these contracts run ``make -n`` and inspect what Make prints.

This module owns that single boundary. It is test-only support: runtime code
and unrelated document contracts must not depend on it, and it decides no
contract's verdict.

Run via ``make test-workflow-contracts``.
"""

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def make_n(*arguments: str, overrides: tuple[str, ...] = ()) -> str:
    """Return the recipes ``make -n`` would run, without directory chatter.

    ``arguments`` are targets or ``name=value`` assignments passed in order;
    ``overrides`` are appended after them, which lets a caller show that a
    command-line assignment cannot override the recipe's own binding.

    >>> "make" in make_n("help")
    True
    """
    result = subprocess.run(
        ["make", "-n", "--no-print-directory", *arguments, *overrides],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout


def recipe_lines(recipe: str) -> list[str]:
    """Join Make's continued recipe lines, keeping command order.

    Make prints a continued recipe across several lines. Comparing line
    positions or searching for a flag would miss a command split at a
    backslash, so the continuation is folded first.
    """
    return [
        line.strip()
        for line in recipe.replace("\\\n", " ").splitlines()
        if line.strip()
    ]
