//! Contract test for the coverage target's codegen backend.
//!
//! `.cargo/config.toml` selects Cranelift for the dev profile, which is what
//! makes ordinary builds fast. Cranelift cannot instrument coverage: with it
//! active, `cargo llvm-cov` fails outright with "`-Cinstrument-coverage` is
//! LLVM specific and not supported by Cranelift". The `coverage` recipe
//! therefore overrides the backend for that one invocation.
//!
//! Nothing else records that dependency. Removing the override leaves the
//! Makefile looking correct and the failure appears three minutes into a
//! build, so the override is a contract. Like the dev-fast contract beside
//! it, this parses `make --dry-run` output rather than running a build, so it
//! stays fast and needs neither a nightly toolchain nor a linker.

use std::{io, process::Command};

use camino::Utf8PathBuf;

/// The environment assignment the coverage recipe must carry.
const BACKEND_OVERRIDE: &str = "CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm";

/// Returns the workspace root, derived from the manifest directory so the
/// test works regardless of the runner's current directory.
fn workspace_root() -> Utf8PathBuf { Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Runs `make --dry-run coverage CARGO=probe-cargo` and returns its stdout.
fn dry_run() -> Result<String, io::Error> {
    let output = Command::new("make")
        .current_dir(workspace_root())
        .args(["--dry-run", "coverage", "CARGO=probe-cargo"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Joins the coverage recipe's continued lines so one command reads as one
/// string.
///
/// `make --dry-run` prints a continued recipe as several lines ending in a
/// backslash. Rebuilding the command from `lines` rather than slicing the
/// output by byte offset keeps this free of string indexing, which the lint
/// configuration denies.
fn recipe(stdout: &str) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if parts.is_empty() && !opens_recipe(trimmed) {
            continue;
        }
        parts.push(trimmed.trim_end_matches('\\').trim_end());
        if !trimmed.ends_with('\\') {
            break;
        }
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

/// Whether `line` is where the coverage command starts.
///
/// The backend override comes first when it is present; when it is not, the
/// cargo invocation itself is the start, so the caller still has a command to
/// report as missing the override.
fn opens_recipe(line: &str) -> bool {
    line.contains(BACKEND_OVERRIDE) || line.contains("probe-cargo")
}

#[test]
fn coverage_forces_the_llvm_backend_before_invoking_cargo() {
    let stdout = dry_run().expect("make --dry-run should run");
    let Some(command) = recipe(&stdout) else {
        panic!("the coverage target did not invoke the injected CARGO override:\n{stdout}");
    };

    let Some(override_position) = command.find(BACKEND_OVERRIDE) else {
        panic!(
            "the coverage recipe does not set {BACKEND_OVERRIDE}; Cranelift cannot instrument \
             coverage, so `cargo llvm-cov` will fail:\n{command}"
        );
    };
    let Some(cargo_position) = command.find("probe-cargo") else {
        panic!("the coverage recipe does not invoke the injected CARGO:\n{command}");
    };

    assert!(
        override_position < cargo_position,
        "the backend override must precede the cargo invocation it applies to:\n{command}"
    );
}

#[test]
fn the_contract_would_notice_a_recipe_without_the_override() {
    // Mutation check for the assertion above: without it, a `find` that
    // matched nothing would be indistinguishable from a passing contract.
    let without = "\tCFLAGS=\"-fuse-ld=lld\" \\\n\tprobe-cargo llvm-cov --lcov\n";
    let with = "\tCARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm \\\n\tprobe-cargo llvm-cov --lcov\n";

    let joined_without = recipe(without).expect("the fixture invokes the injected CARGO");
    let joined_with = recipe(with).expect("the fixture invokes the injected CARGO");

    assert!(!joined_without.contains(BACKEND_OVERRIDE));
    assert!(joined_with.contains(BACKEND_OVERRIDE));
    assert!(
        joined_with.find(BACKEND_OVERRIDE) < joined_with.find("probe-cargo"),
        "the fixture must place the override before the cargo invocation"
    );
}
