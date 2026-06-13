# statelet technical design

Status: Draft v0.2 after design review

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
`smlang`, `sfsm`, `typed-fsm`, `finny`, `stateless`, and `macro-machines`
already cover those product shapes. `stateless` is especially relevant because
it already offers a zero-cost transition-table macro that separates structure
from behaviour. `statelet` must therefore stay out of transition-table
ownership and focus on marking boundaries in code that already exists.

The first technical test is not whether `statelet` can model every finite state
machine. The test is whether it can improve real transition-heavy code,
especially `mdtablefix` `ProcessBuffer` and continuation handling, without
making that code look alien.

The design review exposed two low-confidence bets that now control sequencing:
whether users want a crate at all for this problem, and whether a
`#[transition]` macro adds enough value over `#[tracing::instrument]` plus a
small conventions crate. The implementation must answer those bets before
publishing a macro API.

## 2. Goals and non-goals

### 2.1 Goals

- Provide a small runtime crate with transition naming and observability
  conventions that can stand without a proc macro.
- Build a non-macro `mdtablefix` baseline using `tracing::instrument`, helper
  functions, and any runtime vocabulary before implementing the macro.
- Add an attribute macro only after the baseline shows repetitive boilerplate
  that a macro can remove without hiding control flow.
- Generate transition-specific observability only when the relevant feature is
  enabled.
- Treat fallibility declarations as descriptive in the first macro slice, with
  optional syntactic checking only where the diagnostic is reliable.
- Keep branch logic, state mutation, event modelling, and domain errors in
  user code.
- Keep dependency topology acyclic and checked in routine validation from the
  first commit that introduces a second crate or feature matrix.

### 2.2 Non-goals

- No generated dispatcher.
- No required event enum.
- No required state-machine object.
- No guard/action DSL.
- No graph-first syntax.
- No typestate API.
- No initial claim of `no_std`, interrupt-safety, or embedded suitability.
- No diagram generation in v0.1.
- No transition-table ownership.
- No published `TransitionOutcome` enum unless a validation spike proves that
  users or macros actually consume it.

## 3. Design decisions

### 3.1 Sequence the runtime crate before the macro crate

Decision: start implementation with the runtime/conventions crate. Introduce
`statelet-macros` only after the `mdtablefix` baseline proves that a macro beats
`#[tracing::instrument]` plus helper functions by a stated margin.

Rust procedural macros must be defined in a `proc-macro` crate and cannot be
used from the crate where they are defined. If `statelet` ships a macro, the
split is a Rust language constraint. The split is not justified before the
macro earns its place. This sequence keeps the maintenance and dependency
topology cost proportional to proven value.

### 3.2 Make the runtime crate the vocabulary owner

Decision: `statelet` owns public traits, documentation, feature flags, tracing
field contracts, and any runtime vocabulary that survives validation.
`statelet-macros`, if added, owns token parsing and generated wrapper code only.

This keeps the runtime API usable without procedural macros and prevents macro
implementation detail from becoming the public contract. It also creates the
strongest baseline against which the macro must compete.

### 3.3 Prefer attribute macros over a graph DSL

Decision: if the macro gate passes, the macro surface is
`#[statelet::transition(...)]` for functions and methods. The project never
exposes a transition-table macro.

The user job is to mark existing transition boundaries, not to re-express the
state machine as a table. An attribute macro can wrap a function while leaving
the function body visible to reviewers. A transition-table macro would compete
with `stateless` and with graph-first crates rather than with local convention.

### 3.4 Treat tracing as an integration, not the core model

Decision: the runtime crate's first useful integration is `tracing`. Whether the
`tracing` feature is enabled by default is a release decision made after the
conventions-only baseline. The no-default-feature build must remain useful for
state naming and documentation.

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

The initial repository should contain only crates that earn a boundary. The
runtime crate earns a boundary immediately. The macro crate earns one only if
the baseline gate passes.

<!-- markdownlint-disable MD013 -->

| Crate                    | Publish              | Responsibility                                                                                | May depend on                                                                  |
| ------------------------ | -------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `statelet`               | Yes                  | Runtime traits, tracing field contract, conventions documentation, and any proven vocabulary  | External runtime dependencies; optionally `statelet-macros` for re-export only |
| `statelet-macros`        | Yes, only after gate | Attribute and derive macros, syntax parsing, generated token output, compile-time diagnostics | `statelet`, `syn`, `quote`, `proc-macro2`, diagnostic helpers                  |
| `statelet-test-support`  | No, only if needed   | Shared test fixtures for compile tests and examples                                           | `statelet`; optionally `statelet-macros` as a dev dependency                   |
| `examples/*`             | No                   | Realistic consumer crates, including the `mdtablefix` validation spike if vendored locally    | `statelet` as a consumer would                                                 |
| `xtask` or tooling crate | No, only if needed   | Dependency-topology checks that cannot be expressed cleanly in Makefile targets               | Cargo metadata crates and process helpers                                      |

