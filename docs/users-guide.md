# User Guide

This guide explains the current user-facing shape of Statelet. The crate is in
the design and validation phase, so this page is a signpost rather than an API
manual.

## Current status

Statelet is being evaluated as a small toolkit for marking transition
boundaries in ordinary Rust state machines. It does not yet expose a stable
runtime API or a procedural macro.

The first validation slice is deliberately conventions-only. The project must
prove that stable state names and documented tracing fields improve real
`mdtablefix` code before any macro surface is published.

Statelet may deliberately ship nothing if either validation example shows
little value. [ADR 003](adr-003-v0-1-exit-register.md) records that off-ramp
alongside the conventions-only and macro release scopes.

The shape of the state name itself is likewise unsettled. Whether a
`&'static str` is enough, or whether consumers need a stable identifier as
well, will be decided from observations recorded while the validation examples
are annotated and not before — so no identifier will be added on anticipation
alone. [ADR 004](adr-004-state-name-consumption-evidence.md) defines what those
observations record and how they are read.

## Quick start

Install `rustup` and `uv` before running the validation entrypoint. The
repository pins its Rust nightly and components in `rust-toolchain.toml`; `uv`
provisions managed CPython 3.14 for repository scripts, tests, linting, and
typechecking. On Linux, native development builds need `clang` and the pinned
`mold` linker; `lld` is needed only for coverage. The linker installer uses
`curl`, `sha256sum`, and `tar`. Markdown lint installation needs Node.js and
`npm`.

After those prerequisites are available, `make install-build-tools` installs
the pinned linker and Rust toolchain components. The test gate also requires
`cargo-nextest`, and the lint gate requires Whitaker. Follow the installation
commands in the [developer guide](developers-guide.md#local-workflow) before
running `make all`.

Then run the public validation entrypoint from a fresh checkout:

```bash
git clone https://github.com/leynos/statelet.git
cd statelet
make install-build-tools
make install-mdtablefix
make install-markdownlint
make all
```

The most useful public commands are:

- `make all` runs formatting checks, linting, typechecking, Rust and Python
  tests, shared workflow contracts, and spelling.
- `make lint` runs the repository lint suite.
- `make test` runs the repository test suite.

## What to read first

- [Terms of reference](terms-of-reference.md) explains the problem space,
  intended users, non-goals, and validation test.
- [Technical design](design.md) explains the proposed crate boundary, runtime
  vocabulary, macro gate, and dependency constraints.
- [Roadmap](roadmap.md) explains the delivery phases and the evidence required
  to ship nothing, ship conventions only, or proceed to a macro.

## Expected user model

Use Statelet only if the state machine already belongs in your codebase as
ordinary Rust. Statelet should help name and observe transition boundaries; it
should not move branch logic into a graph DSL or runtime engine.

If you need generated dispatch, transition tables, typestate guarantees,
hierarchical statecharts, or embedded hard real-time behaviour, the design
expects you to use a crate built for those jobs instead.
