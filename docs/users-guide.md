# User Guide

This guide explains the current user-facing shape of Statelet. The crate is in
the design and validation phase, so this page is a signpost rather than an API
manual.

## Current status

Statelet is being evaluated as a small toolkit for marking transition
boundaries in ordinary Rust state machines. Its only runtime API is the
handwritten `StateName` trait described below; it does not yet expose a
procedural macro or a stable public API. The trait's `&'static str` return
shape is provisional until the consumption evidence required by
[ADR 0004](adrs/adr-0004-define-the-statename-consumption-evidence.md) is
recorded.

The first validation slice is deliberately conventions-only. The project must
prove that stable state names and documented tracing fields improve real
`mdtablefix` code before any macro surface is published.

Statelet may deliberately ship nothing if either validation example shows
little value. [ADR 0003](adrs/adr-0003-record-the-v0-1-exit-register.md)
records that off-ramp alongside the conventions-only and macro release scopes.

The installed surface is unchanged by the repository's own validation tooling.
`make topology-check` checks that the workspace is still the `statelet` runtime
library alone and that no `statelet-macros` edge has been declared ahead of the
macro baseline gate; it adds no runtime API, feature, or dependency, and it
does not validate a future multi-crate workspace. The same policy runs as part
of `make all` and in continuous integration, so a checkout cannot drift into a
second workspace member or a forbidden macro edge while its other gates stay
green.

The shape of the state name itself is likewise unsettled. Whether a
`&'static str` is enough, or whether consumers need a stable identifier as
well, will be decided from observations recorded while the validation examples
are annotated and not before — so no identifier will be added on anticipation
alone. [ADR 0004](adrs/adr-0004-define-the-statename-consumption-evidence.md)
defines what those observations record and how they are read.

## Naming a state without `Debug`

Implement `statelet::StateName` for an application-owned enum when a state
needs an observable label but must not implement `Debug`. The implementation is
handwritten: the crate does not generate it, and the label is an explicit
literal chosen by the implementor. A `match` over the enum returns the chosen
literal for each variant, such as `Self::Normalize => "Normalize"`. The
[trait's Rustdoc example](../src/lib.rs) is a complete implementation that the
doctest suite compiles and runs.

A label is independent of a state's payload: `Reading(u64)` and `Reading(0)`
both report `"Reading"` when the implementation matches on the variant rather
than its value. The method borrows the state and returns a static string, so
naming does not consume the state and the returned label is not a fresh
allocation. The trait does not prevent an implementation from doing other work
internally.

The implementor owns label stability. The trait checks the return type and
lifetime, not a policy; deriving a label from `Debug`, `Display`, a type name,
or a numeric discriminant makes a variant rename silently rename the label. Use
explicit literals for any name that operational tooling may observe.

## Quick start

Install `rustup` and `uv` before running the validation entrypoint. Follow the
[uv installation guide](https://docs.astral.sh/uv/getting-started/installation/)
to install `uv`. The repository pins its Rust nightly and components in
`rust-toolchain.toml`; `uv` provisions managed CPython 3.14 for repository
scripts, tests, linting, and typechecking. On Linux, native development builds
need `clang` and the pinned `mold` linker; `lld` is needed only for coverage.
The linker installer uses `curl`, `sha256sum`, and `tar`. Markdown lint
installation needs Node.js and `npm`.

After those prerequisites are available, `make install-build-tools` installs
the pinned linker and Rust toolchain components. The test gate also requires
`cargo-nextest`, and the lint gate requires Whitaker. Follow the installation
commands in the [developer guide](developers-guide.md#local-workflow) before
running `make all`.

From a prepared checkout, inspect the public validation targets:

<!-- tested-example: users-guide-validation-targets -->
```bash
make help
```

The most useful public commands are:

- `make all` runs formatting checks, linting, typechecking, Rust and Python
  tests (including shared workflow contracts), the dependency-topology check,
  feature matrix, and spelling.
- `make lint` runs the repository lint suite.
- `make test` runs the repository test suite, including the feature-matrix
  command-boundary checks. It requires the pinned `cargo-hack`; install it with
  `make install-cargo-hack` before running the validation entrypoint.

Continuous integration installs the pinned `cargo-hack` and runs the feature
matrix after the dependency-topology check. It runs the feature-matrix
command-boundary checks after coverage, separately from the instrumented
coverage run.

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
