//! Failure cases for the metadata acquisition adapter.
//!
//! The command boundary proves that a rejected manifest produces a non-zero
//! exit. That is not the same claim as the adapter reporting the *right*
//! failure: a parse error about empty output is indistinguishable from a
//! missing executable to a contributor reading only "the check failed". These
//! cases pin the category, the manifest context, and the repair instruction of
//! each [`MetadataError`] variant.
//!
//! No fixture is written. A missing executable is simulated by a name that
//! cannot resolve, a Cargo refusal by a manifest path that does not exist, and
//! unreadable output by a successful process that prints its arguments rather
//! than JSON. None of the three needs a writable directory, so there is no
//! scratch tree to leak. The remaining variant cannot be raised from a test at
//! all — see `an_unreadable_environment_value_names_the_variable_and_repair`
//! for why its *rendered* message is asserted instead.

use std::ffi::OsStr;

use camino::Utf8Path;

use super::metadata::{MANIFEST_OVERRIDE_ENV, MetadataError, read_metadata};

/// The Cargo executable this test binary was started with.
const CARGO: &str = env!("CARGO");

/// The manifest every case names, present only in the diagnostics.
const MANIFEST: &str = "/statelet-fixture-does-not-exist/Cargo.toml";

/// Fails a case whose error was not the expected variant.
fn require<T>(result: Result<T, MetadataError>, wanted: &str) -> MetadataError {
    match result {
        Ok(_) => panic!("{wanted} succeeded on a failure fixture"),
        Err(error) => error,
    }
}

/// An executable that cannot be started is a spawn failure, not a parse one.
#[test]
fn a_missing_executable_is_reported_as_a_spawn_failure() {
    let error = require(
        read_metadata(
            OsStr::new("statelet-no-such-cargo-binary"),
            Utf8Path::new(MANIFEST),
        ),
        "the missing executable",
    );
    match error {
        MetadataError::Spawn {
            ref cargo,
            ref manifest,
            ..
        } => {
            assert_eq!(cargo, "statelet-no-such-cargo-binary");
            assert_eq!(manifest, MANIFEST);
        }
        other => panic!("expected a spawn failure, got {other:?}"),
    }
    assert!(error.to_string().contains("Repair:"), "{error}");
}

/// A non-zero Cargo run is reported as Cargo's own failure.
///
/// Cargo leaves stdout empty when it refuses a manifest. A parse-first adapter
/// would report an end-of-input JSON error instead of Cargo's diagnostic, so
/// this case exists to distinguish the two by the variant it names.
#[test]
fn a_cargo_refusal_is_reported_before_stdout_is_parsed() {
    let error = require(
        read_metadata(OsStr::new(CARGO), Utf8Path::new(MANIFEST)),
        "Cargo against an absent manifest",
    );
    match error {
        MetadataError::Failed {
            ref cargo,
            ref manifest,
            ref status,
            ref stderr,
        } => {
            assert_eq!(cargo, CARGO);
            assert_eq!(manifest, MANIFEST);
            assert!(!status.is_empty(), "the status was not retained");
            assert!(!stderr.is_empty(), "Cargo's diagnostic was not retained");
        }
        other => panic!("expected Cargo's failure, got {other:?}"),
    }
    assert!(error.to_string().contains("Repair:"), "{error}");
}

/// An unreadable environment value names its variable and its repair.
///
/// The other variants are raised by *doing* something to the adapter. This one
/// cannot be: `std::env::set_var` is banned repository-wide, and mutating the
/// process environment even under a guard would race every other case in the
/// binary. Constructing the variant therefore asserts the only thing the
/// finding asked for — the rendered diagnostic — and leaves the branch that
/// raises it covered by the `NotUnicode` match arm in `optional_variable`,
/// which is a two-line translation with no decision to get wrong.
///
/// The repair is asserted as a *sentence*, not as the bare word: a message
/// saying "not valid UTF-8" beside an unrelated "Repair:" would satisfy a
/// word-level check while telling a contributor nothing about what to do.
#[test]
fn an_unreadable_environment_value_names_the_variable_and_repair() {
    let rendered = MetadataError::NonUnicodeValue {
        variable: MANIFEST_OVERRIDE_ENV,
    }
    .to_string();
    assert!(
        rendered.contains(MANIFEST_OVERRIDE_ENV),
        "the variable was not named: {rendered}"
    );
    assert!(
        rendered.contains("Repair:") && rendered.contains("UTF-8 path"),
        "the repair did not say how to supply a readable value: {rendered}"
    );
}

/// A successful process printing non-JSON is reported as unreadable output.
///
/// `echo` is used rather than a fixture script because the adapter runs one
/// process with a fixed argument vector; a program that exits zero and prints
/// its arguments is exactly the successful-but-unreadable case. It never
/// reaches the manifest.
#[test]
fn unreadable_stdout_is_reported_with_its_manifest() {
    let error = require(
        read_metadata(OsStr::new("echo"), Utf8Path::new(MANIFEST)),
        "echo",
    );
    match error {
        MetadataError::Unreadable {
            ref manifest,
            ref source,
        } => {
            assert_eq!(manifest, MANIFEST);
            assert!(
                !source.to_string().is_empty(),
                "the JSON error was not retained"
            );
        }
        other => panic!("expected unreadable output, got {other:?}"),
    }
    assert!(error.to_string().contains("Repair:"), "{error}");
}
