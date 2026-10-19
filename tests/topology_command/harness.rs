//! Process, fixture, and assertion helpers for the command-boundary suite.
//!
//! `tests/topology_command.rs` holds the three tests; this module holds the
//! process machinery they share. The manifest *text* lives beside it in
//! `manifests.rs`, because deciding what a fixture contains and running a
//! process against it are separate responsibilities and the pair together
//! breached the repository's 400-line rule. Every helper is the same code,
//! moved; no verdict changed.
//!
//! Fixtures are written through a capability handle rather than ambient
//! filesystem calls, so a path escaping the scratch directory is a refused
//! capability instead of a write somewhere else. Each invocation owns a unique
//! root and removes it on both outcomes.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use cap_std::{ambient_authority, fs::Dir};

use super::manifests::{EMPTY_LIB, PackageName, manifest_for, plain_package};

/// The Cargo executable this test binary was started with.
///
/// `CARGO` is a compile-time constant Cargo exports to every process it
/// starts, so reading it here costs no ambient environment access and cannot
/// disagree with the executable that selected this test.
pub(super) const CARGO: &str = env!("CARGO");

/// The name of one fixture case.
///
/// The case name is both the scratch directory a fixture owns and the word its
/// failure messages use, so it is not interchangeable with a Cargo package name
/// or a path even though all three travel as strings.
#[derive(Clone, Copy)]
pub(super) struct CaseName(pub(super) &'static str);

impl CaseName {
    /// The case name as written in the fixture table.
    pub(super) const fn as_str(self) -> &'static str { self.0 }
}

/// One fixture workspace and the verdict its manifest must produce.
///
/// The shape stays here rather than in `manifests.rs`: a fixture's expected
/// verdict is what the command boundary asserts, while the manifest module only
/// answers what text a declared file contains. Splitting on that line is what
/// separates the two files' responsibilities.
pub(super) struct Fixture {
    /// The case name, used as the scratch directory name and in messages.
    pub(super) name: CaseName,
    /// Every file the fixture writes, relative to its scratch directory.
    pub(super) files: &'static [&'static str],
    /// Whether the command is expected to exit zero.
    pub(super) should_succeed: bool,
    /// Text the failure output must contain, empty when success is expected.
    pub(super) expected_text: &'static str,
}

/// The repository root the target is invoked from.
fn repository_root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// The parent directory every fixture invocation owns one child of.
///
/// The parent is shared and never removed; each invocation's own leaf is
/// unique. A stale leaf from an interrupted earlier run therefore cannot leak a
/// file into a later fixture, and two invocations cannot write the same paths.
fn scratch_root() -> PathBuf { Path::new(env!("CARGO_TARGET_TMPDIR")).join("topology-command") }

/// A fixture root unique to this invocation.
///
/// The process id alone would repeat across enough runs to collide, so the
/// current time in nanoseconds joins it. The pair is unique for the lifetime
/// of a test binary and needs no shared counter.
pub(super) fn unique_scratch_dir(name: CaseName) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    scratch_root().join(format!("{}-{}-{nanos}", name.as_str(), process::id()))
}

/// Writes every file a fixture declares and returns its manifest path.
///
/// Cleanup is owned from the moment the root exists, not from the moment the
/// caller receives it. Materialization can fail *after* the directory is
/// created — opening it, or writing a later file, may refuse — and returning
/// that error with `?` would otherwise leave a half-written tree behind for the
/// next run to inherit. Every failure path therefore removes the root it
/// created before reporting what went wrong, and a cleanup refusal is appended
/// to the original error rather than replacing it.
pub(super) fn materialize(fixture: &Fixture) -> Result<(PathBuf, PathBuf), String> {
    let scratch = unique_scratch_dir(fixture.name);
    let manifest = scratch.join("Cargo.toml");
    let outcome = write_fixture(fixture, &scratch);
    match outcome {
        Ok(()) => Ok((scratch, manifest)),
        Err(error) => Err(append_cleanup(error, remove_scratch(&scratch))),
    }
}