<!-- markdownlint-enable MD013 -->

The first implementation should start with `statelet` only. Add
`statelet-macros` after the macro gate passes. Add `statelet-test-support` or
`xtask` only when tests duplicate enough setup or topology checks to justify
the split.

## 5. Dependency rules

The workspace dependency graph must obey these rules:

- `statelet-macros` may depend on `statelet`, but may not exist until the
  macro baseline gate passes.
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

### 6.1 State naming

The runtime crate should expose a state naming trait and a derive macro:

```rust
pub trait StateName {
    fn state_name(&self) -> &'static str;
}
```

The derive macro should support enums first:

```rust
#[derive(StateName)]
enum ContinuationMode {
    Normalize,
    TightCodeSpan,
    VerbatimFlush,
}
```

Struct support can wait until a real use case needs it. The generated names
should default to variant names and allow explicit renames later only if
examples prove the need.

### 6.2 Transition outcome vocabulary

Do not publish `TransitionOutcome` in the first implementation slice. The
design review identified a compatibility risk: if the macro does not produce or
consume the type, the enum becomes documentation-only public API.

The implementation may prototype a private or example-local outcome enum while
building the `mdtablefix` baseline:

```rust
enum TransitionOutcome<O = ()> {
    Stay(O),
    Move(O),
    Emit(O),
    Ignore,
}
```

Publishing a crate-level outcome type requires one of these proofs:

- the non-macro baseline uses the type in real transition code and reviewers
  find it clearer than a project-local `Decision` enum;
- the macro consumes the type to produce observability without extra user
  boilerplate;
- the macro produces the type in a way that does not force users to reshape
  their domain result model.

If the type is published, do not include duplicated current state in `Stay`.
The macro should read state from the declared state expression rather than
making users restate `self.mode` in their return value.

### 6.3 Attribute macro

The user-facing macro, if the gate passes, should start with this shape:

```rust
#[statelet::transition(
    state(self.mode),
    event(line),
    fallible,
    tracing(level = "trace")
)]
fn handle_line(&mut self, line: &str) -> Result<Decision, Error> {
    // ordinary Rust
}
```

The `state(...)` and `event(...)` entries are real Rust expression tokens
parsed as `syn::Expr`, not string literals. This preserves spans, keeps IDE
rename and completion behaviour closer to ordinary Rust, and avoids the
stringly-typed contract called out by the design review.

The macro may also accept `infallible`:

```rust
#[statelet::transition(state(self.mode), event(event), infallible)]
fn step(&mut self, event: Event) -> Decision {
    // ordinary Rust
}
```

If neither `fallible` nor `infallible` is supplied, the macro should omit
fallibility-specific fields rather than guessing. In v0.1, `fallible` and
`infallible` are descriptive by default: they drive field names and examples.
An explicit `check_return` option may enable syntactic return-shape validation
for users who want hard compile-time checks and accept the alias limitations.

## 7. Macro expansion contract

The `#[transition]` macro wraps the original function body but must preserve
the user's control flow.

The macro may:

- parse the annotated function or method signature;
- validate fallible or infallible return shape only when the user opts into a
  strict syntactic check;
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
- parse state or event expressions from string literals;
- change function visibility or call convention except where wrapping requires
  a private implementation function.

Generated code must use absolute paths such as `::statelet::...` by default. If
crate renaming support is needed, use `proc-macro-crate` rather than relying on
a hard-coded local crate name. This follows the Rust Reference warning that
procedural macros are unhygienic.

## 8. Fallibility declarations

Fallibility is descriptive by default in v0.1. It should not try to prove
semantic error behaviour, and it should not reject valid code merely because
the return type uses an alias or `Result<T, Infallible>`.

When `fallible` is present, the macro may record transition error fields if the
returned value is visibly an error and can be observed without imposing broad
trait bounds. When `infallible` is present, the macro omits error-specific
fields. Both declarations document intent at the transition boundary.

If the user adds `check_return`, the macro performs only a syntactic check:

- `fallible, check_return` accepts visible `Result<T, E>`,
  `core::result::Result<T, E>`, or `std::result::Result<T, E>`;
- `infallible, check_return` rejects a visible `Result<_, _>` return type;
- type aliases are reported as unsupported, not inferred.

