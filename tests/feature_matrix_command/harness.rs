//! Private process and filesystem fixtures for feature-matrix command tests.
//!
//! The harness invokes the repository Makefile from an isolated package path,
//! records the Cargo command boundary, and keeps each fixture in an owned root
//! below Cargo's test temporary directory. Fixture writes use a capability
//! directory so supplied paths cannot escape that root.

use std::{
    path::{Component, Path, PathBuf},
    process::{self, Command},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use cap_std::{ambient_authority, fs::Dir};

const PACKAGE_DIRECTORY: &str = "fixture with spaces";
const WRAPPER_NAME: &str = "cargo-wrapper";
const INVOCATION_LOG_NAME: &str = "cargo-invocations.log";
const BEGIN_MARKER: &str = "BEGIN";
const END_MARKER: &str = "END";

static ROOT_SERIAL: AtomicU64 = AtomicU64::new(0);

/// A feature-matrix fixture and the paths it owns.
pub(super) struct Fixture {
    root: PathBuf,
    root_dir: Dir,
    package: PathBuf,
    package_dir: Dir,
    source: PathBuf,
    wrapper: PathBuf,
}

impl Fixture {
    /// Creates an isolated package and writes the caller-supplied fixture files.
    pub(super) fn new(label: &str, files: &[(&str, &str)]) -> Result<Self, String> {
        let source = repository_root();
        let root = unique_root(label)?;
        let root_dir = Dir::open_ambient_dir(&root, ambient_authority())
            .map_err(|error| format!("cannot open fixture root {}: {error}", root.display()))?;
        let package = root.join(PACKAGE_DIRECTORY);
        root_dir.create_dir(PACKAGE_DIRECTORY).map_err(|error| {
            format!(
                "cannot create package directory {}: {error}",
                package.display()
            )
        })?;
        let package_dir = root_dir.open_dir(PACKAGE_DIRECTORY).map_err(|error| {
            format!(
                "cannot open package directory {}: {error}",
                package.display()
            )
        })?;

        copy_repository_inputs(&source, &package_dir)?;
        for (relative, contents) in files {
            write_fixture_file(&package_dir, relative, contents)?;
        }

        let wrapper = root.join(WRAPPER_NAME);
        if wrapper.to_string_lossy().contains(' ') {
            return Err(format!(
                "Cargo wrapper path must not contain spaces: {}",
                wrapper.display()
            ));
        }
        root_dir
            .write(WRAPPER_NAME, CARGO_WRAPPER)
            .map_err(|error| {
                format!("cannot write Cargo wrapper {}: {error}", wrapper.display())
            })?;
        set_executable(&root_dir, WRAPPER_NAME)?;

        Ok(Self {
            root: root.clone(),
            root_dir,
            package,
            package_dir,
            source,
            wrapper,
        })
    }

    /// Runs the real Make target with Cargo redirected through the recording
    /// wrapper.
    #[expect(
        clippy::disallowed_methods,
        reason = "The command boundary must preserve caller RUSTFLAGS from the test process \
                  without mutating global state."
    )]
    pub(super) fn run(&self, version_override: Option<&str>) -> Result<process::Output, String> {
        let rustflags = std::env::var_os("STATELET_FEATURE_MATRIX_CALLER_FLAGS")
            .or_else(|| std::env::var_os("RUSTFLAGS"));
        let mut command = Command::new("make");
        command
            .arg("feature-matrix")
            .current_dir(&self.package)
            .env("CARGO", &self.wrapper)
            .env("REAL_CARGO", env!("CARGO"))
            .env("CARGO_INVOCATION_LOG", self.root.join(INVOCATION_LOG_NAME))
            .env("CARGO_TARGET_DIR", self.root.join("cargo-target"))
            .env("MOLD_VERSION_FILE", self.source.join("tools/mold/VERSION"))
            .env(
                "MOLD_SHA256SUMS_FILE",
                self.source.join("tools/mold/SHA256SUMS"),
            )
            .env(
                "RUST_TOOLCHAIN_FILE",
                self.source.join("rust-toolchain.toml"),
            );
        match version_override {
            Some(value) => {
                command.env("STATELET_FEATURE_MATRIX_VERSION_OVERRIDE", value);
            }
            None => {
                command.env_remove("STATELET_FEATURE_MATRIX_VERSION_OVERRIDE");
            }
        }
        match rustflags {
            Some(flags) => {
                command.env("RUSTFLAGS", flags);
            }
            None => {
                command.env_remove("RUSTFLAGS");
            }
        }
        command
            .output()
            .map_err(|error| format!("cannot run make feature-matrix: {error}"))
    }

    /// Returns each recorded Cargo invocation as its original argument vector.
    pub(super) fn invocations(&self) -> Result<Vec<Vec<String>>, String> {
        let bytes = self
            .root_dir
            .read(INVOCATION_LOG_NAME)
            .map_err(|error| format!("cannot read Cargo invocation log: {error}"))?;
        let contents = String::from_utf8(bytes)
            .map_err(|error| format!("Cargo invocation log is not UTF-8: {error}"))?;
        parse_invocations(&contents)
    }

    /// Reads a fixture file relative to the package directory.
    pub(super) fn read(&self, relative: &str) -> Result<Vec<u8>, String> {
        let relative_path = checked_relative_path(relative)?;
        self.package_dir
            .read(relative_path)
            .map_err(|error| format!("cannot read fixture file {relative:?}: {error}"))
    }
}

/// The repository root whose Makefile and tool pins the fixture uses.
fn repository_root() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")) }

