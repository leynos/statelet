"""Focused contracts for the pinned build-tool provisioning scripts."""

import hashlib
import os
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CHECK_SCRIPT = ROOT / "scripts" / "check-build-tools.sh"
INSTALL_SCRIPT = ROOT / "scripts" / "install-build-tools.sh"
MOLD_VERSION = (ROOT / "tools" / "mold" / "VERSION").read_text("utf-8").strip()
TOOLCHAIN = tomllib.loads((ROOT / "rust-toolchain.toml").read_text("utf-8"))[
    "toolchain"
]["channel"]


@dataclass(frozen=True, slots=True)
class EnvironmentOptions:
    """Overrides for one fake build-tool environment."""

    mold_version: str | None = MOLD_VERSION
    clang_works: bool = True
    toolchain: str = f"{TOOLCHAIN}-x86_64-unknown-linux-gnu"
    components: tuple[str, ...] | None = None


def _write_executable(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8")
    path.chmod(0o755)


def _check_environment(
    tmp_path: Path, options: EnvironmentOptions = EnvironmentOptions()
) -> dict[str, str]:
    """Create a closed PATH with deterministic Linux prerequisite commands."""
    binary_dir = tmp_path / "bin"
    binary_dir.mkdir()
    for utility in ("bash", "awk", "dirname"):
        resolved = subprocess.run(
            ["which", utility], capture_output=True, check=True, text=True
        ).stdout.strip()
        (binary_dir / utility).symlink_to(resolved)

    _write_executable(binary_dir / "uname", "#!/bin/sh\nprintf 'Linux\\n'\n")
    _write_executable(
        binary_dir / "rustup",
        "#!/bin/sh\n"
        "case \"$*\" in\n"
        "  *'toolchain list'*) printf '%s\\n' \"${RUSTUP_TEST_TOOLCHAIN}\" ;;\n"
        "  *'component list'*) printf '%s\\n' \"${RUSTUP_TEST_COMPONENTS}\" ;;\n"
        "  *) exit 2 ;;\n"
        "esac\n",
    )
    if options.mold_version is not None:
        _write_executable(
            binary_dir / "mold",
            "#!/bin/sh\n"
            "[ \"${1:-}\" = --version ] || exit 2\n"
            "printf 'mold %s (test binary)\\n' \"$MOLD_TEST_VERSION\"\n",
        )
    if options.clang_works:
        _write_executable(
            binary_dir / "clang",
            "#!/bin/sh\n[ \"${1:-}\" = --version ] && exit 0\nexit 2\n",
        )

    installed_components = options.components
    if installed_components is None:
        manifest_components = tomllib.loads(
            (ROOT / "rust-toolchain.toml").read_text("utf-8")
        )["toolchain"].get("components", [])
        installed_components = tuple(
            f"{component.removesuffix('-preview')}-x86_64-unknown-linux-gnu"
            for component in manifest_components
        )
    return {
        **os.environ,
        "PATH": str(binary_dir),
        "MOLD_TEST_VERSION": options.mold_version or "",
        "RUSTUP_TEST_TOOLCHAIN": options.toolchain,
        "RUSTUP_TEST_COMPONENTS": "\n".join(installed_components),
    }


def _run_check(
    tmp_path: Path, options: EnvironmentOptions = EnvironmentOptions()
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(CHECK_SCRIPT)],
        capture_output=True,
        check=False,
        env=_check_environment(tmp_path, options),
        text=True,
    )


