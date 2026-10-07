# Architectural decision record (ADR) 005: Stable transition and outcome names

## Status

Proposed. Extend naming-consumption evidence to transition names and domain
outcomes without pre-empting the `StateName` return-shape decision or publishing
new runtime vocabulary.

## Date

2026-10-07.

## Context and problem statement

[ADR 004](adr-004-state-name-consumption-evidence.md) already distinguishes
label stability across source renames from numeric representation. It defines
an accepted evidence instrument, not the final `StateName` verdict. This record
extends the question to transition names and application-defined outcomes. It
does not supersede ADR 004, change its register, or mark roadmap task 1.1.3
complete.

A trace query or regression assertion can depend on an operation label after
the Rust method that implements it is renamed. Deriving every operational name
from a method or variant risks changing that contract accidentally. Conversely,
assigning numeric identifiers does not automatically supply stable meaning or
reduce the number of distinct values a finite label set contains.

The Trasic investigation supplies a candidate consumer: finding the relevant
elaboration observation after a semantic-scene mismatch. That is a hypothesis
to validate, not evidence that Statelet needs a registry or a new public trait.

## Decision drivers

- Preserve the marker-only boundary and the runtime-first evidence gates.
- Distinguish field-name stability, label-value stability, and representation.
- Keep application meanings and namespaces under application ownership.
- Avoid high-cardinality data in state or outcome labels and metric dimensions.
- Preserve the accepted naming evidence and aggregation rules in ADR 004.

## Options considered

| Option | Benefit | Limitation | Proposed disposition |
| --- | --- | --- | --- |
| Derive all labels from Rust names | Minimal configuration | Source renames can change operational contracts | Retain only where no stability consumer exists |
| Explicit namespaced static strings | Reviewable meanings independent of source spelling | Requires a naming and change policy | Preferred validation baseline |
| Mandatory numeric identifiers | Compact explicit representation | Needs allocation policy; does not supply stability by itself | Defer without a consumer requirement |
| Global registry or generated schema | Central coordination | Adds ownership and maintenance before demonstrated need | Reject for the current scope |

*Table 1: Candidate naming strategies and their evidence requirements.*

## Proposed direction

Use explicit, application-chosen static strings as the first validation
candidate for transition and domain-outcome names. For example:

```plaintext
transition.name    = "trasic.conditional.select_else"
transition.outcome = "duplicate_else"
```

These examples are not reserved Statelet names. Statelet owns its accepted
field schema and any generic completion vocabulary eventually accepted from
RFC 0005. Applications own the meanings and naming policy of their states,
transitions, and domain outcomes. A display description may change without
changing a stable operational label; keep those roles separate when needed.

A source-level rename must not silently change a label that a consumer treats
as stable. A semantic change must not silently reuse an old label for a new
meaning. Review operational compatibility explicitly; document any necessary
rename, migration alias, or versioned schema change. Do not require every
application to adopt a global naming registry or every helper to expose a new
naming trait.

Static names and numeric codes remain representation choices. A consumer must
identify an operation or integration requirement before a stronger type or
numeric representation earns consideration. Neither enum position nor a Rust
discriminant is an implicit operational identity contract.

Source positions, resource paths, invocation IDs, and object IDs belong in
selected diagnostic context. They must not be interpolated into stable state
or outcome names. Metric use needs a finite, reviewed label vocabulary; hashing
or numbering an unbounded set does not make its cardinality bounded.

## Evidence and acceptance criteria

Extend the observation-integration evidence proposed by RFC 0004, using its
separate marker and schema. Do not add rows to ADR 004's accepted register in
this change. Any finding that affects `StateName` still requires a note through
ADR 004's existing instrument and its normal return-shape decision.

For each transition or outcome consumer, record the label, owner, current
representation, source-rename expectation, consumer operation, cardinality
policy, and evidence location. Record whether the requirement concerns stable
meaning, readable display, numeric representation, or none of these.

Validate a source-method rename and an outcome-variant rename while preserving
the declared operational names. Include a negative control that derives the
label from the renamed source item; the affected query or assertion must fail.
Include a genuine semantic rename and show that compatibility review detects
it rather than silently keeping an obsolete meaning.

Acceptance requires concrete evidence from the existing proving grounds and an
explicit decision about which labels need stability. Trasic may add depth but
cannot replace `wireframe` as the non-parser validation domain. If no consumer
needs stable transition or outcome names, leave the convention local and record
that result rather than manufacturing a numeric-ID requirement.

## Consequences

The proposed convention makes operational names reviewable independently of
Rust identifiers and gives tests a stable assertion vocabulary. It adds a
small naming-policy obligation for applications that elect to expose stable
labels. It does not make arbitrary names stable automatically, change
`StateName`'s return type, or justify `TransitionOutcome` publication.

## Migration plan

Roadmap task 6.1.6 reviews this decision. Task 6.3.2 runs rename and cardinality
controls, and 6.3.3 gathers consumer evidence. Task 6.3.4 decides adoption and
updates the technical design before any new naming contract becomes public.
Existing fields and ADR 004 evidence remain authoritative while this ADR is
proposed.

## Known risks and limitations

Explicit strings can drift without review and tests. Namespaces reduce
accidental collisions but do not prove global uniqueness. Stability tests must
compare meaning, not merely demand frozen spellings forever. A tracing span ID
is not a substitute for an application-owned operational label.

## Outstanding decisions

Which consumers require source-rename stability? Should an accepted derive
later support explicit label overrides? Does any real consumer need a numeric
representation rather than stable strings? These decisions remain evidence-led.
