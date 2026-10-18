//! Command-boundary cases for `make topology-check`.
//!
//! The unit, property, and behavioural suites feed the policy a document in
//! memory. This target is the only place the *command* is exercised: it writes
//! isolated Cargo fixtures to a scratch directory, runs the real Make target
//! against each one, and asserts the process boundary — exit status, the
//! absence of success text on failure, and the presence of a repair hint.
//!
//! Every case here is `#[ignore]`d, and the reason is the gate rather than the
//! test. A routine or coverage run discovers these tests too, and each one
//! starts a nested Cargo process; the nested run would compile while the outer
//! binary holds the build directory, and under coverage instrumentation the
//! child would inherit instrumentation it was never meant to have. The
//! dedicated `make test-topology-command` gate runs them after the ordinary
//! tests and after coverage, one thread at a time, with
//! `--run-ignored only --no-tests fail`.
//!
//! The fixture path reaches the command through `TOPOLOGY_MANIFEST_PATH`, the
//! Make variable the recipe forwards, not through the environment: the recipe
//! assigns `STATELET_TOPOLOGY_MANIFEST_PATH` itself from that variable, so an
//! environment value would be overwritten and every case would quietly check
//! the repository manifest instead. A per-case make argument is also the only
//! way to vary the input without mutating process-global state.
//!
//! Fixtures are written through a capability handle rather than ambient
//! filesystem calls, so a path escaping the scratch directory is a refused
//! capability instead of a write somewhere else.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use cap_std::{ambient_authority, fs::Dir};

/// The Cargo executable this test binary was started with.
///
/// `CARGO` is a compile-time constant Cargo exports to every process it
/// starts, so reading it here costs no ambient environment access and cannot
/// disagree with the executable that selected this test.
const CARGO: &str = env!("CARGO");

/// One fixture workspace and the verdict its manifest must produce.
struct Fixture {
    /// The case name, used as the scratch directory name and in messages.
    name: &'static str,
    /// Every file the fixture writes, relative to its scratch directory.
    files: &'static [&'static str],
    /// Whether the command is expected to exit zero.
    should_succeed: bool,
    /// Text the failure output must contain, empty when success is expected.
    expected_text: &'static str,
}

/// A single-package manifest naming `name`, with a library target.
///
/// The package holds no registry dependencies, so `cargo metadata --no-deps`
/// answers without a fetch and without compiling the fixture. That is what
/// makes a fixture cheap enough to write per case and safe to run inside a
/// test.
///
/// `trailing` is appended after the library table, which is where a workspace
/// or dependency declaration belongs. The library table itself is emitted
/// exactly once and only [`proc_macro_lib`] may replace it, because Cargo
/// refuses a repeated table with a duplicate-key error — a fixture that
/// intended to exercise the policy would instead fail in the manifest parser.
fn package_manifest(name: &str, library: &str, trailing: &str) -> String {
    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

{library}{trailing}"#
    )
}

/// The ordinary library table.
const fn library() -> &'static str { "[lib]\npath = \"src/lib.rs\"\n" }

/// The procedural-macro library table.
const fn proc_macro_lib() -> &'static str { "[lib]\npath = \"src/lib.rs\"\nproc-macro = true\n" }

/// An empty library source, so a fixture has the file its target names.
const EMPTY_LIB: &str = "";

/// An ordinary single-package fixture naming `name`.
fn plain_package(name: &str, trailing: &str) -> String {
    package_manifest(name, library(), trailing)
}

