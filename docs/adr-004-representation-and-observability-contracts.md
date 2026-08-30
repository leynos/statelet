# Architectural decision record (ADR) 004: Refine representation and observability contracts

## Status

Proposed.

## Date

2026-08-30.

## Context and problem statement

[ADR 002](adr-002-transition-boundary-scope.md) establishes Statelet as a
marker-only toolkit: user code owns state, events, storage, dispatch,
transition tables, and graph safety.
[ADR 001](adr-001-proving-ground-candidates.md) selects `mdtablefix` and
`wireframe` as the first two proving grounds for that claim.

A review of the proposed `mdtablefix` algebraic data type (ADT) refactors and
`wireframe` protocol state machines exposes several contracts that the current
design leaves implicit:

- whether Statelet should accept a weak representation merely because it can be
  named and traced;
- whether `StateName` supports payload-carrying and generic enum variants
  without imposing bounds on their payloads;
- whether transition records need both the state before a boundary and the
  state after it;
- whether `event(...)` names a stable semantic class or captures the raw input;
- whether a crate-level outcome ontology may displace project-owned domain
  outcomes; and
- what evidence is required before the optional macro claims support for
  asynchronous transition methods.

The distinction matters because type-driven refactoring and transition
instrumentation solve different problems. Typestate and ADTs can make invalid
states or operation orders unrepresentable, while Statelet can make an already
honest transition boundary recognizable and observable. The experience report
that prompted this review reaches a similar qualified conclusion: typestate can
improve faultlessness and testability but adds boilerplate, and is most useful
where branching logic interacts with several invariants; newtypes combined with
"Parse, don't validate" offer a lower-cost way to protect domain boundaries.[^1]

This ADR asks how Statelet should preserve that division of responsibility
while tightening the naming, observability, and validation contracts used by
its proving grounds.

## Decision drivers

- Statelet must not legitimize correlated booleans, optional fields, or
  decorative state projections merely because they satisfy an instrumentation
  API.
- Payload-carrying variants such as `Buffering(Vec<String>)` and generic
  variants such as `ActiveOutput<F, E>` are ordinary Rust state models and must
  not acquire unrelated trait bounds.
- Operational fields should use stable, low-cardinality semantic labels rather
  than raw lines, frames, request identifiers, paths, or payloads.
- A transition outcome and a destination state are separate concepts and may
  both be useful in diagnostics.
- Project-owned outcome enums preserve domain language and exhaustive matching;
  Statelet should not flatten them into a generic ontology without evidence.
- `wireframe` is an asynchronous proving ground, so macro applicability cannot
  be inferred from synchronous examples alone.
- The existing "ship nothing", "ship conventions only", and "ship macro"
  exits must remain available.

## Requirements

### Functional requirements

- Statelet ornaments project-owned ADTs, newtypes, and typestate transitions; it
  does not define or repair those representations.
- Validation must reject a synthetic state projection introduced only to make a
  boundary instrumentable.
- `StateName` must name unit, tuple, and struct enum variants without inspecting
  their payloads.
- Transition observability must distinguish the state before a boundary from an
  optional state after the boundary.
- `event(...)` must represent a semantic event class. Raw input capture is a
  separate concern and is not enabled implicitly.
- Domain-specific outcome enums and existing `Result` error types remain
  first-class.
- Macro support for `async fn` must be tested explicitly or documented as
  unsupported for the release that omits those tests.

### Technical requirements

- A derived `StateName` implementation for a generic enum must add no `Debug`,
  `Display`, `Clone`, `StateName`, or `'static` bounds to payload type
  parameters unless the enum declaration already requires them.
- Generated match arms must ignore payloads and return stable static names.
- The macro must document how many times each configured expression is
  evaluated and must not encourage expressions with side effects.
- `transition.state.after` must be absent when Statelet cannot observe it
  honestly; it must not be populated by guessing from an outcome name.
- Raw errors may be recorded in tracing fields, but error strings, payloads,
  request identifiers, and other unbounded values must not become metric
  labels.
- Async instrumentation must not hold a `tracing::Span::enter` guard across an
  `.await` point.

## Options considered

### Option A: Keep the current design unchanged

The existing marker-only scope already prevents Statelet from owning the state
machine. However, its examples can still imply that replacing a boolean with a
separate mode enum is sufficient even when state-specific data remains stored
beside it. The current field contract also records only the before-state and
uses examples such as `event(line)`, which can be read as encouragement to log
raw input.

This option is too ambiguous for the two proving grounds.

### Option B: Let Statelet supply stronger state and outcome types

Statelet could provide payload containers, typestate marker scaffolding, a
required event enum, and a universal transition outcome. That would make the
instrumentation contract easy to implement, but it would cross ADR 002's
ownership boundary and compete with state-machine frameworks and project domain
models.

This option is rejected.

### Option C: Tighten the marker contract around honest project-owned models

Statelet can retain its narrow scope while making the prerequisites and
observability semantics explicit. The proving grounds first establish honest
project-owned representations, then compare local tracing conventions with the
runtime vocabulary, and only then evaluate a macro.

This option is recommended.

