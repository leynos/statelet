# statelet context

Status: Draft v0.1

Audience: Maintainers, contributors, and reviewers working on `statelet`
requirements, design, implementation, and documentation.

Companion documents:

- `docs/terms-of-reference.md`
- `docs/design.md`
- `docs/roadmap.md`: not yet present

Last substantive revision: 2026-06-13

## Purpose

This document defines the vocabulary used by `statelet` design documents and
implementation work. Terms here are normative: when another project document
uses one of these terms, it should mean the definition recorded here.

## Glossary

### State machine

Explicit stateful logic that changes behaviour according to a current state
and input. In `statelet`, a state machine is normally expressed with
project-owned Rust enums, structs, methods, and `match` expressions.

### Ordinary Rust state machine

A state machine whose state, events, errors, storage, and dispatch flow remain
owned by the user's crate. The code should still look like ordinary Rust after
`statelet` is applied.

### State

The value, mode, phase, or variant that determines how a state machine
interprets input. A state may be a field, enum variant, wrapper type, or
computed expression in user code.

### Event

The input or trigger being handled by a transition boundary. `statelet` does
not require users to define a single event enum; an event may be a function
argument, parsed line, command, protocol message, timer tick, or domain value.

### Transition boundary

A method or function where stateful logic decides to stay in the current
state, move to another state, emit output, ignore input, or fail.

### Transition contract

The declared shape of a transition boundary, including how it identifies state,
which event or input it handles, whether it is fallible, which outcome type it
returns, and which observability fields it emits.

### Transition outcome

The value returned by a transition boundary to describe the visible result of
handling input. An outcome may report staying in state, moving between states,
emitting output, ignoring input, or domain-specific completion.

### Fallible transition

A transition boundary whose signature reports domain failure with `Result`.
The design treats fallibility as part of the transition contract, not as an
implementation detail hidden inside the function body.

### Infallible transition

A transition boundary whose signature does not report domain failure with
`Result`. Panics are not considered part of the declared transition contract.

### State name

A stable, low-cost name for a state used in diagnostics, tracing fields, and
generated documentation. A state name is not required to be the same as
`Debug` output.

### Transition instrumentation

Generated observability around a transition boundary. For `statelet`, this
means transition-specific fields such as state, event, transition name,
outcome, and error. It does not mean a general replacement for
`tracing::instrument`.

### Framework

A crate that owns the state-machine object, dispatch model, lifecycle, or graph
semantics. `statelet` is not a framework.

### Domain-specific language

A macro or language surface where the transition graph is declared outside
ordinary Rust control flow. In this project, `DSL` is reserved for this
graph-first style, not for a small attribute that marks ordinary Rust methods.

### Runtime crate

The publishable crate that exposes user-facing types, traits, feature-gated
integrations, documentation, and optionally macro re-exports. For this project,
the runtime crate is expected to be named `statelet`.

### Proc-macro crate

A Rust crate with `proc-macro = true` that exposes procedural macros. A
proc-macro crate normally depends on runtime/support crates, but those crates
must not depend back on it in a way that creates a cycle.

### Support crate

A crate that holds shared implementation detail, test helpers, policy enums, or
developer tooling. A support crate should exist only when it prevents a real
cycle or keeps a stable boundary testable.

### Dependency topology

The directed graph of crate dependencies across runtime, proc-macro,
test-helper, example, and generated-code crates. The topology must stay
acyclic and understandable.

### Circular dependency

A dependency cycle between crates. Circular dependencies are especially risky
in proc-macro work because macro crates, runtime crates, examples, and
test-support crates can easily start depending on each other for convenience.

### Feature flag

A Cargo feature used to add optional capability or integration. In `statelet`,
feature flags should be additive and should not create incompatible variants of
the core transition contract.

### Maintenance safety

Protection against drift in transition naming, instrumentation, fallibility,
and review conventions. Maintenance safety is not a claim of formal graph
safety or typestate correctness.

### Graph safety

Compile-time proof that a state-machine graph admits only declared
transitions. This is a non-goal for initial `statelet` unless later metadata can
provide it without reshaping user code.

### Typestate

A Rust API pattern that encodes state in types so invalid operation order does
not compile. Typestate is out of scope for `statelet` because the target use
case is dynamic stateful logic written as ordinary Rust.

### `mdtablefix` validation spike

The design validation exercise that applies `statelet` to `mdtablefix`
transition-heavy code, especially `ProcessBuffer` and continuation handling.
The spike passes only if `statelet` improves reviewability or diagnostics
without converting the code into a framework or graph-first DSL.

### `rstest-bdd` prior art

The previously implemented Rust proc-macro project in this project family. It
informs `statelet` crate-boundary, compile-diagnostic, and dependency-topology
decisions, especially the risk of circular dependencies.