/// The cases the command boundary must hold.
///
/// The first four are the plan's acceptance list; `run_case` exercises each
/// through the complete command. The next three are the unit suite's
/// partitions carried across the process boundary — a proc-macro package under
/// the runtime name, a build-kind declaration, an alias to the runtime package
/// itself — because a verdict proven only in memory says nothing about the
/// messages a contributor actually sees. The last two add the boundary
/// conditions a passing-only suite cannot see: a scratch path containing a
/// space, and a manifest Cargo refuses to parse.
///
/// A dangling-member fixture is deliberately absent: Cargo refuses to load a
/// declared member without a manifest, so the policy's `DanglingMember` branch
/// is reachable only from a handwritten document. It stays covered by the
/// in-memory partition instead of being faked here.
fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "one-runtime-package",
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: true,
            expected_text: "",
        },
        Fixture {
            name: "additional-workspace-member",
            files: &["Cargo.toml", "extra/Cargo.toml", "extra/src/lib.rs"],
            should_succeed: false,
            expected_text: "remove the extra member",
        },
        Fixture {
            name: "proc-macro-workspace-member",
            files: &["Cargo.toml", "macros/Cargo.toml", "macros/src/lib.rs"],
            should_succeed: false,
            expected_text: "remove the extra member",
        },
        Fixture {
            name: "inactive-renamed-macro-edge",
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: false,
            expected_text: "statelet-macros",
        },
        Fixture {
            name: "proc-macro-runtime-package",
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: false,
            expected_text: "declares a proc-macro target",
        },
        Fixture {
            name: "build-kind-macro-edge",
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: false,
            // The child's message passes through nextest's own diagnosis, so a
            // quoted needle would have to encode another tool's escaping. The
            // unquoted phrase identifies the verdict and the declaration kind
            // without depending on how that layer prints quotes.
            expected_text: "declares a build dependency on",
        },
        Fixture {
            name: "runtime-alias-dependency",
            files: &[
                "Cargo.toml",
                "src/lib.rs",
                "runtime/Cargo.toml",
                "runtime/src/lib.rs",
            ],
            should_succeed: false,
            expected_text: "declares a normal dependency on",
        },
        Fixture {
            name: "path containing spaces",
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: true,
            expected_text: "",
        },
        Fixture {
            name: "invalid-manifest",
            files: &["Cargo.toml"],
            should_succeed: false,
            expected_text: "metadata failed",
        },
    ]
}

/// Returns the concrete manifest text for a fixture file.
///
/// The table above states each fixture's shape once; this function turns that
/// shape into manifests. Keeping the two apart means a case's expectation is
/// read beside its construction, while the exact manifest text lives in one
/// place instead of in nine nearly identical string literals.
///
/// An unlisted pair is an error rather than an empty file. A fallback that
/// returned `String::new()` made a missing arm indistinguishable from a
/// deliberately empty fixture: Cargo then rejected the manifest for having no
/// package table, and the case failed with a manifest-parser message that said
/// nothing about the real mistake.
fn manifest_for(name: &str, relative: &str) -> Result<String, String> {
    let text = match (name, relative) {
        ("one-runtime-package" | "path containing spaces", "Cargo.toml") => {
            plain_package("statelet", "")
        }
        ("additional-workspace-member", "Cargo.toml") => format!(
            "{}\n[workspace]\nmembers = [\"extra\"]\n",
            plain_package("statelet", "")
        ),
        ("additional-workspace-member", "extra/Cargo.toml") => plain_package("statelet-extra", ""),
        ("proc-macro-workspace-member", "Cargo.toml") => format!(
            "{}\n[workspace]\nmembers = [\"macros\"]\n",
            plain_package("statelet", "")
        ),
        ("proc-macro-workspace-member", "macros/Cargo.toml") => {
            package_manifest("macros-helper", proc_macro_lib(), "")
        }
        ("inactive-renamed-macro-edge", "Cargo.toml") => plain_package(
            "statelet",
            "\n[dependencies]\nmacros = { package = \"statelet-macros\", path = \"../macros\", \
             optional = true }\n",
        ),
        // The kind is carried into the diagnostic, so one case per kind proves
        // the message a contributor sees, not merely that the edge was seen.
        ("build-kind-macro-edge", "Cargo.toml") => plain_package(
            "statelet",
            "\n[build-dependencies]\nstatelet-macros = { path = \"../macros\" }\n",
        ),
        ("runtime-alias-dependency", "Cargo.toml") => plain_package(
            "statelet",
            "\n[dependencies]\nruntime = { package = \"statelet\", path = \"runtime\" }\n",
        ),
        ("runtime-alias-dependency", "runtime/Cargo.toml") => plain_package("statelet", ""),
        ("proc-macro-runtime-package", "Cargo.toml") => {
            package_manifest("statelet", proc_macro_lib(), "")
        }
        ("invalid-manifest", "Cargo.toml") => "[package\nname = \"statelet\"\n".to_owned(),
        // Every library source is empty: the fixture declares a target, and
        // `cargo metadata --no-deps` never compiles it.
        (_, path) if path.ends_with("src/lib.rs") => EMPTY_LIB.to_owned(),
        _ => {
            return Err(format!(
                "fixture {name:?} declares {relative:?} but has no manifest for it"
            ));
        }
    };
    Ok(text)
}

