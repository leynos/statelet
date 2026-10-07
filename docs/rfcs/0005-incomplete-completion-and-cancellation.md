# RFC 0005: Describe incomplete completion without owning lifecycle

## Preamble

- **RFC number:** 0005
- **Status:** Proposed
- **Created:** 2026-10-07
- **Related:** [Technical design](../design.md), sections 11.3 and 14;
  [RFC 0001](0001-transition-capture-semantics.md);
  [RFC 0003](0003-observational-non-interference.md).

## Summary

Define normal return, observed unwinding, and missing completion as different
observation states. Keep synchronous completion explicit and any incomplete
marker best-effort. Record future cancellation questions and tests without
bringing async support forward or giving Statelet cleanup, rollback, or
application lifecycle responsibilities.

## Problem and current state

The proposed return observation does not cover an operation that never reaches
its completion code. Dropping an unfinished guard does not establish why that
happened. It can reflect an omitted finish call, abandonment, unwinding, or a
future dropped before completion.

Tracing span lifetime is not identical to function execution: spans can be
entered repeatedly, and cloned handles affect closure.[^1] A collector must not
interpret span closure as successful return. Rust also permits termination
paths that do not run destructors, so no guard can promise a terminal record
for every invocation.[^2]

## Goals and non-goals

The proposal gives incomplete observations an honest meaning and defines the
limits of synchronous unwind reporting. It specifies the evidence required
before a later asynchronous integration can claim support.

It does not infer cancellation causes, repair state, roll back effects, catch
application panics, guarantee log delivery, or introduce a cancellation token,
executor, or lifecycle manager.

## Proposed design

### Synchronous completion vocabulary

The candidate vocabulary records what the observer knows, not why the domain
operation behaved that way.

| Completion | Meaning | Does not establish |
| --- | --- | --- |
| `returned` | The operation returned and completion capture ran | Domain success, rollback, or post-state availability |
| `unwind_observed` | An unfinished observation noticed active unwinding | Which operation caused the panic or whether cleanup finished |
| `incomplete` | The observation ended without recorded completion | Cancellation, failure, or absence of effects |
| Field absent | No completion observation is available | Any terminal outcome |

*Table 1: Proposed completion labels and inference limits.*

A returned `Err` uses `returned` plus the independent return classification from
RFC 0001. Successful early returns and `?` exits should reach explicit
completion when an implemented wrapper claims to support those shapes. A
handwritten path that omits finish instead yields incomplete evidence.

### Observation handle lifecycle

The baseline has an active observation and at most one explicit completion.
Its handle owns metadata only. Finishing consumes or otherwise terminally marks
that observation so later drop cannot overwrite a completed result. A test
adapter must reject contradictory or duplicate terminal recordings.

An optional drop marker may report incomplete observation using already
captured metadata. It must not inspect application post-state, call domain
cleanup, or rerun result projections. It remains best-effort: filters, bounded
recorders, termination, and subscriber failures can prevent delivery.

Unwind reporting is a separately validated extension to this marker. It must
not attribute panic causality merely because unwinding is active. Observation
callbacks used during unwinding must be non-panicking; Statelet cannot make an
arbitrary subscriber safe. If the baseline cannot satisfy that constraint, omit
unwind emission and report unavailable completion rather than adding a
`catch_unwind` policy or risking a second panic.

### Deferred asynchronous integration

Async support remains out of scope until roadmap task 5.2.1 accepts it. When
that gate opens, use poll-aware future instrumentation rather than holding an
entered-span guard across `.await`. Tracing's `Instrument` provides the relevant
poll-scoped integration.[^3]

Specify when observation begins: future construction or first poll. A future
created and dropped without polling must not look like an executed transition.
If an integration deliberately creates a wrapper observation at construction,
label that construction separately from domain execution. Otherwise no
invocation record is expected before first poll.

A future dropped after suspension can produce an incomplete observation. Call
it cancelled only when the application explicitly provides that reason. A
synchronous scoped dispatcher does not automatically follow work onto another
executor thread; tests must install or propagate the intended context.

## Requirements

- **L1:** Separate completion, return classification, domain outcome, and
  post-state availability.
- **L2:** Record no more than one terminal completion for an invocation; do not
  overwrite a completed result from a destructor.
- **L3:** Incomplete and unwind observations perform no domain reads, cleanup,
  rollback, or panic interception.
- **L4:** Document missing-delivery and destructor limits. Absence of a terminal
  record proves neither failure nor cancellation.
- **L5:** Keep async implementation deferred until its existing acceptance gate;
  any later claim must state observation start and context propagation rules.

## Validation and falsification

First test normal return, returned error, early return, `?`, an omitted finish,
and nested cleanup. Verify that required application restoration matches the
uninstrumented baseline even when an error follows a committed mutation.
Reject a negative control that labels every unfinished observation cancelled.
Reject a duplicate-completion control and one that reads post-state in `Drop`.

Use an isolated unwind test only where the configured panic strategy supports
unwinding. Process-abort and explicit-exit cases must not require a destructor
record. Document whether an optional subprocess fixture exercises these limits;
do not run an abort in the test runner itself.

If async support is later accepted, test drop before first poll, suspension and
resumption, drop after suspension, completion, panic during polling, nested
futures, filtering, and execution-context changes. Assert no cross-task span
attribution or changed domain cleanup. No async result may be inferred from a
synchronous-only baseline.

## Compatibility and migration

Tasks 6.1.5 and 6.2.3 cover the synchronous contract and baseline. The adoption
review in 6.3.4 may retain explicit finish alone. Step 6.6 depends on the
separate async decision and does not block conventions-only publication.

## Alternatives considered

Equating span closure with success loses return semantics. Calling unfinished
work cancelled invents a cause. Automatic rollback exceeds Statelet's scope.
Mandatory destructor logging risks observation failures during unwinding. An
explicit-finish-only baseline is a legitimate outcome if it proves sufficient.

## Open questions

Do real consumers need an unwind marker beyond incomplete observation? Which
capture-start policy makes a future's lifetime least surprising? Can a later
async implementation preserve all claimed ownership and auto-trait properties?

## Recommendation

Adopt only completion facts the observer can establish. Preserve explicit
application ownership of cleanup, effects, and cancellation reasons.

[^1]: [Tracing span lifecycle](https://docs.rs/tracing/latest/tracing/span/index.html),
    consulted 2026-10-07.
[^2]: [Rust Reference: destructors](https://doc.rust-lang.org/reference/destructors.html),
    consulted 2026-10-07.
[^3]: [Tracing Instrument interface](https://docs.rs/tracing/latest/tracing/trait.Instrument.html),
    consulted 2026-10-07.
