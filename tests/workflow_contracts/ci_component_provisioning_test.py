"""Keep shared CI linker installation separate from local Rust components."""

import copy
import os
import shutil
import subprocess
import tomllib
from pathlib import Path

import pytest

from suite_discovery import suite_findings
from suite_provisioning import INSTALL_COMMAND, load_workflows

ROOT = Path(__file__).resolve().parents[2]


@pytest.mark.parametrize("mutation", ["late-setup", "missing-linker", "duplicate-install"])
def test_ci_component_route_rejects_provisioning_regressions(mutation: str) -> None:
    """Missing, late, or duplicate linker installation cannot satisfy CI."""
    workflows = copy.deepcopy(load_workflows())
    steps = workflows["ci.yml"]["jobs"]["build-test"]["steps"]
    setup = next(step for step in steps if "setup-rust@" in step.get("uses", ""))
    components = next(step for step in steps if step.get("run") == INSTALL_COMMAND)
    match mutation:
        case "late-setup":
            steps.remove(setup)
            steps.insert(steps.index(components) + 1, setup)
        case "missing-linker":
            setup["with"].pop("install-mold")
        case "duplicate-install":
            steps.insert(steps.index(components), {"run": "make install-build-tools"})
    findings = suite_findings(workflows)
    assert any("ci.yml:build-test" in finding for finding in findings), findings


@pytest.mark.parametrize("failure", ["none", "toolchain", "components"])
def test_component_only_installer_is_binding_and_never_downloads_mold(
    tmp_path: Path, failure: str
) -> None:
    """Exercise the real entrypoint with no linker/download tools on PATH."""
    binary_dir = tmp_path / "bin"
    binary_dir.mkdir()
    for utility in ("bash", "awk", "dirname"):
        executable = shutil.which(utility)
        assert executable is not None, f"missing test prerequisite {utility}"
        (binary_dir / utility).symlink_to(executable)
    rustup = binary_dir / "rustup"
    rustup.write_text(
        '#!/bin/sh\n'
        'printf "%s\\n" "$*" >> "$CALL_LOG"\n'
        'case "$*" in\n'
        '  "toolchain install "*) [ "$FAIL_AT" != toolchain ] ;;\n'
        '  "component add "*) [ "$FAIL_AT" != components ] ;;\n'
        '  *) exit 7 ;;\n'
        'esac\n',
        encoding="utf-8",
    )
    rustup.chmod(0o755)
    calls = tmp_path / "calls"
    result = subprocess.run(
        [str(ROOT / "scripts/install-build-tools.sh"), "--toolchain-only"],
        env={**os.environ, "PATH": str(binary_dir), "CALL_LOG": str(calls), "FAIL_AT": failure},
        capture_output=True,
        text=True,
        check=False,
    )
    assert (result.returncode == 0) is (failure == "none"), result.stderr
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text("utf-8"))["toolchain"]
    expected_calls = [
        ["toolchain", "install", toolchain["channel"], "--profile", "minimal"],
    ]
    if failure != "toolchain":
        expected_calls.append(
            ["component", "add", "--toolchain", toolchain["channel"], *toolchain["components"]]
        )
    recorded_calls = [line.split() for line in calls.read_text(encoding="utf-8").splitlines()]
    assert recorded_calls == expected_calls, (
        f"component-only installation changed the pinned commands or their order: {recorded_calls}"
    )
    assert "downloading" not in result.stderr, result.stderr
