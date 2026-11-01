"""Check the test-only Cargo command classifier's nested tool routes."""

import pytest

from build_standard_test import _cargo_kind


@pytest.mark.parametrize(
    ("words", "expected"),
    [
        (["hack", "--version"], "hack-version"),
        (["hack", "check"], "hack-check"),
        (["hack", "other"], "hack-other"),
        (["nextest", "--version"], "nextest-version"),
        (["nextest", "run"], "nextest-run"),
        (["nextest"], "nextest-other"),
        (["check"], "check"),
        (["metadata"], "cargo-metadata"),
    ],
)
def test_cargo_command_classification(words: list[str], expected: str) -> None:
    """Each supported route and fallback has a distinct classification."""
    assert _cargo_kind(words, 0) == expected, f"unexpected Cargo route for {words}"
