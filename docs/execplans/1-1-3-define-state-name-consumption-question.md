# Define the `StateName` consumption question (roadmap 1.1.3)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances (exception triggers)`, `Risks`, `Progress`,
`Surprises & discoveries`, `Decision log`, `Outcomes & retrospective`,
`Conformance basis`, and `Verification plan` must be kept up to date as work
proceeds.

Status: DRAFT

## Purpose / big picture

Statelet intends to publish a trait whose whole public surface is one method:

```rust,ignore
pub trait StateName {
    fn state_name(&self) -> &'static str;
}
```

The technical design does not know whether that return type is right. It says
so plainly: "The `mdtablefix` baseline must scrutinize whether `&'static str`
is enough for the first slice. If instrumentation, metrics, or downstream
dashboards need a stable numeric discriminant or other low-cardinality
identifier, record that before publishing `StateName`."

That instruction is currently unexecutable. Nothing in the repository tells a
Phase 2 engineer what to record, what shape the record takes, or what rule
turns a record into a verdict. Roadmap task 3.2.1 is scheduled to "finalize the
`StateName` return shape" from "observed example consumption, not
anticipation", but no instrument exists to observe that consumption with.

After this change, three things are true that are not true today.

First, a Phase 2 engineer annotating `mdtablefix` can open
`docs/phase-2-validation-note-template.md`, copy it, and fill four named fields
whose admissible answers are enumerated. No field can be left blank, and no
field can be answered with a judgement call dressed as an observation.

Second, the rule that turns those four fields into a verdict on `&'static str`
is written down in `docs/adr-004-state-name-consumption-evidence.md` before any
evidence exists, so the rule cannot be reverse-engineered from a result
somebody has already grown fond of. The rule is deliberately capable of saying
"insufficient": a decision procedure that can only ever ratify the default is
not a decision procedure.

Third, both documents are machine-checked. Running `make test` parses the
decision record and the template, checks that the template instantiates exactly
the fields the record defines, checks that the verdict rule is total and obeys
its two exclusion clauses, checks that every quoted upstream clause still says
what is quoted, and checks that every named gate resolves to a live roadmap
task. A future edit that silently unpicks any of this fails the build with a
message naming the file and the repair.

Observable acceptance: from a clean checkout, `make test` passes and reports
the new integration test binary `state_name_consumption_contract` with its
scenarios green. Deleting the word `Insufficient` from the decision record's
register, or blanking one status cell in the template, makes `make test` fail
with a specific, actionable message rather than a generic assertion failure.

This task adds no runtime code. `src/` is untouched. That is not a shortcoming:
roadmap phase 1 exists precisely to settle contracts and kill gates before
feature work starts, so that later slices can falsify weak bets without leaving
speculative public API behind.

## Context and orientation

### What Statelet is

Statelet is an unreleased Rust crate. Its thesis is narrow: ordinary
handwritten Rust state machines are good, and what they lack is a shared
convention for *marking the boundary* where a transition happens, so that
tracing, diagnostics, and review all describe that boundary the same way.
Statelet does not own dispatch, events, storage, transition tables, or graph
safety; ADR 002 records that boundary.

The crate has two possible shapes. The "conventions" shape is a runtime crate
exposing the `StateName` trait and a documented set of `transition.*` tracing
field names, used with ordinary `#[tracing::instrument(fields(...))]`. The
"macro" shape adds a `statelet-macros` crate providing `#[statelet::transition]`.
ADR 003 records that the project may also ship nothing at all. Which of the
three happens is decided later, by evidence, at named roadmap gates.

A term used throughout this plan, defined here because it is easy to
misunderstand: a **state name** is, per `docs/context.md`, "a stable, low-cost
name for a state used in diagnostics, tracing fields, and generated
documentation. A state name is not required to be the same as `Debug` output."
It is a label, not an identity. Two distinct states must not share a name, but
the name carries no ordering, no numeric value, and no storage guarantee.

### At plan inception: current state of the repository

The crate is a stub. `src/lib.rs` is thirteen lines containing one `pub const
fn greet() -> &'static str` and a `TODO` marking it for deletion. There is no
`StateName` trait, no derive macro, no `statelet-macros` crate, and no
`mdtablefix` integration. Every grep for `state_name` in `src/` returns
nothing.

What the repository does have is a growing body of *documentation-as-code*: a
set of integration tests that treat design documents as the artefact under
test. `tests/v0_1_exit_register_contract.rs` and its private children parse
`docs/adr-003-v0-1-exit-register.md`, check the exit register is total, check
that a falsified bet B1 forces the off-ramp, check that quoted clauses still
resolve in `docs/design.md`, `docs/terms-of-reference.md`, and
`docs/context.md`, and check that named gates resolve to live roadmap tasks.
Siblings `tests/codegen_backend_contract.rs`, `tests/coverage_contract.rs`, and
`tests/dev_fast_contract.rs` do the same for build configuration.

This plan adds one more member of that family. Anyone who has not read
`tests/v0_1_exit_register_contract.rs` should read it before starting; it is
the model this work follows, and imitating it is cheaper than reinventing it.

Roadmap tasks 1.1.1 and 1.1.2 are complete (ADR 002 and ADR 003 respectively).
Task 1.1.3 is the next unticked item and is the subject of this plan. Its
declared dependency, 1.1.2, is satisfied.

### The documents this plan depends on

- `docs/design.md` — the technical design. Sections 6.1 (state naming), 6.2
  (why speculative public vocabulary is refused), 9 (the `transition.*` field
  contract), 11.1 (the bet register), 11.2 (baseline comparison), 12 (the
  `mdtablefix` validation spike), 13.6 and 13.7 (failure modes), and 14
  (deferred decisions) all bear on this task.
