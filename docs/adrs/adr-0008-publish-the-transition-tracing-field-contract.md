# Architectural decision record (ADR) 0008: Publish the transition tracing field contract

## Status

Accepted, 2026-11-23. The five field names, their location on the boundary
span, and the semantic-versioning rule are accepted. The presence and value
rules are provisional until roadmap task 3.1.3 and are revisable by a dated
amendment section in this record. Publication itself remains conditional:
[ADR 0003](adr-0003-record-the-v0-1-exit-register.md) still decides whether the
conventions-only scope ships at all.

## Date

2026-11-23.

## Context and problem statement

The technical design, `docs/design.md` §9, lists five transition tracing fields
— `transition.name`, `transition.state.before`, `transition.event`,
`transition.outcome`, and `transition.error` — and calls their names public
operational API. Dashboards, alerts, and log queries may depend on them, so a
rename is a breaking change that the Rust compiler cannot detect. Until this
record, the names existed only as a bullet list in a design section written for
a future macro. Nothing stated what each field means, where it is recorded,
when it is present, or what shape its value takes, and the documents that
mentioned the fields disagreed:

- `docs/design.md` §9 calls all five "default generated fields" of the future
  macro and allows it to record a subset.
- `docs/state-representation-and-observability-design.md` (Proposed) marks
  `transition.outcome` and `transition.error` optional, adds a proposed
  `transition.state.after`, and defines `transition.event` as the stable
  semantic class of the handled input.
- `docs/context.md` defines a transition outcome as "the value returned by a
  transition boundary to describe the visible result", and accepted
  [ADR 0002](adr-0002-scope-statelet-to-transition-boundary-marking.md) says
  the outcome field "must use the project-local decision or return shape
  already present".
- Proposed RFC 0001 separates domain outcome from return classification,
  records an outcome alongside an error return, and adds fields that are not
  part of this contract.
- ADR 0002's own example declares only three fields, which would silently drop
  a late-recorded outcome or error.
- No document chooses `Display`, `Debug`, or a bounded category for
  `transition.error`.

Roadmap task 2.1.2 requires the contract to be documented and requires
downstream code to be able to use the same fields with ordinary
`#[tracing::instrument(fields(...))]`. The question this record answers is what
the five names mean, where and when they appear, what values they carry, and
what stability promise attaches to them.

## Decision drivers

- Accepted documents fix the five names but not their semantics; the
  disagreement above must be settled in one place rather than restated in three.
- Operational consumers depend on field names and meanings, not on the label
  vocabulary of any application's states, events, outcomes, or errors.
- `tracing` silently discards a value recorded for a field that was not
  declared when the span was created, so the contract must state the
  declaration rule rather than leave it to discovery.
- Manual recording can be skipped by a `?` or an early return, so absence
  cannot be given a meaning it does not have.
- Roadmap task 3.1.2 expects the contract to be "proven or revised in Phase
  2", so freezing every rule now would force costly supersessions.
- The runtime crate must remain free of a `tracing` dependency; the contract
  must be usable with the ordinary `tracing` crate alone.

## Options considered

| Option                        | What it publishes                     | Why it was not chosen                                                                                   |
| ----------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Documentation only            | The rules, with no code               | Leaves the runtime crate owning the contract only on paper and gives a later macro nothing to reference |
| `FieldName` newtype           | A wrapper type per name               | `tracing`'s field-name position and `Span::record` take `&str`, so every use would need `.as_str()`     |
| `tracing`-typed constants     | Constants typed by `tracing`          | Requires a runtime `tracing` dependency, which design §3.4 defers                                       |
| Constants behind a feature    | The constants only with `tracing`     | The constants need no `tracing`, so the feature would add matrix rows for no benefit                    |
| `statelet::transition` module | The constants under that path         | Clashes conceptually with the future `#[statelet::transition]` attribute                                |
| `statelet::fields` constants  | Five `pub const` names of type `&str` | Chosen: compiles a misspelt name, needs no dependency, and a later macro can emit the paths             |

*Table 1: Options considered for publishing the field names.*

## Decision outcome / proposed direction

Statelet fixes the five field names, records them in the table below, and
publishes the names as five zero-dependency constants in the module
`statelet::fields`. The dotted string literal form remains a documented, equal
alternative. The contract fixes field names, their location, and their meaning,
presence, and value-shape rules. It does not fix the label vocabulary of any
value: transition names, state labels, event classes, outcome labels, and error
categories remain application-owned.

