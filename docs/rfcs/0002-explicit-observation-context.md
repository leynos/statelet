# RFC 0002: Expose explicit observation context and extension points

## Preamble

- **RFC number:** 0002
- **Status:** Proposed
- **Created:** 2026-10-07
- **Related:** [Technical design](../design.md), section 9;
  [RFC 0001](0001-transition-capture-semantics.md).

## Summary

Provide a conventions-only integration pattern that retains the exact
observation target for an invocation. Let callers supply application context
and attach observations discovered during execution without retaining a borrow
of their mutable state. Compare caller-owned tracing spans with a narrow
handle before designing a new public wrapper.

## Problem and current state

The existing field contract specifies observations but not how application
code finds the correct invocation to update. A Trasic source origin may be
known at entry, an object identifier may appear during elaboration, and the
outcome arrives at return. Nested operations make an implicit current span an
ambiguous destination for those facts.

Tracing fixes a span's field set in its metadata. A declared empty field can
receive a later value; recording an undeclared field does not extend that
set.[^1] An API promising arbitrary late-added fields would therefore need
another explicit representation.

## Goals and non-goals

The proposal enables exact invocation attribution, caller-owned context,
filtered-child safety, and short observation borrows. It does not own source
maps, introduce mutable action hooks, create application identifiers, or
require a new observation type in the core runtime.

## Proposed design

### An explicit target with no application-state borrow

The baseline retains the actual invocation span or an equivalent small handle.
Entry capture releases application borrows before the body executes. Domain
code may attach already computed identifiers or selected values to that target.
Completion records against the same target, regardless of the current nested
operation.

A disabled target remains disabled. Never fall back to `Span::current()` or
`or_current()` when recording a child's completion: doing so could attribute
it to an enabled parent. Selecting a parent for a new context span is a
separate, explicit operation, not a completion fallback.

A handle may retain observation metadata and tracing context. It must not
retain `&self`, `&mut self`, an application callback with mutation authority,
or a deferred state accessor that obstructs the body's borrows.

### Two baseline integration forms

The first form uses a caller-created span. The caller declares the standard
transition fields and its own extension fields at construction, reserving late
values with `Empty` where appropriate.

The second form nests a standard transition span under an application-owned
context span. Source provenance and invocation identity belong to the context;
transition observations belong to the child. Tests must account for whether the
chosen recorder follows parent context rather than assuming all exporters
flatten inherited fields.

A new Statelet wrapper is justified only if comparison with these forms shows
repeated, general-purpose boilerplate. Any tracing-specific public handle must
remain behind the tracing integration feature.

### Ownership and context vocabulary

Statelet owns the accepted `transition.*` field contract. Applications own their
extension namespaces. Trasic-local candidates include the following fields;
they are examples, not additions to Statelet's public vocabulary.

```plaintext
trasic.source_id
trasic.source.start
trasic.source.end
trasic.origin_id
trasic.invocation_id
trasic.object_id
```

Source paths, input text, and error details require a caller-selected disclosure
policy. Do not capture them by default. High-cardinality invocation and object
identifiers are diagnostic context, not state labels or metric dimensions.
Statelet must not depend on Trasic's source-map or scene types.

### Field validation and semantic events

The test adapter must detect attempts to use required undeclared fields rather
than silently accepting an incomplete record. Production integrations may
choose a documented omission or diagnostic policy, but must not invent fields
that tracing cannot attach.

Domain facts such as `DefaultTextureResolved` remain application events. The
handle can associate them with an invocation without turning them into new
control states or Statelet-owned event types.

## Requirements

- **H1:** Late observations target the original invocation, including during
  nested calls and when a child is filtered out.
- **H2:** Entry capture cannot retain an application-state borrow across domain
  execution or change the operation's ownership model.
- **H3:** Caller context has an explicit declaration and attachment path.
  Required missing fields must fail the validation example.
- **H4:** The no-default-feature path introduces no tracing-specific public
  type requirement. Applications retain source and identity ownership.
- **H5:** Statelet emits observations, not application actions. No integration
  hook receives mutable domain state merely to observe it.

## Validation and falsification

Compare both baseline forms on a nested conditional operation and a temporary
context restoration boundary. Enable the parent while filtering the child;
verify that child completion never changes the parent's fields. Then enable
both and verify that late object identity reaches only the intended invocation.

Include a missing `Empty` field declaration, a non-current explicit target,
multiple sibling operations, and a mutable state that cannot be cloned. Verify
that a local recorder distinguishes parent context from child fields.

Reject a wrapper if ordinary `Span` handles solve the problem more clearly.
Reject any variant requiring application restructuring, implicit parent
fallback, or a shared global context registry.

## Compatibility and migration

Prototype the pattern in task 6.2.2 after the contract review in 6.1.2. No
attribute syntax, new wrapper, or dependency is introduced by this RFC. Task
6.3.4 decides whether documented patterns suffice. Any later macro must expose
a way to select the observation target rather than duplicating the complete
tracing attribute grammar.

## Alternatives considered

An implicit current-span API is shorter but ambiguous under nesting and
filtering. A generic mutable observer context crosses the marker-only boundary.
A universal dynamic field map duplicates tracing infrastructure and requires a
consumer that the current investigation has not demonstrated.

## Open questions

Which integration form is clearer in both `mdtablefix` and `wireframe`? Should
missing required fields produce a test-only failure or a runtime diagnostic?
Does an actual second consumer justify a wrapper beyond a tracing span?

## Recommendation

Keep observation targets explicit and application borrows short. Prove a need
for a public handle rather than making one a prerequisite for validation.

[^1]: [Tracing Span::record documentation](https://docs.rs/tracing/latest/tracing/span/struct.Span.html#method.record),
    consulted 2026-10-07.