The compile errors must point at the fallibility argument or return type, not
at the whole function. `syn` span information and `trybuild` snapshots are the
verification mechanism for those diagnostics. This keeps the hard check honest:
it is a lint-like syntactic contract, not semantic proof.

## 9. Observability

The `tracing` feature should generate transition-specific instrumentation. It
should not duplicate every option in `tracing-attributes::instrument`.

The default generated fields are:

- `transition.name`
- `transition.state.before`
- `transition.event`
- `transition.outcome`
- `transition.error`

These field names are public operational API. Dashboards, alerts, and log
queries may depend on them. Renaming or changing their meaning is
semver-relevant even if Rust type checking cannot detect the break.

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

| Feature   | Default                   | Purpose                                     | Design rule                                                              |
| --------- | ------------------------- | ------------------------------------------- | ------------------------------------------------------------------------ |
| `macros`  | No until gate passes      | Re-export `statelet-macros` from `statelet` | May depend on `statelet-macros`; must not be needed for runtime-only use |
| `tracing` | Open                      | Enable transition-specific tracing output   | Adds `tracing`; does not change transition semantics                     |
| `derive`  | Same decision as `macros` | Enable `StateName` derive re-export         | Must not pull in tracing                                                 |
| `serde`   | No                        | Serialize runtime vocabulary if needed      | Must not affect macro expansion                                          |

<!-- markdownlint-enable MD013 -->

The `macros` and `derive` defaults are one decision, not two. If macro
re-export is enabled by default, `StateName` derive follows the same path. If
macro re-export is not default, users depend on `statelet-macros` explicitly.
This keeps feature reasoning simple and avoids two interacting toggles for
macro surface.

The `tracing` default is decided after the conventions baseline. Observability
is the product's main value, but default dependencies are sticky. The design
must record the chosen default and its trade-off before v0.1 is published.

The design must not include mutually exclusive features unless no alternative
exists. Cargo features unify across dependency graphs, so incompatible features
create downstream coordination problems.

## 11. Validation strategy

Validation is part of the design, not an implementation afterthought.

### 11.1 Bet register

The design review identified the following bets. Each bet must have an
evidence-producing gate before the affected API is published.

<!-- markdownlint-disable MD013 -->

| Bet | Claim                                                                          | Confidence | Required evidence                                                                             |
| --- | ------------------------------------------------------------------------------ | ---------- | --------------------------------------------------------------------------------------------- |
| B1  | A real segment prefers hand-written state machines and wants shared convention | Low-medium | `mdtablefix` plus one second non-toy example both improve without framework adoption          |
| B2  | `#[transition]` beats `#[tracing::instrument]` plus helper functions           | Low        | A head-to-head `mdtablefix` baseline comparison states the concrete value added by the macro  |
| B3  | The attribute argument grammar is acceptable to ordinary Rust users            | Medium     | Macro syntax uses real expression tokens, and diagnostics point at user-written expressions   |
| B4  | Fallibility declarations help more than they annoy                             | Medium     | Hard return-shape checks are opt-in, and alias limitations are documented and tested          |
| B5  | Dependency topology stays acyclic and feature leakage is catchable             | High       | CI runs feature-matrix and topology checks from the first macro-crate commit                  |
| B6  | `mdtablefix` is representative enough to generalize the wedge                  | Low-medium | A second validation example is named before release and evaluated before publishing the macro |

<!-- markdownlint-enable MD013 -->

### 11.2 Baseline comparison

Before implementing `statelet-macros`, build the `mdtablefix` baseline without
`#[transition]`. The baseline may use:

- `StateName`;
- documented `transition.*` tracing field names;
- project-local helper functions;
- `#[tracing::instrument(fields(...))]`;
- a project-local outcome enum if the code already benefits from one.

The macro gate passes only if the design note can state, in one paragraph, what
`#[transition]` adds over that baseline. Acceptable added value includes
removing repeated field capture, preventing field-name drift, preserving
consistent entry/exit logging, or reducing boilerplate enough that the
annotated code is easier to review. If the baseline is the more honest result,
the macro crate does not ship in v0.1.

### 11.3 Compile-time diagnostics

Use `trybuild` for macro misuse and compiler-output stability if the macro gate
passes:

- invalid `check_return` fallible function returning a non-`Result`;
- invalid `check_return` infallible function returning `Result`;
- invalid `state` expression syntax;
- invalid `event` expression syntax;
- unsupported item kind;
- renamed crate path support using `proc-macro-crate`;
- diagnostics that land on user-written expression tokens, not synthetic string
  reparses;
- supported `async fn` only if the implementation claims support.

Each compile-fail fixture should target one diagnostic. If rustc output varies
across stable releases, reduce the fixture until the volatile suggestion text
is not part of the assertion.

