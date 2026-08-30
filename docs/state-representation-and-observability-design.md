# State representation and observability design

Status: Proposed v0.1 design refinement

Scope: Downstream state representation, naming, transition fields, and macro
validation for the `mdtablefix` and `wireframe` proving grounds.

Audience: Maintainers reviewing or implementing Statelet's runtime conventions
and optional procedural macro.

Governing and companion documents:

- [ADR 002](adr-002-transition-boundary-scope.md) remains the authority for
  Statelet's marker-only ownership boundary.
- [ADR 001](adr-001-proving-ground-candidates.md) remains the authority for the
  proving-ground selection.
- [ADR 004](adr-004-representation-and-observability-contracts.md) proposes the
  refinements explained by this document.
- [Technical design](design.md) remains the primary design for crate topology,
  feature policy, and the existing macro gate.
- [Focused roadmap](state-representation-and-observability-roadmap.md) sequences
  the validation work introduced here.

## 1. Design goal

Statelet should improve the maintenance safety of an ordinary Rust state
machine without becoming the owner of that machine. The intended layering is:

```plaintext
project-owned ADTs, newtypes, or typestate
                    ↓
ordinary Rust transition methods and matches
                    ↓
Statelet names, contracts, and observability
```

The first layer prevents invalid states or invalid operation order. The second
contains domain predicates, mutation, side effects, and dispatch. Statelet
operates only at the third layer.

This prevents two opposite errors: treating visibility over a weak
representation as correctness, and making Statelet generate a state model.
Typestate and algebraic data types (ADTs) can improve faultlessness and
testability where several invariants interact, but can add boilerplate and
harm readability in simple linear flows. Newtypes combined with "Parse, don't
validate" generally have a lower adoption cost.[^1] Statelet should remain
compatible with these patterns without requiring them everywhere.

## 2. Representation comes before instrumentation

### 2.1 Honest state models

A Statelet boundary should refer to state that already matters to the
application. Suitable representations include:

- an enum whose variants own state-specific data;
- a struct or newtype whose constructors establish an invariant;
- a project-owned typestate wrapper; or
- a computed state expression that reflects an existing domain model.

An enum invented only to translate correlated counters, booleans, or optional
fields into something Statelet can name is not sufficient. Each proving-ground
note must answer two questions separately:

1. Is the downstream representation honest without Statelet?
2. Does Statelet improve naming, diagnostics, or reviewability around it?

A negative first answer returns the work to the downstream repository. A
negative second answer preserves Statelet's ship-nothing exit.

### 2.2 Payload-carrying variants

The `mdtablefix` example should put state-specific data inside the variant that
owns it. This shape is illustrative; `mdtablefix` retains authority over final
names and fields:

```rust,no_run
enum TableRun {
    Streaming,
    Buffering(Vec<String>),
}

struct ProcessBuffer {
    out: Vec<String>,
    table: TableRun,
    ellipsis: bool,
}
```

Statelet sees stable `Streaming` and `Buffering` labels without inspecting or
recording buffered lines. A separate mode enum and table buffer should not
become the canonical example unless both combinations are valid.

### 2.3 Lifecycle and typestate remain downstream

A consuming operation can make required finalization atomic:

```rust,no_run
impl ProcessBuffer {
    fn finish(mut self) -> Vec<String> {
        self.flush();
        self.out
    }
}
```

Statelet may instrument `finish` when useful, but it cannot make a separate
`flush(); into_output()` sequence correct. Constructors, consuming methods,
ADTs, and typestate remain the correctness mechanisms.

The "no typestate API" non-goal does not prohibit instrumentation of a
project-owned transition such as `Server<Bound>::run(self) -> Server<Running>`.
Statelet may name the boundary, but does not generate marker states or legal
moves.

## 3. State naming contract

The runtime candidate remains intentionally small:

```rust,no_run
pub trait StateName {
    fn state_name(&self) -> &'static str;
}
```

A state name is a stable operational label, not `Debug` output or a
serialization format. Renaming it may break dashboards, tests, and log queries.

The derive must support ordinary enum shapes without inspecting payloads.

<!-- markdownlint-disable MD013 -->

| Enum shape | Required behaviour | Additional payload bounds |
| --- | --- | --- |
| Unit variant | Return the stable variant name | None |
| Tuple variant | Ignore every field and return the variant name | None |
| Struct variant | Ignore every named field and return the variant name | None |
| Generic payload | Preserve declared generics and where clauses | None introduced by Statelet |
| Explicit rename | Deferred until a proving ground demonstrates the need | Not applicable |

<!-- markdownlint-enable MD013 -->

_Table 1: Required `StateName` derive behaviour._

For example, `ActiveOutput<F, E>` must not require `F` or `E` to implement
`Debug`, `Display`, `Clone`, or `StateName`. The generated match should ignore
payloads and return stable names for `None`, `Response`, and `MultiPacket`.

The exact default case convention remains open. Stability and absence of
payload inspection are the binding requirements.

## 4. Transition observability contract

### 4.1 Field vocabulary

The baseline and optional macro should use one public operational vocabulary.
Fields may be absent when the boundary cannot expose them honestly.

<!-- markdownlint-disable MD013 -->

| Field | Initial status | Meaning |
| --- | --- | --- |
| `transition.name` | Baseline | Stable name of the marked boundary |
| `transition.state.before` | Baseline when state is declared | State observed before the body runs |
| `transition.state.after` | Proposed optional field | State observed after a completed body when explicitly supportable |
| `transition.event` | Baseline | Stable semantic class of the handled input |
| `transition.outcome` | Optional | Stable project-owned outcome class |
| `transition.error` | Optional | Error context available to tracing, not a metric label |

