# Architectural decision record (ADR) 002: Scope Statelet to transition-boundary marking

## Status

Accepted, 2026-07-22. Statelet is scoped as a transition-boundary toolkit
(marker-only). The full decision is stated once, verbatim, under "Decision
outcome / proposed direction"; this summary does not restate it, so the two
cannot drift.

## Date

2026-07-22.

## Context and problem statement

Statelet enters a crowded market. On 2026-06-13, crates.io listed 154 crates
under the `state-machine` keyword,[^1] spanning hierarchical event-driven
engines,[^2] transition-table domain-specific languages (DSLs),[^3] static
embedded generators,[^4] typestate APIs[^5], and diagram or logging
helpers.[^6] A broad "another Rust state-machine framework" position is
therefore weak.

The terms of reference (§§1-6) and the technical design (§§1-3) instead stake a
narrow claim: Statelet helps teams that have *already* written an ordinary Rust
state machine — enums for modes, structs for accumulated state, methods for
transition boundaries, and `match` expressions for branch logic — by
standardizing how those boundaries are named, made fallible-explicit, and
observed. That claim only holds if Statelet refuses the responsibilities that
turn a helper into a framework.

That refusal is currently expressed as prose distributed across the terms of
reference, the design document, the README, and the user's guide. The roadmap
(item 1.1.1) requires it to be ratified once, as a single accepted decision,
because every later phase branches on it: the conventions-only baseline, the
second proving ground, and the conditional macro are each defined by what
Statelet does not own. Without one citable decision, "ship nothing", "ship
conventions only", and "ship the macro" are hard to separate, and reviewers
have no settled authority for declining framework-shaped requests.

The question this ADR answers is therefore narrow: what does Statelet own, and
what does it explicitly leave to user code and to existing crates?

## Decision drivers

- A defensible wedge requires a boundary that competing crates do not already
  occupy. Across the competing state-machine crates named in this ADR, each
  *generates* the machine; the marker-only position — instrumenting a machine
  the author has already written — is under-served (see Options considered).
- The primary user keeps explicit Rust control flow and wants it treated as a
  strength, not replaced by a generated dispatcher or transition table.