### 11.4 Pass tests

Use pass tests for supported macro shapes:

- free functions;
- inherent methods with `&self`;
- inherent methods with `&mut self`;
- fallible transitions;
- infallible transitions;
- generic functions only after a real example requires them;
- tracing-enabled builds;
- no-default-feature builds.

### 11.5 Feature combinations

From the first commit that introduces optional macro or tracing features, CI or
local gates should check:

- no default features;
- default features;
- `tracing`;
- `serde`;
- all features.

Use `cargo-hack` or an equivalent from the first feature-matrix commit, not
after the matrix becomes painful. Manual inspection is not enough for the
feature-leak risk called out by the design review.

### 11.6 Dependency topology

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

The required automated gate consumes `cargo metadata` and fails on forbidden
workspace edges. Manual `cargo tree` output may aid debugging, but it is not
the acceptance mechanism. `lading` and `whitaker` can mitigate this class of
problem, but their existence does not remove the need for direct topology tests
in this repository.

### 11.7 Compile-time and binary-size budget

The `mdtablefix` spike must measure build-time and binary-size impact before
the macro is accepted. The initial budget is:

- annotating 20 transition boundaries in a downstream crate adds no more than
  10% to clean debug build time compared with the non-macro baseline;
- the release binary or library artefact grows by no more than 2% compared with
  the non-macro baseline;
- generated code size stays understandable in `cargo expand` for one annotated
  fallible transition and one annotated infallible transition.

If these budgets are too strict or too loose in practice, revise the numbers in
the design note before relaxing the gate. Do not ship the macro without a
stated cost budget.

## 12. `mdtablefix` validation spike

The `mdtablefix` spike is the first acceptance test for usefulness. It has two
phases.

Phase 1 builds the non-macro baseline:

- annotate `ProcessBuffer` and continuation handling with ordinary
  `#[tracing::instrument(fields(...))]`;
- use any runtime `StateName` or field-name conventions that `statelet` would
  publish;
- keep helper functions local to `mdtablefix` unless they prove generally
  reusable;
- record the boilerplate, reviewability, diagnostics, build time, and artefact
  size.

Phase 2 may apply the proposed macro API only if Phase 1 shows repetitive
boilerplate or drift risk that a macro can remove. The macro spike must apply
the proposed API to:

- `ProcessBuffer` line handling;
- continuation mode handling;
- at least one fallible transition if the code has one;
- at least one infallible transition.

The macro spike passes only if all of these are true:

- branch logic remains in ordinary Rust;
- no generated dispatcher appears;
- no event enum is introduced only to satisfy `statelet`;
- reviewers can identify transition boundaries faster than before;
- diagnostics or tracing fields show state, event, and outcome information that
  was not already available in a consistent form;
- the diff is small enough that a maintainer would plausibly accept it.
- it beats the non-macro baseline by a stated margin;
- it stays inside the compile-time and binary-size budget in §11.7.

The spike fails if the most honest result is a local helper function or
`tracing::instrument` annotation without `statelet`.

The second non-toy validation candidate is `lading`'s publish workflow phase
coordination. It exercises a different domain from Markdown table repair:
release planning, package ordering, dry-run/live execution, and failure
handling. Before publishing the macro, the project must either validate
`statelet` there or replace it with a better named example and record why.

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

### 13.6 Macro is only `instrument` with extra steps

If the macro cannot beat the baseline in §11.2, the project ships the
conventions/runtime crate and defers `statelet-macros`. This is a successful
validation outcome, not a failed implementation.

### 13.7 Attribute syntax looks alien

String-encoded expressions are forbidden because they break the product thesis:
they lose editor support, produce weaker spans, and make ordinary Rust look
like a mini DSL. Attribute arguments must use real expression tokens.

### 13.8 Feature leakage appears without a cycle

Feature unification can pull `tracing` or macro dependencies into builds where
users disabled them even when the dependency graph is acyclic. The
feature-matrix and topology gates must check for leaks, not only cycles.

## 14. Deferred decisions

The implementation should resolve these before publishing v0.1:

- Whether `statelet` re-exports macros by default.
- Whether `statelet-macros` ships in v0.1 or waits for a later release.
- Whether any `TransitionOutcome` type is published at all.
- Whether tracing is a default feature.
- Whether `StateName` derive follows the `macros` feature or requires an
  explicit proc-macro dependency.
- Whether `async fn` support is explicitly tested in v0.1 or documented as
  unsupported. The default answer is unsupported until tracing across `.await`
  is proven safe and unsurprising.
- Which second validation example is accepted if `lading` is replaced.
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
- `stateless` documentation:
  `https://docs.rs/stateless`
