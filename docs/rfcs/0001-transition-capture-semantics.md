# RFC 0001: Separate transition capture, return, and outcome semantics

## Preamble

- **RFC number:** 0001
- **Status:** Proposed
- **Created:** 2026-10-07
- **Related:** [Technical design](../design.md), sections 7 to 9;
  [ADR 002](../adr-002-transition-boundary-scope.md).

## Summary

Define entry and completion observations independently of application control
flow. Add an optional post-state observation and an explicit, borrowing result
projection. Distinguish how an invocation finishes, how its return value is
classified, and what the operation means in its application. Prototype these
contracts with ordinary Rust and tracing before proposing public helpers or
attribute syntax.

This request for comments (RFC) proposes a contract, not an implemented
application programming interface (API). Acceptance does not bypass the
runtime-first or macro-publication gates.

## Problem and current state

The design names `transition.state.before`, `transition.outcome`, and
`transition.error`, but leaves post-state capture and completion semantics
underspecified. Its descriptive macro handling may omit error fields when
return syntax uses an alias. That omission avoids pretending to perform type
inference, but callers need a supported way to supply the missing knowledge.

A Trasic elaboration operation can return an error after changing a binding,
return successfully without changing its control-state label, or visit several
states and return to its original label. Neither a result variant nor label
equality establishes whether effects occurred or whether cleanup succeeded.

## Goals and non-goals

The proposal makes capture timing, ownership, omission, and result
classification reviewable. It supports move-only arguments, application-owned
errors, and explicitly selected observations of consumed state.

It does not introduce a dispatcher, rollback, state equality, semantic type
inference, a mandatory result enum, or a universal `TransitionOutcome`. Domain
logic and effects remain in the annotated operation.

## Proposed design

### Independent observation dimensions

Use the following candidate fields in the handwritten validation baseline.
Existing field meanings remain unchanged while the RFC is proposed.

| Field | Proposed meaning | Omission rule |
| --- | --- | --- |
| `transition.state.before` | Selected label immediately before domain execution | Omit when no safe entry projection exists |
| `transition.state.after` | Selected label after normal return, including `Err` | Omit when unconfigured or unavailable |
| `transition.completion` | Observation of invocation completion | See RFC 0005; absence is not success |
| `transition.return_kind` | Caller classification such as `ok`, `err`, or `value` | Omit when unclassified or no return occurred |
| `transition.outcome` | Application-defined meaning of the operation | Omit when no outcome projection exists |
| `transition.error` | Explicitly selected, redacted error information | Omit when unavailable or deliberately excluded |

*Table 1: Candidate capture fields and their omission semantics.*

An omitted post-state means that no post-state was observed. It must not mean
that the state was unchanged. Even equal observed labels describe only the
selected projection, not equality of the complete application state.

The following illustrative record describes a rejected conditional directive.
It does not claim rollback or whole-state equality.

```plaintext
transition.name         = "trasic.conditional.select_else"
transition.state.before = "skipping"
transition.state.after  = "skipping"
transition.completion   = "returned"
transition.return_kind  = "err"
transition.outcome      = "duplicate_else"
```

### Capture sequence and ownership

On an enabled path, capture the configured entry label and event projection at
most once, immediately before invoking the domain operation. Release those
borrows before domain execution. Store small labels or identifiers, not a
borrow of the elaborator or a clone of its complete state.

Execute the operation exactly once. After a normal return, borrow the result
for classification, then obtain any configured post-state through a compatible
projection. Document a deterministic order for these projections. Do not
consume or replace the returned value.

For a consuming method, the caller may derive post-state from the returned
replacement. For a borrowed return, a post-state projection is permitted only
when the original borrowing contract still compiles. The baseline must support
an explicit omission path rather than forcing cloning or a new lifetime bound.
No post-state projection runs from a destructor or during unwinding.

### Explicit return classification

Allow a handwritten function or closure to borrow an application result and
provide selected observation values. An alias such as `ElabResult<Decision>`
then requires no syntactic recognition by Statelet. A future macro may invoke
an explicitly supplied projection, with Rust checking its types.

Automatic syntactic recognition remains descriptive and conservative. It must
not become mandatory when a manual projection exists. Explicit projections
must not impose `Debug`, `Display`, `Clone`, or serialization bounds on the
whole state, event, result, or error. Formatting requires only the selected
fields' documented capabilities.

## Requirements

- **C1:** Preserve the operation's return value, errors, effects, and supported
  ownership shapes. Error return must never imply rollback.
- **C2:** Distinguish absent observations from observed labels and outcomes.
  A filtered invocation provides no evidence about domain effects.
- **C3:** Capture entry and post-state independently. Never retain an entry
  borrow across a body that requires mutable access.
- **C4:** Offer explicit borrowing result classification before considering
  additional automatic return-shape inference or public result types.
- **C5:** Apply [RFC 0003](0003-observational-non-interference.md) to evaluation
  counts, filtering, and projection preconditions, and
  [RFC 0005](0005-incomplete-completion-and-cancellation.md) to missing returns.

## Validation and falsification

The bet is that independent observations explain real failures without
reshaping application code. Test a successful unchanged label, an error after
mutation, a leave-and-return sequence, an opaque result alias, a non-`Debug`
error, a moved event, a consuming method, and a borrowed return. Include an
unavailable post-state and verify that the recorder does not synthesize one.

A negative control should incorrectly equate `Err` with rollback; semantic
assertions must reject it. Another should capture entry twice; projection
counters must reject it. Use the structured recorder from RFC 0004 rather than
formatted log comparison.

Reject or narrow the proposal if equivalent plain tracing is clearer, if a
supported signature needs extra cloning or trait bounds, or if the projections
cannot remain observational. Document unsupported shapes explicitly; do not
claim universal function wrapping.

## Compatibility and migration

Start with example-local conventions. Record acceptance, revision, or rejection
for each field before extending the published operational contract.
Existing dashboards and naming evidence retain their current meaning.

Roadmap tasks 6.1.1 and 6.2.1 cover contract review and the handwritten
baseline; 6.3.4 governs adoption. Task 6.5.1 evaluates macro projection syntax
only after the existing macro gate permits implementation.

## Alternatives considered

A single `Stay`/`Move`/`Error` outcome loses information and mistakes selected
labels for complete state. Automatic result inference cannot resolve arbitrary
aliases syntactically. Serializing the entire state introduces ownership,
privacy, and representation coupling that the contract does not require.
Plain tracing with local projections remains the control implementation and a
valid final product outcome.

## Open questions

Which return classifications do actual consumers need? Does the baseline need
an explicit reason for an omitted post-state, or is a documented omission
sufficient? Which borrowed-return shapes justify automated capture?

## Recommendation

Validate independent, optional observations and caller-owned projections first.
Publish only the fields and helpers whose consumers demonstrate value.
