# Architectural decision record (ADR) 003: Record the v0.1 exit register

## Status

Accepted, 2026-08-22. Statelet records three release scopes and the evidence
gates that select them; it does not yet choose one.

## Date

2026-08-22.

## Context and problem statement

Statelet must establish whether its transition-boundary toolkit earns a v0.1
release before it publishes a speculative public API. The technical design
records two relevant bets: B1 asks whether a real segment wants a shared
convention for handwritten state machines, while B2 asks whether
`#[transition]` adds enough value over `#[tracing::instrument]` and helpers.

The design already describes two off-ramps, but its statements are scattered.
This record makes the possible release scopes total, mutually exclusive and
bound to named roadmap gates. An exit is a release scope, not a test exit
criterion and not a decision that Statelet has already made.

## Decision drivers

- B1 has low-medium confidence and requires `mdtablefix` plus one second
  non-toy example to improve without framework adoption.
- B2 has low confidence and needs a head-to-head baseline comparison stating
  the macro's concrete added value.
- B6 asks whether the `mdtablefix` wedge generalizes to the second example.
- Evidence, rather than author preference, must select the final scope.

## Options considered

| Option   | Release scope                       | When it applies              |
| -------- | ----------------------------------- | ---------------------------- |
| E1       | Ship nothing                        | B1 is falsified              |
| E2       | Ship conventions only               | B1 holds and B2 is falsified |
| E3       | Ship the macro                      | B1 and B2 hold               |
| Rejected | Ship an unstable macro behind `cfg` | Bypasses the macro gate      |

*Table 1: Release-scope options considered for Statelet v0.1.*

The rejected option would allow a macro to ship before it had demonstrated
value over the baseline. It therefore defeats the validation discipline this
record is intended to preserve.

## Decision outcome / proposed direction

Statelet records E1, E2 and E3 as its complete set of v0.1 release scopes. The
register below maps every final B1/B2 verdict combination to one scope and
identifies the gate that decides it. The later roadmap gates choose the
outcome; this ADR does not choose one now.

## Exit register

The decision table is deliberately machine-checked by
`tests/v0_1_exit_register_contract.rs`.

<!-- exit-register:begin -->

| B1 verdict | B2 verdict | Exit                     | Gate | Reachable |
| ---------- | ---------- | ------------------------ | ---- | --------- |
| Falsified  | Falsified  | E1 ship nothing          | G2   | yes       |
| Falsified  | Held       | E1 ship nothing          | G2   | no        |
| Held       | Falsified  | E2 ship conventions only | G3   | yes       |
| Held       | Held       | E3 ship macro            | G3   | yes       |

<!-- exit-register:end -->

*Table 2: The v0.1 exit register. Each B1/B2 verdict combination selects one
release scope and gate.*

| Gate | Roadmap task | Decides                                                        |
| ---- | ------------ | -------------------------------------------------------------- |
| G1   | 2.2.3        | Whether the `mdtablefix` baseline is strong enough to continue |
| G2   | 3.1.3        | B1 across both validation examples                             |
| G3   | 4.3.1        | B2 from the head-to-head macro comparison                      |

*Table 3: Gates at which the relevant evidence is recorded.*

E1 is the off-ramp: Statelet is not published and its naming and tracing-field
conventions stay project-local. B1 is falsified when either validation example
fails to improve, because B1 requires both examples to improve. G1 may provide
an early off-ramp when the `mdtablefix` baseline is already weak; G2 records B1
finally.

E2 publishes the conventions/runtime crate with `StateName` and the documented
`transition.*` tracing-field contract, while deferring `statelet-macros`. E3
ships that same scope plus `statelet-macros`; it is a strict superset of E2,
not an unrelated product.

The `(Falsified, Held)` row is unreachable by construction. G2 selects E1 when
B1 is falsified, so G3 is never reached and B2 cannot be recorded as held. It
remains in the register to make the mapping total and to prevent a promising
macro from appearing to rescue a wedge that has already failed.

The split case, where one validation example improves and the other does not,
falsifies B1 and selects E1. This is B6 failing. It broadens the corresponding
conditions in `design.md` section 13.7 and `terms-of-reference.md` section 7.1
from both examples to either example.

## Evidence the register preserves

The register rests on the following load-bearing statements, quoted so the
contract test can detect a semantic rewrite in their source documents:

- B1 requires examples that "both improve without framework adoption".
- B2 requires a comparison that "states the concrete value added by the macro".
- The baseline result means "the macro crate does not ship in v0.1".
- A failed macro gate means "the project ships the conventions/runtime crate
  and defers" the macro crate.
- A failed conventions gate means "the project should ship nothing and keep the
  pattern local".

## Goals and non-goals

- Goals:
  - Make every v0.1 scope and trigger easy to review in one place.
  - Bind the triggers to the roadmap's evidence-producing gates.
  - Preserve a valid, explicit outcome in which Statelet ships nothing.
- Non-goals:
  - Choose a release scope before the validation evidence exists.
  - Add runtime code, a public API or a proc-macro crate.
  - Alter ADR 002's marker-only ownership boundary.

## Known risks and limitations

- The gates are managerial validation points, not a claim about a Rust language
  guarantee.
- A later rewrite to the design or roadmap can make this record stale. The
  contract test intentionally turns that drift into a test failure.
- The final B1/B2 verdicts remain unknown until the roadmap gates run.

## Outstanding decisions

- Whether validation selects E1, E2 or E3 remains open until G2 and G3.
- The technical design's deferred decisions remain governed by
  `docs/design.md` section 14.

## Architectural rationale

ADR 002 establishes that Statelet marks transition boundaries without owning
dispatch, events, storage, transition tables or graph safety. This ADR is its
delivery counterpart: it preserves that narrow boundary while making the
product's evidence-based off-ramp and macro gate explicit.
