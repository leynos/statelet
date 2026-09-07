//! Contract test for the repository's codegen-backend and linker standard.
//!
//! Dev-profile Cranelift with the mold linker, configured in
//! `.cargo/config.toml`, is the estate standard for development, test, lint
//! and proof builds. Release builds use LLVM because `--release` selects a
//! profile that file does not configure, and coverage overrides the backend
//! for its own invocation because `-Cinstrument-coverage` is LLVM-specific.
//!
//! This is a contract rather than a comment because the repository previously
//! documented the opposite rule, that Cranelift must never appear in
//! `.cargo/config.toml` (withdrawn in #60). That text is still in the history,
//! and acting on it would silently make every debug build slower while leaving
//! nothing to fail.
//!
//! The assertions read the configuration file rather than describing it, so
//! they need no toolchain and no build.

/// The auto-discovered Cargo configuration, read at compile time.
///
/// `include_str!` rather than a runtime read: the workspace's capability-based
/// filesystem policy forbids `std::fs` here, and baking the file in also makes
/// Cargo rebuild this test whenever the configuration changes, which is
/// exactly when the contract needs re-checking.
const CARGO_CONFIG: &str = include_str!("../.cargo/config.toml");

/// Returns the body of a named table, up to the next table header.
fn table<'a>(config: &'a str, header: &str) -> Option<&'a str> {
    let after = config.split_once(header)?.1;
    Some(after.split_once("\n[").map_or(after, |(body, _)| body))
}

#[test]
fn the_dev_profile_uses_cranelift() {
    let Some(profile) = table(CARGO_CONFIG, "[profile.dev]") else {
        panic!("`.cargo/config.toml` declares no [profile.dev] table:\n{CARGO_CONFIG}");
    };

    assert!(
        profile.contains(r#"codegen-backend = "cranelift""#),
        "[profile.dev] must select Cranelift, the standard for development, test, lint and proof \
         builds:\n{profile}"
    );
    assert!(
        CARGO_CONFIG.contains("codegen-backend = true"),
        "the [unstable] table must enable codegen-backend, or Cargo rejects the profile \
         setting:\n{CARGO_CONFIG}"
    );
}

#[test]
fn the_release_profile_is_left_on_the_default_backend() {
    assert!(
        table(CARGO_CONFIG, "[profile.release]").is_none(),
        "`.cargo/config.toml` must not configure the release profile; release builds use \
         LLVM:\n{CARGO_CONFIG}"
    );
}

/// The table header selecting every Linux target, whatever the architecture.
const LINUX_TARGET: &str = r#"[target.'cfg(target_os = "linux")']"#;

#[test]
fn linux_links_with_mold_through_clang() {
    let Some(target) = table(CARGO_CONFIG, LINUX_TARGET) else {
        panic!("`.cargo/config.toml` declares no Linux target table:\n{CARGO_CONFIG}");
    };

    assert!(
        target.contains(r#"linker = "clang""#),
        "the Linux target must link through clang, which is what invokes mold:\n{target}"
    );
    assert!(
        target.contains("-fuse-ld=mold"),
        "the Linux target must select the mold linker:\n{target}"
    );
}

#[test]
fn the_linker_selector_covers_every_linux_architecture() {
    // A target-triple table would leave aarch64 Linux on the default linker,
    // which is the state this configuration was in before #61. mold supports
    // every Linux architecture, so the selector is keyed on the operating
    // system; asserting the header, not merely the keys inside it, is what
    // makes a narrowing visible.
    assert!(
        CARGO_CONFIG.contains(LINUX_TARGET),
        "the Linux linker settings must be keyed on `cfg(target_os = \"linux\")`, not on one \
         target triple:\n{CARGO_CONFIG}"
    );
    assert!(
        !CARGO_CONFIG.contains("[target.x86_64-unknown-linux-gnu]"),
        "an x86_64-only table is a narrowing unless it carries a genuinely architecture-specific \
         flag; the linker settings belong in the cfg table:\n{CARGO_CONFIG}"
    );
}

#[test]
fn the_table_reader_stops_at_the_next_header() {
    // Mutation check: a reader that ran to the end of the file would find a
    // later table's keys and report them as the requested table's, which would
    // make every assertion above pass for the wrong reason.
    let sample = "[profile.dev]\nkey = 1\n\n[target.other]\nlinker = \"clang\"\n";

    let Some(profile) = table(sample, "[profile.dev]") else {
        panic!("the reader must find a table that is present");
    };

    assert!(profile.contains("key = 1"));
    assert!(!profile.contains("linker"));
    assert!(table(sample, "[profile.release]").is_none());
}