### Option D: Keep every refinement project-local

`mdtablefix` and `wireframe` could each define their own labels and tracing
fields. This remains the correct fallback if the shared contracts do not reduce
drift or improve reviewability in both repositories. It is preserved by the
existing ship-nothing exit.

## Decision outcome / proposed direction

Statelet should standardize transition-boundary naming and observability only
after the downstream project has chosen an honest representation. **Statelet
may ornament a project-owned ADT, newtype, or typestate transition, but must not
supply a synthetic state model merely to make weakly represented code
instrumentable.**

The recommended contract has these refinements:

1. `StateName` covers unit, tuple, and struct variants, including generic
   payload-carrying enums, without new payload bounds.
2. `transition.state.before` remains the baseline state field, while
   `transition.state.after` becomes an optional operational field whose capture
   semantics must be decided before the macro ships.
3. `event(...)` denotes a stable semantic class such as `line_kind` or
   `frame_kind`; raw payload capture is separate and absent by default.
4. Statelet accepts project-owned outcome and error models. A shared
   `TransitionOutcome` remains deferred until real consumers prove that it adds
   value without flattening domain language.
5. The conventions-only validation may apply to async code immediately, but a
   public macro either passes a `wireframe`-shaped async test matrix or states
   that `async fn` is unsupported.
6. Consuming lifecycle operations such as `finish(self)` remain downstream
   encapsulation mechanisms. Statelet may observe them when diagnostically
   useful, but does not make multi-call ceremony atomic.

## Goals and non-goals

- Goals:
  - Keep the Statelet layer below project state modelling and above ordinary
    transition methods.
  - Make the `mdtablefix` example demonstrate state-specific payload ownership,
    not merely an enum-shaped boolean.
  - Make event and outcome labels safe for operational aggregation.
  - Establish honest after-state and async publication gates before macro work.
  - Preserve the selected proving grounds and all existing release exits.
- Non-goals:
  - Generate typestate wrappers, transition tables, dispatchers, event enums, or
    project outcome enums.
  - Require every stateful helper or consuming method to be instrumented.
  - Infer an after-state for consuming typestate transitions or from arbitrary
    return values in v0.1.
  - Capture raw protocol frames, Markdown lines, request identifiers, or user
    payloads by default.
  - Require `wireframe` to refactor a representation solely for Statelet's
    benefit.

## Migration plan

1. Add a focused companion design and roadmap that make these proposed
   contracts reviewable without changing runtime code.
2. In the `mdtablefix` spike, establish or confirm the payload-carrying ADT and
   lifecycle encapsulation before applying Statelet conventions.
3. Apply the conventions baseline to `wireframe`'s existing `RunState` and
   `ActiveOutput` seams. Use `MessageSeries` only after `wireframe`
   independently adopts an honest ADT representation for its correlated state.
4. Record the observed need for state-after, event labels, outcome labels, and
   identifier shape in both validation notes.
5. Before macro publication, settle after-state syntax and evaluation semantics,
   then exercise synchronous and asynchronous success, error, early-return,
   cancellation, and borrowing cases.
6. If this ADR is accepted, integrate its settled requirements into the primary
   technical design and roadmap; if it is rejected, retain the existing design
   and document the reason.

## Known risks and limitations

- Requiring an honest representation first can delay the instrumentation spike
  or expose downstream refactoring work that belongs in another repository.
- `transition.state.after` is difficult for consuming methods, moved values,
  error paths that mutate state, and expressions that cannot be evaluated after
  the body.
- Semantic event and outcome labels add small project-local helpers until a
  reusable naming contract proves itself.
- A richer observability contract can make the macro less attractive if plain
  `#[tracing::instrument]` remains clearer.
- The `mdtablefix` and `wireframe` examples may not generalize beyond parser,
  stream-processing, actor, and protocol code.

## Outstanding decisions

- Whether after-state uses an explicit `state_after(...)` expression, a safe
  re-evaluation of `state(...)`, an outcome adapter, or no macro support in
  v0.1.
- Whether after-state is recorded on every completed call or only successful
  calls when a visible `Result` permits that distinction.
- Whether semantic event and outcome names use new traits, closures, local
  helpers, or only documented conventions in the first release.
- Whether `StateName` needs a stable identifier in addition to
  `&'static str` after the two proving grounds report their metrics needs.
- Whether asynchronous macro support belongs in v0.1 or is explicitly deferred.

## Architectural rationale

The proposal preserves ADR 002's marker-only boundary while making it harder to
misread marker support as state-model ownership. It also strengthens ADR 001's
validation logic: `mdtablefix` tests a payload-carrying parser state, while
`wireframe` tests generic protocol states, fallibility, and async boundaries.
The shared layer remains naming and observability; invalid-state prevention,
lifecycle correctness, domain outcomes, and proof obligations remain with the
project that owns the machine.

[^1]: Leon Heuer, Falk Woldmann Lu, and Jan Haase, "Functional State Machines in
    Rust: Typestate and Newtype Patterns (Experience Report)", FUNARCH '26,
    2026, <https://doi.org/10.1145/3830438.3830958>.
