# statelet technical design

Status: Draft v0.1

Audience: Maintainers and contributors implementing or reviewing the initial
`statelet` crate design.

Companion documents:

- `docs/terms-of-reference.md`
- `docs/context.md`
- `docs/roadmap.md`: not yet present

Last substantive revision: 2026-06-13

## 1. Design context

`statelet` is a transition-boundary toolkit for ordinary Rust state machines.
It exists for codebases that already want explicit Rust enums, structs,
methods, and `match` expressions, but want shared discipline around transition
contracts, fallibility, and observability.

The design deliberately avoids the full state-machine framework shape. It does
not own dispatch, storage, events, lifecycle hooks, async orchestration,
typestate, or graph validation. Existing crates such as `statig`, `rust-fsm`,
`smlang`, `sfsm`, `typed-fsm`, `finny`, and `macro-machines` already cover
those product shapes.

The first technical test is not whether `statelet` can model every finite
state machine. The test is whether it can improve real transition-heavy code,
especially `mdtablefix` `ProcessBuffer` and continuation handling, without
making that code look alien.

## 2. Goals and non-goals

### 2.1 Goals

- Provide a small runtime crate with transition outcome and state naming
  vocabulary.
- Provide an attribute macro that marks ordinary Rust functions and methods as
  transition boundaries.
- Generate transition-specific observability only when the relevant feature is
  enabled.
- Check declared fallibility contracts at compile time where Rust syntax makes
  that possible.
- Keep branch logic, state mutation, event modelling, and domain errors in
  user code.
- Keep dependency topology acyclic and checked in routine validation.

### 2.2 Non-goals

- No generated dispatcher.
- No required event enum.
- No required state-machine object.
- No guard/action DSL.
- No graph-first syntax.
- No typestate API.
- No initial claim of `no_std`, interrupt-safety, or embedded suitability.
- No diagram generation in v0.1.

## 3. Design decisions

### 3.1 Use a workspace from the start

Decision: create a Cargo workspace with a runtime crate and a proc-macro crate
from the first implementation slice.

Rust procedural macros must be defined in a `proc-macro` crate and cannot be
used from the crate where they are defined. A split is therefore not premature
architecture; it is a Rust language constraint. A workspace lets the project
share package metadata, dependency versions, lint policy, and validation
commands while keeping the publishable crates separate.

### 3.2 Make the runtime crate the vocabulary owner

Decision: `statelet` owns public types, traits, documentation, feature flags,
and optional macro re-exports. `statelet-macros` owns token parsing and code
generation only.

This keeps the runtime API usable without procedural macros and prevents macro
implementation detail from becoming the public contract. It also gives
examples, tests, and downstream users one stable vocabulary crate.

### 3.3 Prefer attribute macros over a graph DSL

Decision: v0.1 exposes `#[transition(...)]` for functions and methods. It does
not expose a transition-table macro.

The user job is to mark existing transition boundaries, not to re-express the
state machine as a table. An attribute macro can wrap a function while leaving
the function body visible to reviewers.

### 3.4 Treat tracing as an integration, not the core model

Decision: tracing support is behind a `tracing` feature. The default feature
set is empty until an implementation spike proves that observability must be
on by default.

Cargo default features are hard to remove without breaking downstream users.
The ToR calls observability central, but not every state machine consumer wants
a tracing dependency. The design should make tracing easy and documented
without making it unavoidable.

### 3.5 Defer diagrams and transition-table metadata

Decision: v0.1 records no graph metadata and generates no diagrams.

Diagram metadata risks pulling the crate toward graph ownership. That is the
line where `statelet` starts competing with existing frameworks and DSLs. The
`mdtablefix` spike should happen before any graph-shaped feature is accepted.

## 4. Workspace topology

The initial workspace should contain only crates that earn a boundary.

<!-- markdownlint-disable MD013 -->

