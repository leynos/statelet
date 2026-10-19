//! The fixture table the command-boundary suite runs.
//!
//! Each fixture states one verdict the command must produce and the exact
//! repair text a contributor should see. The manifests are built by
//! `manifests::manifest_for`, so this table holds the shape and the expectation
//! while the text lives in one place.

use super::harness::{CaseName, Fixture};

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
pub(super) fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            name: CaseName("one-runtime-package"),
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: true,
            expected_text: "",
        },
        Fixture {
            name: CaseName("additional-workspace-member"),
            files: &["Cargo.toml", "extra/Cargo.toml", "extra/src/lib.rs"],
            should_succeed: false,
            expected_text: "remove the extra member",
        },
        Fixture {
            name: CaseName("proc-macro-workspace-member"),
            files: &["Cargo.toml", "macros/Cargo.toml", "macros/src/lib.rs"],
            should_succeed: false,
            expected_text: "remove the extra member",
        },
        Fixture {
            name: CaseName("inactive-renamed-macro-edge"),
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: false,
            expected_text: "statelet-macros",
        },
        Fixture {
            name: CaseName("proc-macro-runtime-package"),
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: false,
            expected_text: "declares a proc-macro target",
        },
        Fixture {
            name: CaseName("build-kind-macro-edge"),
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: false,
            // The child's message passes through nextest's own diagnosis, so a
            // quoted needle would have to encode another tool's escaping. The
            // unquoted phrase identifies the verdict and the declaration kind
            // without depending on how that layer prints quotes.
            expected_text: "declares a build dependency on",
        },
        Fixture {
            name: CaseName("runtime-alias-dependency"),
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
            name: CaseName("path containing spaces"),
            files: &["Cargo.toml", "src/lib.rs"],
            should_succeed: true,
            expected_text: "",
        },
        Fixture {
            name: CaseName("invalid-manifest"),
            files: &["Cargo.toml"],
            should_succeed: false,
            expected_text: "metadata failed",
        },
    ]
}
