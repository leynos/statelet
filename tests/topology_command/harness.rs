//! Process, fixture, and assertion helpers for the command-boundary suite.
//!
//! `tests/topology_command.rs` holds the three tests; this module holds the
//! machinery they share. Splitting the file keeps the entrypoint readable and
//! the repository's 400-line rule satisfied without changing a single verdict:
//! every helper is the same code, moved.
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

/// The `name` field of one fixture's `[package]` table.
#[derive(Clone, Copy)]
pub(super) struct PackageName(pub(super) &'static str);

impl PackageName {
    /// The package name as it appears in the manifest.
    pub(super) const fn as_str(self) -> &'static str { self.0 }
}

/// The library target a fixture package declares.
///
/// The two tables differ only in the `proc-macro` key, but which one a fixture
/// declares is the policy's subject, so the choice is an enum rather than a
/// free-form manifest fragment that a caller could mistype.
#[derive(Clone, Copy)]
pub(super) enum Library {
    /// An ordinary library target.
    Ordinary,
    /// A procedural-macro library target.
    ProcMacro,
}

impl Library {
    /// The `[lib]` table this target kind is written as.
    const fn table(self) -> &'static str {
        match self {
            Self::Ordinary => "[lib]\npath = \"src/lib.rs\"\n",
            Self::ProcMacro => "[lib]\npath = \"src/lib.rs\"\nproc-macro = true\n",
        }
    }
}

/// One fixture workspace and the verdict its manifest must produce.
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

/// A single-package manifest naming `name`, with `library`, then `trailing`.
///
/// The package holds no registry dependencies, so `cargo metadata --no-deps`
/// answers without a fetch and without compiling the fixture. That is what
/// makes a fixture cheap enough to write per case and safe to run inside a
/// test.
///
/// `trailing` is appended after the library table, which is where a workspace
/// or dependency declaration belongs. The library table itself is emitted
/// exactly once and cannot be replaced by a `trailing` fragment, because Cargo
/// refuses a repeated table with a duplicate-key error — a fixture that
/// intended to exercise the policy would instead fail in the manifest parser.
fn package_manifest(name: PackageName, library: Library, trailing: &str) -> String {
    let package = name.as_str();
    let lib = library.table();
    format!(
        r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2024"

{lib}{trailing}"#
    )
}

/// An empty library source, so a fixture has the file its target names.
pub(super) const EMPTY_LIB: &str = "";

