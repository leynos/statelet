//! Acquisition of Cargo's metadata document for the topology policy.
//!
//! This module owns the one place the check talks to the outside world. It
//! spawns `cargo metadata --no-deps --format-version 1`, checks the exit status
//! before reading stdout, and parses only a successful run. The policy module
//! stays pure, so its cases never need Cargo and this adapter's failures stay
//! distinguishable from a policy rejection.
//!
//! The two process inputs are injected from [`process_inputs`]: the Cargo
//! executable the invoking Cargo supplies, and an optional manifest path the
//! end-to-end gate sets. Ordinary runs read the manifest beneath
//! `CARGO_MANIFEST_DIR`. The environment read is the boundary itself — there is
//! no caller to inject it from inside a test binary — so it is a scoped,
//! reason-bearing exemption from the repository's `disallowed_methods` policy
//! rather than an ambient read scattered through the check.

use std::{
    ffi::{OsStr, OsString},
    process::Command,
};

use camino::{Utf8Path, Utf8PathBuf};
use serde_json::Value;
use thiserror::Error;

/// The variable naming the Cargo executable that started this test binary.
const CARGO_ENV: &str = "CARGO";

/// The private variable selecting another manifest for the end-to-end gate.
///
/// Visible to the sibling case module so a diagnostic assertion can name the
/// real variable rather than repeat its spelling.
pub(crate) const MANIFEST_OVERRIDE_ENV: &str = "STATELET_TOPOLOGY_MANIFEST_PATH";

/// The metadata format version requested from Cargo.
const METADATA_ARGUMENTS: [&str; 5] = [
    "metadata",
    "--no-deps",
    "--format-version",
    "1",
    "--manifest-path",
];

/// The Cargo executable and manifest one check reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessInputs {
    /// The Cargo executable, as the invoking Cargo named it.
    pub(crate) cargo: OsString,
    /// The manifest to read, absolute when the workspace root is known.
    pub(crate) manifest: Utf8PathBuf,
}

/// One acquisition failure, kept distinct from a policy rejection.
///
/// A failure here says the check could not *see* the metadata; a
/// [`super::policy::TopologyViolation`] says it saw metadata it refuses. The
/// distinction survives into the message so a contributor does not repair a
/// policy that was never consulted.
#[derive(Debug, Error)]
pub(crate) enum MetadataError {
    /// The process environment did not supply one of the two inputs.
    #[error(
        "the environment does not define {variable}. Repair: run the check through `make \
         topology-check`, which supplies it."
    )]
    MissingVariable { variable: &'static str },

    /// An environment value is not valid UTF-8 where a path must be.
    ///
    /// The unreadable value cannot be quoted back: it has no valid UTF-8 form.
    /// The variable name is the only locator a contributor needs, so the repair
    /// names it and says where the check expects the value to come from.
    #[error(
        "the value of {variable} is not valid UTF-8. Repair: set {variable} to a UTF-8 path, or \
         unset it to read the repository's own manifest through `make topology-check`."
    )]
    NonUnicodeValue { variable: &'static str },

    /// The Cargo process could not be started.
    #[error(
        "cannot run {cargo} for {manifest}: {source}. Repair: check that the named executable is \
         a Cargo binary and that the manifest path exists."
    )]
    Spawn {
        /// The Cargo executable as it was named.
        cargo: String,
        /// The manifest the check asked Cargo to read.
        manifest: String,
        /// The operating-system failure.
        source: std::io::Error,
    },

    /// Cargo ran and refused the metadata request.
    #[error(
        "{cargo} metadata failed for {manifest} with {status}: {stderr}. Repair: correct the \
         manifest or the workspace it declares, then rerun the check."
    )]
    Failed {
        /// The Cargo executable as it was named.
        cargo: String,
        /// The manifest the check asked Cargo to read.
        manifest: String,
        /// The exit status, or the signal that replaced it.
        status: String,
        /// Cargo's standard error, preserved whole.
        stderr: String,
    },

    /// Cargo's standard output was not the JSON document it claims.
    #[error(
        "Cargo printed unreadable metadata for {manifest}: {source}. Repair: confirm the manifest \
         is valid JSON-emitting Cargo metadata and rerun the check."
    )]
    Unreadable {
        /// The manifest the check asked Cargo to read.
        manifest: String,
        /// The JSON syntax or encoding failure.
        source: serde_json::Error,
    },
}

