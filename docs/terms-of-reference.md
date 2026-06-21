# statelet - terms of reference

Status: Draft v0.2 after design review

Audience: Product owner, engineering lead, maintainers, crate reviewers, and
contributors evaluating whether `statelet` should exist before technical design
starts.

Companion documents:

- `docs/context.md`
- `docs/design.md`
- `docs/roadmap.md`
- `docs/adr-001-proving-ground-candidates.md`

Last substantive revision: 2026-06-14

## 1. Background and motivation

Rust already has a crowded state-machine crate ecosystem. On 2026-06-13,
crates.io listed 154 crates under the `state-machine` keyword. Established
options cover hierarchical state machines, event-driven dispatch, transition
table DSLs, static embedded generators, typestate-oriented APIs, logging, and
diagram output.

That market reality makes a broad "another Rust state-machine framework"
position weak. The useful question is narrower: whether teams that already
write ordinary enum- or struct-backed Rust state machines need a small library
that standardizes transition boundaries without taking over dispatch, storage,
event modelling, or control flow.

The motivating pattern comes from parser, stream-processing, and protocol code
where the hand-written state machine is the right model, but the surrounding
discipline drifts. Maintainers want transition methods to be recognizable,
observable, and explicit about fallibility. They do not necessarily want a
statechart engine, a graph-first DSL, or a generated runtime.

`statelet` exists if that wedge is real. It should be evaluated as a
maintenance-safety toolkit for ordinary Rust state machines, not as a framework
competing to own the whole state-machine model.

The first validation step is not a macro. It is the strongest non-macro
baseline: documented conventions, `StateName`, stable tracing field names,
local helpers, and `#[tracing::instrument(fields(...))]`. A macro belongs in
`statelet` only if it beats that baseline on real code.

## 2. Domain

The domain is Rust state-machine implementation for small to medium application
logic. The relevant field is not formal modelling by itself; it is the
day-to-day practice of implementing parser modes, protocol phases, retry
states, import pipelines, command workflows, and stream processors in ordinary
Rust.

The dominant local convention in this domain is explicit Rust control flow:
enums for modes, structs for accumulated state, methods for transition
boundaries, and `match` expressions for branch logic. This convention keeps
domain predicates and side effects visible in code review. It also creates
maintenance risks when teams copy a pattern by memory instead of naming and
checking the pattern.

Prior art falls into four broad groups:

<!-- markdownlint-disable MD013 -->

| Group                           | Examples                                | What they optimize for                                                    |
| ------------------------------- | --------------------------------------- | ------------------------------------------------------------------------- |
| Event-driven frameworks         | `statig`, `rust-fsm`                    | A state machine as an object that accepts events through a runtime API    |
| Transition-table DSLs           | `smlang`, `state-machines`, `stateless` | A graph or transition list as the centre of the model                     |
| Static and embedded generators  | `sm`, `sfsm`, `typed-fsm`, `finny`      | Compile-time validation, low overhead, `no_std`, and embedded suitability |
| Graph and logging macro helpers | `macro-machines`                        | Macro-defined machines with logging and Graphviz-style output             |

<!-- markdownlint-enable MD013 -->

`statelet` should coexist with these crates. Its domain boundary is the
hand-written state machine whose author wants a named transition contract and
consistent observability, while keeping the machine's state, event, error,
storage, and dispatch decisions in ordinary project code.

Internal prior art also matters. The `rstest-bdd` project is an example of a
proc-macro crate previously implemented in this project family. It shows that
the team can ship Rust macro tooling, but it also supplies a caution: avoiding
circular dependencies is difficult once macro crates, runtime crates, test-only
support crates, and generated examples start depending on each other.
`statelet` should treat dependency topology as a product risk from the start,
not as a clean-up task after the API settles.

## 3. Market context

The direct competitive landscape is active and varied:

<!-- markdownlint-disable MD013 -->

