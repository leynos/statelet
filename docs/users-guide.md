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

## Quick start

Run the public validation entrypoint from a fresh checkout:

```bash
git clone https://github.com/leynos/statelet.git
cd statelet
make all
```

The most useful public commands are:

- `make all` builds confidence by running formatting checks, linting, and
  tests.
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
