"""Prove a failing Whitaker binary fails the Make composite target."""

import os
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _executable(path: Path, body: str) -> None:
    path.write_text(body, encoding="utf-8")
    path.chmod(0o755)


def test_parallel_composite_propagates_whitaker_failure(tmp_path: Path) -> None:
    """A failing Whitaker executable aborts sequential Make lint after Clippy."""
    tool_bin = tmp_path / "bin"
    tool_bin.mkdir()
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text("utf-8"))["toolchain"]
    channel = toolchain["channel"]
    components = [
        f"{name.removesuffix('-preview')}-x86_64-unknown-linux-gnu"
        for name in toolchain.get("components", [])
    ]
    linker_version = (ROOT / "tools" / "mold" / "VERSION").read_text("utf-8").strip()
    events = tmp_path / "events.log"
    _executable(
        tool_bin / "mold",
        f"#!/bin/sh\n[ \"${{1:-}}\" = --version ] || exit 2\n"
        f"printf 'mold {linker_version} (fixture)\\n'\n",
    )
    _executable(tool_bin / "clang", "#!/bin/sh\n[ \"${1:-}\" = --version ] && exit 0\nexit 2\n")
    _executable(
        tool_bin / "rustup",
        "#!/bin/sh\ncase \"$*\" in\n"
        " 'toolchain list') echo \"$RUSTUP_TEST_TOOLCHAIN\" ;;\n"
        " 'component list'*) echo \"$RUSTUP_TEST_COMPONENTS\" ;;\n"
        " *) exit 2 ;;\nesac\n",
    )
    cargo = tool_bin / "fake-cargo"
    whitaker = tool_bin / "fake-whitaker"
    _executable(
        cargo,
        "#!/bin/sh\nprintf 'cargo:%s\\n' \"$*\" >> \"$WHITAKER_TEST_EVENT_LOG\"\n"
        "exit 0\n",
    )
    _executable(
        whitaker,
        "#!/bin/sh\nprintf 'whitaker:%s:RUSTFLAGS=%s\\n' \"$*\" "
        "\"${RUSTFLAGS-unset}\" >> \"$WHITAKER_TEST_EVENT_LOG\"\n"
        "exit 23\n",
    )
    env = {
        **os.environ,
        "BUILD_TOOLS_PREFIX": str(tmp_path),
        "RUSTUP_TEST_TOOLCHAIN": f"{channel}-x86_64-unknown-linux-gnu",
        "RUSTUP_TEST_COMPONENTS": "\n".join(components),
        "WHITAKER_TEST_EVENT_LOG": str(events),
    }
    result = subprocess.run(
        ["make", "--no-print-directory", "-j", "lint", f"CARGO={cargo}", f"WHITAKER={whitaker}"],
        cwd=ROOT, env=env, capture_output=True, text=True, check=False,
    )
    observed = events.read_text(encoding="utf-8").splitlines()
    kinds = [event.split(":", 1)[0] for event in observed]
    assert result.returncode != 0, (
        f"Whitaker failure was swallowed: {result.stdout}\n"
        f"{result.stderr}\n{observed}"
    )
    assert kinds[-1:] == ["whitaker"] and kinds.count("cargo") >= 2, (
        "test_parallel_composite_propagates_whitaker_failure contract failed"
    )
    assert observed[-1].endswith("RUSTFLAGS="), observed
    cargo_calls = [
        (i, event.partition(":")[2].split()[0])
        for i, event in enumerate(observed)
        if event.startswith("cargo:")
    ]
    doc = next(i for i, subcommand in cargo_calls if subcommand == "doc")
    clippy = next(i for i, subcommand in cargo_calls if subcommand == "clippy")
    suite = next(i for i, kind in enumerate(kinds) if kind == "whitaker")
    assert doc < clippy < suite, observed