| Crate            | Current signal                                           | Relevance to `statelet`                                                      |
| ---------------- | -------------------------------------------------------: | ---------------------------------------------------------------------------- |
| `statig`         | 0.4.1; 805,049 recent downloads on crates.io, 2026-06-13 | Mature hierarchical, event-driven machine with generated state machinery     |
| `rust-fsm`       | 0.8.0; 177,910 recent downloads on crates.io, 2026-06-13 | Framework plus DSL for finite state machines                                 |
| `smlang`         | 0.8.0; 163,007 recent downloads on crates.io, 2026-06-13 | Procedural macro DSL with guards, actions, async support, and generated docs |
| `sm`             | 0.9.0; updated 2019-11-09 on crates.io                   | Static macro-defined state machines; older but still downloaded              |
| `typed-fsm`      | 0.4.8 on docs.rs                                         | Embedded-oriented event-driven FSM generator with compile-time validation    |
| `finny`          | 0.2.0 on docs.rs                                         | Builder-style procedural macro with dispatcher, guards, and hierarchy        |
| `sfsm`           | 0.4.3 on docs.rs                                         | Static, embedded-oriented generator with transition and state traits         |
| `stateless`      | docs.rs describes a zero-cost transition-table macro     | Closest warning against owning transition tables                             |
| `macro-machines` | 0.10.8; 141 recent downloads on crates.io, 2026-06-13    | Relevant prior art for logging and Graphviz generation                       |

<!-- markdownlint-enable MD013 -->

The current default that `statelet` competes against is not mainly another
crate. It is local convention: teams write methods such as `handle_line`,
`apply_chunk`, or `step`, rely on code review to spot transition boundaries,
and hand-roll tracing or debug output when a state bug appears.

The market gap is therefore a narrow ergonomics and maintenance gap:

> An ordinary Rust enum/struct state machine is already the desired model.
> Provide just enough macro help to make the pattern uniform, observable, and
> hard to botch, without taking over user-owned control flow.

The gap disappears if `statelet` becomes a framework or DSL. Features such as a
required dispatch loop, required event enum, graph ownership, lifecycle hooks,
typestate claims, async orchestration, or hierarchical statecharts move the
crate into well-served territory.

## 4. Users and stakeholders

<!-- markdownlint-disable MD013 -->

| Type           | Description                                          | Context                                                                                                        | Cares about                                                                                | Will dislike                                                                         | Current alternative                                                               |
| -------------- | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------- |
| Primary user   | Rust maintainer of small stateful logic              | Works on parsers, protocol handlers, stream processors, CLIs, retry loops, or import pipelines                 | Explicit control flow, reviewable transitions, low dependency cost, consistent diagnostics | Framework lock-in, hidden branch logic, generated models that look unlike local code | Hand-written methods plus ad hoc tracing and review discipline                    |
| Primary user   | Library author preserving a small public API         | Maintains reusable Rust crates where a full FSM framework would be too large a commitment                      | Small surface area, opt-in macros, clear contracts, feature-gated dependencies             | Required runtime engine, macro magic that changes ownership semantics                | Local helper types, `Debug`, and documentation                                    |
| Secondary user | Code reviewer or future maintainer                   | Reads transition-heavy code after the original author has moved on                                             | Predictable naming, state/event/result fields in logs, visible fallibility contracts       | Needing to learn a new graph DSL to review ordinary branch logic                     | Manual inspection and reviewer memory                                             |
| Stakeholder    | Project sponsor or crate owner                       | Decides whether the crate earns maintenance effort                                                             | A defensible wedge, low support burden, clear non-goals                                    | A product surface that grows into a general FSM framework                            | Do nothing; keep patterns project-local                                           |
| Non-user       | Developer seeking a graph-first state-machine engine | Needs generated dispatch, statecharts, graph validation, lifecycle hooks, or formal transition tables          | Existing framework capability                                                              | A crate that refuses to own the graph                                                | `statig`, `smlang`, `rust-fsm`, `typed-fsm`, `sfsm`, `finny`, or `state-machines` |
| Non-user       | Embedded hard real-time FSM author                   | Needs `no_std`, no allocation, ISR safety, static dispatch guarantees, or microcontroller-specific constraints | Predictable generated code and platform support                                            | A library optimized for application-code maintainability                             | `typed-fsm`, `sfsm`, `sm`, `finny`, or hand-written static code                   |

<!-- markdownlint-enable MD013 -->

## 5. Job to be done

When a Rust maintainer is working on explicit stateful logic that is too
bespoke for a framework but too important for ad hoc discipline, they want to
mark transition boundaries and contracts consistently, so they can review,
instrument, and evolve the state machine without converting it into a DSL.

Functional dimension:

- Identify transition methods in ordinary Rust code.
- Standardize state, event, outcome, and error vocabulary around those methods.
- Make fallible and infallible transition contracts visible and checkable.
- Emit consistent observability fields when the project opts into tracing.

Emotional dimension:

- The maintainer should feel that the crate reduces drift without making their
  code alien.
- The reviewer should feel that the transition boundary is named and
  inspectable.

Social dimension:

- The crate should let teams say "this is our transition pattern" without
  requiring them to justify a full state-machine framework.

