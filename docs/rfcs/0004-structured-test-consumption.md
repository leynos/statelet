# RFC 0004: Add a structured observation consumption path for tests

## Preamble

- **RFC number:** 0004
- **Status:** Proposed
- **Created:** 2026-10-07
- **Related:** [Technical design](../design.md), sections 9 and 11;
  [RFC 0001](0001-transition-capture-semantics.md);
  [RFC 0002](0002-explicit-observation-context.md).

## Summary

Prototype an example-local recorder for structured transition observations.
Consume creation fields, late recordings, events, and context without parsing
formatted logs. Make filtering, truncation, and missing completion visible.
Promote a typed observer or shared test-support crate only when multiple real
consumers demonstrate duplicated requirements.

## Problem and current state

The design concentrates on producing observations. Tests also need a reliable
way to read them. A collector that reads only span creation misses outcomes and
post-state recorded at return. Formatted log snapshots couple tests to display
choices and volatile timestamps rather than semantic contracts.

Trasic can use a trace to explain an incorrect semantic scene, but must not
mistake an incomplete capture for proof that an event never happened. It also
must not require HgPovRay to follow the same internal transition sequence.

## Goals and non-goals

The proposal enables test-local recording, stable semantic assertions, context
correlation, and explicit evidence completeness. It does not replace application
unit tests, scene conformance snapshots, model checking, or tracing itself.
There is no new public observer API or serialization dependency in this RFC.

## Proposed design

### Start with a tracing layer

Build a small local `tracing-subscriber::Layer` that handles span creation,
field updates, relevant events, and span closure. The layer interface exposes
these integration points.[^1] Merge selected late fields into the invocation
record and keep explicit completion distinct from span closure.

Retaining another span handle can delay closure. Tests must therefore inspect
explicitly completed observations and report outstanding records at the end of
the collection scope rather than treating `on_close` as operation success.
A missing post-state remains missing; the recorder must not copy the entry
label into it.

Install the recorder with a scoped dispatcher for synchronous tests rather than
a global subscriber. Tracing provides `with_default` for a scoped default.[^2]
A later asynchronous test must propagate its dispatcher with the future or
task; a synchronous thread-local scope alone is not a propagation guarantee.

### Stable assertions and identity

Assert selected field values, domain outcomes, explicit parentage, and relevant
ordering. Do not assert timestamps, addresses, raw formatted strings, or a
global total order across independently executing tasks.

Subscriber-assigned span IDs are local correlation keys, not persistent
application identity.[^3] Applications may supply session and invocation IDs
through RFC 0002's context seam. Normalize volatile identities in golden
records while preserving the parent/child relationships that the test checks.
Do not turn per-invocation identifiers into metric labels.

### Completeness is part of the result

A recorder may be bounded. Its result must carry overflow or dropped-record
counts, outstanding observations, and the configured filtering scope. The
collector must not silently evict older observations and then certify absence.

The following illustrative result distinguishes a failed evidence capture from
a successful domain operation.

```plaintext
capture.scope       = "all selected conditional operations"
capture.overflow    = 1
capture.outstanding = 0
assertion           = "inconclusive: capture incomplete"
```

A test asserting that an event did not occur must first establish coverage and
completeness for that event class. Filters can intentionally exclude other
classes; completeness need not mean recording every event in the application.
Unknown required fields and malformed completion records must fail the test or
produce an explicit unusable-evidence result, not disappear silently.

### Keep evidence formats separate

Store observation-integration notes under a distinct marker and schema from
ADR 004's `StateName` consumption notes. Do not change that accepted register
or feed new note fields into its parser implicitly. Naming evidence relevant
to `StateName` still uses its existing instrument.

Each integration note records the compared baseline, fixture revision, capture
configuration, semantic assertions, observed defect, and keep/revise/reject
verdict. Missing or truncated evidence cannot support a publication claim.
The schema and marker must be reviewed before executable tests consume them.

## Requirements

- **T1:** Capture late fields and events against the correct invocation.
- **T2:** Use test-local installation without global subscriber mutation.
- **T3:** Distinguish completion, span closure, filtering, and truncation.
- **T4:** Make absence assertions conditional on declared capture completeness.
- **T5:** Keep application identifiers separate from subscriber IDs and keep
  integration evidence separate from ADR 004's accepted note schema.

## Validation and falsification

Exercise nested operations, sibling operations, late return classification,
missing post-state, delayed span closure, filtered children, and bounded
recorder overflow. Introduce negative controls that discard `on_record`, attach
child completion to a parent, or suppress overflow reporting. The tests must
reject all three.

Use semantic assertions alongside focused snapshots. For Trasic, compare scenes
for conformance and use observation records only to explain a mismatch. A
reference renderer with different internal control flow is not a failing trace
fixture.

Reject a shared recorder abstraction if a short local layer remains clearer.
Consider a typed record and observer only after at least two consumers repeat
the same reconstruction and can state what the new interface removes.

## Compatibility and migration

Tasks 6.1.4 and 6.3.1 cover schema review and the local recorder. Task 6.3.3
collects evidence from `mdtablefix` and `wireframe`; task 6.3.4 decides the
publication boundary. Trasic remains supplementary in step 6.4. No new test
crate, public trait, or dependency follows automatically from RFC acceptance.

## Alternatives considered

Formatted-log parsing is brittle and loses structured omissions. A global test
subscriber risks cross-test interference. Publishing an `Observer` trait now
would spend public API before proving a consumer. Unbounded collection avoids
one overflow path but does not solve filtering or missing completion.

## Open questions

What is the smallest record shape both proving grounds consume? Should a test
scope fail immediately on overflow or return an explicit incomplete result?
Which contexts require stable application IDs rather than local span identity?

## Recommendation

Validate structured consumption using existing tracing extension points first.
Treat evidence completeness as a requirement, not a reporting embellishment.

[^1]: [Tracing subscriber Layer interface](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/layer/trait.Layer.html),
    consulted 2026-10-07.
[^2]: [Tracing scoped dispatcher](https://docs.rs/tracing/latest/tracing/dispatcher/fn.with_default.html),
    consulted 2026-10-07.
[^3]: [Tracing Subscriber identity contract](https://docs.rs/tracing/latest/tracing/subscriber/trait.Subscriber.html#tymethod.new_span),
    consulted 2026-10-07.