/// Creates one fixture root and writes every file it declares into it.
fn write_fixture(fixture: &Fixture, scratch: &Path) -> Result<(), String> {
    Dir::create_ambient_dir_all(scratch, ambient_authority())
        .map_err(|error| format!("cannot create {}: {error}", scratch.display()))?;
    let directory = Dir::open_ambient_dir(scratch, ambient_authority())
        .map_err(|error| format!("cannot open {}: {error}", scratch.display()))?;
    fixture.files.iter().try_for_each(|declared| {
        let relative = Path::new(declared);
        let contents = manifest_for(fixture.name, relative)?;
        write_file(&directory, relative, &contents)
    })
}

/// Writes the V5 pin control's two-member workspace under `root`.
///
/// The control needs a tree whose manifest Cargo can load but whose shape the
/// policy refuses, so a manifest naming a member with no manifest of its own
/// would fail during acquisition instead. Cleanup follows the same rule as
/// [`materialize`]: the root is removed on a write failure, so the caller never
/// inherits a partially written tree.
pub(super) fn write_pinned_fixture(root: &Path, forbidden: &str) -> Result<(), String> {
    let outcome = (|| {
        Dir::create_ambient_dir_all(root, ambient_authority())
            .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
        let directory = Dir::open_ambient_dir(root, ambient_authority())
            .map_err(|error| format!("cannot open {}: {error}", root.display()))?;
        write_file(&directory, Path::new("Cargo.toml"), forbidden)?;
        write_file(&directory, Path::new("src/lib.rs"), EMPTY_LIB)?;
        write_file(
            &directory,
            Path::new("extra/Cargo.toml"),
            &plain_package(PackageName("statelet-extra"), ""),
        )?;
        write_file(&directory, Path::new("extra/src/lib.rs"), EMPTY_LIB)
    })();
    match outcome {
        Ok(()) => Ok(()),
        Err(error) => Err(append_cleanup(error, remove_scratch(root))),
    }
}

/// Reports a cleanup refusal beside the error that made cleanup necessary.
fn append_cleanup(error: String, cleanup: Result<(), String>) -> String {
    match cleanup {
        Ok(()) => error,
        Err(cleanup_error) => format!("{error}; additionally, {cleanup_error}"),
    }
}

/// Writes one fixture file, creating its parent directory first.
pub(super) fn write_file(directory: &Dir, relative: &Path, contents: &str) -> Result<(), String> {
    let Some(parent) = relative.parent() else {
        return Ok(());
    };
    let name = relative.to_str().unwrap_or_default();
    directory
        .create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    directory
        .write(name, contents)
        .map_err(|error| format!("cannot write {name}: {error}"))
}

/// Starts one Make invocation of `target`, ready for its own arguments.
///
/// Both entrypoints run Make from the repository root, forward the Cargo
/// executable this test binary was started with, and hand Make the caller's
/// original `RUSTFLAGS`.
///
/// The restore matters because this binary is started by `cargo nextest run`,
/// which passes the composed `RUSTFLAGS` — the caller's value plus the
/// repository flags — to the test process. A fixture case then runs
/// `make topology-check`, whose recipe composes `RUSTFLAGS` again, so passing
/// the inherited value would make every repository flag appear twice. This
/// toolchain tolerates that duplication: a nested run with the composed value
/// still compiles, which was measured rather than assumed. The copy is kept
/// anyway so composition stays idempotent — a caller flag that is not
/// duplication-safe would otherwise change meaning in the nested build.
/// `test-topology-command` therefore copies the caller's value beside the
/// composed one, and that copy is preferred here.
///
/// `RUSTFLAGS` is set from whichever value is found, or removed from the child
/// when neither is present, rather than left to inherit. `None` covers a direct
/// `cargo test` invocation, where inheriting the composed value would reintroduce
/// the duplication. The copy ends in `FLAGS`, not `RUSTFLAGS`, because the build
/// standard's contract reads the first `RUSTFLAGS="` on an evaluated recipe line.
///
/// The environment read is a scoped, reason-bearing exemption from the
/// repository's `disallowed_methods` policy: the caller's flags can only arrive
/// through the environment of this process, exactly as in the acquisition
/// adapter, and no caller can inject them into a test binary.
#[expect(
    clippy::disallowed_methods,
    reason = "This module is the command boundary: the caller's RUSTFLAGS and the composed copy \
              reach the test binary only through this process's environment. The developer guide \
              records the scope."
)]
fn make_command(target: &str) -> Command {
    let rustflags = std::env::var_os("STATELET_TOPOLOGY_CALLER_FLAGS")
        .or_else(|| std::env::var_os("RUSTFLAGS"));
    let mut command = Command::new("make");
    command
        .arg(target)
        .current_dir(repository_root())
        .env("CARGO", OsString::from(CARGO));
    match rustflags {
        Some(inherited) => command.env("RUSTFLAGS", inherited),
        None => command.env_remove("RUSTFLAGS"),
    };
    command
}