Competing alternatives:

- Keep the pattern local and enforce it through review.
- Adopt a full event-driven framework.
- Adopt a graph-first DSL.
- Use an embedded/static generator even when the problem is not embedded.
- Add project-specific macros that never mature into reusable tooling.

## 6. Scope

### 6.1 Goals

- Define `statelet` as a lightweight toolkit for transition boundaries in
  ordinary Rust state machines.
- Prove the conventions-only baseline before publishing the macro surface.
- Preserve user-owned state, event, output, error, storage, and dispatch
  models.
- Provide only the shared vocabulary that real examples consume; do not publish
  an outcome ontology before it has a user.
- Support opt-in transition instrumentation with predictable state, event,
  transition, result, and error fields.
- Support explicit fallible and infallible transition declarations without
  pretending to prove semantic failure behaviour.
- Keep the core surface small enough that a maintainer can understand the
  generated contract from the documentation and examples.
- Make the crate useful in parser, protocol, stream-processing, CLI workflow,
  retry, and import/export pipeline code.

### 6.2 Non-goals

- A runtime state-machine engine is out of scope; users needing one should use
  `statig`, `rust-fsm`, `smlang`, or a similar framework.
- A required dispatch loop is out of scope; projects keep their own control
  flow.
- A required event enum, action model, guard model, context model, or storage
  model is out of scope.
- Compile-time graph safety is out of scope unless later metadata can provide
  it without reshaping user code.
- Typestate APIs are out of scope; users needing type-level protocol states
  should use a typestate-oriented design or crate.
- Embedded hard real-time guarantees, ISR safety, and `no_std` leadership are
  out of scope for the initial product definition.
- GUI statecharts, workflow orchestration, business process modelling, and
  actor-like async systems are out of scope.
- Hiding branch logic is out of scope. The crate should make transitions easier
  to recognize, not harder to read.

## 7. Success criteria

### 7.1 User-facing success

- A maintainer can apply `statelet` to an existing hand-written state machine
  without introducing a generated dispatch loop or moving branch logic into a
  transition table.
- Examples show parser, protocol, stream-processing, and retry-style code where
  the state machine still looks like ordinary Rust after instrumentation.
- Reviewers can identify transition boundaries and fallibility contracts from
  the function signature, attributes, and generated documentation.
- The README clearly redirects users who need a full state-machine engine to
  stronger existing crates.
- A validation spike against `mdtablefix` demonstrates usefulness on
  transition-heavy production-shaped code. The test passes only if `statelet`
  can annotate or instrument the existing `ProcessBuffer` and continuation
  handling flow while preserving ordinary Rust control flow, avoiding a
  generated dispatch loop, and making review or diagnostics measurably clearer
  than the unannotated version.
- The `mdtablefix` spike includes a non-macro baseline using
  `#[tracing::instrument]` and local helpers. A macro passes only if it beats
  that baseline by a stated margin.
- A second non-toy validation example is named before release. ADR 001 selects
  `wireframe` connection actor and active-output transitions as the next
  proving ground after `mdtablefix`.
- If both validation examples show that `StateName`, documented field names,
  and helper functions add little over local `#[tracing::instrument]`
  annotations, the correct outcome is to ship nothing and keep the convention
  project-local.

### 7.2 Operational success

- The default dependency footprint remains small and defensible.
- Optional dependencies are feature-gated, especially tracing, diagram, serde,
  and test-helper integrations.
- Macro expansion is explainable through documentation and examples.
- The crate has compatibility tests for representative transition signatures,
  fallible/infallible contracts, and feature combinations.
- The crate regularly tests its dependency graph for circular dependencies
  across runtime, proc-macro, test-helper, example, and generated-code crates.
  The test is required even while supporting tooling such as `lading` and
  `whitaker` matures, because those tools mitigate the risk rather than remove
  it.

### 7.3 Strategic success

- The crate earns adoption as a pattern-enforcing helper rather than as another
  general FSM framework.
- The issue tracker does not become dominated by requests for owned dispatch,
  graph validation, lifecycle hooks, async orchestration, or embedded runtime
  guarantees.
- The project can say no to framework-shaped requests by pointing to explicit
  non-goals and named alternatives.

## 8. Constraints and assumptions

### 8.1 Hard constraints

- `statelet` must not require users to hand over their state-machine model.
  This is the core product boundary.
- The initial documentation must describe the crate as a transition-boundary
  toolkit, not as a state-machine framework.
- The design must treat explicit Rust branch logic as a strength for the target
  users.
