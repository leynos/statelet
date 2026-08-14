//! Contract tests for the opt-in dev-fast Make targets.
//!
//! `dev-build` and `dev-test` must honour the injectable `CARGO` variable
//! and pass the dev-fast fragment to Cargo explicitly with `--config`,
//! rather than relying on Cargo auto-discovering `.cargo/config.toml`.
//! These tests parse `make --dry-run` output rather than running a real
//! build, so they stay fast and do not require a nightly toolchain or the
//! mold linker. Invoking `make` is process spawning, which Whitaker's
//! capability-based filesystem rules do not constrain; only file access
//! goes through `camino`.

use std::{io, process::Command};

use camino::{Utf8Path, Utf8PathBuf};
use rstest::rstest;

/// Returns the workspace root, derived from the manifest directory so the
/// test works regardless of the runner's current directory.
fn workspace_root() -> Utf8PathBuf { Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Runs `make --dry-run <target> CARGO=probe-cargo` and returns its stdout.
fn dry_run(target: &str) -> Result<String, io::Error> {
    let output = Command::new("make")
        .current_dir(workspace_root())
        .args(["--dry-run", target, "CARGO=probe-cargo"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Locates `probe-cargo`, `--config`, and the dev-fast fragment marker on a
/// recipe line, returning their byte offsets if all three are present.
fn substitution_offsets(line: &str) -> Option<(usize, usize, usize)> {
    let cargo = line.find("probe-cargo")?;
    let config = line.find("--config")?;
    let dev_fast = line.find("dev-fast")?;
    Some((cargo, config, dev_fast))
}

#[rstest]
#[case::dev_build("dev-build")]
#[case::dev_test("dev-test")]
fn dev_fast_target_substitutes_injected_cargo(#[case] target: &str) {
    let stdout = dry_run(target).expect("make --dry-run should run");

    let Some(recipe_line) = stdout.lines().find(|line| line.contains("probe-cargo")) else {
        panic!("target {target} did not invoke the injected CARGO override:\n{stdout}");
    };

    let Some((cargo_pos, config_pos, dev_fast_pos)) = substitution_offsets(recipe_line) else {
        panic!(
            "target {target} recipe line is missing probe-cargo, --config, or a dev-fast \
             reference:\n{recipe_line}"
        );
    };

    assert!(
        cargo_pos < config_pos && config_pos < dev_fast_pos,
        "target {target} emitted an out-of-order command:\n{recipe_line}"
    );
}

#[test]
fn dev_fast_config_fragment_exists() {
    let config_path: Utf8PathBuf = [
        workspace_root().as_str(),
        "tools",
        "dev-fast",
        "config.toml",
    ]
    .iter()
    .collect();

    assert!(
        Utf8Path::new(&config_path).exists(),
        "{config_path} must exist for the dev-fast targets to function"
    );
}
