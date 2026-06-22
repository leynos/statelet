# Architectural decision record (ADR) 001: Select proving ground candidates

## Status

Proposed.

## Date

2026-06-15.

## Context and problem statement

Statelet's first validation target remains `mdtablefix`. The project still
needs a second, non-toy proving ground before it publishes macro behaviour or
claims that the runtime conventions generalize beyond Markdown table repair.

The current design constrains that choice. Statelet is a transition-boundary
toolkit for ordinary Rust code. It must not own dispatch, storage, event
modelling, lifecycle orchestration, graph validation, or control flow. Its
first useful surface is stable naming and observability around transitions:
`StateName`, documented `transition.*` fields, local helpers, and ordinary
`#[tracing::instrument(fields(...))]`.

The question is which downstream repository should validate that surface after
`mdtablefix`, without encouraging Statelet to become a full state-machine
framework.

## Decision drivers

- Prefer existing hand-written state machines where Statelet can ornament a
  transition boundary without replacing local control flow.
- Prefer repositories that already use, or can plausibly benefit from,
  `tracing`-oriented transition diagnostics.
- Prefer seams that expose state, event, outcome, and error context without
  forcing a generated dispatcher or event model.
- Avoid proving grounds where the interesting behaviour is a build graph,
  parser tree, token classification table, or semantic collection pass rather
  than a transition boundary.
- Treat the non-macro conventions baseline as the acceptance test. A macro must
  later beat that baseline by a stated margin.

## Options considered

<!-- markdownlint-disable MD013 -->

| Repository  | Fit            | Where Statelet would help                                                                        | Where Statelet should not intrude                                                                         |
| ----------- | -------------- | ------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------- |
| `wireframe` | High           | Connection lifecycle, output-source transitions, protocol and event tracing, verification names  | Do not replace Stateright, routing, middleware, or protocol modelling                                     |
| `weaver`    | High, targeted | Language Server Protocol (LSP) process lifecycle, daemon health states, apply-patch parser modes | Do not model the whole daemon or command-line interface (CLI) as one machine                              |
| `netsuke`   | Medium         | Pipeline and stage observability, manifest load phases, progress-state transitions               | Do not treat the build directed acyclic graph (DAG) or Ninja graph as a Statelet concern                  |
| `ddlint`    | Low to medium  | Parser mode and guard tracing if those states become more troublesome                            | Do not wrap tokenization, general abstract syntax tree (AST) traversal, or semantic collection by default |

<!-- markdownlint-enable MD013 -->

_Table 1: Candidate proving grounds after `mdtablefix`._

### Option A: `wireframe`

`wireframe` is the strongest application. Its connection actor already has the
kind of hand-written state machine Statelet is intended to make more legible.
`ActorState` wraps `RunState` variants such as active, shutting down, and
finished, and the actor loop polls typed events before dispatching them through
explicit matches.

The strongest seam is `ConnectionActor::dispatch_event`, with supporting
instrumentation around queue processing, shutdown handling, empty-queue
handling, response assignment, and multi-packet correlation. `ActiveOutput`
also encodes a compact state machine for no active output, a response, and a
multi-packet stream. A `StateName` implementation would provide stable
operational labels for those variants without changing their semantics.

`wireframe` also has a verification crate with production-adjacent state names,
action variants, and transition functions that feed a Stateright model.
Statelet should not replace that model. Formal graph validation and transition
ownership remain out of scope. The useful validation question is whether
Statelet can align production actor names, test names, logs, and verification
model names while leaving the model in charge of proof obligations.

### Option B: `weaver`

`weaver` is also strong, but the fit is localized. Its LSP adapter has an
explicit lifecycle with not-started, running, and stopped states. The process
language server stores that lifecycle behind a mutex, moves into the running
state after spawning, rejects transport operations when it is not running, and
replaces running state with stopped on shutdown and drop.

That lifecycle is a clean Statelet seam. Stable names and transition tracing
around initialization, running-state installation, shutdown, and drop could
make existing `tracing` output easier to query. The project should not model
the whole daemon or CLI as a single state machine.

The apply-patch parser is a secondary pocket. It has operation modes, a mode
transition detector, line classification, and parser sub-state represented by
optional fields. That area may benefit from naming if parser diagnostics become
hard to reason about, but a conventions-only spike should come before any
macro. Otherwise Statelet risks becoming decorative parser scaffolding.

### Option C: `netsuke`