/// Creates a unique fixture root below Cargo's test temporary directory.
fn unique_root(label: &str) -> Result<PathBuf, String> {
    let temp_root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    Dir::create_ambient_dir_all(&temp_root, ambient_authority()).map_err(|error| {
        format!(
            "cannot create test temporary directory {}: {error}",
            temp_root.display()
        )
    })?;
    let fixture_parent = temp_root.join("feature-matrix-command");
    Dir::create_ambient_dir_all(&fixture_parent, ambient_authority()).map_err(|error| {
        format!(
            "cannot create fixture parent {}: {error}",
            fixture_parent.display()
        )
    })?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock is before the Unix epoch: {error}"))?
        .as_nanos();
    let serial = ROOT_SERIAL.fetch_add(1, Ordering::Relaxed);
    let safe_label = label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    let name = format!(
        "{}-{}-{timestamp}-{serial}",
        nonempty_label(&safe_label),
        process::id()
    );
    let parent = Dir::open_ambient_dir(&fixture_parent, ambient_authority()).map_err(|error| {
        format!(
            "cannot open fixture parent {}: {error}",
            fixture_parent.display()
        )
    })?;
    parent
        .create_dir(&name)
        .map_err(|error| format!("cannot create unique fixture root {name:?}: {error}"))?;
    Ok(fixture_parent.join(name))
}

/// Keeps an empty or punctuation-only label useful in the fixture path.
fn nonempty_label(label: &str) -> &str {
    if label.is_empty() || label.chars().all(|character| character == '-') {
        "fixture"
    } else {
        label
    }
}

/// Copies the live Makefile and tool-check scripts, preserving script permissions.
fn copy_repository_inputs(source: &Path, package: &Dir) -> Result<(), String> {
    package
        .create_dir_all("scripts")
        .map_err(|error| format!("cannot create fixture scripts directory: {error}"))?;
    let source_dir = Dir::open_ambient_dir(source, ambient_authority()).map_err(|error| {
        format!(
            "cannot open source repository {}: {error}",
            source.display()
        )
    })?;
    source_dir
        .copy("Makefile", package, "Makefile")
        .map_err(|error| format!("cannot copy repository Makefile: {error}"))?;
    for script in ["check-build-tools.sh", "build-tools-common.sh"] {
        source_dir
            .copy(
                Path::new("scripts").join(script),
                package,
                Path::new("scripts").join(script),
            )
            .map_err(|error| format!("cannot copy repository script {script}: {error}"))?;
    }
    Ok(())
}

/// Writes one supplied UTF-8 file using the package capability directory.
fn write_fixture_file(package: &Dir, relative: &str, contents: &str) -> Result<(), String> {
    let path = checked_relative_path(relative)?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        package
            .create_dir_all(parent)
            .map_err(|error| format!("cannot create parent directory for {relative:?}: {error}"))?;
    }
    package
        .write(path, contents)
        .map_err(|error| format!("cannot write fixture file {relative:?}: {error}"))
}

/// Rejects absolute paths and traversal before passing a fixture path to cap-std.
fn checked_relative_path(relative: &str) -> Result<&Path, String> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "fixture path must be a non-empty relative path: {relative:?}"
        ));
    }
    Ok(path)
}

/// Sets the wrapper's executable bit without changing process-global state.
#[cfg(unix)]
fn set_executable(root: &Dir, relative: &str) -> Result<(), String> {
    use cap_std::fs::{Permissions, PermissionsExt};

    root.set_permissions(relative, Permissions::from_mode(0o755))
        .map_err(|error| format!("cannot make Cargo wrapper executable: {error}"))
}

/// Reports unsupported hosts rather than creating a wrapper that cannot run.
#[cfg(not(unix))]
fn set_executable(_root: &Dir, _relative: &str) -> Result<(), String> {
    Err("the Cargo wrapper requires a Unix host".to_owned())
}

/// Parses the wrapper's line-delimited BEGIN/END records.
fn parse_invocations(contents: &str) -> Result<Vec<Vec<String>>, String> {
    let mut invocations = Vec::new();
    let mut current = None::<Vec<String>>;
    for line in contents.lines() {
        match line {
            BEGIN_MARKER if current.is_none() => current = Some(Vec::new()),
            END_MARKER => match current.take() {
                Some(arguments) => invocations.push(arguments),
                None => return Err("Cargo invocation log has an unmatched END marker".to_owned()),
            },
            BEGIN_MARKER => {
                return Err("Cargo invocation log has a nested BEGIN marker".to_owned());
            }
            argument => match current.as_mut() {
                Some(arguments) => arguments.push(argument.to_owned()),
                None => return Err("Cargo invocation log has text outside a record".to_owned()),
            },
        }
    }
    if current.is_some() {
        return Err("Cargo invocation log ends before an END marker".to_owned());
    }
    Ok(invocations)
}

/// The test-owned wrapper records arguments, optionally controls the version
/// probe, and delegates all other invocations to the Cargo used for this test.
const CARGO_WRAPPER: &str = r#"#!/usr/bin/env bash
set -euo pipefail
{
  printf '%s\n' 'BEGIN'
  printf '%s\n' "$@"
  printf '%s\n' 'END'
} >> "$CARGO_INVOCATION_LOG"

if [[ ${1-} == hack && ${2-} == --version && ${STATELET_FEATURE_MATRIX_VERSION_OVERRIDE+x} ]]; then
  if [[ $STATELET_FEATURE_MATRIX_VERSION_OVERRIDE == missing ]]; then
    exit 1
  fi
  printf '%s\n' "$STATELET_FEATURE_MATRIX_VERSION_OVERRIDE"
  exit 0
fi

exec "$REAL_CARGO" "$@"
"#;