## The field contract

The table is the single source of each field's meaning, presence, and value
rule. `docs/design.md` §9.1, `docs/users-guide.md`, and the `statelet::fields`
rustdoc name the same fields and link here rather than restating the rules.
`tests/transition_fields/docs_agree.rs` checks that this table, those name
lists, and the exported constants stay in agreement.

<!-- transition-fields:begin -->

| Field                     | Meaning                                                    | Recorded when                                                  | Value rule                                                                                                                             |
| ------------------------- | ---------------------------------------------------------- | -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `transition.name`         | Stable label naming the transition boundary                | Always, when the span is created                               | A stable, low-cardinality label chosen by the application, usually the boundary's name                                                 |
| `transition.state.before` | The `StateName` label of the state observed on entry       | When the boundary observes a state                             | A stable label; it should be the value `StateName::state_name` returns                                                                 |
| `transition.event`        | Stable semantic class of the input the boundary handles    | When the boundary handles an input                             | A low-cardinality class of the input rather than the input itself; the form used by a future macro's `event(...)` argument is deferred |
| `transition.outcome`      | Stable label of the result the boundary produced           | When the application has such a label for the completed call   | A stable, low-cardinality label chosen by the application, for example the variant of its own decision enum                            |
| `transition.error`        | Bounded category of the error a fallible boundary returned | Only by a fallible boundary, and only when it returns an error | A stable, low-cardinality category, for example the variant label of the application's error enum; not arbitrary error text            |

<!-- transition-fields:end -->

*Table 2: The transition tracing field contract.*

## Location and presence rules

- The fields belong to the span that represents one invocation of the
  transition boundary. Span name, level, and target are outside this contract.
- Every contracted field the boundary may record must be declared when the
  span is created. A field known only after the body runs is declared
  `tracing::field::Empty` and filled through `Span::record`. An undeclared
  field's recorded value is discarded silently.
- `transition.name` is recorded when the span is created.
- `transition.state.before` is recorded when the span is created, by a
  boundary that observes a state.
- `transition.event` is recorded when the span is created, by a boundary that
  handles an input.
- `transition.outcome` follows the meaning in Table 2. This record does not
  say whether it is recorded when the boundary returns an error; consumers must
  not assume it is absent on the error path. Proposed RFC 0001 records an
  outcome alongside an error, and forbidding that now would turn its possible
  acceptance into a breaking change.
- `transition.error` is declared only by fallible boundaries and recorded only
  when the boundary returns an error. Infallible boundaries do not declare it,
  matching design §8: "When `infallible` is present, the macro omits
  error-specific fields."
- An absent field means "not captured by this boundary". It never stands for a
  default or a guessed value, and it never implies success or failure. A
  boundary that returns through `?` before its `record` call leaves
  `transition.error` absent even though it failed.

## Value rules

- All five fields carry string values, recorded as strings: a `&str` or a
  `Display` value, not `Debug` output. A label `stay` is recorded as `stay`, not
  `"stay"`.
- `transition.name`, `transition.state.before`, `transition.event`, and
  `transition.outcome` carry stable, low-cardinality labels.
  `transition.state.before` should be the `StateName::state_name` label.
  `transition.event` should be a low-cardinality semantic class of the input
  rather than the input itself.
- `transition.error` carries a stable, low-cardinality category of the error,
  for example the variant label of the application's error enum, not arbitrary
  error text. The free-text message belongs to the application's own logging or
  to `#[instrument(err)]`, which emits a separate `ERROR` event with a field
  named `error`.
- No field may contain secrets or raw payloads. Letter case of values is not
  fixed.

## Prefix ownership and extension

Within a span that carries `transition.name`, Statelet owns every
`transition.*` key, and Statelet may add further names under the `transition.`
prefix. An application that uses the prefix for its own fields on such a span
accepts the collision risk.

A new facet uses a sibling segment rather than extending an existing name —
`transition.state.after` or `transition.state_id.before`, never
`transition.state.before.id` — so no name is ever both a field and a namespace.
The rule keeps the dot-prefix invariant that prevents object-versus-leaf
mapping clashes in OpenTelemetry Protocol (OTLP) JSON and document stores.