- Optional observability must not make tracing a hidden runtime requirement for
  projects that do not use it.

### 8.2 Assumptions

<!-- markdownlint-disable MD013 -->

| Assumption                                                                                                                     | Consequence if false                                                                                                          |
| ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| A meaningful segment of Rust users prefers hand-written enum/struct state machines over frameworks for small application logic | The crate becomes a style preference with weak adoption pull                                                                  |
| Users will accept an attribute macro when it removes repetitive instrumentation and contract boilerplate                       | The macro surface becomes harder to justify than a conventions-only crate                                                     |
| Real expression-token attribute arguments are acceptable to users                                                              | If not, the macro may still feel too alien even without string literals                                                       |
| The narrow wedge is easier to maintain than a general framework                                                                | Feature pressure may turn the crate into the crowded product it was meant to avoid                                            |
| Existing crates do not already satisfy this exact transition-boundary pattern                                                  | `statelet` should either narrow further or not be built                                                                       |
| Tracing consistency is valuable enough to appear early                                                                         | If not, tracing should become optional sugar rather than the central thesis                                                   |
| Dependency topology can stay simple enough for regular validation                                                              | If not, circular dependencies may make the proc-macro split harder to maintain than the state-machine pattern it standardizes |

<!-- markdownlint-enable MD013 -->

### 8.3 Dependencies

- Current crates.io and docs.rs ecosystem evidence informs the market
  position.
- A later technical design must settle the actual macro shape, generated code,
  trait vocabulary, feature flags, and compile-time checks.
- A later roadmap must sequence validation around examples from real
  transition-heavy code, not only toy state machines.
- The `rstest-bdd` repository is internal prior art for proc-macro crate
  delivery and dependency-graph risk.
- `lading` and `whitaker` are developing mitigation tools for dependency and
  validation concerns, but `statelet` must still test for circular dependencies
  directly.

## 9. Open questions

<!-- markdownlint-disable MD013 -->

| Question                                                                                                | Why it matters                                                                                          | Resolution criteria                                                                                                                                                      | Owner            | Suggested path           |
| ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- | ------------------------ |
| Is the first-class product a macro crate, a conventions crate, or a small trait crate with macro sugar? | This determines API shape, dependency cost, and how users adopt the pattern                             | A design note compares at least two shapes against the user job and non-goals                                                                                            | Project owner    | Technical design         |
| Should tracing be a default feature or fully opt-in?                                                    | Observability is part of the wedge, but default dependencies affect adoption                            | A dependency and user-experience trade-off is recorded before v0.1 implementation                                                                                        | Project owner    | ADR candidate            |
| What is the minimum transition outcome vocabulary, if any?                                              | Too much vocabulary becomes a framework; unused public vocabulary becomes compatibility debt            | The design names the smallest set consumed by real examples or defers the type                                                                                           | Engineering lead | Implementation spike     |
| Can fallible/infallible declarations be useful without hard false-positive checks?                      | Alias-blind syntactic checking can reject valid code or miss aliased `Result`                           | The macro treats declarations as descriptive by default and makes strict checking opt-in                                                                                 | Engineering lead | Implementation spike     |
| What does `#[transition]` add over `#[tracing::instrument]` plus helpers?                               | This is the make-or-break value claim for the macro                                                     | A head-to-head `mdtablefix` baseline comparison states the concrete added value or defers the macro                                                                      | Engineering lead | Validation spike         |
| Should diagram or test metadata exist in v0.1?                                                          | These features risk pulling the crate towards graph ownership                                           | v0.1 either excludes them or defines metadata that does not shape user code                                                                                              | Project owner    | Roadmap decision         |
| Does `statelet` improve `mdtablefix` enough to justify extraction?                                      | `mdtablefix` is the motivating Day 2 usefulness test, not a toy example                                 | A spike applies the proposed API to `ProcessBuffer` and continuation handling, then records whether reviewability or diagnostics improved without obscuring branch logic | Project owner    | Validation spike         |
| Does `lading` publish workflow coordination validate the wedge outside parsers?                         | The crate should not overfit to `mdtablefix`                                                            | `lading` is evaluated or replaced with a better named non-toy example before macro publication                                                                           | Project owner    | Example selection        |
| How will `statelet` prevent circular dependency drift?                                                  | Prior proc-macro work in `rstest-bdd` showed that avoiding dependency cycles is a real maintenance risk | CI or local gates include a repeatable dependency-topology check covering runtime, proc-macro, test-helper, example, and generated-code crates                           | Engineering lead | Validation tooling spike |
| What project licence, maintenance policy, and MSRV should apply?                                        | Crate consumers need these commitments before adoption                                                  | The repository declares them before publishing                                                                                                                           | Project owner    | Repository setup         |