/// An ordinary single-package fixture naming `name`.
pub(super) fn plain_package(name: PackageName, trailing: &str) -> String {
    package_manifest(name, Library::Ordinary, trailing)
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

/// Returns the concrete manifest text for a fixture file.
///
/// `fixtures.rs` states each fixture's shape once; this function turns that
/// shape into manifests. Keeping the two apart means a case's expectation is
/// read beside its construction, while the exact manifest text lives in one
/// place instead of in nine nearly identical string literals.
///
/// An unlisted pair is an error rather than an empty file. A fallback that
/// returned `String::new()` made a missing arm indistinguishable from a
/// deliberately empty fixture: Cargo then rejected the manifest for having no
/// package table, and the case failed with a manifest-parser message that said
/// nothing about the real mistake.
pub(super) fn manifest_for(name: CaseName, relative: &Path) -> Result<String, String> {
    let case = name.as_str();
    let text = match (case, relative.to_str().unwrap_or_default()) {
        ("one-runtime-package" | "path containing spaces", "Cargo.toml")
        | ("runtime-alias-dependency", "runtime/Cargo.toml") => {
            plain_package(PackageName("statelet"), "")
        }
        ("additional-workspace-member", "Cargo.toml") => format!(
            "{}\n[workspace]\nmembers = [\"extra\"]\n",
            plain_package(PackageName("statelet"), "")
        ),
        ("additional-workspace-member", "extra/Cargo.toml") => {
            plain_package(PackageName("statelet-extra"), "")
        }
        ("proc-macro-workspace-member", "Cargo.toml") => format!(
            "{}\n[workspace]\nmembers = [\"macros\"]\n",
            plain_package(PackageName("statelet"), "")
        ),
        ("proc-macro-workspace-member", "macros/Cargo.toml") => {
            package_manifest(PackageName("macros-helper"), Library::ProcMacro, "")
        }
        ("inactive-renamed-macro-edge", "Cargo.toml") => plain_package(
            PackageName("statelet"),
            "\n[dependencies]\nmacros = { package = \"statelet-macros\", path = \"../macros\", \
             optional = true }\n",
        ),
        // The kind is carried into the diagnostic, so one case per kind proves
        // the message a contributor sees, not merely that the edge was seen.
        ("build-kind-macro-edge", "Cargo.toml") => plain_package(
            PackageName("statelet"),
            "\n[build-dependencies]\nstatelet-macros = { path = \"../macros\" }\n",
        ),
        ("runtime-alias-dependency", "Cargo.toml") => plain_package(
            PackageName("statelet"),
            "\n[dependencies]\nruntime = { package = \"statelet\", path = \"runtime\" }\n",
        ),
        ("proc-macro-runtime-package", "Cargo.toml") => {
            package_manifest(PackageName("statelet"), Library::ProcMacro, "")
        }
        ("invalid-manifest", "Cargo.toml") => "[package\nname = \"statelet\"\n".to_owned(),
        // Every library source is empty: the fixture declares a target, and
        // `cargo metadata --no-deps` never compiles it.
        (_, path) if path.ends_with("src/lib.rs") => EMPTY_LIB.to_owned(),
        _ => {
            return Err(format!(
                "fixture {case:?} declares {} but has no manifest for it",
                relative.display()
            ));
        }
    };
    Ok(text)
}

/// Writes every file a fixture declares and returns its manifest path.
///
/// The caller owns the returned directory and removes it once the command has
/// finished, whether the fixture passed or failed.
pub(super) fn materialize(fixture: &Fixture) -> Result<(PathBuf, PathBuf), String> {
    let scratch = unique_scratch_dir(fixture.name);
    Dir::create_ambient_dir_all(&scratch, ambient_authority())
        .map_err(|error| format!("cannot create {}: {error}", scratch.display()))?;
    let directory = Dir::open_ambient_dir(&scratch, ambient_authority())
        .map_err(|error| format!("cannot open {}: {error}", scratch.display()))?;
    for declared in fixture.files {
        let relative = Path::new(declared);
        write_file(&directory, relative, &manifest_for(fixture.name, relative)?)?;
    }
    Ok((scratch.clone(), scratch.join("Cargo.toml")))
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
/// Both entrypoints run Make from the repository root and forward the Cargo
/// executable this test binary was started with. Sharing that setup keeps the
/// two invocations differing only in the arguments that give them their
/// separate meanings.
fn make_command(target: &str) -> Command {
    let mut command = Command::new("make");
    command
        .arg(target)
        .current_dir(repository_root())
        .env("CARGO", OsString::from(CARGO));
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
pub(super) fn check_observable_contract(fixture: &Fixture, manifest: &Path) -> Result<(), String> {
    let output = invoke_target(manifest)?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let status = output.status;
    if status.success() != fixture.should_succeed {
        return Err(format!(
            "{} exited {status}: {combined}",
            fixture.name.as_str()
        ));
    }
    if fixture.should_succeed {
        return if combined.contains("Topology check passed") {
            Ok(())
        } else {
            Err(format!(
                "{} printed no success message: {combined}",
                fixture.name.as_str()
            ))
        };
    }
    if combined.contains("Topology check passed") {
        return Err(format!(
            "{} printed success text: {combined}",
            fixture.name.as_str()
        ));
    }
    if combined.contains(fixture.expected_text) {
        Ok(())
    } else {
        Err(format!(
            "{} did not mention {:?}: {combined}",
            fixture.name.as_str(),
            fixture.expected_text
        ))
    }
}

/// Checks one fixture's observable contract, then removes its root.
pub(super) fn run_case(fixture: &Fixture) -> Result<(), String> {
    let (scratch, manifest) = materialize(fixture)?;
    let outcome = check_observable_contract(fixture, &manifest);
    let cleanup = remove_scratch(&scratch);
    outcome.and(cleanup)
}
