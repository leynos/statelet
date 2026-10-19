//! Manifest text for the command-boundary fixtures.
//!
//! `harness.rs` runs processes and owns scratch trees; this module only decides
//! what text a fixture file contains. Keeping the two apart matters because the
//! fixture table states a case's *shape* and expectation in one place, while the
//! exact manifest text lives here instead of in nine nearly identical string
//! literals. Neither module decides a verdict.

use std::path::Path;

use super::harness::CaseName;

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

/// An empty library source, so a fixture has the file its target names.
pub(super) const EMPTY_LIB: &str = "";

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

/// An ordinary single-package fixture naming `name`.
pub(super) fn plain_package(name: PackageName, trailing: &str) -> String {
    package_manifest(name, Library::Ordinary, trailing)
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