| Crate | Publish | Responsibility | May depend on |
| --- | --- | --- | --- |
| `statelet` | Yes | Runtime vocabulary, traits, feature-gated integrations, documentation, and optional macro re-exports | External runtime dependencies; optionally `statelet-macros` for re-export only |
| `statelet-macros` | Yes | Attribute and derive macros, syntax parsing, generated token output, compile-time diagnostics | `statelet`, `syn`, `quote`, `proc-macro2`, diagnostic helpers |
| `statelet-test-support` | No, only if needed | Shared test fixtures for compile tests and examples | `statelet`; optionally `statelet-macros` as a dev dependency |
| `examples/*` | No | Realistic consumer crates, including the `mdtablefix` validation spike if vendored locally | `statelet` as a consumer would |
| `xtask` or tooling crate | No, only if needed | Dependency-topology checks that cannot be expressed cleanly in Makefile targets | Cargo metadata crates and process helpers |

<!-- markdownlint-enable MD013 -->

The first implementation should start with `statelet` and `statelet-macros`
only. Add `statelet-test-support` or `xtask` only when tests duplicate enough
setup to justify the split.

## 5. Dependency rules

The workspace dependency graph must obey these rules:

- `statelet-macros` may depend on `statelet`.
- `statelet` must not require `statelet-macros` for its core runtime API.
- If `statelet` re-exports macros, that dependency must be optional and
  feature-gated.
- Test crates and examples may depend on both `statelet` and
  `statelet-macros`.
- No support crate may depend on a crate that already depends on it.
- Generated-code tests must compile as downstream users, not by reaching into
  private macro internals.

The routine gate must include a dependency-topology check. At minimum, it
should run `cargo tree --workspace --edges normal,build,dev` and a focused
reverse-dependency inspection for `statelet`, `statelet-macros`, and any
support crate. If that is too noisy, the project should add a small validation
tool that consumes `cargo metadata` and fails on forbidden workspace edges.

## 6. Public API

### 6.1 Runtime vocabulary

The runtime crate should expose concrete types first. Traits are introduced
only when a concrete type prevents a real use case.

```rust
pub enum TransitionOutcome<S, O = ()> {
    Stay { state: S, output: O },
    Move { from: S, to: S, output: O },
    Emit(O),
    Ignore,
}
```

This type is intentionally small. It is not a graph model, and it does not
encode every state-machine theory term. It gives transition boundaries a common
language for the results that matter to maintainers.

The `Stay` variant includes `state` rather than leaving it implicit because
explicit state reporting gives instrumentation a stable field. If the
implementation spike finds that this duplicates too much user code, v0.1 may
replace it with `Stay(O)`.

### 6.2 State naming

The runtime crate should expose a state naming trait and a derive macro:

```rust
pub trait StateName {
    fn state_name(&self) -> &'static str;
}
```

The derive macro should support enums first. Struct support can wait until a
real use case needs it. The generated names should default to variant names and
allow explicit renames later only if examples prove the need.

### 6.3 Attribute macro

The user-facing macro should start with this shape:

```rust
#[transition(
    state = "self.mode",
    event = "line",
    fallible,
    tracing(level = "trace")
)]
fn handle_line(&mut self, line: &str) -> Result<Decision, Error> {
    // ordinary Rust
}
```

The `state` and `event` entries are Rust expressions encoded as string
literals in v0.1. This avoids inventing a nested DSL in the attribute grammar
and lets the macro parse them as expressions with `syn`.

The macro may also accept `infallible`:

```rust
#[transition(state = "self.mode", event = "event", infallible)]
fn step(&mut self, event: Event) -> Decision {
    // ordinary Rust
}
```

If neither `fallible` nor `infallible` is supplied, the macro should infer the
shape when it can, and emit a warning or compile error only if the design
explicitly chooses strict declaration. The safer v0.1 default is to require the
declaration. That makes examples noisier but keeps the contract reviewable.

## 7. Macro expansion contract

The `#[transition]` macro wraps the original function body but must preserve
the user's control flow.

The macro may:

- parse the annotated function or method signature;
- validate fallible or infallible return shape;
- evaluate configured state and event expressions at the transition boundary;
- emit tracing fields when the `tracing` feature is enabled;
- record the returned outcome or error when supported;
- preserve attributes that should remain on the function;
- produce compile errors with spans on invalid macro input.

