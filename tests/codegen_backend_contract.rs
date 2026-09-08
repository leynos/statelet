//! Contract test for the repository's codegen-backend and linker standard.
//!
//! Dev-profile Cranelift with the mold linker, configured in
//! `.cargo/config.toml`, is the standard for development, test, lint, and
//! proof builds. Release builds use LLVM because `--release` selects a profile
//! that file does not configure, and coverage runs override the backend for
//! their own invocation because `-Cinstrument-coverage` is LLVM-specific.
//!
//! This is a contract rather than a comment because the repository previously
//! documented the opposite rule, that Cranelift must never appear in
//! `.cargo/config.toml` (withdrawn in #60). That text is still in the history,
//! and acting on it would silently make every debug build slower while leaving
//! nothing to fail.
//!
//! The configuration is parsed as TOML rather than searched as text. A
//! substring search cannot tell a live table from a commented-out one, so
//! `# [profile.dev]` would satisfy it while Cargo ignored the setting.

use toml::Value;

/// The auto-discovered Cargo configuration, read at compile time.
///
/// `include_str!` rather than a runtime read: the workspace's capability-based
/// filesystem policy forbids `std::fs` here, and baking the file in also makes
/// Cargo rebuild this test whenever the configuration changes, which is
/// exactly when the contract needs re-checking.
const CARGO_CONFIG: &str = include_str!("../.cargo/config.toml");

/// The table key selecting every Linux target, whatever the architecture.
const LINUX_TARGET: &str = r#"cfg(target_os = "linux")"#;

/// Parses a TOML document into a value that [`table`] can walk.
///
/// `toml::from_str` into a `Table` rather than `str::parse::<Value>`, which
/// parses a bare value rather than a document.
fn parse(text: &str) -> Result<Value, toml::de::Error> {
    toml::from_str::<toml::Table>(text).map(Value::Table)
}

/// Parses the configuration, returning the error rather than unwrapping it.
///
/// This is a helper, so a parse failure is the test body's verdict to report.
fn config() -> Result<Value, toml::de::Error> { parse(CARGO_CONFIG) }

/// Reads a nested table by path, for example `["profile", "dev"]`.
fn table<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(root, |value, key| value.get(*key))
}

/// Reads a string-valued key from a nested table.
fn string_at<'a>(root: &'a Value, path: &[&str], key: &str) -> Option<&'a str> {
    table(root, path)?.get(key)?.as_str()
}

/// Joins a target table's `rustflags` array into one string for searching.
fn rustflags(target: &Value) -> String {
    target
        .get("rustflags")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}

/// Whether a target table chooses a linker, by driver or by `-fuse-ld` flag.
fn selects_a_linker(target: &Value) -> bool {
    target.get("linker").is_some() || rustflags(target).contains("-fuse-ld=")
}

#[test]
fn the_dev_profile_uses_cranelift() {
    let config = config().expect("`.cargo/config.toml` must be valid TOML");

    assert_eq!(
        string_at(&config, &["profile", "dev"], "codegen-backend"),
        Some("cranelift"),
        "[profile.dev] must select Cranelift, the standard for development, test, lint, and proof \
         builds:\n{CARGO_CONFIG}"
    );
    assert_eq!(
        table(&config, &["unstable"]).and_then(|t| t.get("codegen-backend")),
        Some(&Value::Boolean(true)),
        "[unstable] must enable codegen-backend, or Cargo rejects the profile \
         setting:\n{CARGO_CONFIG}"
    );
}

#[test]
fn the_release_profile_does_not_select_cranelift() {
    // The release table is allowed to exist and to carry tuning such as `lto`
    // or `opt-level`. What it must not do is put release builds on Cranelift.
    let config = config().expect("`.cargo/config.toml` must be valid TOML");
    let backend = string_at(&config, &["profile", "release"], "codegen-backend");

    assert_ne!(
        backend,
        Some("cranelift"),
        "release builds must not use Cranelift:\n{CARGO_CONFIG}"
    );
}