- `docs/terms-of-reference.md` — sections 7 (success criteria) and 9 (open
  questions, including "Does `statelet` improve `mdtablefix` enough to justify
  extraction?").
- `docs/context.md` — the glossary. The entries "State name", "Non-macro
  baseline", and "Transition instrumentation" are load-bearing here.
- `docs/roadmap.md` — task 1.1.3 itself, and the later tasks 2.2.1, 2.2.2,
  3.1.2, and 3.2.1 that consume this work.
- `docs/adr-001-proving-ground-candidates.md` — selects `wireframe` as the
  second proving ground, and carries the outstanding decision this plan
  operationalizes.
- `docs/adr-002-transition-boundary-scope.md` — the marker-only boundary, and
  the worked `mdtablefix` and `wireframe` sketches that name real state enums.
- `docs/adr-003-v0-1-exit-register.md` — the v0.1 release scopes and their
  gates.
- `docs/documentation-style-guide.md` — the governing standard for ADR
  structure, Markdown formatting, and en-GB-oxendict spelling.

### The verbatim evidence this plan is built on

These passages are quoted because the contract test will assert that each still
resolves in its source document. A rewrite that changes their meaning must
break the build, not pass silently.

From `docs/design.md` section 6.1:

```plaintext
The `mdtablefix` baseline must scrutinize whether `&'static str` is enough for
the first slice. If instrumentation, metrics, or downstream dashboards need a
stable numeric discriminant or other low-cardinality identifier, record that
before publishing `StateName`. The default remains `&'static str` until a real
example consumes something stronger.
```

From `docs/context.md`, glossary entry "State name":

```plaintext
A stable, low-cost name for a state used in diagnostics, tracing fields, and
generated documentation. A state name is not required to be the same as `Debug`
output.
```

From `docs/adr-001-proving-ground-candidates.md`, "Outstanding decisions":

```plaintext
Whether `StateName` needs only stable string labels or a stronger
low-cardinality identifier after `mdtablefix` and `wireframe`.
```

From `docs/design.md` section 6.2, the discipline that governs the default:

```plaintext
if the macro does not produce or consume the type, the enum becomes
documentation-only public API
```

From `docs/roadmap.md`, task 3.2.1, the gate this plan feeds:

```plaintext
the public trait shape is backed by observed example consumption, not
anticipation
```

From `docs/adr-002-transition-boundary-scope.md`, the `wireframe` sketch, which
names the one concrete cross-tool consumer of state labels that exists anywhere
in the documentation set:

```plaintext
`crates/wireframe-verification` retains its Stateright model and proof
responsibilities, while `StateName` can give production logs, tests, and the
model the same `Active`, `ShuttingDown`, and `Finished` labels.
```

### The gap this plan closes

Roadmap task 1.1.3 states the gap in two halves:

```plaintext
Decide what `mdtablefix` must consume to prove whether `&'static str` is
enough or whether a stable numeric identifier is needed.
Success: the Phase 2 validation note template has fields for state display
name, optional identifier need, metrics cardinality, and tracing use.
```

The first half is a decision; the second half is an instrument. This plan
delivers both, plus the machine-checked link between them.

The decision half is harder than it first looks, and the plan takes a definite
position on it. Three findings shape that position.

**Finding one: a numeric identifier cannot reduce metric cardinality.**
`StateName` is a total function from a state to a label. Any stable numeric
identifier would be a total function from the same state to a number. Swapping
one for the other is a relabelling of the same domain, so the number of
distinct values an observability backend sees is unchanged. Prometheus guidance
warns against labels holding "dimensions with high cardinality (many different
label values), such as user IDs, email addresses, or other unbounded sets of
values"; OpenTelemetry likewise treats low cardinality as a property of the
*value set*, not of the value's Rust type. It follows that an unbounded state
name set is a naming defect, and never an argument for a numeric identifier.
Any register that lets "cardinality is too high" select "add a numeric id" is
recording a non sequitur, and the contract test will refuse it.

**Finding two: no stable numeric identifier is available for free.**
`std::mem::discriminant` is the obvious candidate and does not qualify. Its
documented stability clause is that "the discriminant of an enum variant may
change if the enum definition changes", and reading a numeric value out of a
`Discriminant<T>` by `transmute` is undefined behaviour. A numeric value can be
obtained by an `as` cast only for fieldless enums, or by reading memory only
for enums that opt into `#[repr(u8)]` or similar. So a "stable numeric
identifier" necessarily means a *hand-assigned* value: an explicit
discriminant, or an attribute on the derive. That is new user-visible API
surface and new user obligation, which is exactly the speculative public API
that design section 6.2 refuses without a named consumer. This raises the
evidential bar rather than lowering it.

**Finding three: `&'static str` already satisfies every consumer the
documentation names.** `tracing` records a `&'static str` directly as a
`Value`. The `mdtablefix` state universe sketched in ADR 002 is
`ContinuationMode` (three variants) plus `BufferMode` (two variants): five
labels, statically bounded by the type. The one cross-tool consumer named
anywhere — `wireframe`'s Stateright model in `crates/wireframe-verification` —
needs *label equality* across production logs, tests, and the model, which a
string serves better than a number, because a number has to be decoded before a
human can read a counter-example.

Taken together, the honest prior is that `&'static str` is sufficient. The
purpose of this plan is therefore *not* to argue that case. It is to build an
instrument that could overturn it, and to write the overturning rule down
before the evidence arrives. An instrument that cannot return "insufficient" is
worthless, so the contract test asserts that the register contains at least one
row selecting `Insufficient`.

### Open questions for the approval gate

Three decisions in this plan exceed what roadmap task 1.1.3 literally asks for.
Each is separable: declining any one leaves the rest intact. They are raised
here rather than buried in `Decision log` because they should be settled before
implementation starts.

**Q1 — should a bet B7 be added to `docs/design.md` section 11.1?** Section
11.1 states its own rule: "Each bet must have an evidence-producing gate before
the affected API is published." The sufficiency of `&'static str` is exactly
such a bet, affecting exactly such an API, and it has no row. Adding
`B7 | &'static str is a sufficient state identity for v0.1 | Medium-high |
Two validation notes record a bounded name set and no named consumer requiring
a stronger identifier` closes that gap and gives the new gate table an upstream
anchor. The cost is an edit to a design table that another contract test reads.
That test's check is a presence check for rows B1 and B2, so adding B7 is safe,
but the edit is still upstream scope. Recommended: yes.

**Q2 — should the template be a separate file from the ADR?** ADR 003 embeds
its machine-checked register inline. Doing the same here would give one file
instead of two. The plan proposes two because their lifecycles differ: an ADR
is a frozen record, whereas the template is a blank form that Phase 2 tasks
copy and fill three or more times (2.2.1, 2.2.2, 3.1.2). Filling a form that
lives inside an accepted ADR would mean editing the ADR, which is wrong.
Roadmap task 1.2.3 will add a sibling benchmark note template, so a template
family in `docs/` is coherent rather than ad hoc. Recommended: two files.

**Q3 — should this plan add `rstest-bdd` and `proptest` as dev-dependencies?**
The repository currently has five dev-dependencies and no behavioural or
property tests. The governing instruction for this task asks for behavioural
coverage where applicable and property coverage where the change introduces an
invariant over a range of inputs. The plan judges both applicable and says why
in `Verification plan` under "Methods chosen and methods refused". The honest
counter-argument is that roadmap task 1.1.2 deliberately dropped both to keep
new dev-dependencies to two. Recommended: add both, because the workflow being
specified here is one a non-Rust reader must be able to review, and because the
Markdown parser is new hand-rolled code over adversarial input. If declined,
the plan still delivers every invariant through `rstest` alone; see D8.

### Files this plan reads or writes

Written (new):

- `docs/adr-004-state-name-consumption-evidence.md`
- `docs/phase-2-validation-note-template.md`
- `tests/state_name_consumption_contract.rs`
- `tests/state_name_consumption_contract/support.rs`
- `tests/state_name_consumption_contract/fixtures.rs`
- `tests/state_name_consumption_contract/regression_controls.rs`
- `tests/features/phase_2_validation_note.feature` (subject to Q3)
- `tests/phase_2_validation_note_bdd.rs` (subject to Q3)

Written (modified):

- `docs/design.md`, `docs/terms-of-reference.md`, `docs/context.md`,
  `docs/roadmap.md`, `docs/contents.md`, `docs/users-guide.md`,
  `docs/developers-guide.md`, `docs/repository-layout.md`
- `Cargo.toml`, `Cargo.lock`

Read only, never modified by this plan:

- `src/lib.rs` and everything else under `src/`
- `tests/v0_1_exit_register_contract.rs` and its children
- `Makefile`, `clippy.toml`, `rust-toolchain.toml`, `typos.toml`

### Documentation to read before starting

- `docs/design.md` sections 6.1, 6.2, 9, 11.1, 12, 13.6, 13.7, and 14.
- `docs/context.md` in full; it is short and every term used here is defined
  there.
- `docs/adr-003-v0-1-exit-register.md` — the structural model for ADR 004.
- `docs/documentation-style-guide.md` — ADR section order and template,
  Markdown rules, and en-GB-oxendict spelling. Note in particular: sentence
  case headings, a caption under every table, a language identifier on every
  fenced block, 80-column prose wrap, 120-column code wrap, and no first- or
  second-person pronouns outside `README.md`.
- `AGENTS.md` — the 400-line file limit, the commit-gate list, and the Rust
  conventions. The `[lints.clippy]` block in `Cargo.toml` is stricter than the
  prose: `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, and
  `shadow_unrelated` are denied, which shapes how the parser must be written.
- `docs/rust-testing-with-rstest-fixtures.md` — fixture and parameterized-case
  idiom.
- `docs/rstest-bdd-users-guide.md` — required only if Q3 is accepted. Note the
  `#[scenario(path = "...", name = "...")]` form and that feature files live
  under `tests/features/`.
- `docs/complexity-antipatterns-and-refactoring-strategies.md` — the
  conditional-complexity threshold that forced task 1.1.2 to split its parser
  into `is_structural_row`, `is_register_header`, and `is_register_divider`.
  The same threshold applies here.
- `docs/reliable-testing-in-rust-via-dependency-injection.md` and
  `docs/rust-doctest-dry-guide.md` — background; neither is load-bearing for
  this task, because the contract test reads documents through `include_str!`
  rather than through the filesystem, so it has no injectable dependency.

### Skills to load before starting

- `execplans` — to keep this document conformant as work proceeds.
- `rust-router` — the entry point for Rust work. It routes onward; do not load
  a language skill speculatively.
- `rust-unit-testing` — for the `rstest` table and assertion shapes, and for
  the rule that a boolean assertion should become a matcher or an exact
  equality.
- `arch-decision-records` — only as background. The repository's ADR template
  is `docs/documentation-style-guide.md`, not the skill's Y-Statement form, and
  the repository template wins.
- `proptest` — only if Q3 is accepted, for the parser round-trip strategy.
- `en-gb-oxendict-style` — for the prose in both new documents.
- `leta` — for navigation of the existing contract-test modules.

## Conformance basis

Upstream artefacts and their revisions at the time of writing:

- Terms of reference: `docs/terms-of-reference.md`, "Draft v0.2 after design
  review", last substantive revision 2026-08-22.
- Technical design: `docs/design.md`, "Draft v0.2 after design review", last
  substantive revision 2026-08-22.
- Decision records: `docs/adr-001-proving-ground-candidates.md`;
  `docs/adr-002-transition-boundary-scope.md` (Accepted, 2026-07-22);
  `docs/adr-003-v0-1-exit-register.md` (Accepted, 2026-08-22).
- Roadmap: `docs/roadmap.md` at commit `bad9a04`.
- Governing standard: `docs/documentation-style-guide.md`.
- External interfaces treated as axioms: `std::mem::discriminant` stability and
  `Discriminant<T>` opacity as documented by the Rust standard library;
  `tracing`'s acceptance of `&'static str` as a field `Value`; Prometheus and
  OpenTelemetry guidance that label and attribute cardinality is a property of
  the value set.

Traced items:

```plaintext
TDD-6.1-default-str   -> ROADMAP-1.1.3 -> EP-M1 -> ADR-004 §Sufficiency register -> tests::sufficiency_mapping_is_total
TDD-6.1-record-need   -> ROADMAP-1.1.3 -> EP-M1 -> ADR-004 §Field register       -> tests::field_register_is_complete
TDD-6.2-no-speculative-api -> ADR-004 R-DEFAULT -> EP-M1 -> tests::absent_consumer_never_selects_insufficient
TDD-9-transition-fields -> ADR-004 F4 tracing-use -> EP-M2 -> template field row 4
ADR-001-outstanding-2 -> ADR-004 §Decision outcome -> EP-M1 -> tests::quoted_passages_still_resolve
ROADMAP-2.2.1/2.2.2/3.1.2 -> ADR-004 gates S1, S2, S3 -> EP-M3 -> tests::gate_bindings_resolve
ROADMAP-3.2.1         -> ADR-004 gate S4 -> EP-M3 -> tests::gate_bindings_resolve
ROADMAP-1.1.3         -> EP-M1..M5 -> ADR-004 + template + roadmap checkbox ticked
TDD-11.1-B7           -> ADR-004 §Decision drivers -> EP-M4 -> design.md §11.1 row (subject to Q1)
```

This plan does not deviate from ADR 002 or ADR 003. ADR 002's non-goals
explicitly leave the `StateName` identifier question open; ADR 001's
outstanding decisions name it. This plan discharges the preparatory half of
that open question and leaves the substantive half to roadmap task 3.2.1, which
is where the roadmap puts it.

No upstream artefact defines a document type called a "validation note". The
roadmap uses the phrase seven times without defining it. This plan interprets
it as the record produced by a validation task and instantiated from the
template delivered here; see `Decision log` D1.

## Constraints

Hard invariants. If satisfying the objective requires violating one, stop and
escalate rather than working around it.

1. **No runtime code.** Nothing is added to or changed under `src/`. Roadmap
   phase 1 forbids feature work, and design section 6.2 forbids publishing
   vocabulary that no example consumes. A trait definition added "to make the
   template concrete" would violate both.
2. **No decision on the `StateName` return shape.** This plan defines the
   evidence and the rule. Roadmap task 3.2.1 applies them. An ADR that
   concludes `&'static str` is sufficient would pre-empt a gate that has no
   evidence yet, and would make the whole instrument decorative.
3. **No new runtime dependencies.** `[dependencies]` in `Cargo.toml` stays
   empty. Dev-dependencies may grow only as approved under Q3.
4. **No renumbering of roadmap tasks.** Tasks 2.2.1, 2.2.2, 3.1.2, and 3.2.1
   are gate targets checked by `INV-GATES`, and tasks 2.2.3, 3.1.3, and 4.3.1
   are gate targets checked by the existing exit-register contract. Renumbering
   breaks both.
5. **`tests/v0_1_exit_register_contract.rs` and its children are not
   modified.** The new contract is a sibling, not an extension. Sharing a
   parser between the two would couple ADR 003's fate to ADR 004's.
6. **Every source file stays under 400 lines**, per `AGENTS.md`. Task 1.1.2 hit
   this limit and split its support module; budget for the same.
7. **`make check-fmt`, `make lint`, and `make test` pass before every commit**,
   and `make markdownlint`, `make nixie`, `make audit`, and
   `make test-workflow-contracts` pass before delivery.
8. **British English, Oxford spelling** throughout both new documents, per
   `docs/documentation-style-guide.md`. `make markdownlint` enforces this
   through the pinned `typos` configuration and will fail on a lapse.

## Tolerances (exception triggers)

1. **Scope.** If delivery requires touching a file not listed under "Files this
   plan reads or writes", stop and escalate.
2. **Dependencies.** Q3 authorizes at most `rstest-bdd`, `rstest-bdd-macros`,
   and `proptest` as dev-dependencies. A fourth addition, or any runtime
   dependency, stops the work.
3. **Design amendment.** Q1 authorizes exactly one new row in `docs/design.md`
   section 11.1 plus one cross-reference sentence in section 6.1. Any further
   change to design prose that alters a requirement rather than adding a
   pointer stops the work and becomes a recorded deviation.
4. **Iterations.** If a contract test still fails after four attempts to make
   it green, stop; a parser that resists four repairs is the wrong parser.
5. **Size.** If `tests/state_name_consumption_contract/support.rs` exceeds 350
   lines before every invariant is implemented, stop and re-plan the module
   split rather than continuing towards the 400-line wall.
6. **Ambiguity.** If the register's four rows admit a second defensible
   verdict assignment, stop and present both with trade-offs rather than
   picking one.
7. **Gate time.** If `make lint` or `make test` takes more than twenty minutes,
   stop and report; the repository has no runtime code and should not.

## Risks

- Risk: the register's exclusion rules look like arbitrary house rules to a
  reviewer who has not followed the cardinality argument, and get "simplified"
  away in a later edit.
  Severity: high. Likelihood: medium.
  Mitigation: state the argument in the ADR's "Architectural rationale" in
  full, not by reference; bind each rule to a named invariant and a named
  negative control so that removing the rule fails the build with a message
  that restates the argument.

- Risk: the template's four fields are answerable only for enums, and Phase 2
  encounters a state represented as a `bool` (`ProcessBuffer` currently tracks
  table mode as `bool in_table`).
  Severity: medium. Likelihood: high — ADR 002 says this is the present state
  of `mdtablefix`.
  Mitigation: the `state-display-name` field's evidence cell requires the type
  path and the enumerated names; a `bool` has two states and is therefore
  answerable. ADR 002 already notes that `mdtablefix`'s own roadmap proposes
  promoting the boolean to an enum, so the field records the situation rather
  than being blocked by it.

- Risk: a new hand-rolled Markdown table parser repeats the defects task 1.1.2
  spent many commits repairing — indented headings, code fences mistaken for
  tables, divider rows parsed as data.
  Severity: medium. Likelihood: high.
  Mitigation: read `tests/v0_1_exit_register_contract/support.rs` first and
  reuse its shape (explicit header recognition, explicit divider recognition,
  structural rows filtered before parsing). Under Q3, add the `proptest`
  round-trip property that would have caught them.

- Risk: `make markdownlint` rejects the new tables for line length, and
  disabling MD013 around them becomes habitual.
  Severity: low. Likelihood: high.
  Mitigation: the repository already brackets wide tables with
  `<!-- markdownlint-disable MD013 -->` and `<!-- markdownlint-enable MD013 -->`
  in `docs/design.md`; follow that precedent exactly, and keep the disable
  region as tight as the table.

- Risk: `mdtablefix` (invoked by `make fmt` and `make check-fmt`) reflows the
  new tables and shifts the delimiter columns the parser depends on.
  Severity: medium. Likelihood: medium.
  Mitigation: run `make fmt` *before* writing the contract test's expected
  strings, so the parser is written against the formatter's output rather than
  against hand-typed alignment. The parser trims every cell, so column width is
  not load-bearing, but the test's negative controls contain literal table
  lines and those must match post-format.

- Risk: Q1 is accepted and the new B7 row disturbs
  `tests/v0_1_exit_register_contract.rs`.
  Severity: low. Likelihood: low.
  Mitigation: verified during planning — `has_table_bet` in
  `tests/v0_1_exit_register_contract/support.rs` performs a presence check for
  rows B1 and B2 using `lines().any(...)`, not an exact row-set comparison, so
  an added row is inert. Re-run `make test` immediately after the edit anyway.

- Risk: the plan's own analysis of the cardinality question is wrong, and a
  numeric identifier does help some consumer nobody has named.
  Severity: medium. Likelihood: low.
  Mitigation: this is precisely why the register retains an `Insufficient` row
  and why `INV-FALSIFIABLE` asserts that row exists. The plan's analysis
  determines the *prior*, not the verdict; the instrument can overturn it.

## Progress

- [ ] Stage A — orient and confirm the conformance basis (no changes).
- [ ] EP-M1 — ADR 004 exists and its registers are guarded.
- [ ] EP-M2 — the template exists and conforms to ADR 004's field register.
- [ ] EP-M3 — gate bindings and quoted anchors are guarded.
- [ ] EP-M4 — companion documentation is coherent.
- [ ] EP-M5 — delivery: full gates, review, roadmap ticked.

Timestamps are added as each item completes.

## Surprises & discoveries

Recorded during implementation. At plan inception, three findings from the
planning phase are worth carrying forward as pre-recorded discoveries, because
each was not obvious from the repository alone:

- Observation: `std::mem::discriminant` cannot supply the "stable numeric
  discriminant" design section 6.1 speculates about.
  Evidence: the standard library documents that "the discriminant of an enum
  variant may change if the enum definition changes", and that transmuting
  `Discriminant<T>` to a primitive is undefined behaviour; a numeric value is
  reachable only by an `as` cast on a fieldless enum or through an explicit
  `#[repr(u8)]`-style representation.
  Impact: any numeric identifier must be hand-assigned, which makes it new
  user-visible API and raises the evidential bar. ADR 004 records this under
  "Known risks and limitations" so task 3.2.1 does not rediscover it.

- Observation: a numeric identifier cannot reduce observability cardinality.
  Evidence: `StateName` and any identifier are both total functions on the same
  state domain, so the image size is identical; Prometheus and OpenTelemetry
  both frame cardinality as a property of the value set.
  Impact: this is the justification for rule R-CARDINALITY, which forbids an
  unbounded name set from selecting `Insufficient`.

- Observation: the only concrete cross-tool consumer of state labels named
  anywhere in the documentation is `wireframe`'s Stateright model, and its
  requirement is label *equality*, which favours a string.
  Evidence: `docs/adr-002-transition-boundary-scope.md`, `wireframe` section.
  Impact: strengthens the default and gives the `identifier-need` field a
  worked example of a consumer that does *not* establish a need.

## Decision log

- D1: Interpret the roadmap's undefined phrase "validation note" as the record
  a validation task produces, instantiated from the template delivered here.
  Rationale: the roadmap uses the phrase seven times across tasks 2.2.1, 2.2.2,
  3.1.2, and 4.2.1 without defining it, and task 1.1.3's success criterion
  presupposes a template for it. No competing definition exists.
  Date/Author: 2026-09-18, planning agent.

- D2: Deliver two documents — a decision record and a separate blank template —
  rather than one combined ADR.
  Rationale: their lifecycles differ. An accepted ADR is frozen; the template
  is copied and filled at least three times. See Q2.
  Date/Author: 2026-09-18, planning agent.

- D3: Make the sufficiency register a total mapping over two binary axes
  (named consumer present or absent; name set bounded or unbounded) rather than
  a free-text verdict.
  Rationale: the repository has a working precedent in ADR 003 for a total,
  machine-checked decision table, and a four-row domain is small enough for
  exhaustive verification to *be* the proof.
  Date/Author: 2026-09-18, planning agent.

- D4: Reframe "metrics cardinality" from "what a metrics backend observed" to
  "the bound on the set of distinct names the annotated code can emit".
  Rationale: `mdtablefix` installs no metrics recorder, so the first reading is
  unanswerable in Phase 2 and the field would be filled with "not exercised"
  every time, which is the same as not having the field. The second reading is
  determinable from the state type, answers the question task 3.2.1 actually
  needs, and still detects the dangerous case of a name synthesized from data.
  Date/Author: 2026-09-18, planning agent.

- D5: Give each field a closed status vocabulary with no blank and no free-text
  status, and require a separate evidence cell.
  Rationale: the failure mode this instrument exists to prevent is a judgement
  recorded as an observation. Separating the enumerated status from the
  supporting evidence makes the judgement checkable and the evidence auditable.
  Date/Author: 2026-09-18, planning agent.

- D6: Assert that the register contains at least one row selecting
  `Insufficient` (`INV-FALSIFIABLE`).
  Rationale: the plan's own analysis concludes `&'static str` is probably
  sufficient. That conclusion makes it tempting, later, to simplify the
  register into an unconditional ratification. A decision procedure that cannot
  return the inconvenient answer is not one.
  Date/Author: 2026-09-18, planning agent.

- D7: Do not use `insta` snapshots.
  Rationale: the multivariant output here is the set of repair messages. Task
  1.1.2 pinned those with exact `assert_eq!` comparisons against literal
  strings, which gives the same protection, reads better in a diff, adds no
  dependency, and — unlike an accepted snapshot — cannot be re-blessed without
  the reviewer seeing the new text.
  Date/Author: 2026-09-18, planning agent.

- D8: Do not use `kani` or `verus`.
  Rationale: `kani` is for bounded exhaustive exploration of arithmetic,
  memory, or transition safety; the only bounded domain here has four elements
  and is already exhaustively enumerated by an `rstest` table, so `kani` would
  add tooling without adding coverage. `verus` is for lemmas the implementation
  depends on. The load-bearing proposition behind rule R-CARDINALITY — that a
  relabelling of a domain preserves the size of its image — is a claim about
  observability backends and about a function that does not exist in this
  repository. Encoding it in Verus would prove that an injective function has
  an image the size of its domain, which is a restatement of injectivity and
  precisely the vacuous proof the ExecPlan standard forbids. The argument is
  therefore recorded as prose in ADR 004's rationale, where it can be reviewed,
  and enforced as a table constraint, where it can be violated and caught.
  Date/Author: 2026-09-18, planning agent.

- D9: Use `rstest-bdd` for the note-filling workflow and `proptest` for the
  parser round-trip, subject to Q3.
  Rationale: the deliverable's user is a Phase 2 engineer filling a form and
  running `make test`; that accept-or-reject loop is an externally observable
  workflow, and expressing it in Gherkin makes it reviewable by someone who
  will fill the form but not read the parser. The parser is new hand-rolled
  code over adversarial input, which is the textbook case for a round-trip
  property. If Q3 is declined, every invariant below is still discharged by
  `rstest`; only the review ergonomics and the parser's input coverage are
  lost, and `INV-PARSE` falls back to its handwritten controls alone.
  Date/Author: 2026-09-18, planning agent.

- D10: Number the new ADR 004 and name it
  `adr-004-state-name-consumption-evidence.md`.
  Rationale: 001, 002, and 003 are taken; the style guide mandates
  `adr-NNN-short-description.md`. The name says "evidence" rather than
  "decision" because the ADR decides what evidence counts, not what the answer
  is.
  Date/Author: 2026-09-18, planning agent.

## Outcomes & retrospective

To be completed at EP-M5. Before setting this plan to `COMPLETE`, reconcile
every implementation discovery against the artefacts named in
`Conformance basis`: amend `docs/design.md` if a discovery falsifies section
6.1's premise, amend `docs/adr-001-proving-ground-candidates.md`'s outstanding
decision if this work resolves it, and record a purely mechanical difference in
`Decision log`.

## Verification plan

This change adds no runtime behaviour, so there is nothing to verify about
program execution. It introduces eight non-trivial propositions about two
*documents* and the relationship between them, and every one is checkable.

**Design for falsifiability.** Both documents are embedded at compile time with
`include_str!`, so the tests have no filesystem dependency and no injectable
collaborator. Parsing is separated from policy: `parse_field_register`,
`parse_sufficiency_register`, and `parse_note_register` are pure functions from
`&str` to `Result<Vec<Row>, ParseError>` that perform syntax only — locate the
block between two HTML-comment delimiters, split rows on `|`, trim cells, and
map cell text to closed token types. They apply no policy. Each obligation
below is a separate pure predicate over the parsed rows, so a failure is
attributable to one obligation rather than swallowed by a parse error. Purity
means every negative control is a string literal.

`ParseError` distinguishes `MissingDelimiters`, `MalformedRow { line }`,
`UnknownToken { column, found }`, and `EmptyRegister`. Reporting a formatting
break as a missing register rather than as a content failure is deliberate:
mislabelling a formatting problem as missing content is the silent-rot failure
this suite exists to prevent.

**Axioms.** The following are assumed and not verified, because they are
third-party or external:

- `std::mem::discriminant`'s documented stability and opacity clauses.
- `tracing`'s acceptance of `&'static str` as a field `Value`.
- Prometheus and OpenTelemetry guidance that cardinality is a property of the
  value set rather than of the representing type.
- `markdownlint-cli2` and `mdtablefix` produce stable table formatting for a
  given input, so a formatted document round-trips through `make check-fmt`.

None of these is verified here. Where repository-owned logic depends on one —
specifically, the parser's assumption about `mdtablefix`'s output shape — the
dependency is exercised against the real formatter by running `make fmt` before
the expected strings are written, and by `make check-fmt` in every gate run.

### INV-FIELDS — the template instantiates exactly the record's fields

- **Obligation**: the ordered list of field identifiers in
  `docs/phase-2-validation-note-template.md`'s note register equals the ordered
  list in `docs/adr-004-state-name-consumption-evidence.md`'s field register.
  No extra field, no missing field, no reordering.
- **Method**: exact equality assertion over two parsed vectors, with
  `pretty_assertions::assert_eq!` so a divergence prints as a readable diff.
- **Rationale**: this is a schema-versus-instance conformance edge. It is the
  single most likely thing to rot, because the two files will be edited months
  apart by different people, and it is trivially checkable.
- **Domain**: the two live documents.
- **Artefact**: `tests/state_name_consumption_contract.rs`, test
  `template_instantiates_the_field_register`.
- **Evidence**: `make test`.
- **Non-vacuity**: four controls. (a) A template with a field removed must fail
  naming the missing identifier. (b) A template with an extra field must fail
  naming the extra. (c) A template with the four fields reordered must fail,
  because the note's reading order carries meaning: the two deciding fields
  must be read after the field that establishes admissibility. (d) An empty
  template register must yield `EmptyRegister`, not a vacuously equal pair of
  empty vectors — this is the classic vacuity hole, and without control (d) a
  parser stubbed to return `Ok(vec![])` would pass.

### INV-BLANK — the blank form pre-judges nothing

- **Obligation**: in the template, every `Status` cell and every `Evidence`
  cell holds the literal token `TBD`.
- **Method**: parameterized test with one `#[case]` per field.
- **Rationale**: a template shipped with a plausible default status would be
  filled by copying, not by observing. This obligation is what makes the
  instrument an instrument rather than a suggestion.
- **Domain**: four rows, two cells each.
- **Artefact**: test `template_ships_unfilled`.
- **Evidence**: `make test`.
- **Non-vacuity**: a control template with `Bounded` pre-filled in the
  `metrics-cardinality` status cell must fail with
  `docs/phase-2-validation-note-template.md: metrics-cardinality status is
  "Bounded". Repair: the blank template must hold TBD in every status and
  evidence cell.` A second control with an empty cell must fail the same way,
  proving that "blank" is not accepted as a synonym for `TBD`.

### INV-TOTAL — the sufficiency register is total and unambiguous

- **Obligation**: for every combination of the two deciding axes — named
  consumer in `{Present, Absent}`, name set in `{Bounded, Unbounded}` — the
  register names exactly one verdict. Four combinations, no gaps, no
  duplicates.
- **Method**: exhaustive parameterized test, one `#[case]` per combination,
  over the parsed rows.
- **Rationale**: the domain has four elements, so exhaustive enumeration is the
  proof. The obligation's real content is not "the table has four rows" but
  "the parser faithfully reflects the document", which is where the risk lies
  and which the controls target.
- **Domain**: `{Present, Absent} × {Bounded, Unbounded}`.
- **Artefact**: test `sufficiency_mapping_is_total`.
- **Evidence**: `make test`.
- **Non-vacuity**: six controls, each asserting a *specific* message rather than
  `is_err()`. (a) The empty string yields `MissingDelimiters`. (b) Delimiters
  with no rows yield `EmptyRegister`. (c) A register missing the
  `Present/Bounded` row reports that combination by name before any generic
  row-count check. (d) A duplicated combination fails as ambiguous. (e) A
  spurious fifth combination retains the cardinality error. (f) An unrecognized
  axis token yields `UnknownToken` naming the column. The live register is the
  accepting witness. A stub `parse_sufficiency_register` returning `Ok(vec![])`
  fails controls (a) through (f).

### INV-DEFAULT — an unnamed consumer never overturns the default

- **Obligation**: no row whose named-consumer axis is `Absent` selects the
  verdict `Insufficient`.
- **Method**: predicate over the parsed rows, plus the handwritten cases in
  `INV-CASES`.
- **Rationale**: this encodes design section 6.1's "The default remains
  `&'static str` until a real example consumes something stronger" and section
  6.2's refusal of documentation-only public API. Without it, a Phase 2 note
  could record "an identifier would be tidier" and that would be enough to
  change a published trait. The rule is the difference between evidence and
  preference.
- **Domain**: the two rows where the consumer axis is `Absent`.
- **Artefact**: test `absent_consumer_never_selects_insufficient`.
- **Evidence**: `make test`.
- **Non-vacuity**: a control register whose `Absent/Bounded` row selects
  `Insufficient` must fail with
  `docs/adr-004-state-name-consumption-evidence.md: Absent/Bounded selects
  Insufficient. Repair: only a named consumer may overturn the &'static str
  default.` The rule is violable, and the violation is exactly the mistake a
  well-meaning reviewer would make.

### INV-CARDINALITY — cardinality pressure is never identifier evidence

- **Obligation**: no row whose name-set axis is `Unbounded` selects the verdict
  `Insufficient`.
- **Method**: predicate over the parsed rows, plus `INV-CASES`.
- **Rationale**: this is the substantive claim of the whole record, and the one
  a careless reader gets backwards. `StateName` and any numeric identifier are
  both total functions on the same state domain, so substituting one for the
  other cannot change how many distinct label values a backend sees. An
  unbounded name set therefore diagnoses a naming defect — a name synthesized
  from data, or a leaked `String` — and repairing the names is the response. A
  register that let `Unbounded` select `Insufficient` would encode a non
  sequitur into the project's decision procedure.
- **Domain**: the two rows where the name-set axis is `Unbounded`.
- **Artefact**: test `unbounded_names_never_select_insufficient`.
- **Evidence**: `make test`.
- **Non-vacuity**: a control register whose `Present/Unbounded` row selects
  `Insufficient` must fail with
  `docs/adr-004-state-name-consumption-evidence.md: Present/Unbounded selects
  Insufficient. Repair: an unbounded name set is a naming defect; repair the
  names before recording an identifier need.` Note that this control uses
  `Present`, not `Absent`, so it cannot pass by accident through `INV-DEFAULT`;
  the two rules are independently violable and independently detected.

### INV-FALSIFIABLE — the register can return the inconvenient answer

- **Obligation**: at least one row selects the verdict `Insufficient`.
- **Method**: existence assertion over the parsed rows.
- **Rationale**: `INV-DEFAULT` and `INV-CARDINALITY` are exclusion rules. Taken
  alone they are satisfied perfectly by a register in which every row selects
  `Sufficient` — that is, by an instrument that cannot fail the default. Since
  the plan's own analysis expects the default to survive, the pressure to
  arrive at exactly that degenerate register is real. This obligation is the
  guard against it, and it is the reason the suite is not circular.
- **Domain**: all four rows.
- **Artefact**: test `register_can_select_insufficient`.
- **Evidence**: `make test`.
- **Non-vacuity**: a control register with `Present/Bounded` changed from
  `Insufficient` to `Sufficient` — which violates no other rule in this suite —
  must fail with
  `docs/adr-004-state-name-consumption-evidence.md: no row selects
  Insufficient. Repair: a register that cannot overturn the default is not a
  decision procedure.` This control is the proof that the three register rules
  are not jointly vacuous.

### INV-AXES — the field vocabulary and the register vocabulary agree

- **Obligation**: the admissible statuses that the field register assigns to
  `identifier-need` map onto the register's named-consumer axis tokens
  bijectively, and likewise for `metrics-cardinality` and the name-set axis.
- **Method**: parameterized test over the two axis pairs.
- **Rationale**: the two tables live in the same file but are edited
  independently. Renaming a status in one without the other produces a note
  that is correctly filled and cannot be resolved — a failure that is silent
  under every other invariant here.
- **Domain**: two axes, two tokens each.
- **Artefact**: test `axis_vocabularies_agree`.
- **Evidence**: `make test`.
- **Non-vacuity**: a control ADR that renames the `identifier-need` status
  `Named consumer` to `Consumer named` while leaving the register's `Present`
  mapping untouched must fail naming both cells. A second control that drops
  one status must fail as a cardinality mismatch, not as a missing key.

### INV-ANCHORS — every quoted upstream passage still says what is quoted

- **Obligation**: each load-bearing clause quoted in ADR 004's "Evidence the
  record preserves" section still occurs verbatim in its named source:
  `docs/design.md` sections 6.1 and 6.2, `docs/context.md`'s "State name"
  entry, `docs/adr-001-proving-ground-candidates.md`'s outstanding decisions,
  and `docs/roadmap.md` task 3.2.1.
- **Method**: substring resolution of each clause against the embedded source,
  scoped to the named section so that a clause that migrated elsewhere is
  reported as moved rather than silently accepted.
- **Rationale**: the record's authority comes from those clauses. A design
  revision that reverses one while leaving the ADR standing produces a decision
  procedure resting on a premise its own project has abandoned.
- **Domain**: five clauses across four documents.
- **Artefact**: test `quoted_passages_still_resolve`.
- **Evidence**: `make test`.
- **Non-vacuity**: three controls. (a) A `design.md` in which "The default
  remains `&'static str`" is rewritten to "The default is a numeric identifier"
  must fail naming the clause and the file. (b) A `design.md` in which the
  clause is moved out of section 6.1 into section 14 must fail as relocated,
  proving the check is section-scoped rather than whole-file. (c) An ADR
  quoting a clause that appears in no source must fail, proving the check is
  not satisfied by the ADR quoting itself.

### INV-GATES — every named gate resolves to a live roadmap task

- **Obligation**: each of the four gates S1 to S4 names a roadmap task that
  exists, is still unticked, and whose title text matches the gate's stated
  purpose. S1 binds 2.2.1, S2 binds 2.2.2, S3 binds 3.1.2, and S4 binds 3.2.1.
- **Method**: parameterized test, one `#[case]` per gate, against the embedded
  roadmap.
- **Rationale**: a gate pointing at a renumbered or already-completed task is
  an instrument with no consumer. Task 1.1.2 learned this the hard way, having
  bound a gate to the wrong task in its first draft.
- **Domain**: four gates.
- **Artefact**: test `gate_bindings_resolve`.
- **Evidence**: `make test`.
- **Non-vacuity**: four controls. (a) A roadmap with task 3.2.1 renumbered must
  fail naming the gate. (b) A roadmap with 2.2.2 already ticked must fail,
  because a gate cannot be satisfied by a task that ran before the instrument
  existed. (c) A gate naming a task prefix such as `3.2` rather than a full
  task number must be rejected, so that a prefix cannot match several tasks.
  (d) A gate naming a task whose title does not mention the gate's subject must
  fail, proving the check is not satisfied by any number of the right shape.

### INV-CASES — handwritten expectations agree with the live documents

- **Obligation**: the handwritten register in
  `tests/state_name_consumption_contract/fixtures.rs` parses to the same rows as
  the live ADR, and the handwritten field list matches the live field register.
- **Method**: equality assertion between fixture-derived and document-derived
  values.
- **Rationale**: every negative control is a mutation of the fixture. If the
  fixture drifts from the document, the controls are testing a register the
  project no longer has, and the whole suite becomes decorative while staying
  green.
- **Domain**: the fixture and the two live registers.
- **Artefact**: test `fixture_matches_the_live_record`.
- **Evidence**: `make test`.
- **Non-vacuity**: this obligation is itself the non-vacuity guard for the six
  preceding ones. Its own control is a fixture with one cell altered, which
  must fail with a diff naming the cell.

### INV-PARSE — the parser round-trips every well-formed register

- **Obligation**: for any vector of rows drawn from the closed token
  vocabularies, rendering those rows as a delimited Markdown table and parsing
  the result returns the original rows.
- **Method**: `proptest` property over generated row vectors, subject to Q3.
- **Rationale**: the four handwritten combinations exercise the register the
  project has. They do not exercise cell padding, an empty evidence cell, a
  cell containing an escaped pipe, a row indented by two spaces, or a table
  preceded by a fenced code block containing table-like text. Task 1.1.2's
  commit history shows each of those classes costing a repair commit. A
  round-trip property covers the class rather than the instance.
- **Domain**: vectors of one to eight rows over the closed axis and verdict
  vocabularies, with evidence cells drawn from a regex excluding the pipe
  character; plus a shrink-friendly wrapper that prepends optional indentation
  and a decoy fenced block.
- **Artefact**: `tests/state_name_consumption_contract/regression_controls.rs`,
  property `parse_render_round_trip`, with the regression seed file committed.
- **Evidence**: `make test`. A discovered counter-example is written to
  `proptest-regressions/` and must be committed with its fix.
- **Non-vacuity**: the property records generator classification with
  `proptest::prop_assert!` guarded counters, and the test asserts that
  generated cases include at least one indented table and at least one decoy
  fence across the run; a generator that produced only unindented,
  fence-free tables would be reported as a failure rather than passing
  quietly. The seeded fault is a deliberate one-character mutation of the
  renderer's delimiter emission, which the property must reject.
- **If Q3 is declined**: this obligation degrades to five additional
  handwritten controls covering indentation, padding, an empty evidence cell,
  a decoy fence, and a trailing-pipe-free row. The residual gap — untested
  cell content outside those five shapes — is then explicit rather than
  assumed away, and is recorded here as accepted.

### Behavioural specification

Subject to Q3. The workflow under specification is the one a Phase 2 engineer
performs: copy the template, fill it, run the gate, and either proceed or read
the repair message. `tests/features/phase_2_validation_note.feature`:

```gherkin
Feature: Recording StateName consumption evidence in Phase 2

  Background:
    Given the Phase 2 validation note template

  Scenario: A complete note with no identifier pressure keeps the default
    When the engineer records a bounded name set and no named consumer
    Then the note resolves to the verdict "Sufficient"
    And the note advises keeping the &'static str return type

  Scenario: A named consumer with a bounded name set overturns the default
    When the engineer records a bounded name set and names a consumer
    Then the note resolves to the verdict "Insufficient"
    And the note advises recording the consumer for roadmap task 3.2.1

  Scenario: Cardinality pressure alone is treated as a naming defect
    When the engineer records an unbounded name set and names a consumer
    Then the note resolves to the verdict "Naming defect"
    And the note advises repairing the state names before re-recording

  Scenario: A note with an unfilled field cannot be resolved
    When the engineer leaves the tracing use field unfilled
    Then the note is rejected
    And the rejection names the tracing-use field and its admissible statuses
```

These scenarios drive `INV-TOTAL`, `INV-DEFAULT`, `INV-CARDINALITY`, and
`INV-BLANK` through the workflow rather than through the parser, so they add
review value rather than coverage. Step definitions live in
`tests/phase_2_validation_note_bdd.rs` and call the same pure predicates the
`rstest` suite calls; no logic is duplicated. If Q3 is declined, this section
and its two files are dropped, and the workflow is documented in prose in
`docs/developers-guide.md` instead.

### Methods chosen and methods refused

Chosen: exhaustive parameterized `rstest` tables for the four-element register
domain and the four gates; exact-string equality with `pretty_assertions` for
diff-readable structural comparisons; `googletest` matchers where an assertion
is about shape rather than exact value; a `proptest` round-trip for the parser;
`rstest-bdd` scenarios for the fill-and-gate workflow.

Refused, with reasons: `insta` (D7 — the multivariant output is the repair
message set, and exact literals protect it better than a re-blessable
snapshot); `kani` (D8 — the only bounded domain has four elements and is
already exhausted); `verus` (D8 — the load-bearing proposition is about
external systems and a function this repository does not contain; encoding it
would restate injectivity, which the standard forbids as vacuous);
`cargo-mutants` (the suite is new, so its mutation score would be measured
against tests written the same day; the project's separate mutation-testing
workflow covers the repository on its own cadence); end-to-end tests (there is
no runtime, no binary, and no externally observable workflow beyond `make
test`, which is itself the acceptance command).

## Plan of work

### Stage A — orient and confirm (no changes)

Read `tests/v0_1_exit_register_contract.rs` and its four children end to end.
Confirm `docs/roadmap.md` task 1.1.3 is still unticked and that tasks 2.2.1,
2.2.2, 3.1.2, and 3.2.1 still carry those numbers. Confirm
`docs/design.md` section 6.1 still contains the quoted clauses. Confirm
`git branch --show-current` reports
`1-1-3-define-state-name-consumption-question`. Confirm `make test` is green
before any edit, so a later failure is attributable.

Stage A ends with no diff. If any confirmation fails, stop: the conformance
basis has moved and the plan needs revision before code.

### Stage B — red: the contract test before the documents (EP-M1, EP-M2)

Write `tests/state_name_consumption_contract.rs` and its support children in
full, against documents that do not yet exist. Add
`docs/adr-004-state-name-consumption-evidence.md` and
`docs/phase-2-validation-note-template.md` containing their prose and their
delimiter comments but *not* their register tables. Run `make test` and observe
the red state: `MissingDelimiters` for both registers, reported per document.

This is the honest red. It fails for the intended reason — the registers are
absent — rather than because the parser is unwritten. Record the transcript.

Do not use an expected-failure marker. `AGENTS.md` requires every commit to
pass the gates, so the red state is observed and recorded within a working
session and is not committed. The first commit is the green one, and it carries
the red transcript in its body.

### Stage C — green: insert the registers (EP-M1, EP-M2, EP-M3)

Insert the field register and the sufficiency register into ADR 004, and the
note register into the template. Run `make fmt` first so `mdtablefix` sets the
column widths, then copy the formatted table lines into
`tests/state_name_consumption_contract/fixtures.rs` so the fixture and the
document agree byte for byte. Run `make test` and observe green. Add the gate
table and the quoted-evidence section, and observe `INV-GATES` and
`INV-ANCHORS` go from red to green in turn.

Commit at each green point. Each commit is a coherent plateau.

### Stage D — companion sync, behaviour, and delivery (EP-M4, EP-M5)

Apply the documentation sync map. Add the `proptest` property and the
`rstest-bdd` scenarios if Q3 is accepted. Run every gate, obtain review, tick
the roadmap, and set this plan to `COMPLETE`.

## Milestones and plateaus

### EP-M1 — the decision record exists and its registers are guarded

- **Identifier and outcome**: `docs/adr-004-state-name-consumption-evidence.md`
  exists, carries both registers, and `INV-TOTAL`, `INV-DEFAULT`,
  `INV-CARDINALITY`, `INV-FALSIFIABLE`, `INV-AXES`, and `INV-CASES` are green.
- **Requirements and gaps**: discharges `TDD-6.1-default-str`,
  `TDD-6.2-no-speculative-api`, and the decision half of `ROADMAP-1.1.3`.
- **Acceptance evidence**: `make test` passes; the six named tests appear in
  the `state_name_consumption_contract` binary; every negative control asserts
  a specific message.
- **Conformance check**: no runtime code added; no verdict on the `StateName`
  return shape pronounced; ADR 002's boundary untouched; no roadmap
  renumbering; trace links current.
- **Recovery**: the ADR is additive. Deleting the file and the test restores
  the prior state with no other change.
- **Remaining gaps**: the template does not exist yet, so the record defines a
  schema with no instance.
- **Compatibility decision**: none. Nothing is released, nothing is consumed
  externally, and `src/` is a stub.

### EP-M2 — the template exists and conforms

- **Identifier and outcome**: `docs/phase-2-validation-note-template.md` exists
  and `INV-FIELDS` and `INV-BLANK` are green.
- **Requirements and gaps**: discharges the instrument half of
  `ROADMAP-1.1.3`; this is the roadmap's literal success criterion.
- **Acceptance evidence**: `make test` passes; a manual check that copying the
  template and filling the four fields produces a note whose verdict is
  derivable by reading ADR 004's register and nothing else.
- **Conformance check**: template fields match the record exactly; all cells
  hold `TBD`; no verdict pre-filled.
- **Recovery**: additive; delete both new documents and the test.
- **Remaining gaps**: gates and anchors are not yet bound.
- **Compatibility decision**: none.

### EP-M3 — gates and anchors are guarded

- **Identifier and outcome**: ADR 004 carries its gate table and its quoted
  evidence section; `INV-GATES` and `INV-ANCHORS` are green.
- **Requirements and gaps**: binds `ROADMAP-2.2.1`, `ROADMAP-2.2.2`,
  `ROADMAP-3.1.2`, and `ROADMAP-3.2.1` to the instrument.
- **Acceptance evidence**: `make test` passes; each of the eight controls
  fails for its own reason when applied.
- **Conformance check**: gate bindings match live task numbers and titles;
  quoted clauses resolve in their named sections.
- **Recovery**: the gate table and evidence section are two contiguous blocks;
  removing them and their tests reverts to EP-M2.
- **Remaining gaps**: companion documentation still does not mention either new
  document.
- **Compatibility decision**: none.

### EP-M4 — documentation is coherent

- **Identifier and outcome**: every document in the sync map references the new
  artefacts; `make markdownlint`, `make nixie`, and `make check-fmt` pass.
- **Requirements and gaps**: discharges `AGENTS.md`'s documentation-maintenance
  obligation and, subject to Q1, adds bet B7.
- **Acceptance evidence**: `docs/contents.md` lists both new documents; a
  reader arriving at `docs/design.md` section 6.1 finds the pointer;
  `make test` still passes, including the pre-existing exit-register contract.
- **Conformance check**: no requirement altered, only pointers added, except
  the Q1-authorized B7 row; `docs/design.md` and
  `docs/terms-of-reference.md` "Last substantive revision" updated.
- **Recovery**: each sync edit is a small, independent diff; revert
  individually.
- **Remaining gaps**: behavioural and property coverage, if Q3 accepted.
- **Compatibility decision**: none.

### EP-M5 — delivery

- **Identifier and outcome**: property and behavioural coverage present per Q3;
  every gate green; review clean; roadmap task 1.1.3 ticked; this plan
  `COMPLETE`.
- **Requirements and gaps**: `ROADMAP-1.1.3` fully discharged.
- **Acceptance evidence**: transcripts for `make check-fmt`, `make lint`,
  `make test`, `make markdownlint`, `make nixie`, `make audit`, and
  `make test-workflow-contracts` recorded in `Artefacts and notes`; a
  zero-finding independent review.
- **Conformance check**: the full checklist from
  `Traceability and architecture conformance`, plus reconciliation of every
  entry in `Surprises & discoveries` against the conformance basis.
- **Recovery**: the branch is revertible as a unit; nothing is published.
- **Remaining gaps**: none for 1.1.3. The substantive verdict on `StateName`
  belongs to task 3.2.1 and is deliberately left open.
- **Compatibility decision**: none.

## Concrete steps

All commands run from the repository root, which is the worktree at
`/home/leynos/.lody/repos/github---leynos---statelet/worktrees/99bdf268-61e5-4cd8-bf98-c238f1c0ee37`.

### Step 1 — confirm the starting state

```bash
git branch --show-current
git status --short
make test 2>&1 | tee /tmp/test-statelet-$(git branch --show-current).out
```

Expected: the branch is `1-1-3-define-state-name-consumption-question`, the
tree is clean, and the test run ends with every test passing across the
`stub`, `dev_fast_contract`, `codegen_backend_contract`, `coverage_contract`,
and `v0_1_exit_register_contract` binaries.

### Step 2 — add dev-dependencies (subject to Q3)

Edit `Cargo.toml` `[dev-dependencies]`, keeping the existing alphabetical
order and caret requirements:

```toml
proptest = "1.11.0"
rstest-bdd = "0.6.0"
rstest-bdd-macros = "0.6.0"
```

Then:

```bash
cargo fetch 2>&1 | tail -5
make audit 2>&1 | tee /tmp/audit-statelet-$(git branch --show-current).out
```

Expected: `Cargo.lock` updates, and `cargo audit` reports no vulnerabilities.
If it reports one, stop and escalate rather than adding an ignore.

### Step 3 — write the contract test in full

Create the four test files described in `Interfaces and dependencies`. Write
every test and every negative control before either document carries a
register.

### Step 4 — observe red

```bash
make test 2>&1 | tee /tmp/test-statelet-red-$(git branch --show-current).out
```

Expected transcript fragment:

```plaintext
---- template_instantiates_the_field_register stdout ----
docs/adr-004-state-name-consumption-evidence.md: no field register found
between <!-- field-register:begin --> and <!-- field-register:end -->.
Repair: add the register block to the Field register section.
```

Every register-dependent test fails with a `MissingDelimiters` message naming
its own document. `INV-ANCHORS` passes already, because the quoted clauses are
present in their sources from the start; that is expected and is recorded here
so it is not mistaken for a test that never runs.

### Step 5 — insert the registers and go green

Insert both registers into ADR 004 and the note register into the template.
Then, in this order:

```bash
make fmt
make test 2>&1 | tee /tmp/test-statelet-green-$(git branch --show-current).out
```

`make fmt` runs before the test because `mdtablefix` normalizes column widths;
running it afterwards would change the documents under a green test. Copy the
post-format table lines into `fixtures.rs`.

Expected: every test passes. Commit.

### Step 6 — gate table and quoted evidence

Add ADR 004's gate table and "Evidence the record preserves" section, then:

```bash
make fmt && make check-fmt && make test 2>&1 | tee /tmp/test-statelet-gates-$(git branch --show-current).out
```

Expected: `gate_bindings_resolve` and `quoted_passages_still_resolve` pass.
Commit.

### Step 7 — companion sync

Apply every item in `Documentation sync map`, then:

```bash
make fmt
make check-fmt 2>&1 | tee /tmp/checkfmt-statelet-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/mdlint-statelet-$(git branch --show-current).out
make nixie 2>&1 | tee /tmp/nixie-statelet-$(git branch --show-current).out
make test 2>&1 | tee /tmp/test-statelet-sync-$(git branch --show-current).out
```

Expected: all pass, including the pre-existing `v0_1_exit_register_contract`
tests, which read `docs/design.md`, `docs/roadmap.md`, and `docs/context.md`
and are therefore sensitive to this step. Commit.

### Step 8 — property and behavioural coverage (subject to Q3)

Add `tests/features/phase_2_validation_note.feature`,
`tests/phase_2_validation_note_bdd.rs`, and the `proptest` property. Run the
property with an increased case count once to shake out shrink failures:

```bash
PROPTEST_CASES=4096 make test 2>&1 | tee /tmp/test-statelet-prop-$(git branch --show-current).out
```

Expected: passes. Any counter-example lands in `proptest-regressions/` and must
be committed together with its fix. Commit.

### Step 9 — full gates and delivery

```bash
make check-fmt 2>&1 | tee /tmp/checkfmt-statelet-$(git branch --show-current).out
make lint      2>&1 | tee /tmp/lint-statelet-$(git branch --show-current).out
make test      2>&1 | tee /tmp/test-statelet-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/mdlint-statelet-$(git branch --show-current).out
make nixie     2>&1 | tee /tmp/nixie-statelet-$(git branch --show-current).out
make audit     2>&1 | tee /tmp/audit-statelet-$(git branch --show-current).out
make test-workflow-contracts 2>&1 | tee /tmp/wfc-statelet-$(git branch --show-current).out
```

Run these sequentially, never in parallel; the repository relies on build
caching and concurrent cargo jobs contend for the package-cache lock.

Tick roadmap task 1.1.3, append the ADR link to its success bullet in the form
tasks 1.1.1 and 1.1.2 use, set this plan's status to `COMPLETE`, and record
outcomes.

## Documentation sync map

1. `docs/contents.md`, "Decision records": add a bullet after the ADR 003
   entry, in the same link-plus-description form. Add a second bullet under
   "Product and design" for the template, describing it as the form Phase 2 and
   Phase 3 validation tasks fill.
2. `docs/design.md`: add both new documents to the companion-documents list. At
   the end of section 6.1, add one sentence pointing to ADR 004 as the record
   that defines what evidence settles the return type, and to the template as
   the instrument that collects it. **Subject to Q1**, add the B7 row to the
   section 11.1 bet register inside the existing
   `<!-- markdownlint-disable MD013 -->` region. Update "Last substantive
   revision" to the delivery date.
3. `docs/terms-of-reference.md`: add both documents to the companion list, and
   add ADR 004 as a linked bullet in section 10.2. Update "Last substantive
   revision".
4. `docs/context.md`: add two glossary entries. **State identifier** — "a
   stable value distinguishing one state from another for machine consumption;
   distinct from a state name, which is a human-readable label. Statelet
   publishes no state identifier and will not do so without a named consumer."
   **Validation note** — "the record a validation task produces, instantiated
   from `docs/phase-2-validation-note-template.md`." Both are treated as
   citation targets by `INV-ANCHORS` so they cannot be silently dropped.
5. `docs/roadmap.md`: tick task 1.1.3 and append the ADR link to its "Success:"
   bullet. Do **not** renumber anything; `INV-GATES` depends on 2.2.1, 2.2.2,
   3.1.2, and 3.2.1 keeping their numbers, and the existing exit-register
   contract depends on 2.2.3, 3.1.3, and 4.3.1 keeping theirs.
6. `docs/users-guide.md`, "Current status": two sentences telling a prospective
   consumer that the `StateName` return type is not yet settled, that the
   project will not add a numeric identifier without a named consumer, and
   where that decision will be recorded. A consumer deciding whether to depend
   on this crate needs that.
7. `docs/developers-guide.md`: a subsection recording that ADR 004 and the
   template are machine-checked, that
   `tests/state_name_consumption_contract.rs` is the guard, that a revision
   touching `docs/design.md` sections 6.1 or 6.2, the `docs/context.md`
   entries, or roadmap tasks 2.2.1, 2.2.2, 3.1.2, or 3.2.1 will break
   `make test` by design, and how to repair such a break. Include the
   instruction that filling a validation note means copying the template, not
   editing it.
8. `docs/repository-layout.md`: note the new test files and the
   `tests/features/` directory alongside the existing `tests/` entries.

## Validation and acceptance

Run from the repository root. Acceptance is phrased as behaviour.

**Red evidence.** Before the registers exist, `make test` fails and the failure
names a document and a repair, as shown in Step 4. The failure must be
`MissingDelimiters`, not a panic, not an index-out-of-bounds, and not a generic
`assertion failed`.

**Green evidence.** After Step 5, `make test` passes. The new binary
`state_name_consumption_contract` reports, at minimum,
`template_instantiates_the_field_register`, `template_ships_unfilled`,
`sufficiency_mapping_is_total` (four cases),
`absent_consumer_never_selects_insufficient`,
`unbounded_names_never_select_insufficient`,
`register_can_select_insufficient`, `axis_vocabularies_agree` (two cases),
`quoted_passages_still_resolve`, `gate_bindings_resolve` (four cases), and
`fixture_matches_the_live_record`.

**Negative-control evidence.** Each control listed under an invariant is a
committed test asserting an exact message with `pretty_assertions::assert_eq!`,
not `is_err()`. A control that asserts only that an error occurred does not
discharge its obligation and must be rewritten.

**Behavioural evidence.** Subject to Q3, the four scenarios in
`tests/features/phase_2_validation_note.feature` pass, and each is visible in
the test output by its scenario name.

**Manual acceptance.** Copy `docs/phase-2-validation-note-template.md` to a
scratch file, fill the four fields for `mdtablefix`'s `ContinuationMode` as
ADR 002 sketches it — three variant names, no named consumer, a bounded set of
three, and the `transition.state.before` field — and read ADR 004's register.
The verdict must be derivable without consulting any other document and without
a judgement call. If it is not, the instrument has failed its purpose
regardless of what the tests say.

Quality criteria:

- Tests: every gate above passes; every invariant has a passing test and a
  failing control.
- Verification: `INV-FIELDS`, `INV-BLANK`, `INV-TOTAL`, `INV-DEFAULT`,
  `INV-CARDINALITY`, `INV-FALSIFIABLE`, `INV-AXES`, `INV-ANCHORS`,
  `INV-GATES`, and `INV-CASES` discharged; `INV-PARSE` discharged or its
  residual gap recorded per Q3.
- Lint and typecheck: `make lint` reports no `clippy` or Whitaker findings and
  `cargo doc` emits no warnings. No `#[allow]` is added; a finding is fixed in
  the code.
- Performance: not applicable; no runtime code.
- Security: `make audit` reports no advisories.

Quality method: the gate commands in Step 9, run sequentially, each captured to
its own log under `/tmp`, plus an independent review before merge.

## Idempotence and recovery

Every step is re-runnable. `make fmt` is idempotent; running it twice produces
no second diff. The contract test reads documents through `include_str!` and
writes nothing, so re-running `make test` has no side effect.

The work is additive up to the sync map. Recovery from any point before Step 7
is `git checkout -- .` plus deleting the new files. Recovery after Step 7 is a
revert of the sync commit, which is a separate commit from the artefact
commits precisely so that it can be reverted alone.

The one destructive-adjacent action is editing `docs/design.md` section 11.1
under Q1. Before that edit, confirm `make test` is green; after it, re-run
`make test` immediately, because `tests/v0_1_exit_register_contract.rs` reads
that section. If the pre-existing contract fails, revert the single edit rather
than adapting the other contract.

No scratch directories are created outside `/tmp`, and nothing is written to
`target/` beyond ordinary build output.

## Artefacts and notes

Transcripts are appended here as steps complete.

### The field register

To be inserted into ADR 004 under "Field register", between
`<!-- field-register:begin -->` and `<!-- field-register:end -->`:

```markdown
| Field                | Admissible statuses            | Records                                                                                                           | Decides                                  |
| -------------------- | ------------------------------ | ----------------------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| state-display-name   | Recorded / Not reached         | Each annotated state type's path, the name returned for every variant, and whether any name differs from `Debug`  | Note admissibility                       |
| identifier-need      | None observed / Named consumer | Whether a consumer required an operation a `&'static str` cannot perform, naming the consumer, operation and site | The named-consumer axis                  |
| metrics-cardinality  | Bounded / Unbounded            | The number of distinct names each recording site can emit and whether the type fixes that number                  | The name-set axis                        |
| tracing-use          | Recorded / Not exercised       | Which `transition.*` fields carried the name, the `tracing` value form used, and any conversion or allocation     | Whether the default meets design §9      |
```

*Table: The four fields a Phase 2 validation note must record.*

### The sufficiency register

To be inserted into ADR 004 under "Sufficiency register", between
`<!-- sufficiency-register:begin -->` and `<!-- sufficiency-register:end -->`:

```markdown
| Named consumer | Name set  | Verdict       | Action                                                          |
| -------------- | --------- | ------------- | --------------------------------------------------------------- |
| Absent         | Bounded   | Sufficient    | Keep `&'static str`; record the note as evidence for 3.2.1      |
| Absent         | Unbounded | Naming defect | Repair the state names; do not infer an identifier need         |
| Present        | Bounded   | Insufficient  | Record the consumer and required operation for 3.2.1            |
| Present        | Unbounded | Naming defect | Repair the state names, then re-record the consumer             |
```

*Table: How a filled validation note resolves to a verdict on `&'static str`.*

### The template's note register

To be inserted into `docs/phase-2-validation-note-template.md` between
`<!-- validation-note:begin -->` and `<!-- validation-note:end -->`:

```markdown
| Field                | Status | Evidence |
| -------------------- | ------ | -------- |
| state-display-name   | TBD    | TBD      |
| identifier-need      | TBD    | TBD      |
| metrics-cardinality  | TBD    | TBD      |
| tracing-use          | TBD    | TBD      |
```

*Table: The blank Phase 2 validation note. Copy this file; do not edit it.*

### ADR 004's section skeleton

Following `docs/documentation-style-guide.md`'s required order:

```plaintext
# Architectural decision record (ADR) 004: Define the StateName consumption evidence

## Status            -> Accepted, <delivery date>
## Date
## Context and problem statement
## Decision drivers
## Options considered            -> A: free-text note; B: enumerated field register
                                    with a total verdict table; C: defer to 3.2.1.
                                    Table 1 compares them.
## Decision outcome / proposed direction
## Field register                -> delimited, machine-checked
## Sufficiency register          -> delimited, machine-checked
## Gates                         -> S1..S4 table
## Evidence the record preserves -> the five quoted clauses
## Goals and non-goals
## Known risks and limitations   -> includes the discriminant finding
## Outstanding decisions         -> the verdict itself, left to 3.2.1
## Architectural rationale       -> the cardinality argument in full
```

### The gate table

```markdown
| Gate | Roadmap task | Records or decides                                                  |
| ---- | ------------ | --------------------------------------------------------------------- |
| S1   | 2.2.1        | Records the note for `mdtablefix` `ProcessBuffer`                     |
| S2   | 2.2.2        | Records the note for `mdtablefix` continuation handling               |
| S3   | 3.1.2        | Records the note for `wireframe`, using the same template             |
| S4   | 3.2.1        | Decides the `StateName` return shape from the recorded notes          |
```

*Table: Gates at which each validation note is recorded and the gate at which
the verdict is taken.*

## Interfaces and dependencies

### New dev-dependencies (subject to Q3)

`Cargo.toml`, `[dev-dependencies]`, caret requirements only:

```toml
proptest = "1.11.0"
rstest-bdd = "0.6.0"
rstest-bdd-macros = "0.6.0"
```

`camino`, `googletest`, `pretty_assertions`, `rstest`, and `toml` are already
present and unchanged. No runtime dependency is added.

### Test module shape

`tests/state_name_consumption_contract.rs` owns the integration-test scenarios
and nothing else. It declares its children with `#[path]`, as the
exit-register contract does:

```rust,ignore
#[path = "state_name_consumption_contract/fixtures.rs"]
mod fixtures;
#[path = "state_name_consumption_contract/regression_controls.rs"]
mod regression_controls;
#[path = "state_name_consumption_contract/support.rs"]
mod support;

const ADR: &str = include_str!("../docs/adr-004-state-name-consumption-evidence.md");
const TEMPLATE: &str = include_str!("../docs/phase-2-validation-note-template.md");
const DESIGN: &str = include_str!("../docs/design.md");
const CONTEXT: &str = include_str!("../docs/context.md");
const ADR_001: &str = include_str!("../docs/adr-001-proving-ground-candidates.md");
const ROADMAP: &str = include_str!("../docs/roadmap.md");
```

`support.rs` owns the pure parsers and policy predicates and no scenarios:

```rust,ignore
pub(super) enum Consumer { Absent, Present }
pub(super) enum NameSet { Bounded, Unbounded }
pub(super) enum Verdict { Sufficient, Insufficient, NamingDefect }

pub(super) struct FieldRow {
    pub(super) id: String,
    pub(super) statuses: Vec<String>,
}
pub(super) struct SufficiencyRow {
    pub(super) consumer: Consumer,
    pub(super) names: NameSet,
    pub(super) verdict: Verdict,
    action: String,
}
pub(super) struct NoteRow {
    pub(super) id: String,
    pub(super) status: String,
    pub(super) evidence: String,
}

pub(super) enum ParseError {
    MissingDelimiters { document: &'static str, begin: &'static str },
    EmptyRegister { document: &'static str },
    MalformedRow { document: &'static str, line: usize },
    UnknownToken { document: &'static str, column: &'static str, found: String },
}

pub(super) fn parse_field_register(adr: &str) -> Result<Vec<FieldRow>, ParseError>;
pub(super) fn parse_sufficiency_register(adr: &str) -> Result<Vec<SufficiencyRow>, ParseError>;
pub(super) fn parse_note_register(template: &str) -> Result<Vec<NoteRow>, ParseError>;

pub(super) fn check_fields_match(adr: &[FieldRow], note: &[NoteRow]) -> Result<(), String>;
pub(super) fn check_unfilled(note: &[NoteRow]) -> Result<(), String>;
pub(super) fn check_totality(rows: &[SufficiencyRow]) -> Result<(), String>;
pub(super) fn check_default_rule(rows: &[SufficiencyRow]) -> Result<(), String>;
pub(super) fn check_cardinality_rule(rows: &[SufficiencyRow]) -> Result<(), String>;
pub(super) fn check_falsifiable(rows: &[SufficiencyRow]) -> Result<(), String>;
pub(super) fn check_axis_vocabularies(fields: &[FieldRow], rows: &[SufficiencyRow]) -> Result<(), String>;
pub(super) fn check_quoted_clauses(adr: &str, design: &str, context: &str, adr_001: &str, roadmap: &str) -> Result<(), String>;
pub(super) fn check_gate_bindings(adr: &str, roadmap: &str) -> Result<(), String>;
```

Every policy function returns `Result<(), String>` whose `Err` is the exact
repair message a negative control asserts. No function panics; `unwrap_used`,
`expect_used`, `panic`, and `indexing_slicing` are denied in `Cargo.toml`, so
the parser must use `split_once`, `get`, slice patterns, and `let ... else`
throughout, exactly as `tests/v0_1_exit_register_contract/support.rs` does.

If `support.rs` approaches 350 lines, split the row-shape recognition into
`tests/state_name_consumption_contract/support/register_rows.rs`, mirroring the
`support/split_case.rs` precedent, rather than relaxing the limit.

`fixtures.rs` owns the canonical handwritten registers as string constants.
`regression_controls.rs` owns the mutated variants and, under Q3, the
`proptest` property and its renderer.

### Behavioural test shape (subject to Q3)

`tests/phase_2_validation_note_bdd.rs` binds the feature file:

```rust,ignore
use rstest_bdd_macros::{given, scenario, then, when};

#[scenario(
    path = "tests/features/phase_2_validation_note.feature",
    name = "A named consumer with a bounded name set overturns the default"
)]
fn named_consumer_overturns_default(note: NoteUnderTest) {
    // Assertions live in the Then steps.
}
```

Step definitions call the same `support` predicates the `rstest` suite calls.
No policy logic is written twice.

## Revision note

- 2026-09-18: initial draft. Establishes the two-document structure (decision
  record plus blank template), the two-axis sufficiency register with its
  exclusion rules, ten invariants with named negative controls, and three
  approval-gate questions (Q1 bet B7, Q2 two files, Q3 `rstest-bdd` and
  `proptest`). No implementation has started; the plan awaits approval.
