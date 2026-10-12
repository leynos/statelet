"""Hermetic coverage of pinned archive installation and entrypoint failures.

The fake commands belong only to these installer boundary tests. Their call
log observes orchestration order without invoking Cargo or a real download.
"""

import hashlib
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

import pytest

from build_tools_scripts_test import INSTALL_SCRIPT, MOLD_VERSION, TOOLCHAIN, _write_executable


@dataclass(frozen=True, slots=True)
class InstallerEnvironment:
    """Files and process configuration for one isolated installation."""

    environment: dict[str, str]
    calls: Path
    prefix: Path
    archive_name: str


def _fake_installer_commands(binary_dir: Path) -> None:
    """Replace only download, extraction, platform, and toolchain boundaries."""
    _write_executable(
        binary_dir / "uname",
        '#!/bin/sh\ncase "$1" in -s) echo Linux ;; -m) echo "$TEST_ARCH" ;; esac\n',
    )
    _write_executable(
        binary_dir / "curl",
        '#!/bin/sh\nprintf "curl %s\\n" "$*" >> "$CALL_LOG"\n'
        '[ "$FAIL_AT" != download ] || exit 7\n'
        'while [ "$#" -gt 0 ]; do\n'
        '  if [ "$1" = --output ]; then cp "$ARCHIVE_FIXTURE" "$2"; exit; fi\n'
        '  shift\ndone\nexit 8\n',
    )
    _write_executable(
        binary_dir / "tar",
        '#!/bin/sh\nprintf "tar %s\\n" "$*" >> "$CALL_LOG"\n'
        '[ "$FAIL_AT" != extract ] || exit 7\n'
        'while [ "$#" -gt 0 ]; do\n'
        '  if [ "$1" = --directory ]; then\n'
        '    mkdir -p "$2/bin"; printf "installed fixture\\n" > "$2/bin/mold"; exit\n'
        '  fi\n  shift\ndone\nexit 8\n',
    )
    _write_executable(
        binary_dir / "rustup",
        '#!/bin/sh\nprintf "rustup %s\\n" "$*" >> "$CALL_LOG"\n'
        'case "$*" in\n'
        '  "toolchain install "*) [ "$FAIL_AT" != toolchain ] ;;\n'
        '  "component add "*) [ "$FAIL_AT" != components ] ;;\n'
        '  *) exit 8 ;;\nesac\n',
    )


def _installation_environment(
    tmp_path: Path, failure: str, architecture: str = "x86_64"
) -> InstallerEnvironment:
    """Build a closed PATH and local archive/checksum inputs."""
    binary_dir = tmp_path / "bin"
    binary_dir.mkdir()
    for utility in ("bash", "awk", "dirname", "grep", "sha256sum", "mktemp", "rm", "mkdir", "cp"):
        executable = shutil.which(utility)
        assert executable is not None, f"missing test prerequisite {utility}"
        (binary_dir / utility).symlink_to(executable)
    _fake_installer_commands(binary_dir)
    archive = tmp_path / "archive"
    archive.write_bytes(b"local pinned archive fixture")
    name = f"mold-{MOLD_VERSION}-{architecture}-linux.tar.gz"
    digest = "0" * 64 if failure == "checksum" else hashlib.sha256(archive.read_bytes()).hexdigest()
    checksums = tmp_path / "SHA256SUMS"
    checksums.write_text(f"{digest}  {name}\n", encoding="utf-8")
    calls = tmp_path / "calls"
    prefix = tmp_path / "installed"
    environment = {
        **os.environ,
        "PATH": str(binary_dir),
        "ARCHIVE_FIXTURE": str(archive),
        "MOLD_SHA256SUMS_FILE": str(checksums),
        "MOLD_RELEASE_BASE_URL": "https://fixture.invalid/mold",
        "BUILD_TOOLS_PREFIX": str(prefix),
        "CALL_LOG": str(calls),
        "FAIL_AT": failure,
        "TEST_ARCH": architecture,
    }
    return InstallerEnvironment(environment, calls, prefix, name)


@pytest.mark.parametrize("architecture", ["x86_64", "aarch64"])
@pytest.mark.parametrize("entrypoint", ["default", "install-mold"])
def test_installer_uses_the_pinned_asset_and_prefix(
    tmp_path: Path, architecture: str, entrypoint: str
) -> None:
    """Both routes verify the named archive before extracting into the prefix."""
    fixture = _installation_environment(tmp_path, "none", architecture)
    arguments = [str(INSTALL_SCRIPT)]
    if entrypoint == "install-mold":
        arguments = [
            "bash", "-c", 'source "$1"; install_mold "$2"',
            "installer-test", str(INSTALL_SCRIPT), MOLD_VERSION,
        ]
    result = subprocess.run(
        arguments, env=fixture.environment, capture_output=True, text=True, check=False
    )
    assert result.returncode == 0, result.stderr
    calls = fixture.calls.read_text(encoding="utf-8").splitlines()
    assert f"https://fixture.invalid/mold/v{MOLD_VERSION}/{fixture.archive_name}" in calls[0], calls
    assert calls[1].startswith("tar --extract --gzip --strip-components=1"), calls
    assert f"--directory {fixture.prefix}" in calls[1], calls
    assert (fixture.prefix / "bin/mold").read_text(encoding="utf-8") == "installed fixture\n", (
        "the verified archive must be extracted into the installation prefix"
    )
    assert "verified" in result.stderr, result.stderr
    if entrypoint == "default":
        assert calls[2] == f"rustup toolchain install {TOOLCHAIN} --profile minimal", calls
        assert "rustc-codegen-cranelift-preview" in calls[3], calls
        assert "rust-analyzer" in calls[3], calls
    else:
        assert len(calls) == 2, calls


@pytest.mark.parametrize(
    ("failure", "message", "last_command"),
    [
        ("download", "failed to download", "curl"),
        ("checksum", "checksum mismatch", "curl"),
        ("extract", "failed to unpack", "tar"),
        ("toolchain", "failed to install toolchain", "rustup toolchain"),
        ("components", "failed to install components", "rustup component"),
    ],
)
def test_default_installer_preserves_each_boundary_failure(
    tmp_path: Path, failure: str, message: str, last_command: str
) -> None:
    """An earlier failure prevents extraction or later toolchain operations."""
    fixture = _installation_environment(tmp_path, failure)
    result = subprocess.run(
        [str(INSTALL_SCRIPT)], env=fixture.environment, capture_output=True, text=True, check=False
    )
    assert result.returncode != 0, result.stderr
    assert message in result.stderr, result.stderr
    calls = fixture.calls.read_text(encoding="utf-8").splitlines()
    assert calls[-1].startswith(last_command), calls
    if failure in {"download", "checksum", "extract"}:
        assert not (fixture.prefix / "bin/mold").exists(), calls