- The crate must be able to fail cleanly: if marking boundaries adds little
  over plain `#[tracing::instrument]`, the honest outcome is to ship nothing
  (the design's "Conventions baseline is also too weak" failure mode, §13.7). A
  clear scope makes that falsifiable.
- Reviewers need one authority to redirect framework-shaped feature requests
  to existing crates rather than re-litigating scope per issue.
- The decision must not pre-empt still-open questions (macro versus
  conventions, tracing defaults, second proving ground); it fixes only the
  ownership boundary.

## Validation evidence

The following reconnaissance examples test the boundary; they do not broaden it
or decide that the optional macro should ship. They use the proposed surface
from design §§6-9: an enum `StateName` derive returning stable labels,
documented `transition.*` fields, and, only if it beats the baseline, an
attribute whose `state(...)` and `event(...)` arguments are Rust expressions.
None supplies a dispatcher, event enum, or transition table.

### `mdtablefix`: conventions baseline

`mdtablefix` remains the first spike (design §12). `ContinuationMode` is an
existing enum in `src/wrap/paragraph/pending.rs`, while `ProcessBuffer` tracks
table mode as `bool in_table` in `src/process/buffer.rs`. The latter project
roadmap already proposes promoting that boolean to a small enum.
Instrumentation is currently uneven: `ProcessBuffer` emits `debug!`,
continuation handling uses `trace!`, and paths such as `handle_fence_line` are
silent.

The first phase must therefore keep the existing branches and apply a common
convention with ordinary `tracing` instrumentation:

```rust,ignore
#[derive(Debug, PartialEq, StateName)]
enum ContinuationMode {
    Normalize,
    TightCodeSpan,
    VerbatimFlush,
}

#[derive(StateName)]
enum BufferMode {
    Text,
    Table,
}

impl ProcessBuffer {
    #[tracing::instrument(
        level = "trace",
        skip(self, line),
        fields(
            transition.name = "handle_table_line",
            transition.state.before = self.mode.state_name(),
            transition.event = "source_line",
        )
    )]
    pub(super) fn handle_table_line(&mut self, line: String) -> Option<String> {
        // Existing branch logic remains here.
    }
}
```

Only demonstrated repetition or drift may justify the equivalent macro form:

```rust,ignore
#[statelet::transition(
    state(self.mode),
    event(line),
    infallible,
    tracing(level = "trace")
)]
pub(super) fn handle_table_line(&mut self, line: String) -> Option<String> {
    // Body remains byte-for-byte ordinary Rust.
}
```

The relevant `ProcessBuffer`, fence-tracker, and continuation paths do not
return `Result`. The design's requirement for a fallible transition therefore
does not apply to this spike. It remains a useful test of whether a convention
eliminates `debug!`-versus-`trace!` drift and names silent boundaries, but not
of `fallible` or `transition.error`.

### `wireframe`: primary proving ground

`wireframe` supplies the complementary case selected by
[ADR 001](adr-001-proving-ground-candidates.md). Its connection actor has the
explicit `RunState { Active, ShuttingDown, Finished }` in
`src/connection/state.rs`, generic `ActiveOutput<F, E>` in
`src/connection/output.rs`, and a fallible `ConnectionActor::dispatch_event` in
`src/connection/dispatch.rs`. The connection module has no tracing today, so
Statelet would add observability rather than duplicate an existing local
convention.

The boundary demonstrates both generic enum derivation and why the state
argument must accept an expression rather than a string: the actor state is a
function parameter, not a field on `self`.

```rust,ignore
#[derive(StateName)]
enum RunState {
    Active,
    ShuttingDown,
    Finished,
}

#[derive(StateName)]
enum ActiveOutput<F, E> {
    None,
    Response(FrameStream<F, E>),
    MultiPacket(MultiPacketContext<F>),
}

#[statelet::transition(
    state(state.run_state),
    event(event),
    fallible,
    tracing(level = "debug")
)]
fn dispatch_event(
    &mut self,
    event: Event<F, E>,
    state: &mut ActorState,
    out: &mut Vec<F>,
) -> Result<(), WireframeError<E>> {
    // The existing event match remains the dispatcher.
}
```

An error from `Event::Response` can then carry `transition.name`, the prior
state, and `transition.error` without a Statelet-shaped domain type.
`ActiveOutput::shutdown`, which replaces the active output with `None`, is the
matching infallible boundary. The distinct benefit is label alignment:
`crates/wireframe-verification` retains its Stateright model and proof
responsibilities, while `StateName` can give production logs, tests, and the
model the same `Active`, `ShuttingDown`, and `Finished` labels.

### `ddlint`: negative control

`ddlint` confirms why [ADR 001](adr-001-proving-ground-candidates.md) treats it
as the weakest fit. Its `StructLiteralState` in
`src/parser/expression/pratt.rs` is two `usize` counters, `active` and
`suspension`, manipulated by activation and suspension wrappers around closures.
`allows_struct_literals()` derives a boolean from those counters; there is no
state enum to name, and parser warnings use `log`, not `tracing`.

Forcing Statelet into this seam would first require a synthetic projection:

```rust,ignore
#[derive(StateName)]
enum GuardMode {
    Inactive,
    Active,
    Suspended,
}

impl StructLiteralState {
    fn mode(&self) -> GuardMode {
        // Derived solely from `active` and `suspension`.
    }
}
```

That would be decorative parser scaffolding. The projection would exist only to
satisfy a marker; its before-state describes a scoped region rather than a
boundary decision, and the important invariant remains counterbalance and
underflow prevention. `transition.*` fields do not express that invariant.
Statelet should therefore not be introduced unless `ddlint` promotes this to a
real mode representation and moves the relevant diagnostics to `tracing`.

Together, the examples bracket the validation bet: `mdtablefix` tests whether
conventions beat ad hoc instrumentation but cannot exercise fallibility;
`wireframe` tests a fallible parameter-held state expression, generic derives,
and verification-name alignment; and `ddlint` is the negative control where the
correct outcome is "do not use Statelet". `TransitionOutcome` remains
unpublished (design §6.2), so any initial `transition.outcome` field must use
the project-local decision or return shape already present at the boundary.

## Options considered

<!-- markdownlint-disable MD013 -->

| Option                                  | What Statelet would own                                               | Closest existing crates              | Verdict           |
| --------------------------------------- | --------------------------------------------------------------------- | ------------------------------------ | ----------------- |
| 0. Ship nothing / keep convention local | Nothing; redirect users to `#[tracing::instrument]` plus helpers      | n/a (the status quo)                 | Deferred fallback |
| A. Transition-boundary marker (chosen)  | Naming and observability conventions at boundaries that already exist | No crate at the marker-only position | Selected          |
| B. Transition-table or graph owner      | The state set and legal moves, generated from a macro or DSL          | `stateless`, `smlang`                | Rejected          |
| C. Dispatch and event-engine owner      | An event-accepting runtime and dispatch loop                          | `statig`, `rust-fsm`, `finny`        | Rejected          |
| D. Diagram and graph-metadata owner     | A transition graph rendered to diagrams or validated for safety       | `macro-machines`; graph-first crates | Rejected          |

<!-- markdownlint-enable MD013 -->

*Table 1: Scope postures considered for Statelet and the crates that already
serve each posture.*

### Option 0: Ship nothing / keep the convention project-local

Statelet need not exist as a crate at all. If a stable state-naming contract
and documented tracing fields add little over plain
`#[tracing::instrument(fields(state = %self.mode))]`, the honest outcome is to
ship nothing and keep the pattern project-local. The design preserves this
outcome deliberately (its "Conventions baseline is also too weak" failure mode,
§13.7). This option is not selected *now* — it is the fallback the validation
phases may reach — but it is recorded here because the scope decision and the
ship/no-ship decision are distinct: choosing the marker-only scope does not
commit the project to publishing anything.

### Option A: Transition-boundary marker (chosen)

Statelet owns a small vocabulary and a set of conventions for *marking* the
points where an existing machine transitions: a stable state-naming contract,
documented `transition.*` tracing fields, and local helper guidance. The
machine's states, events, storage, error types, dispatch, and control flow stay
in user code. This marker-only position is under-served rather than crowded: of
the surveyed crates, every one generates the state machine, and the nearest
neighbours still do so even where they overlap Statelet's framing — `stateless`
shares the "separate structure from behaviour" wording but generates the table,
and `macro-machines` adds logging but generates the machine. Statelet instead
instruments one the author has already written.

This option fixes *scope*, not *delivery vehicle*. Whether the marker surface
ships as conventions only, as a trait crate, or eventually as an attribute
macro is explicitly **not** decided here; that fork is owned by the validation
phases and the design's deferred decisions (§14). The macro is added only if it
later beats the non-macro baseline (the "Macro is only `instrument` with extra
steps" gate, §13.6).

### Option B: Transition-table or graph owner

Statelet could generate the transition table — the states and the legal moves —
as `stateless` does with its zero-cost macro that separates structure from
behaviour. This is the closest dangerous edge: `stateless` already shares the
"separate structure from behaviour" framing, yet it still owns the generated
enums and lookup function. Taking this on would put Statelet in direct
competition with a mature crate and would move branch logic out of the user's
`match` expressions, contradicting the product thesis. Rejected.

### Option C: Dispatch and event-engine owner

Statelet could provide an event-accepting runtime and dispatch loop, as
`statig` and `rust-fsm` do. This is the well-served centre of the market and
would require users to hand over their control flow — the one thing the target
user most wants to keep. Rejected.

### Option D: Diagram and graph-metadata owner

Statelet could record a transition graph and render diagrams or assert graph
safety, as `macro-machines` does for Graphviz output. Diagram and safety
metadata pull the crate toward graph ownership and toward competing with
graph-first and typestate crates. v0.1 records no graph metadata and generates
no diagrams (the design's "Defer diagrams and transition-table metadata"
decision, §3.5). Rejected for now; any future metadata must not reshape user
code.

## Decision outcome / proposed direction

Statelet is scoped as a transition-boundary toolkit. **Statelet marks
boundaries and does not own dispatch, events, storage, transition tables, or
graph safety.** Its initial surface is expected to be state naming and
documented `tracing` fields around boundaries that already exist in ordinary
Rust code, but the delivery vehicle (conventions, trait crate, or macro) and
the tracing-default question remain open and are decided elsewhere; this ADR
fixes only the ownership boundary.

In the context of entering a crowded state-machine market, facing the risk that
Statelet drifts into a framework and competes with mature crates, the project
decides for a marker-only scope, and against owning transition tables (as
`stateless` does), dispatch and events (as `statig` and `rust-fsm` do),
storage, or graph safety, to keep the user's explicit Rust control flow as the
model and keep the wedge falsifiable, accepting that the resulting product is
smaller than a framework and may, on validation, prove too thin to ship at all.

## Goals and non-goals

- Goals:
  - Record one accepted boundary that later roadmap phases can cite.
  - Keep user-owned state, event, output, error, storage, and dispatch models
    in user code.
  - Position Statelet as complementary to, not competitive with, existing
    state-machine crates, with explicit redirections.
  - Preserve the ability to ship nothing if the boundary-marking surface proves
    too weak (the design's "Macro is only `instrument` with extra steps" and
    "Conventions baseline is also too weak" failure modes, §§13.6-13.7).
- Non-goals:
  - Own a generated dispatcher, a required event enum, a guard or action DSL, a
    transition table, or a state-machine object.
  - Provide compile-time graph safety or diagram generation in v0.1.
  - Provide a typestate API or claim `no_std`, interrupt-safety, or embedded
    suitability.
  - Hide branch logic; transitions should be easier to recognize, not harder to
    read.
  - Decide the still-open questions of macro-versus-conventions, the tracing
    default, or the second proving ground. Those are tracked separately.

## Known risks and limitations

- A marker-only scope may add too little over plain
  `#[tracing::instrument(fields(...))]`. This is an accepted, designed-for
  outcome: the validation phases may conclude "ship nothing" (design §13.7).
- The boundary against `stateless` is narrow. Reviewers must watch for feature
  requests that quietly reintroduce table or graph ownership.
- Observability polish can be mistaken for product validation. A clear scope
  reduces, but does not remove, that risk; the baseline comparison in design
  §11.2 remains the real gate.
- This ADR fixes scope only. It does not justify the macro, the second proving
  ground, or any dependency-default decision.

## Architectural rationale

This decision preserves Statelet's core boundary: ordinary Rust owns the state
machine, while Statelet standardizes names and observability at selected
transition boundaries. It aligns with the terms of reference (§§1-6, §6.2), the
design context and non-goals (§§1-3, §2.2), the README, and the user's guide,
consolidating their shared position into one citable record. It complements
[ADR 001](adr-001-proving-ground-candidates.md), which selects `wireframe` as
the proving ground that will test this boundary on production-shaped code
without Statelet taking ownership of `wireframe`'s Stateright model.

[^1]: crates.io keyword API for `state-machine`, reporting 154 crates on
    2026-06-13: `https://crates.io/api/v1/keywords/state-machine`
[^2]: docs.rs documentation for `statig`, accessed 2026-06-13:
    `https://docs.rs/statig/latest/statig/`
[^3]: docs.rs documentation for `smlang`, accessed 2026-06-13:
    `https://docs.rs/smlang/latest/smlang/`
[^4]: crates.io API for `sm`, accessed 2026-06-13:
    `https://crates.io/api/v1/crates/sm`
[^5]: docs.rs documentation for `sfsm` and `typed-fsm`, accessed 2026-06-13:
    `https://docs.rs/sfsm/latest/sfsm/`;
    `https://docs.rs/typed-fsm/latest/typed_fsm/`
[^6]: docs.rs source page for `macro-machines`, accessed 2026-06-13:
    `https://docs.rs/crate/macro-machines/latest/source/`; crates.io API for
    `macro-machines`, accessed 2026-06-13:
    `https://crates.io/api/v1/crates/macro-machines`