<!-- markdownlint-enable MD013 -->

## 10. Handoff

### 10.1 Candidate `docs/context.md` terms

- State machine: explicit stateful logic that changes behaviour according to a
  current state and input.
- Transition boundary: a method or function where stateful logic decides to
  stay, move, emit output, ignore input, or fail.
- Transition contract: the declared shape of a transition boundary, including
  state access, event access, outcome, and fallibility.
- Ordinary Rust state machine: a state machine expressed with project-owned
  enums, structs, methods, `match` expressions, and domain errors.
- Proc-macro crate: a Rust crate that exposes procedural macros and usually
  sits beside one or more runtime/support crates.
- Framework: a crate that owns the dispatch model, state-machine object, or
  graph semantics.
- DSL: a macro or language surface where the transition graph is declared
  outside ordinary Rust control flow.
- Maintenance safety: protection against drift in naming, instrumentation,
  fallibility, and review conventions; not a claim of formal graph safety.
- Dependency topology: the directed graph of crate dependencies that must stay
  acyclic and understandable across runtime, proc-macro, test-helper, example,
  and generated-code crates.

### 10.2 ADR candidates

- ADR: `statelet` is a transition-boundary toolkit, not a state-machine
  framework.
- ADR: Macro-first versus trait-first API shape.
- ADR: Default dependency and feature policy for tracing.
- ADR: Whether v0.1 includes diagram or test metadata.
- ADR or validation note: `mdtablefix` spike result and whether it justifies
  extracting `statelet` as an external Day 2 library.
- [ADR 001: candidate proving grounds after `mdtablefix`](adr-001-proving-ground-candidates.md).
- ADR or validation note: dependency-topology checks for proc-macro crate
  splits, informed by `rstest-bdd`.

### 10.3 Downstream readiness

This terms of reference is complete enough to begin `docs/design.md`, provided
the design treats the open questions as active gates. It is also complete
enough to draft a roadmap that sequences discovery spikes before implementation
commitments. It is not complete enough to publish a crate API without resolving
the macro-versus-trait question and the tracing feature policy.

## Appendix A. References

- crates.io keyword API for `state-machine`, reporting 154 crates on
  2026-06-13: `https://crates.io/api/v1/keywords/state-machine`
- crates.io API for `statig`, accessed 2026-06-13:
  `https://crates.io/api/v1/crates/statig`
- docs.rs documentation for `statig`, accessed 2026-06-13:
  `https://docs.rs/statig/latest/statig/`
- crates.io API for `rust-fsm`, accessed 2026-06-13:
  `https://crates.io/api/v1/crates/rust-fsm`
- docs.rs documentation for `rust-fsm`, accessed 2026-06-13:
  `https://docs.rs/rust-fsm/latest/rust_fsm/`
- crates.io API for `smlang`, accessed 2026-06-13:
  `https://crates.io/api/v1/crates/smlang`
- docs.rs documentation for `smlang`, accessed 2026-06-13:
  `https://docs.rs/smlang/latest/smlang/`
- crates.io API for `sm`, accessed 2026-06-13:
  `https://crates.io/api/v1/crates/sm`
- docs.rs documentation for `sfsm`, accessed 2026-06-13:
  `https://docs.rs/sfsm/latest/sfsm/`
- docs.rs documentation for `typed-fsm`, accessed 2026-06-13:
  `https://docs.rs/typed-fsm/latest/typed_fsm/`
- docs.rs documentation for `finny`, accessed 2026-06-13:
  `https://docs.rs/finny/latest/finny/`
- docs.rs documentation for `stateless`, accessed 2026-06-13:
  `https://docs.rs/stateless`
- crates.io API for `macro-machines`, accessed 2026-06-13:
  `https://crates.io/api/v1/crates/macro-machines`
- docs.rs source page for `macro-machines`, accessed 2026-06-13:
  `https://docs.rs/crate/macro-machines/latest/source/`
- `rstest-bdd` repository, internal prior art for Rust proc-macro crate
  delivery, accessed 2026-06-13: `https://github.com/leynos/rstest-bdd/`
- `lading` repository, developing mitigation tooling, accessed 2026-06-13:
  `https://github.com/leynos/lading`
- `whitaker` repository, developing validation tooling, accessed 2026-06-13:
  `https://github.com/leynos/whitaker`
