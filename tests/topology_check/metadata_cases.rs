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
//! scratch tree to leak.

use std::ffi::OsStr;

use camino::Utf8Path;

use super::metadata::{MetadataError, read_metadata};

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
