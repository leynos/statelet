# Developer Guide

This guide explains the contributor workflow and implementation tooling for the
generated Statelet project.

## Normative references

- [Technical design](design.md) defines crate boundaries, public API decisions,
  and validation gates.
- [Repository layout](repository-layout.md) explains the top-level files,
  directories, and ownership boundaries.
- [Documentation contents](contents.md) indexes the full documentation set.

## Generated project shape

Statelet uses Rust 2024, a pinned nightly toolchain, strict lint settings, and
documented starter code. The current repository is a library project and renders
`src/lib.rs`.

## Local Workflow

Use `make all` as the public entrypoint for formatting, linting, and tests.
`make lint` runs rustdoc, Clippy, and Whitaker. `make test` prefers
`cargo nextest run` and falls back to `cargo test` when cargo-nextest is not
available. `make audit` derives the Rust workspace root with `cargo metadata`,
logs workspace member manifests, and runs `cargo audit` once from the workspace
root. `make coverage` uses `cargo llvm-cov` with `lld`.

The generated `Makefile` exposes these public targets:

- `make all` runs formatting checks, linting, and tests.
- `make check-fmt` verifies Rust formatting.
- `make lint` runs rustdoc, Clippy, and Whitaker with warnings denied.
- `make test` runs `cargo nextest run` when cargo-nextest is installed and
  falls back to `cargo test` otherwise. All projects also run doctests.
- `make build` builds the debug target.
- `make release` builds the release target.
- `make coverage` writes `lcov.info` using `cargo llvm-cov` and `lld`.
- `make audit` derives the Rust workspace root with `cargo metadata` and runs
  `cargo audit` once from that root.
- `make markdownlint` checks Markdown files.
- `make spelling` checks the shared en-GB-oxendict configuration for drift,
  runs the consumer phrase scanner, and checks tracked Markdown prose with the
  pinned `typos` release.
- `make nixie` validates Mermaid diagrams.

GitHub Actions Act validation lives in `.github/workflows/act-validation.yml`.
The main `.github/workflows/ci.yml` workflow deliberately does not run
`make test WITH_ACT=1`; the separate Act workflow runs those slower
container-backed checks in parallel.

## Tooling

Development builds use Cranelift for debug code generation. On Linux targets,
`.cargo/config.toml` configures clang to link with `mold` so debug builds link
quickly. Coverage generation uses `lld` because LLVM coverage tooling expects
LLVM-compatible linker behaviour.

Coverage also overrides the codegen backend. `-Cinstrument-coverage` is an LLVM
feature that Cranelift does not implement, so with the dev profile's Cranelift
backend in force `cargo llvm-cov` stops at the first crate:

```text
error: `-Cinstrument-coverage` is LLVM specific and not supported by Cranelift
```

The `coverage` recipe therefore sets `CARGO_PROFILE_DEV_CODEGEN_BACKEND=llvm`
for that one invocation, which leaves every other target on Cranelift.
`tests/coverage_contract.rs` asserts the override is present and precedes the
cargo invocation, because removing it leaves the Makefile looking correct and
surfaces as a build failure minutes later.

Install `clang`, `lld`, `mold`, `python3`, and `cargo-audit` before running the
full generated workflow locally on Linux.

## Fast development builds

`make dev-build` and `make dev-test` offer an opt-in, faster iteration loop
for local debug work. `dev-build` compiles debug binaries and `dev-test` runs
the test suite; both use the Cranelift codegen backend and the mold linker
configured in `tools/dev-fast/config.toml`.

The `DEV_FAST_CONFIG` variable names that fragment, defaulting to
`tools/dev-fast/config.toml`, and both targets pass it to Cargo explicitly
with `--config "$(DEV_FAST_CONFIG)"`. Cargo never auto-discovers this
fragment; it takes effect only when a target invokes it directly, so other
`make` targets are unaffected.

Using the fragment requires a nightly toolchain, because the Cranelift
codegen backend is unstable. On Linux it also requires the mold linker on
`PATH`; the fragment gates the linker flag behind a `target_os = "linux"`
`cfg` table, so other platforms fall back to their default linker.

Cranelift configuration must never be copied into `.cargo/config.toml`.
Cargo auto-discovers that file and applies it to every invocation, which
would silently degrade release, coverage, and verification builds to the
faster but less optimizing backend. Keep the fast-build configuration
isolated in `tools/dev-fast/config.toml` and reach it only through
`make dev-build` and `make dev-test`.

## Spelling policy

The tracked `typos.toml` is generated from the shared estate dictionary and the
repository-specific `typos.local.toml` overlay. Never edit generated entries by
hand. Add only narrow repository terminology to the overlay.

The configuration builder is pinned to commit
`d6da92f02240a79a945c835f69bdd08a888da1d0`. Regenerate the configuration with:

```sh
TYPOS_CONFIG_BUILDER_COMMIT=d6da92f02240a79a945c835f69bdd08a888da1d0
uvx --python 3.14 \
  --from "git+https://github.com/leynos/typos-config-builder.git@${TYPOS_CONFIG_BUILDER_COMMIT}" \
  typos-config-builder
```

Use the same command with `--check` in quality gates to detect drift without
rewriting `typos.toml`. The builder refreshes the shared dictionary into the
untracked `.typos-oxendict-base.toml` cache only when the authority is newer,
records refresh metadata in `.typos-oxendict-base.json`, and reuses a valid
local cache when the authority is unavailable.

Typos splits hyphenated phrases into separate words. The consumer-owned
`scripts/typos_rollout_check.py` therefore reads phrase corrections from the
shared cache and local overlay, while taking ignore patterns and file
exclusions from generated `typos.toml`. It reports prohibited phrases without
duplicating the builder's validation, cache, merge, or rendering behaviour.
Quoted APIs and identifiers retain their upstream spelling; put them in
backticks or fenced code blocks where practical rather than adding broad
word-level exceptions.

## Workflow pins and Dependabot

Dependabot owns the upgrade of GitHub Actions and reusable workflows, including
calls into `leynos/shared-actions`. Contract tests that assert a caller's exact
commit SHA create a lockstep dependency: every time Dependabot opens a bump PR,
the test fails until a human edits the pinned constant to match. That defeats
the purpose of automated dependency updates and turns a routine bump into a
manual chore.

Contract tests may still verify the *shape* of a reusable-workflow caller. They
must not verify the specific SHA value.

- Do assert the workflow references the correct reusable workflow path.
- Do assert the ref is pinned to a full 40-character commit SHA, not a
  mutable branch such as `main` or `rolling`.
- Do assert the expected `on:` triggers, least-privilege `permissions:`, and
  the inputs the caller relies on.
- Do not hard-code the current SHA value as an expected string. Match it with
  a pattern instead.
- Do not fail a test purely because Dependabot bumped the pinned SHA.

```python
import re

SHA_RE = re.compile(r"^[0-9a-f]{40}$")

def test_uses_pinned_full_sha(caller_step):
    ref = caller_step["uses"].split("@")[-1]
    assert SHA_RE.match(ref), f"expected a 40-hex commit SHA, got {ref!r}"
```

If a workflow's behaviour genuinely depends on a feature only present from a
particular commit onwards, express that as a comment or a changelog note, not
as a test assertion on the SHA string.

### Security audit ignores

Security audit jobs may set `CARGO_AUDIT_IGNORES` for narrowly scoped RustSec
advisories that affect unused or tooling-only dependency paths. Keep each
ignore tied to a documented runtime impact analysis, and remove it when the
affected dependency leaves the graph or the project starts using the advised
runtime path.