/// The repository root the target is invoked from.
fn repository_root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// The scratch directory one fixture owns.
fn scratch_dir(fixture: &Fixture) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("topology-command")
        .join(fixture.name)
}

/// Writes every file a fixture declares and returns its manifest path.
fn materialise(fixture: &Fixture) -> Result<PathBuf, String> {
    let scratch = scratch_dir(fixture);
    Dir::create_ambient_dir_all(&scratch, ambient_authority())
        .map_err(|error| format!("cannot create {}: {error}", scratch.display()))?;
    let directory = Dir::open_ambient_dir(&scratch, ambient_authority())
        .map_err(|error| format!("cannot open {}: {error}", scratch.display()))?;
    for relative in fixture.files {
        write_file(&directory, relative, &manifest_for(fixture.name, relative)?)?;
    }
    Ok(scratch.join("Cargo.toml"))
}

/// Writes one fixture file, creating its parent directory first.
fn write_file(directory: &Dir, relative: &str, contents: &str) -> Result<(), String> {
    let Some(parent) = Path::new(relative).parent() else {
        return Ok(());
    };
    directory
        .create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    directory
        .write(relative, contents)
        .map_err(|error| format!("cannot write {relative}: {error}"))
}

/// Runs the Make target against one fixture and returns its combined output.
fn invoke_target(manifest: &Path) -> Result<std::process::Output, String> {
    Command::new("make")
        .arg("topology-check")
        .arg(format!("TOPOLOGY_MANIFEST_PATH={}", manifest.display()))
        .current_dir(repository_root())
        .env("CARGO", OsString::from(CARGO))
        .output()
        .map_err(|error| format!("cannot run make: {error}"))
}

/// Checks one fixture's observable contract.
fn run_case(fixture: &Fixture) -> Result<(), String> {
    let manifest = materialise(fixture)?;
    let output = invoke_target(&manifest)?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let status = output.status;
    if status.success() != fixture.should_succeed {
        return Err(format!("{} exited {status}: {combined}", fixture.name));
    }
    if fixture.should_succeed {
        return if combined.contains("Topology check passed") {
            Ok(())
        } else {
            Err(format!(
                "{} printed no success message: {combined}",
                fixture.name
            ))
        };
    }
    if combined.contains("Topology check passed") {
        return Err(format!("{} printed success text: {combined}", fixture.name));
    }
    if combined.contains(fixture.expected_text) {
        Ok(())
    } else {
        Err(format!(
            "{} did not mention {:?}: {combined}",
            fixture.name, fixture.expected_text
        ))
    }
}

/// Every fixture produces the verdict and text the contract promises.
///
/// Failures are collected rather than short-circuited so one run reports every
/// disagreeing fixture, which keeps a diagnosis from needing one gate run per
/// case.
#[test]
#[ignore = "run by `make test-topology-command`, which nests a Cargo invocation; routine and \
            coverage runs must not start it"]
fn every_fixture_produces_its_documented_verdict() -> Result<(), String> {
    let failures: Vec<String> = fixtures()
        .iter()
        .filter_map(|fixture| run_case(fixture).err())
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