class BuildToolsScriptTests(unittest.TestCase):
    """Exercise script checks with fake tools and no Cargo invocation."""

    def setUp(self) -> None:
        self.tmp_path = Path(tempfile.mkdtemp(prefix="statelet-build-tools-"))
        self.addCleanup(shutil.rmtree, self.tmp_path, ignore_errors=True)

    def test_check_accepts_the_pinned_linker_toolchain_and_components(self) -> None:
        """Accept the complete set of pinned local build tools."""
        result = _run_check(self.tmp_path)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"mold {MOLD_VERSION}", result.stderr)
        self.assertIn(f"toolchain {TOOLCHAIN} available", result.stderr)
        self.assertIn("all rust-toolchain.toml components are installed", result.stderr)

    def test_check_rejects_missing_linker_with_install_instructions(self) -> None:
        """Report the missing linker and the supported installation target."""
        result = _run_check(self.tmp_path, EnvironmentOptions(mold_version=None))

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("mold not found on PATH", result.stderr)
        self.assertIn("make install-build-tools", result.stderr)

    def test_check_rejects_a_linker_version_that_differs_from_the_pin(self) -> None:
        """Reject a linker whose version differs from the repository pin."""
        result = _run_check(self.tmp_path, EnvironmentOptions(mold_version="2.40.0"))

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(f"does not match the pin {MOLD_VERSION}", result.stderr)

    def test_check_rejects_missing_clang_and_names_the_external_prerequisite(self) -> None:
        """Name the missing Linux Clang prerequisite."""
        result = _run_check(self.tmp_path, EnvironmentOptions(clang_works=False))

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("clang not found on PATH", result.stderr)
        self.assertIn("Linux clang runner prerequisite", result.stderr)

    def test_check_rejects_a_missing_pinned_toolchain(self) -> None:
        """Reject a Rust toolchain different from the pinned nightly."""
        result = _run_check(
            self.tmp_path,
            EnvironmentOptions(toolchain="stable-x86_64-unknown-linux-gnu"),
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(f"toolchain {TOOLCHAIN} is not installed", result.stderr)
        self.assertIn("make install-build-tools", result.stderr)

    def test_check_reads_component_names_from_the_toolchain_manifest(self) -> None:
        """Require every component declared by the selected toolchain."""
        manifest = self.tmp_path / "rust-toolchain.toml"
        manifest.write_text(
            f'[toolchain]\nchannel = "{TOOLCHAIN}"\n'
            'components = ["clippy", "rust-analyzer"]\n',
            encoding="utf-8",
        )
        env = _check_environment(
            self.tmp_path,
            EnvironmentOptions(components=("clippy-x86_64-unknown-linux-gnu",)),
        )
        env["RUST_TOOLCHAIN_FILE"] = str(manifest)
        result = subprocess.run(
            [str(CHECK_SCRIPT)], capture_output=True, check=False, env=env, text=True
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("toolchain component rust-analyzer is not installed", result.stderr)

    def _run_checksum_check(self, rows: str) -> subprocess.CompletedProcess[str]:
        """Run the installer's archive verifier against a controlled checksum file."""
        archive = self.tmp_path / "mold-fixture.tar.gz"
        archive.write_bytes(b"trusted fixture bytes")
        sums_file = self.tmp_path / "SHA256SUMS"
        sums_file.write_text(rows, encoding="utf-8")
        return subprocess.run(
            [
                "bash",
                "-c",
                'source "$1"; verify_mold_archive "$2" "$3"',
                "build-tools-test",
                str(INSTALL_SCRIPT),
                str(archive),
                archive.name,
            ],
            capture_output=True,
            check=False,
            env={**os.environ, "MOLD_SHA256SUMS_FILE": str(sums_file)},
            text=True,
        )

    def test_installer_checksum_verification_accepts_the_matching_archive(self) -> None:
        """Accept the archive when its digest matches the published checksum."""
        archive = self.tmp_path / "mold-fixture.tar.gz"
        archive.write_bytes(b"trusted fixture bytes")
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()

        result = self._run_checksum_check(f"{digest}  {archive.name}\n")

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("verified mold-fixture.tar.gz", result.stderr)

    def test_installer_checksum_verification_rejects_mismatch_and_ambiguity(self) -> None:
        """Reject incorrect digests and ambiguous duplicate checksum records."""
        wrong_digest = "0" * 64
        wrong = self._run_checksum_check(
            f"{wrong_digest}  mold-fixture.tar.gz\n"
        )
        duplicate = self._run_checksum_check(
            f"{wrong_digest}  mold-fixture.tar.gz\n"
            f"{wrong_digest}  mold-fixture.tar.gz\n"
        )

        self.assertNotEqual(wrong.returncode, 0)
        self.assertIn("checksum mismatch", wrong.stderr)
        self.assertNotEqual(duplicate.returncode, 0)
        self.assertIn("checksums recorded", duplicate.stderr)

    def test_installer_adds_every_manifest_component_to_the_pinned_toolchain(self) -> None:
        """Install all manifest components after installing the pinned toolchain."""
        manifest = self.tmp_path / "rust-toolchain.toml"
        manifest.write_text(
            f'[toolchain]\nchannel = "{TOOLCHAIN}"\n'
            'components = ["clippy", "llvm-tools-preview", '
            '"rustc-codegen-cranelift-preview", "rustfmt", "rust-analyzer"]\n',
            encoding="utf-8",
        )
        binary_dir = self.tmp_path / "installer-bin"
        binary_dir.mkdir()
        call_log = self.tmp_path / "rustup-calls"
        _write_executable(
            binary_dir / "rustup",
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$RUSTUP_CALL_LOG\"\n",
        )
        env = {
            **os.environ,
            "PATH": f"{binary_dir}:{os.environ['PATH']}",
            "RUST_TOOLCHAIN_FILE": str(manifest),
            "RUSTUP_CALL_LOG": str(call_log),
        }
        result = subprocess.run(
            [
                "bash",
                "-c",
                'source "$1"; install_toolchain "$2"',
                "build-tools-test",
                str(INSTALL_SCRIPT),
                TOOLCHAIN,
            ],
            capture_output=True,
            check=False,
            env=env,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            call_log.read_text("utf-8").splitlines(),
            [
                f"toolchain install {TOOLCHAIN} --profile minimal",
                "component add --toolchain "
                f"{TOOLCHAIN} clippy llvm-tools-preview "
                "rustc-codegen-cranelift-preview rustfmt rust-analyzer",
            ],
        )