`netsuke` is a medium fit. It has named manifest-load and pipeline stages, and
the runner already reports those stages while generating and executing the
Ninja manifest. Statelet could standardize `StateName` labels and
`transition.*` fields around phase changes, especially because `netsuke`
already uses `tracing`, `metrics`, and progress reporting.

The caveat is that the pipeline is mostly linear. Existing stage enums and
progress reporting already express much of the state shape. Statelet would
offer observability polish rather than architectural validation. It should be
considered only if repeated stage-transition instrumentation becomes noisy
across CLI, manifest loading, and task progress.

### Option D: `ddlint`

`ddlint` is the weakest fit. It sits near Statelet's motivating parser and
linter domain, but most observed seams do not need Statelet. Tokenization is a
`logos` classification table. The semantic model builder is a sequence of
declaration, traversal, and collection passes over the AST. Those are not
transition-boundary problems.

The plausible exception is parser mode state, such as struct-literal guard
state with activation and suspension counters. That could become a useful
Statelet spike if parser ambiguities grow into a maintenance problem. Today,
`ddlint` also documents `log` for parser warnings and depends on `log`, while
Statelet's planned product pressure is `tracing`-oriented. It should not be a
near-term proving ground unless parser diagnostics move towards `tracing` or
the parser gains more explicit modes.

## Decision outcome / proposed direction

Select `wireframe` as the primary proving ground after `mdtablefix`. Keep
`weaver` as the second candidate if `wireframe` is unavailable or if a second
post-`mdtablefix` validation pass is needed. Let `netsuke` borrow the
naming/tracing convention only where stage-transition logging becomes
repetitive. Do not pull Statelet into `ddlint` unless parser modes become an
explicit maintenance problem.

In the context of validating Statelet after `mdtablefix`, facing the risk that
the crate overfits one parser-shaped example or drifts into framework
territory, the project decides for `wireframe` as the next proving ground, and
against broad adoption across `weaver`, `netsuke`, and `ddlint`, to validate
transition-boundary naming and observability on production-shaped code,
accepting that the first follow-up spike is narrower than a multi-repository
rollout.

## Goals and non-goals

- Goals:
  - Validate Statelet's conventions on `wireframe` connection actor and
    active-output transitions after the `mdtablefix` baseline.
  - Preserve explicit Rust control flow, queue handling, domain invariants, and
    verification ownership in downstream code.
  - Compare the conventions baseline with plain `#[tracing::instrument]` before
    accepting any macro value claim.
  - Use the candidate ranking to update the roadmap and future validation
    notes.
- Non-goals:
  - Add a Statelet dependency to all candidate repositories.
  - Replace Stateright, Ninja, parser tokenization, semantic collection, daemon
    orchestration, or CLI control flow.
  - Publish a transition macro because a candidate contains enums or matches.
  - Treat observability polish alone as proof that Statelet should own a public
    macro API.

## Migration plan

1. Keep `mdtablefix` as the first validation slice.
2. Replace the previous second-example placeholder with this ADR's `wireframe`
   recommendation in the design and roadmap.
3. After `mdtablefix`, apply the conventions-only baseline to the `wireframe`
   connection actor seam.
4. Record whether `StateName`, documented `transition.*` fields, local helpers,
   and `#[tracing::instrument(fields(...))]` make logs, tests, and reviews
   clearer than the local baseline.
5. Use `weaver` only as the next pocket candidate if more evidence is needed
   before deciding whether the runtime/conventions crate earns publication.

## Known risks and limitations

- The exploration did not implement a spike in any candidate repository, so the
  ranking remains evidence-informed but unproven.
- `wireframe`'s verification model may tempt Statelet towards graph ownership.
  The spike must keep Stateright responsible for model checking.
- `weaver` and `netsuke` may produce useful observability examples without
  proving that Statelet needs a public macro.
- `ddlint` may become a stronger fit later if its parser mode state grows, but
  using it now would mostly validate parser-adjacent enthusiasm rather than the
  current Statelet design.

## Outstanding decisions

- Whether the `wireframe` spike demonstrates enough value to remain the
  established second validation domain.
- Whether `StateName` needs only stable string labels or a stronger
  low-cardinality identifier after `mdtablefix` and `wireframe`.
- Whether the macro remains deferred after both validation examples, ships as a
  later feature, or is dropped in favour of the runtime conventions crate.

## Architectural rationale

This direction preserves the core Statelet boundary: ordinary Rust owns the
state machine, while Statelet standardizes names and observability at selected
transition boundaries. `wireframe` gives the project a production-shaped actor,
runtime logs, and verification vocabulary without requiring Statelet to own
dispatch or graph semantics.