The macro must not:

- rewrite branch logic into a transition table;
- introduce a dispatcher;
- require a state enum or event enum;
- require a specific error type;
- require `TransitionOutcome` as the only return type;
- hide domain errors behind a crate-defined error type;
- change function visibility or call convention except where wrapping requires
  a private implementation function.

Generated code must use absolute paths such as `::statelet::...` by default.
If crate renaming support is needed, use `proc-macro-crate` rather than relying
on a hard-coded local crate name. This follows the Rust Reference warning that
procedural macros are unhygienic.

## 8. Fallibility checking

The fallibility check is syntactic. It should not try to prove semantic error
behaviour.

For `fallible`, the macro accepts return types whose outer shape is
`Result<T, E>` or a path resolved syntactically as `core::result::Result` or
`std::result::Result`. Type aliases are not reliably knowable from token
syntax, so v0.1 should reject aliases unless a later spike proves a better
diagnostic strategy.

For `infallible`, the macro rejects a syntactic `Result<_, _>` return type.
It does not reject panics, because panics are not part of the declared domain
failure contract.

The compile errors should point at the fallibility argument or the return type,
not at the whole function. `syn` span information and `trybuild` snapshots are
the verification mechanism for those diagnostics.

## 9. Observability

The `tracing` feature should generate transition-specific instrumentation. It
should not duplicate every option in `tracing-attributes::instrument`.

The default generated fields are:

- `transition.name`
- `transition.state.before`
- `transition.event`
- `transition.outcome`
- `transition.error`

The first implementation may record only a subset if the return type cannot be
inspected without imposing trait bounds. The macro should prefer explicit,
documented omissions over broad `Debug` bounds that make user functions harder
to compile.

The macro should use `tracing` spans or events directly, not nest
`#[tracing::instrument]` inside the generated output. `tracing-attributes`
already handles generic function instrumentation; `statelet` adds state
transition semantics.

## 10. Feature policy

The initial feature set should be:

<!-- markdownlint-disable MD013 -->

| Feature | Default | Purpose | Design rule |
| --- | --- | --- | --- |
| `macros` | Probably yes | Re-export `statelet-macros` from `statelet` | May depend on `statelet-macros`; must not be needed for runtime-only use |
| `tracing` | No | Enable transition-specific tracing output | Adds `tracing`; does not change transition semantics |
| `derive` | Probably yes if derives live in `statelet-macros` | Enable `StateName` derive re-export | Must not pull in tracing |
| `serde` | No | Serialize runtime vocabulary if needed | Must not affect macro expansion |

<!-- markdownlint-enable MD013 -->

The `macros` default is the main open trade-off. Re-exporting macros from the
runtime crate is ergonomic, but it creates a runtime-to-macro optional edge.
The edge is publishable if optional and acyclic, but the project must check it.
If topology checks become noisy, v0.1 should require users to depend on
`statelet-macros` explicitly.

The design must not include mutually exclusive features unless no alternative
exists. Cargo features unify across dependency graphs, so incompatible features
create downstream coordination problems.

## 11. Validation strategy

Validation is part of the design, not an implementation afterthought.

### 11.1 Compile-time diagnostics

Use `trybuild` for macro misuse and compiler-output stability:

- invalid fallible function returning a non-`Result`;
- invalid infallible function returning `Result`;
- invalid `state` expression syntax;
- invalid `event` expression syntax;
- unsupported item kind;
- renamed crate path if crate renaming support exists;
- supported `async fn` only if the implementation claims support.

Each compile-fail fixture should target one diagnostic. If rustc output varies
across stable releases, reduce the fixture until the volatile suggestion text
is not part of the assertion.

### 11.2 Pass tests

Use pass tests for supported macro shapes:

- free functions;
- inherent methods with `&self`;
- inherent methods with `&mut self`;
- fallible transitions;
- infallible transitions;
- generic functions only after a real example requires them;
- tracing-enabled builds;
- no-default-feature builds.

### 11.3 Feature combinations

At minimum, CI or local gates should check:

- no default features;
- default features;
- `tracing`;
- `serde`;
- all features.