/// Returns the Cargo executable and manifest this invocation must read.
///
/// The manifest override exists for the end-to-end gate, which points the
/// checker at isolated fixtures. Routine entrypoints clear the variable at
/// their own boundary and pin the repository manifest on the Make command
/// line, so a stray fixture value cannot redirect routine validation.
///
/// # Errors
///
/// Returns [`MetadataError::MissingVariable`] or
/// [`MetadataError::NonUnicodeValue`] when an input is absent or unreadable.
pub(crate) fn process_inputs() -> Result<ProcessInputs, MetadataError> {
    Ok(ProcessInputs {
        cargo: required_variable(CARGO_ENV)?,
        manifest: manifest_path()?,
    })
}

/// The manifest the policy evaluates: the override, or the repository's.
fn manifest_path() -> Result<Utf8PathBuf, MetadataError> {
    optional_variable(MANIFEST_OVERRIDE_ENV)?.map_or_else(
        || Ok(Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")),
        |override_path| Ok(override_path.into()),
    )
}

/// Reads one required variable as an `OsString`.
///
/// The scoped allowance is the module's whole interaction with process state;
/// every other environment question is answered by an argument.
#[expect(
    clippy::disallowed_methods,
    reason = "This module is the injected boundary: it recovers the Cargo and manifest inputs the \
              invoking process supplied, and no caller can inject them into a test binary. The \
              developer guide records the scope."
)]
fn required_variable(name: &'static str) -> Result<OsString, MetadataError> {
    std::env::var_os(name).ok_or(MetadataError::MissingVariable { variable: name })
}

/// Reads one optional variable as UTF-8 text.
///
/// The second scoped allowance, for the same reason as [`required_variable`]:
/// the manifest override is supplied per child process by the end-to-end gate
/// and cannot be passed through a test runner's arguments.
#[expect(
    clippy::disallowed_methods,
    reason = "The manifest override is read from this process's environment by design; see the \
              module note and the developer guide's acquisition-owner entry."
)]
fn optional_variable(name: &'static str) -> Result<Option<String>, MetadataError> {
    classify_optional(name, std::env::var(name))
}

/// Decides what one optional-variable read means.
///
/// The caller supplies the `Result` the reader already produced, so every arm is
/// reachable from a case without `set_var` and without process-global state.
/// `VarError::NotUnicode` carries a public `OsString`, so a case builds that
/// error directly instead of trying to coerce this binary's own environment.
///
/// Visible to the sibling case module: `optional_variable` is the only live
/// caller, and a case that could not name this function could only test it by
/// mutating the process environment, which the repository forbids.
pub(crate) fn classify_optional(
    name: &'static str,
    read: Result<String, std::env::VarError>,
) -> Result<Option<String>, MetadataError> {
    match read {
        // An empty value is how Make spells "not set": `VAR="$(EMPTY)"` exports
        // the empty string rather than omitting the variable, and reading that
        // as an override would send Cargo an empty manifest path.
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            Err(MetadataError::NonUnicodeValue { variable: name })
        }
    }
}

/// Runs Cargo and parses the metadata document it prints.
///
/// The exit status is checked before stdout is read, so a failed run reports
/// Cargo's own diagnostic instead of a JSON syntax error about the empty or
/// partial output it left behind. Arguments are passed as separate elements,
/// never through a shell, so a manifest path containing spaces stays one
/// argument.
///
/// # Errors
///
/// Returns the spawn, status, or parse failure in [`MetadataError`].
pub(crate) fn read_metadata(cargo: &OsStr, manifest: &Utf8Path) -> Result<Value, MetadataError> {
    let executable = cargo.to_string_lossy().into_owned();
    let path = manifest.as_str().to_owned();
    let output = Command::new(cargo)
        .args(METADATA_ARGUMENTS)
        .arg(manifest.as_str())
        .output()
        .map_err(|source| MetadataError::Spawn {
            cargo: executable.clone(),
            manifest: path.clone(),
            source,
        })?;
    if !output.status.success() {
        return Err(MetadataError::Failed {
            cargo: executable,
            manifest: path,
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    serde_json::from_slice(&output.stdout).map_err(|source| MetadataError::Unreadable {
        manifest: path,
        source,
    })
}