#[test]
fn linux_links_with_mold_through_clang() {
    let config = config().expect("`.cargo/config.toml` must be valid TOML");
    let Some(target) = table(&config, &["target", LINUX_TARGET]) else {
        panic!("`.cargo/config.toml` declares no Linux target table:\n{CARGO_CONFIG}");
    };

    assert_eq!(
        target.get("linker").and_then(Value::as_str),
        Some("clang"),
        "the Linux target must link through clang, which is what invokes mold:\n{target}"
    );
    let flags = target
        .get("rustflags")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    assert!(
        flags.contains("-fuse-ld=mold"),
        "the Linux target must select the mold linker, got {flags:?}"
    );
}

#[test]
fn the_linker_selector_covers_every_linux_architecture() {
    // A target-triple table would leave aarch64 Linux on the default linker,
    // which is the state this configuration was in before #61. mold supports
    // every Linux architecture, so the selector is keyed on the operating
    // system; asserting the key, not merely the settings inside it, is what
    // makes a narrowing visible.
    let config = config().expect("`.cargo/config.toml` must be valid TOML");
    let Some(targets) = table(&config, &["target"]).and_then(Value::as_table) else {
        panic!("`.cargo/config.toml` declares no [target] tables:\n{CARGO_CONFIG}");
    };

    assert!(
        targets.contains_key(LINUX_TARGET),
        "the Linux linker settings must be keyed on `{LINUX_TARGET}`, not on one target \
         triple:\n{CARGO_CONFIG}"
    );
    // A Linux target-triple table is legitimate for a genuinely
    // architecture-specific setting. What must not reappear there is the
    // linker selection, because that is what narrowed it before.
    let narrowed: Vec<&String> = targets
        .iter()
        .filter(|(key, _)| key.ends_with("-linux-gnu") || key.ends_with("-linux-musl"))
        .filter(|(_, value)| selects_a_linker(value))
        .map(|(key, _)| key)
        .collect();
    assert!(
        narrowed.is_empty(),
        "the linker selection belongs in the `{LINUX_TARGET}` table, not in a target-triple \
         table; found it in {narrowed:?}"
    );
}

#[test]
fn a_triple_table_is_allowed_unless_it_takes_back_the_linker() {
    // Mutation check for the selector assertion. A target-triple table is
    // legitimate for an architecture-specific flag; what it must not do is
    // reclaim the linker, whether by naming a driver or by a `-fuse-ld` flag.
    let cases = [
        ("rustflags = [\"-Ctarget-cpu=neoverse-n1\"]", false),
        ("linker = \"clang\"", true),
        ("rustflags = [\"-Clink-arg=-fuse-ld=mold\"]", true),
    ];

    for (body, narrows) in cases {
        let document = format!("[target.aarch64-unknown-linux-gnu]\n{body}\n");
        let parsed = parse(&document).expect("valid TOML");
        let Some(target) = table(&parsed, &["target", "aarch64-unknown-linux-gnu"]) else {
            panic!("the fixture declares the table: {document}");
        };

        assert_eq!(
            selects_a_linker(target),
            narrows,
            "{body} should {} count as reclaiming the linker",
            if narrows { "" } else { "not" }
        );
    }
}

#[test]
fn the_parser_ignores_commented_out_configuration() {
    // Mutation check for the parsing itself. A substring search would accept
    // each of these as a live setting, which is what this test exists to stop.
    let commented = "# [profile.dev]\n# codegen-backend = \"cranelift\"\n";
    let from_comments = parse(commented).expect("a comment-only document is valid TOML");

    assert!(table(&from_comments, &["profile", "dev"]).is_none());

    let live = "[profile.dev]\ncodegen-backend = \"cranelift\"\n";
    let from_live = parse(live).expect("valid TOML");

    assert_eq!(
        string_at(&from_live, &["profile", "dev"], "codegen-backend"),
        Some("cranelift")
    );
}

#[test]
fn the_release_check_tolerates_unrelated_tuning() {
    // Mutation check for the release assertion: adding `lto` or `opt-level`
    // must not fail it, but selecting Cranelift must.
    let tuned = "[profile.release]\nlto = true\nopt-level = 3\n";
    let from_tuning = parse(tuned).expect("valid TOML");
    assert_ne!(
        string_at(&from_tuning, &["profile", "release"], "codegen-backend"),
        Some("cranelift")
    );

    let wrong = "[profile.release]\ncodegen-backend = \"cranelift\"\n";
    let from_cranelift = parse(wrong).expect("valid TOML");
    assert_eq!(
        string_at(&from_cranelift, &["profile", "release"], "codegen-backend"),
        Some("cranelift")
    );
}
