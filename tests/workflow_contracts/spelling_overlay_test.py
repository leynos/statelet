"""Contracts for Statelet's narrow spelling exceptions."""

import re
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LOCAL_OVERLAY = ROOT / "typos.local.toml"
GENERATED_CONFIG = ROOT / "typos.toml"
INLINE_CODE_MASK = r"`[^`\n]+`"
MISSPELLED_VARIABLE = "var.iam" + "ge_id"
OPENTOFU_EXAMPLE = r"`var\.iam" + r"ge_id` instead of `var\.image_id`"
# Derive the product name from its upstream CLI spelling for negative controls.
LINKER_NAME = "-fuse-ld=mold".removeprefix("-fuse-ld=")


def _patterns(path: Path, table: str, key: str) -> list[str]:
    """Read one pattern list from a TOML document."""
    document = tomllib.loads(path.read_text(encoding="utf-8"))
    return document[table][key]


def test_inline_code_is_not_a_blanket_spelling_exemption() -> None:
    """The overlay and its generated output must not mask every code span."""
    local_patterns = _patterns(LOCAL_OVERLAY, "patterns", "ignore")
    generated_patterns = _patterns(GENERATED_CONFIG, "default", "extend-ignore-re")

    assert INLINE_CODE_MASK not in local_patterns
    assert INLINE_CODE_MASK not in generated_patterns


def test_linker_exceptions_match_only_executable_contexts() -> None:
    """Fixed product spellings pass while ordinary linker prose remains checked."""
    patterns = _patterns(LOCAL_OVERLAY, "patterns", "ignore")
    generated_patterns = _patterns(GENERATED_CONFIG, "default", "extend-ignore-re")
    assert set(patterns) <= set(generated_patterns)
    assert "GitHub" + r"\s+" + "Fla" + "vored" + r"\s+Markdown" not in patterns
    examples = {
        "-fuse-ld=mold": "-Clink-arg=-fuse-ld=mold",
        "tools/mold/": "tools/mold/VERSION",
        "command -v mold": "command -v mold",
        r'binary_dir / \"' + LINKER_NAME + r'\"': 'binary_dir / "mold"',
        r'tool_bin / \"' + LINKER_NAME + r'\"': 'tool_bin / "mold"',
        '"tools" / "mold" / "VERSION"': 'ROOT / "tools" / "mold" / "VERSION"',
        r"\bMOLD_(?:RELEASE_BASE_URL|SHA256SUMS_FILE|TEST_VERSION|VERSION|VERSION_FILE)\b": "MOLD_VERSION_FILE",
        "`mold`": "a `mold` binary",
    }

    for pattern, example in examples.items():
        assert pattern in patterns
        assert re.search(pattern, example)

    ordinary_prose = (
        LINKER_NAME + " is the wrong spelling for the linker in this sentence",
        f'The word "{LINKER_NAME}" is misspelled here',
    )
    for prose in ordinary_prose:
        assert not any(re.search(pattern, prose) for pattern in patterns)
        assert not any(re.search(pattern, prose) for pattern in generated_patterns)


def test_ci_api_exceptions_do_not_mask_ordinary_spellings() -> None:
    """Only the exact Cargo key and GitHub event sequence retain US spelling."""
    patterns = _patterns(LOCAL_OVERLAY, "patterns", "ignore")
    assert any(re.search(pattern, "CARGO_TERM_COLOR: always") for pattern in patterns)
    assert any(
        re.search(pattern, "types: [opened, reopened, synchronize, labeled, ready_for_review]")
        for pattern in patterns
    )
    for prose in ("CO" + "LOR", "The pull request was lab" + "eled yesterday"):
        assert not any(re.search(pattern, prose) for pattern in patterns)


def test_historical_tokens_are_exact_quotations_or_commit_ids() -> None:
    """The immutable plan's rejected words are exempt only in recorded forms."""
    patterns = _patterns(LOCAL_OVERLAY, "patterns", "ignore")
    for recorded in ("`recognise`", "`mis-spelling`", "`mis`", "`8ba3e4a`", "`d4fb5ba`"):
        assert any(re.search(pattern, recorded) for pattern in patterns)
    for prose in ("recog" + "nise", "mi" + "s-spelling", "8bb3e4a"):
        assert not any(re.search(pattern, prose) for pattern in patterns)


def test_shared_opentofu_exemption_matches_only_the_worked_example() -> None:
    """The #170 example is retained, but real misspellings remain visible."""
    patterns = _patterns(GENERATED_CONFIG, "default", "extend-ignore-re")

    assert OPENTOFU_EXAMPLE in patterns
    example = f"`{MISSPELLED_VARIABLE}` instead of `var.image_id`"
    assert example == "`var.iam" + "ge_id` instead of `var.image_id`"
    assert re.search(OPENTOFU_EXAMPLE, example)

    misspellings = (
        MISSPELLED_VARIABLE,
        f"ami = {MISSPELLED_VARIABLE}",
        f"`{MISSPELLED_VARIABLE}`",
        f"`{MISSPELLED_VARIABLE}` instead of `var.image_ids`",
    )
    for text in misspellings:
        assert not any(re.search(pattern, text) for pattern in patterns)
