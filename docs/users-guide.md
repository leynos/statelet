# User Guide

This guide explains the current user-facing shape of Statelet. The crate is in
the design and validation phase, so this page is a signpost rather than an API
manual.

## Current status

Statelet is being evaluated as a small toolkit for marking transition
boundaries in ordinary Rust state machines. Its runtime API is the handwritten
`StateName` trait and the `statelet::fields` constants described below; it does
not yet expose a procedural macro or a stable public API. The trait's
`&'static str` return shape is provisional until the consumption evidence
required by
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

The examples treat a label as independent of a state's payload: `Reading(42)`
and `Reading(0)` both report `"Reading"` because the implementation matches on
the variant rather than its value. The trait does not require that choice, and
an implementation may return different literals for different payloads. The
method borrows the state and returns a static string, so naming does not
consume the state and the returned reference remains valid for the rest of the
program. The return type alone does not guarantee that an implementation avoids
allocation; it can build or leak a string before returning it. The trait also
does not prevent an implementation from doing other work internally.

The implementor owns label stability. The trait checks the return type and
lifetime, not a policy. A label taken from `Debug` output changes when the
derived variant name changes, though it can carry other content too; one
derived from a handwritten `Display`, a type name, or a numeric discriminant
can stay unchanged across a rename or change for a reason unrelated to the
name. Use explicit literals for any name that operational tooling may observe,
so a rename and its label change are one deliberate edit.

## Transition tracing fields

Statelet fixes five `tracing` field names for a transition boundary, so
dashboards and log queries can depend on them however the boundary is written:

- `transition.name` — the boundary's stable label.
- `transition.state.before` — the label of the observed state; the
  `StateName::state_name` label is recommended.
- `transition.event` — a stable label for the handled input; a
  low-cardinality semantic class is recommended.
- `transition.outcome` — the label of the result the boundary produced.
- `transition.error` — the bounded category of a returned error.

The
[ADR 0008 field table](adrs/adr-0008-publish-the-transition-tracing-field-contract.md#the-field-contract)
is the authoritative statement of each field's meaning, when it is recorded,
and what its value may contain. The
[`statelet::fields` Rustdoc example](../src/fields.rs) compiles and runs an
instrumented boundary that uses all five.

Name the fields either as dotted field names in the `fields(...)` list — for
example `transition.name = "handle_line"` — or through the exported constants,
which move the spelling under the compiler: a misspelt constant fails to
compile, while a misspelt name silently creates a field nothing reads. Dotted
field names need a resolved `tracing-attributes` 0.1.9 or later, the first
release that parses a dotted key; `tracing` 0.1.9 permits an older
`tracing-attributes`, so the requirement is on the resolved attributes crate,
not on the `tracing` version alone. The compiled constant form needs both
elements of a working pair: a `tracing` facade whose macros accept braced field
names, and a resolved `tracing-attributes` 0.1.31 or later — in practice
`tracing` 0.1.43 or later (`tracing` 0.1.42 is yanked) with
`tracing-attributes` 0.1.31 or later. A newer `tracing-attributes` with an
older facade is not sufficient. Dotted names are the equal alternative.

Two habits keep a span readable. First, use `skip_all` on the instrumented
boundary: without it every argument is recorded with `Debug`, including raw
input that the contract forbids in a trace. Second, declare every late field
when the span is created — `tracing::field::Empty` is the idiomatic declaration
— because a value recorded for an undeclared field is discarded silently.

A declared field that is never recorded stays absent, and absence means only
that the boundary did not capture it. It never implies success or failure: a
`?` or an early return can leave `transition.error` absent even though the
boundary failed, so do not build an alert on a missing field alone.

Recording through `tracing::Span::current()` targets the nearest enabled span.
If the subscriber's filter disables the boundary's own span, the record can
reach an enclosing span that declares the same field, so keep the boundary span
enabled when the field's location matters.

Each boundary call creates one span, filtered by the subscriber's level like
any other `tracing` span, so record a boundary only where the transition is
worth observing. The field names are semver-relevant once a release contains
them: adding a field is compatible, while renaming one or changing its meaning
is a breaking change. Whether Statelet publishes at all is still
[ADR 0003](adrs/adr-0003-record-the-v0-1-exit-register.md)'s decision, so no
release promise attaches to the names yet.

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