## Semantic versioning

Once a release containing this contract exists, adding a field is additive and
compatible. Renaming or removing a field, or changing its meaning, location, or
accepted value shape incompatibly, is a breaking change; while the crate is 0.x
that requires a minor version bump. Before such a release, and under ADR 0003's
exit E1, nothing is promised.

## Relationship to `#[instrument(err)]` and span status

`#[instrument(err)]` does not record a span field. It emits an `ERROR`-level
event carrying a field named `error`. `transition.error` complements it: the
contracted field carries a bounded category on the span, while the event
carries the error's `Debug` rendering for diagnosis. `transition.error` does
not set an OpenTelemetry span status, and no field in this contract does.

## Goals and non-goals

- Goals: make the five names, their location, and their value shape citable
  and testable; let a downstream crate name them through the compiler; give a
  future macro one authority to emit.
- Non-goals: publish an outcome type or vocabulary (design §6.2, ADR 0002);
  fix the label vocabulary of any value; add `transition.state.after`,
  `transition.completion`, `transition.return_kind`, or any other field
  proposed by RFCs 0001–0005 or ADR 0007; add a `tracing` feature or runtime
  dependency; provide an outcome-recording helper or a macro.

## Known risks and limitations

- The contract can be mistaken for a release commitment. ADR 0003 places the
  published field contract in exits E2 and E3 only; exit E1 keeps the
  conventions project-local.
- The five constants are public API added on anticipation. Roadmap task 3.2
  decides whether they stay, and the outstanding decisions below require usage
  evidence first.
- The presence and value rules are provisional and may be contradicted by a
  later accepted RFC. They are revisable by a recorded amendment; the names and
  the semver rule are not.
- Authors can forget to declare a late field, and `Span::record` silently
  drops the value. The presence rules require declaration, and the negative
  control `AXIOM-undeclared-dropped` in the 2.1.2 plan demonstrates the drop.
- Consumers may read absence as success. The contract says absence never
  implies success or failure, and a behavioural scenario shows a `?` early
  return leaving `transition.error` absent.

## Outstanding decisions

- Whether the five constants or dotted literals are used in practice. The
  validation notes of roadmap tasks 2.2.1 and 2.2.2 record which form
  `mdtablefix` used and whether the `{fields::X}` form hurt reviewability.
  Roadmap step 3.2 decides whether the constants stay before any release.
- Whether the macro's `event(...)` argument records the expression itself or a
  projected low-cardinality class. Design §14 records the question as deferred;
  this contract recommends a class without forbidding the expression form
  accepted in design §6.3 and ADR 0002.
- Whether `transition.outcome` is recorded on the error path. Proposed
  RFC 0001 settles it if accepted; until then consumers must not assume either
  way.

## Provisional rules and amendment route

The presence and value rules are provisional until the B1 gate at roadmap 3.1.3
(G2). A provisional rule changes through a dated `## Amendments` section in
this record, stating the old rule, the new rule, and the evidence that forced
the change. The accepted parts — the five names, their location on the boundary
span, and the semantic-versioning rule — change only by supersession.

## Architectural rationale

ADR 0002 makes Statelet a marker over application-owned state machines, and
design §3.2 makes the runtime crate the owner of the shared vocabulary. This
record is that ownership made citable: it publishes the names the runtime crate
already owns, keeps every application-specific label out of the contract, and
gives the no-dependency build the same names through `statelet::fields`. It
adds no framework surface and leaves ADR 0003's release decision untouched.

## References

- `docs/design.md` §§3.2, 3.4, 6.2, 8, 9, 14.
- [ADR 0002](adr-0002-scope-statelet-to-transition-boundary-marking.md),
  [ADR 0003](adr-0003-record-the-v0-1-exit-register.md), and
  [ADR 0004](adr-0004-define-the-statename-consumption-evidence.md).
- Proposed RFC 0001 and
  `docs/state-representation-and-observability-design.md`, as context only.
- `docs/context.md`, "Transition outcome" and "State name".
- [OpenTelemetry semantic conventions for exceptions][otel-exceptions]
  (`error.type` follows the same low-cardinality practice).

[otel-exceptions]: https://opentelemetry.io/docs/specs/semconv/exceptions/exceptions-spans/
