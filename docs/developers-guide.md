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

Install `clang`, `lld`, `mold`, `python3`, and `cargo-audit` before running the
full generated workflow locally on Linux.

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

### Security audit ignores

Security audit jobs may set `CARGO_AUDIT_IGNORES` for narrowly scoped RustSec
advisories that affect unused or tooling-only dependency paths. Keep each
ignore tied to a documented runtime impact analysis, and remove it when the
affected dependency leaves the graph or the project starts using the advised
runtime path.