<!-- markdownlint-enable MD013 -->

_Table 2: Proposed transition field vocabulary._

Field names and meanings are semver-relevant operational API. Absence means
"not captured by this boundary", not an empty or guessed value.

### 4.2 Before-state and after-state

An after-state is useful when a runtime machine mutates:

```plaintext
transition.state.before = "Tracking"
transition.state.after  = "Complete"
transition.event        = "LastFrame"
transition.outcome      = "MessageAssembled"
```

Outcome and destination are not interchangeable. Several outcomes can leave a
machine in one state, and one outcome can be reached from several states.

The macro must not infer an after-state universally. Consuming `self`, moving
the state, errors after mutation, and early returns can make the original
expression unavailable. Candidate mechanisms are:

- an explicit `state_after(...)` expression;
- documented re-evaluation of `state(...)` for supported mutable boundaries;
- an adapter over a project-owned return value; or
- conventions-only support while macro support remains deferred.

The selected mechanism must document evaluation count, success and error
semantics, borrow behaviour, and unsupported signatures.

### 4.3 Semantic event and outcome labels

`event(...)` should name a stable operational class such as `line_kind`,
`frame_kind`, or `ConnectionEvent::PeerClosed`. It should not capture a raw
Markdown line, protocol frame, request identifier, or user payload by default.
Raw capture would require a separate opt-in with redaction and size semantics.

Statelet should also preserve project-owned outcomes. A domain enum such as
`TableTransition` or `AssemblyOutcome` can carry more useful language than
`Stay`, `Move`, or `Emit`. A local helper may map variants to stable labels. A
shared outcome-name trait should ship only when both proving grounds consume it
without reshaping their domain models.

Expected domain outcomes belong in project enums where exhaustive matching is
useful. Technical failures remain `Result` errors. Tracing may record bounded
diagnostic values, but metrics must use stable low-cardinality classes and
never raw errors, paths with parameters, identifiers, lines, or payloads.

## 5. Macro contract refinements

### 5.1 Expression and return semantics

Every expression argument must state whether it is evaluated zero, one, or more
times. Generated code should borrow for naming where possible and must not
require domain state to be `Copy`. Examples should discourage side effects in
`state(...)`, `state_after(...)`, and `event(...)`.

The wrapper must preserve `?`, explicit `return`, fall-through expressions, and
project error types. If after-state is enabled, tests must cover state changes,
stays, errors before and after mutation, and early returns. The field contract
must state which completed paths emit after-state and which omit it.

### 5.2 Asynchronous boundaries

The conventions-only baseline can apply to async functions immediately. A
public macro must either pass a `wireframe`-shaped async matrix or state that
`async fn` is unsupported for that release.

The matrix must include `&mut self`, a visible `Result`, `?`, an early return,
at least one `.await`, and state used on both sides of the await. Generated
instrumentation must not hold a `tracing::Span::enter` guard across `.await`.
Cancellation does not complete the body and cannot report an ordinary
completed after-state.

## 6. Proving-ground application

### 6.1 `mdtablefix`

The first slice should:

1. establish or confirm payload-carrying ADTs;
2. encapsulate mandatory finalization;
3. define semantic line-kind and outcome labels locally;
4. apply `StateName` and ordinary `#[tracing::instrument(fields(...))]`;
5. compare before-state with honestly observable after-state; and
6. consider a macro only if repeated boilerplate or field drift remains.

The spike should not combine a large downstream refactor and macro adoption in
one diff. Representation correctness and instrumentation value must remain
independently reviewable.

### 6.2 `wireframe`

The first Wireframe slice remains the connection actor and `ActiveOutput`. It
should test a nested or parameter-held state expression, generic payload enum
derivation, project-owned errors, verification-name alignment, and async
conventions without granting the macro an async claim.

`MessageSeries` is a useful secondary seam only after Wireframe independently
decides that an ADT refactor is the honest representation. Statelet must not
introduce a computed enum solely for tracing. Stateright, Kani, Verus, and
project tests retain graph and invariant proof responsibilities.

### 6.3 Negative controls

The correct result is "do not use Statelet" when a state enum would exist only
for Statelet, the important invariant is counter balance or ownership, the code
is primarily a token table or build graph, or local tracing is clearer.

## 7. Publication evidence

<!-- markdownlint-disable MD013 -->

| Surface | Required evidence before publication |
| --- | --- |
| `StateName` derive | Unit, tuple, struct, and generic enum pass tests with no new payload bounds |
| Field conventions | `mdtablefix` and `wireframe` use the same semantic vocabulary |
| `transition.state.after` | Explicit success, error, move, and unsupported-signature semantics |
| Event labels | No raw payload or unbounded identifier is captured by default |
| Domain outcomes | Proving grounds keep their own enums or justify a shared naming trait |
| Synchronous macro | Existing diagnostic, cost, feature, and topology gates plus these refinements |
| Asynchronous macro | Wireframe-shaped async tests, or an explicit unsupported statement |

<!-- markdownlint-enable MD013 -->

_Table 3: Evidence required for each refined public surface._

Deferred extensions remain: raw payload capture, automatic destination
extraction, typestate generation, graph metadata, graph validation, and a
universal transition outcome enum.

The largest design risk is after-state capture. A conventions-only release may
remain the most honest product even if the field vocabulary is accepted.

[^1]: Leon Heuer, Falk Woldmann Lu, and Jan Haase, "Functional State Machines in
    Rust: Typestate and Newtype Patterns (Experience Report)", FUNARCH '26,
    2026, <https://doi.org/10.1145/3830438.3830958>.