If more optional features appear, add a small feature-matrix tool or
`cargo-hack` equivalent before the matrix becomes unreviewable.

### 11.4 Dependency topology

The dependency-topology gate must fail on forbidden workspace cycles or
unexpected edges. `rstest-bdd` showed that macro, runtime, harness, examples,
and test-support crates can grow complex quickly. `statelet` should keep the
first topology small and checked.

The minimum manual gate is:

```text
cargo tree --workspace --edges normal,build,dev
cargo tree --workspace --edges features
cargo tree --workspace --duplicates
```

A later tool may replace manual inspection with a machine-readable
`cargo metadata` check. `lading` and `whitaker` can mitigate this class of
problem, but their existence does not remove the need for direct topology
tests in this repository.

## 12. `mdtablefix` validation spike

The `mdtablefix` spike is the acceptance test for usefulness.

The spike must apply the proposed API to:

- `ProcessBuffer` line handling;
- continuation mode handling;
- at least one fallible transition if the code has one;
- at least one infallible transition.

The spike passes only if all of these are true:

- branch logic remains in ordinary Rust;
- no generated dispatcher appears;
- no event enum is introduced only to satisfy `statelet`;
- reviewers can identify transition boundaries faster than before;
- diagnostics or tracing fields show state, event, and outcome information that
  was not already available in a consistent form;
- the diff is small enough that a maintainer would plausibly accept it.

The spike fails if the most honest result is a local helper function or
`tracing::instrument` annotation without `statelet`.

## 13. Failure modes

### 13.1 Macro hides control flow

If the macro moves predicates or side effects out of the function body,
`statelet` has crossed into DSL territory. The design prevents this by limiting
the macro to boundary instrumentation and syntactic contract checks.

### 13.2 Diagnostics point at the wrong code

Bad macro diagnostics are worse than ordinary Rust errors. The macro parser
must preserve spans for attribute arguments and return types, and `trybuild`
must lock down the user-facing output that matters.

### 13.3 Feature creep turns the crate into a framework

Requests for dispatch, graph validation, entry/exit hooks, lifecycle, async
orchestration, or typestate should be redirected to existing crates unless a
new ToR revision changes the product boundary.

### 13.4 Dependency cycles appear through convenience re-exports

Optional macro re-exports are useful but dangerous. The topology gate must
catch cycles and unexpected support-crate edges before they become release
constraints.

### 13.5 Tracing becomes mandatory by accident

The default build and no-default-feature build must compile without `tracing`.
Any public type that requires `tracing` belongs behind the `tracing` feature.

## 14. Deferred decisions

The implementation should resolve these before publishing v0.1:

- Whether `statelet` re-exports macros by default.
- Whether `TransitionOutcome::Stay` carries the current state.
- Whether fallibility declaration is required or inferred.
- Whether `StateName` derive belongs behind `derive`, `macros`, or both.
- Whether type aliases for `Result` are rejected or accepted with a documented
  limitation.
- Whether `async fn` support is explicitly tested in v0.1 or documented as
  unsupported.
- Project licence, MSRV, and crates.io metadata.

## 15. References

- `docs/terms-of-reference.md`
- `docs/context.md`
- Rust Reference, procedural macros:
  `https://doc.rust-lang.org/reference/procedural-macros.html`
- Cargo Book, workspaces:
  `https://doc.rust-lang.org/cargo/reference/workspaces.html`
- Cargo Book, features:
  `https://doc.rust-lang.org/cargo/reference/features.html`
- Cargo Book, `cargo tree`:
  `https://doc.rust-lang.org/cargo/commands/cargo-tree.html`
- `tracing-attributes::instrument` documentation:
  `https://docs.rs/tracing-attributes/latest/tracing_attributes/attr.instrument.html`
- `syn` documentation:
  `https://docs.rs/syn/latest/syn/`
- `quote` documentation:
  `https://docs.rs/quote/latest/quote/`
- `trybuild` documentation:
  `https://docs.rs/trybuild/latest/trybuild/`
- `rstest-bdd` repository:
  `https://github.com/leynos/rstest-bdd/`
