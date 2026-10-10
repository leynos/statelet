# RFC 0003: Make observational non-interference a tested contract

## Preamble

- **RFC number:** 0003
- **Status:** Proposed
- **Created:** 2026-10-07
- **Related:** [Technical design](../design.md), sections 7, 10, and 11;
  [RFC 0001](0001-transition-capture-semantics.md);
  [RFC 0002](0002-explicit-observation-context.md).

## Summary

Define and test the claim that adding Statelet observation preserves supported
application behaviour. Specify projection evaluation counts, disabled-path
behaviour, ownership constraints, and the limits of the guarantee. Compare
uninstrumented, recorded, runtime-filtered, and feature-disabled executions
before publishing helpers or generated wrappers.

## Problem and current state

The design requires preservation of ordinary control flow but does not yet
state all the ways that observation can interfere. A projection can evaluate a
scene expression twice, consume a token, advance a random stream, or repeat a
geometry query. A deferred state accessor can extend a borrow. Unnecessary
formatting bounds can reject otherwise valid application types.

The relevant promise is semantic preservation, not identical timing, allocation
counts, scheduling, or immunity to malicious observation callbacks.

## Goals and non-goals

The proposal makes semantic preservation falsifiable through small fixtures,
compile checks, and property tests. It protects application results, effects,
and required cleanup without introducing transactional semantics.

It does not prove arbitrary callback purity, guarantee behaviour after process
termination, suppress application panics, promise zero overhead, or require a
new model-checking dependency in this documentation change.

## Proposed design

### Separate library guarantees from callback obligations

Statelet-controlled code must execute the domain operation exactly once,
preserve its returned value and error, avoid extra domain queries, and retain
no unnecessary state borrow. Observation must not acquire application locks or
repeat domain callbacks merely to rediscover a value already computed.

Caller-supplied projections must be observational and non-panicking. They may
read an already available label or identifier, but must not mutate bindings,
consume input, advance random state, or query a resource with domain effects.
Subscriber and formatter behaviour also lies outside Statelet's complete
control. Document these preconditions instead of claiming that arbitrary
callbacks cannot alter semantics.

The baseline uses selected primitive values or static labels where possible.
Statelet must not introduce a general observer callback with mutable access to
the application. A future typed observer needs separate consumer evidence and
the same preconditions.

### Evaluation-count contract

For an invocation selected for observation, evaluate each configured entry
projection at most once before domain execution. Evaluate each configured
completion projection at most once after a normal return. Repeated formatting
or subscriber visits must not re-evaluate the domain projection.

When the observation is disabled before capture, do not evaluate its
observation-only projections. A runtime filter may change or an exporter may
drop data after selection; this does not authorize replay of a projection or
promise that a record reaches storage. Zero capture on a disabled target and
successful delivery are different contracts.

Use lazy capture around the actual enabled target rather than eagerly
constructing strings and then discarding them. Compile-time disabled and
runtime-filtered paths require separate evidence.

### Ownership and signature preservation

Do not add blanket `Debug`, `Display`, `Clone`, `Serialize`, `Send`, or
`'static` bounds to state, events, results, or errors. Selected observation
values can carry narrowly documented requirements. Do not clone an event to
observe it before its original move into the operation.

Consuming methods and functions returning borrows require positive controls.
If a capture combination cannot preserve a valid signature, require explicit
omission or report the combination as unsupported. Do not silently change the
return type, drop timing of application values, or domain error type.

Automatic wrapping must preserve supported early returns and `?` propagation.
Unwind observation follows RFC 0005 and must not change cleanup, catch a panic,
or evaluate post-state from a destructor.

## Requirements

- **N1:** Domain execution occurs exactly once under every supported observation
  configuration.
- **N2:** Observation-only projections obey the stated capture counts and
  disabled-target rule; output delivery is not assumed.
- **N3:** The same input produces the same return, diagnostics, semantic
  effects, and required restoration, subject to the documented callback
  preconditions.
- **N4:** Supported signatures retain their ownership, lifetime, and trait-bound
  requirements. Unsupported combinations have precise diagnostics or omissions.
- **N5:** No guarantee implies identical timing, concurrent interleaving,
  arbitrary subscriber safety, or rollback of application effects.

## Validation and falsification

Use injected token, random, resource, and geometry services with call counters.
Run the same deterministic operation without observation, with recording, with
runtime filtering, and without the integration feature. Compare semantic
outputs and service interactions; exclude observation records themselves.

The following matrix separates assertions that otherwise become conflated.

| Fixture | Required evidence |
| --- | --- |
| Disabled target | No observation-only projection executes |
| Enabled target | Each configured projection executes no more than once |
| Error after mutation | The same effects and returned error survive |
| Early return and `?` | The same exit and required cleanup occur |
| Non-`Clone` event and non-`Debug` error | Supported code still compiles |
| Consuming method and borrowed return | No retained entry borrow or new lifetime requirement |
| Nested observations | Service calls and frame restoration remain unchanged |

*Table 1: Non-interference validation fixtures.*

Add negative controls that duplicate the operation, evaluate a projection
before filtering, or rerun a geometry query for logging. Tests must reject each
control. Use bounded generated operation sequences to exercise nesting and
cleanup invariants; shrink failures to reviewable examples. Use compile-fail
fixtures for unsupported capture shapes rather than claiming runtime testing
proves ownership preservation universally.

Reject a helper or macro shape if preservation requires application cloning,
new broad bounds, or compensating domain logic. The local tracing baseline
remains a valid product outcome.

## Compatibility and migration

Task 6.1.3 reviews the guarantee and preconditions. Tasks 6.2.1 to 6.2.4
establish handwritten evidence. Task 6.5.2 repeats the relevant tests against
generated code only if the macro gate opens. Feature tests exercise features
that exist; this RFC does not create tracing, macros, or serialization
dependencies early.

## Alternatives considered

Testing only output logs cannot detect changed domain effects. A promise that
observation is always pure is not enforceable for arbitrary Rust callbacks.
Catching all observation panics introduces its own error and unwind semantics.
A code-review-only rule cannot falsify duplicated queries or disabled-path
capture reliably.

## Open questions

Which signatures occur in both proving grounds? What minimal event projection
avoids allocation and sensitive data? Which invariants justify a bounded model
check beyond deterministic and property tests once the helper exists?

## Recommendation

Make non-interference a conditional, executable contract with explicit limits.
Do not substitute an unrestricted purity claim for evidence.