/// Runs the Make target against one fixture and returns its combined output.
pub(super) fn invoke_target(manifest: &Path) -> Result<process::Output, String> {
    make_command("topology-check")
        .arg(format!("TOPOLOGY_MANIFEST_PATH={}", manifest.display()))
        .output()
        .map_err(|error| format!("cannot run make: {error}"))
}

/// Runs `check-topology` from an explicit root with an explicit override.
///
/// Separate from [`invoke_target`] because this case supplies its own
/// `CURDIR`: the target's pin is written as `$(CURDIR)/Cargo.toml`, so a
/// caller-visible root is the only way to show that an inherited
/// `TOPOLOGY_MANIFEST_PATH` is overwritten rather than honoured.
pub(super) fn invoke_pinned(root: &Path, supplied: &Path) -> Result<process::Output, String> {
    make_command("check-topology")
        .arg(format!("TOPOLOGY_MANIFEST_PATH={}", supplied.display()))
        .arg(format!("CURDIR={}", root.display()))
        .output()
        .map_err(|error| format!("cannot run make check-topology: {error}"))
}

/// Removes one fixture root, reporting a refusal rather than hiding it.
///
/// Cleanup runs on both outcomes: a leftover tree is how a later invocation
/// inherits stale files, and a silent failure here would be discovered only as
/// a confusing later result.
pub(super) fn remove_scratch(root: &Path) -> Result<(), String> {
    let parent = root
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", root.display()))?;
    let name = root
        .file_name()
        .and_then(|leaf| leaf.to_str())
        .ok_or_else(|| format!("{} has no usable directory name", root.display()))?;
    Dir::open_ambient_dir(parent, ambient_authority())
        .and_then(|directory| directory.remove_dir_all(name))
        .map_err(|error| format!("cannot remove {}: {error}", root.display()))
}

/// Checks the combined output against an expected verdict and rule.
pub(super) fn check_verdict(
    case: &str,
    output: &process::Output,
    should_succeed: bool,
    expected_text: &str,
) -> Result<(), String> {
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let status = output.status;
    if status.success() != should_succeed {
        return Err(format!("{case} exited {status}: {combined}"));
    }
    let printed_success = combined.contains("Topology check passed");
    if should_succeed {
        return if printed_success {
            Ok(())
        } else {
            Err(format!("{case} printed no success message: {combined}"))
        };
    }
    if printed_success {
        return Err(format!("{case} printed success text: {combined}"));
    }
    if combined.contains(expected_text) {
        Ok(())
    } else {
        Err(format!(
            "{case} did not mention {expected_text:?}: {combined}"
        ))
    }
}

/// Checks status and message of one already-materialized fixture.
///
/// The verdict is [`check_verdict`] over the fixture's own expectations; this
/// helper only supplies the process and the words it is judged against.
pub(super) fn check_observable_contract(fixture: &Fixture, manifest: &Path) -> Result<(), String> {
    let output = invoke_target(manifest)?;
    check_verdict(
        fixture.name.as_str(),
        &output,
        fixture.should_succeed,
        fixture.expected_text,
    )
}

/// Checks one fixture's observable contract, then removes its root.
///
/// A materialization failure already cleaned up after itself, so the `?` here
/// returns a tree-free error rather than skipping cleanup.
pub(super) fn run_case(fixture: &Fixture) -> Result<(), String> {
    let (scratch, manifest) = materialize(fixture)?;
    let outcome = check_observable_contract(fixture, &manifest);
    let cleanup = remove_scratch(&scratch);
    outcome.and(cleanup)
}
