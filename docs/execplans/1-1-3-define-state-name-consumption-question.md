# Define the `StateName` consumption question (roadmap 1.1.3)

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances (exception triggers)`, `Risks`, `Progress`,
`Surprises & discoveries`, `Decision log`, `Outcomes & retrospective`,
`Conformance basis`, and `Verification plan` must be kept up to date as work
proceeds.

Status: IN PROGRESS — resumed 2026-09-19 after the approval gate settled Q5.
The filesystem-lint deviation is accepted in the path-scoped form; see Q5, D18,
and D20. Stage A and the two documents in Step 2 are done.

## Purpose / big picture

Statelet intends to publish a trait whose whole public surface is one method:

```rust,ignore
pub trait StateName {
    fn state_name(&self) -> &'static str;
}
```

The technical design does not know whether that return type is right, and says
so: the `mdtablefix` baseline "must scrutinize whether `&'static str` is enough
for the first slice".

That instruction is currently unexecutable. Nothing in the repository tells a
Phase 2 engineer what to record, where to record it, or what rule turns a
record into a verdict. Roadmap task 3.2.1 is scheduled to "finalize the
`StateName` return shape" from "observed example consumption, not
anticipation", but no instrument exists to observe that consumption with.

After this change, four things are true that are not true today.

First, a Phase 2 engineer annotating `mdtablefix` copies
`docs/phase-2-validation-note-template.md` into `docs/validation-notes/`, fills
four named fields whose admissible answers are enumerated, and commits it. The
note has a home, a naming convention, and a reader.

Second, the rule that turns those fields into a verdict is written down in
`docs/adr-004-state-name-consumption-evidence.md` before any evidence exists,
so it cannot be reverse-engineered from a result somebody has grown fond of.
The rule separates *admissibility* (is this note usable evidence at all?) from
*verdict* (does the evidence overturn the default?), because conflating them
produces a decision procedure that loops instead of terminating.

Third, the rule for reading several notes *together* is also written down.
Three notes reach task 3.2.1, from tasks 2.2.1, 2.2.2, and 3.1.2. A rule that
resolves one note and says nothing about three is total over the wrong domain.

Fourth, all of it is machine-checked, including a committed worked example, so
the suite exercises a filled note and not merely an empty form.

Observable acceptance: from a clean checkout, `make test` passes and reports
the new integration-test binary `state_name_consumption_contract`. Deleting the
`Insufficient` row from the status register, adding a residual `TBD` to the
committed worked example, or renaming a field in one document but not the
other, each makes `make test` fail with a message naming the file and the
repair.

This task adds no runtime code. `src/` is untouched.

## Context and orientation

### What Statelet is

Statelet is an unreleased Rust crate. Its thesis is narrow: handwritten Rust
state machines are fine, and what they lack is a shared convention for *marking
the boundary* where a transition happens, so that tracing, diagnostics, and
review describe that boundary the same way. Statelet does not own dispatch,
events, storage, transition tables, or graph safety; ADR 002 records that
boundary.

Two terms, defined because both are easy to misread:

A **state name** is, per `docs/context.md`, "a stable, low-cost name for a
state used in diagnostics, tracing fields, and generated documentation. A state
name is not required to be the same as `Debug` output." It is a label.

A **validation note** is the record a validation task produces. The roadmap
uses the phrase seven times without defining it; this plan defines it and gives
it a home. See `Decision log` D1.

### At plan inception: current state of the repository

The crate is a stub: `src/lib.rs` is twelve lines containing one
`pub const fn greet() -> &'static str` and a `TODO` marking it for deletion.
There is no `StateName` trait, no derive macro, and no `mdtablefix` integration.

What the repository does have is a body of *documentation-as-code*: integration
tests that treat design documents as the artefact under test.
`tests/v0_1_exit_register_contract.rs` and its four private children parse
`docs/adr-003-v0-1-exit-register.md`, check the exit register is total, check
that a falsified bet B1 forces the off-ramp, check quoted clauses still resolve
in their sources, and check named gates resolve to roadmap tasks. Siblings
`tests/codegen_backend_contract.rs`, `tests/coverage_contract.rs`, and
`tests/dev_fast_contract.rs` do the same for build configuration.

This plan adds one more member of that family. Read
`tests/v0_1_exit_register_contract.rs` and all four children before starting;
this plan repeatedly cites specific lines in them, and imitating the good parts
while avoiding the documented traps is most of the work.

Roadmap tasks 1.1.1 and 1.1.2 are complete (ADR 002 and ADR 003). Task 1.1.3 is
the next unticked item and is the subject of this plan. Its declared
dependency, 1.1.2, is satisfied.

### The question this plan answers, and why the obvious reading is wrong

Roadmap task 1.1.3 says:

```plaintext
Decide what `mdtablefix` must consume to prove whether `&'static str` is
enough or whether a stable numeric identifier is needed.
Success: the Phase 2 validation note template has fields for state display
name, optional identifier need, metrics cardinality, and tracing use.
```

The obvious reading is "string versus number". That reading is wrong, and
building the instrument around it would produce an instrument that cannot find
the answer. Four findings establish this.

**Finding one: the operative word is *stable*, not *numeric*.** Roadmap task
3.2.1, the gate that consumes this work, asks whether "a stable identifier is
needed" — it does not say numeric. Design §6.1 speaks of "a stable numeric
discriminant **or other low-cardinality identifier**". A `&'static str`
defaulting to the Rust variant name is *not* stable: renaming a variant
silently renames a label that design §9 declares "public operational API", and
no compiler error results. The requirement Phase 2 is most likely to surface is
therefore a *property* — label stability across releases — not an operation
that a string cannot perform. Design §6.1 already names the cheap remedy,
"allow explicit renames later only if examples prove the need", and no other
roadmap task collects evidence for it. An instrument that asks only "did a
consumer need an operation `&'static str` cannot do?" records "no" and loses
this entirely.

**Finding two: a numeric identifier cannot reduce metric cardinality.**
`StateName` is a total function from a state to a label; any identifier is a
total function from the same state to a value. Substituting one for the other
relabels the same domain, so the number of distinct values a backend sees is
unchanged. Prometheus guidance warns against labels holding "dimensions with
high cardinality (many different label values) … or other unbounded sets of
values"; OpenTelemetry likewise treats low cardinality as a property of the
value set, not of the representing type. An unbounded name set is therefore a
naming defect — a name synthesized from data, or a leaked `String` — and never
an argument for an identifier. Cardinality gates whether a note is *usable*; it
never decides the verdict.

**Finding three: no stable numeric identifier is available for free.**
`std::mem::discriminant` does not qualify: the standard library documents that
"the discriminant of an enum variant may change if the enum definition
changes", and transmuting `Discriminant<T>` to a primitive is undefined
behaviour. A numeric value is reachable only by an `as` cast on a fieldless
enum or through an explicit `#[repr(u8)]`-style representation. So any stable
identifier must be *hand-assigned*, which is new user-visible API and new user
obligation — exactly the speculative surface design §6.2 refuses without a
named consumer.

**Finding four: the one concrete cross-tool consumer favours the string.** ADR
002's `wireframe` sketch names `crates/wireframe-verification`, whose
Stateright model wants "production logs, tests, and the model the same `Active`,
`ShuttingDown`, and `Finished` labels". Its requirement is *equality* across
tools, which a string serves better than a number, because a number must be
decoded before a human can read a counter-example. Note carefully: this is a
*named consumer* that does *not* establish a need. An axis labelled "named
consumer" would be ticked here by a careless filler and would produce the wrong
verdict. The axis must therefore be labelled by the *requirement*, not by the
existence of a consumer.

Taken together the honest prior is that the current default survives, possibly
amended with explicit rename support. This plan does not argue that case. It
builds an instrument that could overturn it, and asserts the instrument retains
a verdict capable of doing so.

### Approval-gate decisions

Five decisions were referred to the approval gate. All five were settled on
2026-09-18, each as recommended; the reasoning is retained because it explains
why the artefacts have the shape they do. Nothing below is still open.

**Q0 — is the reframing in "Findings one and two" accepted?** The roadmap names
a field "optional identifier need" and a field "metrics cardinality". This plan
reads the first as *which property a consumer requires that variant-name
`&'static str` fails to provide* (equality, stability across releases,
ordering, compact encoding), and the second as *the bound on the set of
distinct names the annotated code can emit*, used as an admissibility gate
rather than as a verdict axis. Both readings are defensible and both are
consequential: the first is what makes the label-stability case recordable, the
second is what makes the procedure terminate. Neither is the naive reading.
**Decided: accepted.** This is the substance of the task. See D4, D11 and D17.

**Q1 — how should this question be made evidence-gated in `docs/design.md`?**

Two upstream registers could carry it, and the plan's first two drafts
considered only one of them.

`docs/design.md` §11.1 holds a table named the bet register: six rows, B1 to
B6, each naming a claim the design is betting on, a confidence, and the
evidence that would settle it. The section states its own rule — "Each bet must
have an evidence-producing gate before the affected API is published" — and by
that rule a bet is missing. "B7" is this plan's name for the row that would
close the gap. It does not exist anywhere yet; the proposal is:

<!-- markdownlint-disable MD013 -->

```markdown
| B7  | Variant-name `&'static str` is a sufficient state identity for v0.1 | Medium | Every admissible validation note records no required property the default fails to supply |
```

<!-- markdownlint-enable MD013 -->

*Table 1: The proposed bet B7, sized to the existing column widths.*

`docs/design.md` §14 "Deferred decisions" is the second candidate, and the
better fit. It is introduced as "The implementation should resolve these before
publishing v0.1"; it already carries a `StateName` item, on whether the derive
follows the `macros` feature; and the return shape — the very decision this
task prepares — is conspicuously absent from it. One bullet closes that:

```markdown
- Whether `StateName` returns `&'static str`, or a value carrying a stronger
  stability guarantee, as decided by roadmap task 3.2.1 from the validation
  notes defined in ADR 004.
```

**Decided: add the §14 bullet in this task; raise B7 separately, using the row
above.**

The §14 bullet is one line and verified inert. The existing contract touches
§14 only as a relocation anchor — `tests/v0_1_exit_register_contract.rs:142` and
`regression_controls.rs:16` substitute on the heading string
`"## 14. Deferred decisions"` — so adding a bullet beneath that heading changes
nothing it reads.

B7 is deferred on scope rather than safety, and the safety objection raised in
review is discharged. The hazard is real and invisible on inspection:
`tests/v0_1_exit_register_contract.rs:119-126` and `:274-290` embed the §11.1
B1 row byte-exactly including its trailing pad spaces and use it with
`DESIGN.replace(...)`, while `mdtablefix` canonicalizes every column to
`max(content) + 2`. Current maxima are 3, 77, 10 and 93 characters. Both halves
were measured against the real repository rather than reasoned about: a B7 row
with a 37-character Confidence cell plus `make fmt` repads the table and grows
the B1 row from 197 to 224 bytes, whereas the fitted row above — 67, 6 and 89
characters — leaves every pre-existing row byte-identical and `make test` then
reports 40 passed. So B7 is deliverable; it is simply upstream design prose
that roadmap task 1.1.3 did not ask for, and the pre-drafted row makes the
follow-up mechanical rather than a research task.

**Q2 — should the template be a separate file from the ADR?** **Decided: yes**,
with the template *generated* from the ADR rather than hand-maintained
alongside it (see `INV-TEMPLATE`). Their lifecycles differ: an ADR is frozen, a
template is copied at least three times, and filling a form inside an accepted
ADR would mean editing the ADR. Roadmap task 1.2.3 adds a sibling benchmark
note template, so a template family is coherent. Generation removes the
hand-maintained drift edge that would otherwise be the main argument against
two files.

**Q3 — should `rstest-bdd` and `proptest` be added as dev-dependencies?**
**Decided: no, both declined.** Measured: the current `Cargo.lock` resolves 45
packages; a probe lockfile with `rstest 0.27` plus `rstest-bdd 0.6` and
`rstest-bdd-macros 0.6` resolves **185**. That is a fourfold dependency-graph
increase — pulling in `tokio`, `fluent`, `i18n-embed`, `rust-embed`, and
`serde_json` — on a crate whose `src/lib.rs` is twelve lines, to express four
scenarios over a Markdown table. No workflow caches `target/`, so the cost is
paid on every CI run of four workflows. `proptest`'s case is separately weak:
the proposed property round-trips the parser against a renderer written in the
same file, so it proves `parse ∘ render = id` for a renderer no document uses,
while the failure class that actually bit roadmap task 1.1.2 was *malformed*
input. Five named handwritten controls cover that class exactly, with no
dependency. The behavioural coverage the governing instruction asks for is
delivered as scenario-named `rstest` cases over the committed worked example
and its mutations, which exercise the same workflow; `docs/developers-guide.md`
carries the prose walkthrough. This is a proportionality judgement, not a
refusal — if the dependency cost is acceptable, the scenarios convert to
Gherkin mechanically.

**Q4 — may this plan add pointers to roadmap tasks 2.2.1, 2.2.2, and 3.1.2, and
to design §12?** Without them a Phase 2 engineer will not find the template:
task 2.2.1 cites only design §12, which would not mention it. The edits add one
`- See ...` bullet each and renumber nothing. **Decided: yes.**

**Q5 — how should the notes-directory scan read a note's contents?** **Decided
2026-09-19: Option A, a path-scoped `dylint.toml` exclusion.** The approving
authority directed the path-scoped form in preference to the crate-wide one.
Recorded in D20. This question was not in the planning phase's set because the
review lens that examined the scan concluded it was a read-only `camino`
operation. Measurement says otherwise.

`INV-FILLED` requires the contract test to enumerate `docs/validation-notes/`,
select the files declaring `<!-- state-name-note -->`, and check each one's
contents: no residual `TBD`, admissible statuses, citation-shaped evidence
cells, no missing field. Enumeration is settled and measured clean —
`Utf8Path::read_dir_utf8()` plus `Utf8DirEntry::file_name()` passed `whitaker`
under `-D warnings`, and `include_str!` remains available for the fixed-path
documents. The *content* read of an enumerated file is the problem, and it had
no answer that was free of a standing constraint:

- `camino` cannot read contents. Measured: `grep read_to_string` over
  `camino-1.2.5/src/lib.rs` returns nothing. `camino` is a path type, not a
  capability handle; it exposes only `read_dir` and metadata queries.
- `std::fs` is denied in test crates by `no_std_fs_operations`, and — measured
  on 2026-09-19 — **no Rust attribute suppresses it**: not item-level
  `#[allow]`, not item-level `#[expect]` (which additionally raises
  `unfulfilled_lint_expectations`), and not a crate-level `#![allow]`. Only
  `dylint.toml` suppresses it. See `Surprises & discoveries` for the probe
  matrix.
- `include_str!` cannot serve an enumerated file. It needs a literal path at
  compile time, and the file set is deliberately open: a Phase 2 engineer adds
  a note months from now, and its arrival must not require a Rust edit.
  `concat!` with a `macro_rules!` name argument does work — probed and clean —
  but it still requires the list of names to be known at compile time, which is
  the property the scan exists to avoid.

Two remedies are viable, and each breaches a standing constraint:

- **Option A — a `dylint.toml` exclusion, scoped by path.** Create
  `dylint.toml` at the workspace root with
  `[no_std_fs_operations] excluded_paths = ["state_name_consumption_contract::notes"]`,
  and read through `std::fs` inside that one module. Measured working, in both
  the crate-wide and the path-scoped form, the latter being the narrower one.
  Cost: one new tracked file that this plan's "Files this plan reads or writes"
  list does not contain, plus a permanent, visible exemption in the very
  configuration that enforces the estate's filesystem policy. The lint stays
  active everywhere else. It sets a precedent each future contract test can
  cite, so the exemption generalizes rather than remaining exceptional.
- **Option B — add `cap-std` as a dev-dependency**, and read through
  `cap_std::fs::Dir` (`read_to_string` at `fs_utf8/dir.rs:264`), which is what
  the lint's own diagnostic message and `users-guide.md` instruct. Measured
  cost: **45 → 84 packages** in a probe lockfile built from this repository's
  real dev-dependency set. Breaches constraint 3 ("No dependency change.
  `Cargo.toml` and `Cargo.lock` are untouched") and contradicts the settled Q3
  precedent, which declined `rstest-bdd` and `proptest` on a 45 → 185 increase.
  At 39 added packages this is not the same order, but it is the same kind of
  decision, and Q3's reasoning was about a twelve-line `src/lib.rs` paying an
  estate-wide cost. Also note `cap-std`'s `camino` feature pulls `camino`
  itself, so the existing direct dependency would need an eye kept on it.
- **Option C — keep the scan read-only in the material sense and require
  `include_str!`.** Every note would be embedded at compile time and the scan
  would verify that the compiled-in set matches the directory. Rejected on
  inspection: it inverts the requirement. A new note would fail to compile
  until someone edited Rust, which is exactly the coupling `INV-FILLED`'s
  marker design was chosen to avoid (D15). Listed here so the rejection is
  visible rather than implicit.

Option A was recommended and is adopted. It is the smaller breach of the two
live options: one new file, no dependency change, and the exemption is named
and path-scoped rather than crate-wide. Option B is the more principled remedy
against the lint's own stated intent, and would be preferable if a `cap-std`
dev-dependency is acceptable for other reasons — but it is a dependency
decision that the plan was approved without, and Q3 shows this project weighs
those carefully rather than by default.

The exemption is narrowed further than the option text above proposes. The
module that reads file contents will be named for its single job and the
`excluded_paths` entry will name that module alone, so the exemption covers the
one function that needs it rather than the whole test crate. Every other module
of the contract test — the parsers, the policies, the fixtures — remains under
the lint, and so does every other test crate in the workspace.

### Files this plan reads or writes

Written (new):

- `docs/adr-004-state-name-consumption-evidence.md`
- `docs/phase-2-validation-note-template.md`
- `docs/validation-notes/README.md`
- `dylint.toml` — the path-scoped `no_std_fs_operations` exemption settled as Q5
  option A and ratified in D20. One entry, naming the single module that reads
  a note's contents; rationale comment required, per the Whitaker convention.
- `tests/state_name_consumption_contract.rs`
- `tests/state_name_consumption_contract/types.rs`
- `tests/state_name_consumption_contract/parse.rs`
- `tests/state_name_consumption_contract/policy.rs`
- `tests/state_name_consumption_contract/notes.rs`
- `tests/state_name_consumption_contract/fixtures.rs`
- `tests/state_name_consumption_contract/clauses.rs`, `registers.rs` — the two
  modules D21 records as a deviation from this list's first draft.
- `tests/state_name_consumption_contract/anchor_scenarios.rs`,
  `note_scenarios.rs`, `register_scenarios.rs`, `scan_scenarios.rs` — the four
  scenario modules D26 records, split from the crate root so the 400-line cap
  binds every part of the contract alike.

Written (modified): `docs/design.md`, `docs/terms-of-reference.md`,
`docs/context.md`, `docs/roadmap.md`, `docs/contents.md`, `docs/users-guide.md`,
`docs/developers-guide.md`, `docs/repository-layout.md`.

Read only, never modified: everything under `src/`;
`tests/v0_1_exit_register_contract.rs` and its children; `Makefile`,
`clippy.toml`, `Cargo.toml`, `rust-toolchain.toml`, `typos.toml`.

No dependency change. `Cargo.toml` and `Cargo.lock` are untouched.

### Documentation to read before starting

- `docs/design.md` §§6.1, 6.2, 9, 11.1, 12, 13.6, 13.7, 14.
- `docs/context.md` in full; it is short.
- `docs/adr-001-proving-ground-candidates.md` "Outstanding decisions", and
  `docs/adr-002-transition-boundary-scope.md`'s `mdtablefix` and `wireframe`
  sketches. These name the real state types Phase 2 will meet.
- `docs/adr-003-v0-1-exit-register.md` — the structural model for ADR 004.
- `docs/documentation-style-guide.md` — ADR section order and template.
  Note: sentence-case headings, a **numbered** caption under every table
  (`*Table N: ...*`, per ADR 001, 002, and 003), a language identifier on every
  fence, 80-column prose, 120-column code, and no first- or second-person
  pronouns outside `README.md`.
- `AGENTS.md` — the 400-line file cap and the gate list.
- `clippy.toml` — `cognitive-complexity-threshold = 9`,
  `too-many-arguments-threshold = 4`, `too-many-lines-threshold = 70`,
  `excessive-nesting-threshold = 4`, and `allow-expect-in-tests = true`. These
  bind harder than the prose in `AGENTS.md`.
- `Cargo.toml` `[lints.clippy]` — `unwrap_used`, `indexing_slicing`,
  `option_if_let_else`, and `self_named_module_files` are denied. Note that
  `clippy::panic` is **not** denied (only `panic_in_result_fn`), and that
  `expect_used`, though denied, is re-permitted in tests by `clippy.toml`.
- `docs/complexity-antipatterns-and-refactoring-strategies.md` — the threshold
  that forced roadmap task 1.1.2 to split its parser twice.
- `docs/rust-testing-with-rstest-fixtures.md` — fixture and case idiom.
- `.markdownlint-cli2.jsonc` — `MD013` is `line_length: 80`,
  `code_block_line_length: 120`, `tables: false`, `headings: false`. Table rows
  are already exempt; fenced-block lines are **not**.

### Skills to load before starting

`execplans`; `rust-router` (which routes onward — do not load a language skill
speculatively); `rust-unit-testing` for case tables and assertion shape;
`en-gb-oxendict-style` for both new documents; `leta` for navigating the
existing contract modules. `arch-decision-records` is background only: the
repository's ADR template is `docs/documentation-style-guide.md`, not the
skill's Y-Statement form, and the repository template wins.

## Conformance basis

Upstream artefacts and revisions at the time of writing:

- Terms of reference: `docs/terms-of-reference.md`, "Draft v0.2 after design
  review", last substantive revision 2026-08-22.
- Technical design: `docs/design.md`, same status and date.
- Decision records: ADR 001; ADR 002 (Accepted 2026-07-22); ADR 003 (Accepted
  2026-08-22).
- Roadmap: `docs/roadmap.md` at commit `bad9a04`.
- Governing standard: `docs/documentation-style-guide.md`.
- External interfaces treated as axioms, not verified here:
  `std::mem::discriminant`'s stability and opacity clauses; `tracing`'s
  acceptance of `&'static str` as a field value; Prometheus and OpenTelemetry
  guidance that cardinality is a property of the value set; `mdtablefix`'s
  column canonicalization to `max(content) + 2`.

Traced items:

```plaintext
ROADMAP-1.1.3-success -> EP-M1 -> ADR-004 status register
                      -> anchor_scenarios::success_criterion_still_maps
TDD-6.1-stable-id     -> EP-M1 -> ADR-004 status register
                      -> register_scenarios::status_register_matches_fixture
TDD-6.1-default-str   -> EP-M1 -> ADR-004 R-DEFAULT
                      -> register_scenarios::default_survives_without_a_required_property
TDD-6.1-cardinality   -> EP-M1 -> ADR-004 admissibility
                      -> register_scenarios::register_confines_insufficient_to_the_required_property,
                         note_scenarios::blocked_notes_resolve_to_not_resolved
TDD-6.2-no-speculative-api -> ADR-004 rationale -> EP-M1
                      -> anchor_scenarios::quoted_passages_still_resolve
ADR-002-wireframe-labels -> Finding four -> EP-M1
                      -> anchor_scenarios::quoted_passages_still_resolve
TDD-9-transition-fields -> field tracing-use -> EP-M2 -> ADR-004 worked example
ROADMAP-2.2.1/2.2.2/3.1.2 -> gates S1..S3 -> EP-M3
                      -> anchor_scenarios::gate_titles_resolve
ROADMAP-3.2.1         -> gate S4 + aggregation register -> EP-M3
                      -> register_scenarios::aggregation_register_is_total
```

Each leaf names its module as well as its test, because the contract is eleven
modules and two of its scenario names differ by one letter:
`note_scenarios::committed_state_name_note_is_usable` is the accepting witness,
and `scan_scenarios::committed_state_name_notes_are_usable` is the
committed-note scan. A bare `tests::` prefix would leave a reader to grep for
which of the two a line meant.

This plan does not deviate from ADR 002 or ADR 003. ADR 002's non-goals leave
the identifier question open and ADR 001's outstanding decisions name it; this
plan discharges the preparatory half and leaves the substantive half to task
3.2.1.

One consequence must be stated plainly rather than discovered later: writing
the rule before the evidence converts task 3.2.1 from a *decision* into an
*evaluation*. That is the intent, but it means ADR 004 constrains 3.2.1's
outcome, and a reviewer should approve it on that understanding.

## Constraints

1. **No runtime code.** Nothing is added to or changed under `src/`.
2. **No verdict.** ADR 004 defines evidence and rule. It must not conclude that
   `&'static str` is or is not sufficient.
3. **No dependency change.** `Cargo.toml` and `Cargo.lock` are untouched.
4. **`tests/v0_1_exit_register_contract.rs` and its children are not
   modified.** If an edit to `docs/design.md`, `docs/context.md`, or
   `docs/roadmap.md` breaks that contract, revert the edit; do not adapt the
   other contract.
5. **No `docs/design.md` §11.1 edit.** Q1 was settled the other way: the §14
   bullet is added instead, and bet B7 is a separate change. The width budget
   recorded in Q1 binds that separate change, not this one.
6. **No roadmap renumbering.** Tasks 2.2.3, 3.1.3, and 4.3.1 are gate targets
   of the existing contract. This plan's own gates bind by task *title*, not
   number, and therefore add no new frozen numbers.
7. **Every source file under 400 lines**, every function under 70, cognitive
   complexity under 9, at most 4 arguments, nesting under 4.
8. **British English, Oxford spelling** in all new prose.
9. **No fenced-block line over 120 columns**, no prose line over 80.
10. **The filesystem exemption is confined to one module.** `dylint.toml`
    exempts `state_name_consumption_contract::notes` and nothing else. No other
    module of the contract test, and no other test crate, may call `std::fs`.
    If a second module needs the exemption, that is a scope change and stops the
    work.

## Tolerances (exception triggers)

1. **Scope.** Touching a file not listed under "Files this plan reads or
   writes" stops the work.
2. **Dependencies.** Any dependency addition stops the work; Q3 was settled
   by declining both candidates.
3. **Upstream prose.** Adding a pointer to an upstream document is in scope.
   Changing a requirement, a bet, or a success criterion is not.
4. **Iterations.** If a contract test still fails after four repair attempts,
   stop; a parser resisting four repairs is the wrong parser.
5. **Size.** The module split is pre-declared below rather than discovered; if
   any of `types.rs`, `parse.rs`, or `policy.rs` passes 300 lines, stop and
   re-plan the split.
6. **Ambiguity.** If the status register admits a second defensible reading,
   stop and present both.
7. **Gate time.** If `make lint` or `make test` exceeds twenty minutes, stop
   and report.

## Risks

- Risk: a filled note is written from the type declaration rather than from
  observation — ninety seconds of reading an enum, dressed as evidence.
  Severity: high. Likelihood: high. Mitigation: every evidence cell must be a
  citation of the shape `<repo>@<sha>:<path>` and is checked for it;
  `identifier-need` requires an enumerated *consumers considered* list, so
  "none required" carries a search set rather than being a bare negative
  existential; the committed worked example shows what an adequate cell looks
  like.

- Risk: the admissibility blocker "repair the state names" names work in
  `mdtablefix` or `wireframe`, repositories this roadmap does not own.
  Severity: medium. Likelihood: medium. Mitigation: ADR 004 states that a
  blocked note records an upstream issue link and blocks the *Statelet* gate;
  it never instructs a Phase 2 engineer to land a refactor they cannot merge.

- Risk: the parser repeats the defects roadmap task 1.1.2 spent roughly ten
  commits repairing. Severity: medium. Likelihood: medium. Mitigation: one
  generic delimited-table parser with typed mappers layered on top, rather than
  three parsers; five named controls for the five documented failure classes;
  and, specifically, section bounds taken with `split_once` on the *next*
  heading text rather than by scanning for a leading `#`, because
  `docs/design.md` §6.1 contains `#[derive(StateName)]` at column zero inside a
  fence and a naive scanner truncates the section there.

- Risk: quoted clauses fail to resolve because `mdtablefix --wrap` rewraps
  ADR 004's prose at different break points from the source document. Severity:
  high. Likelihood: certain without mitigation — three of the six clauses in
  this plan's own first draft did not resolve. Mitigation: fold whitespace on
  both sides before comparison, exactly as
  `tests/v0_1_exit_register_contract/support.rs:318` does.

- Risk: the register accumulates coupling that makes ordinary documentation
  edits into Rust-test maintenance. Severity: medium. Likelihood: medium.
  Mitigation: `INV-ANCHORS` guards three clauses, not six; the new glossary
  entries are added but deliberately *not* made citation targets.

## Progress

- [x] Stage A — orient and confirm the conformance basis (no changes).
      Confirmed 2026-09-19: branch
      `1-1-3-define-state-name-consumption-question`, clean tree at `6e4b532`,
      green `make test` (40 nextest cases and one doctest), roadmap task 1.1.3
      unticked, and all three `INV-ANCHORS` clauses present in their named
      sections. No repository artefact changed; this tick and the timestamp
      are the only diff.
- [x] Step 2 — ADR 004, the notes-directory `README.md`, and the template's
      home are committed. Ticked retroactively on 2026-09-19: ADR 004 (322
      lines, four empty delimiter pairs per D19) and
      `docs/validation-notes/README.md` landed in `65b59c5`. The template
      itself belongs to Step 6 and is not yet written.
- [x] Q5 settled 2026-09-19 — path-scoped `dylint.toml` exemption; D18 lifted
      by D20; constraint 10 added; `dylint.toml` and `notes.rs` added to the
      file list. This tick and D20 are the only diff of the resumption commit.
- [x] Steps 3 and 4 — the contract test's seven files and the `dylint.toml`
      exemption are written, and red is observed and recorded before any
      register content exists. Twenty of the 39 scenarios pass at that point —
      every one an accepting or structural control — and the 19 that fail do so
      with `MissingDelimiters` naming their own register, with
      `quoted_passages_still_resolve` failing on the empty-clause-list control
      and `unmarked_notes_are_ignored` passing throughout. That is the Step 4
      prediction exactly as D22 corrects it. Transcript:
      `/tmp/red-statelet-1-1-3-define-state-name-consumption-question.out`.
- [x] Steps 5 to 7 — both ADR registers, the template, the illustrative
      example, the gate table and the evidence section are in place and
      formatted. All 39 contract scenarios pass, alongside the pre-existing
      nextest cases and the doctest: `make test` reports
      `79 tests run: 79 passed, 0 skipped`, and `make fmt` completes with
      `Summary: 0 error(s)` and is idempotent. Committed as `261ebc3`; the six
      latent contract defects found on the way are in
      `Surprises & discoveries` and D23/D24.
- [x] EP-M1 — ADR 004 exists and both its registers are guarded.
- [x] EP-M2 — the template matches the status register; ADR 004 carries the
      illustrative example.
- [x] EP-M3 — gates and anchors are guarded.
- [x] Step 8 — the eight-item sync map is applied and committed as `1e1afd7`.
      Every companion document named in `Conformance basis` now points at
      ADR 004, the template or the notes directory, and every Step 8 gate was
      verified green before the commit. The roadmap's task 1.1.3 is ticked, and
      the two crossing guards that could have objected — the success-criterion
      `contains` check and the exit register's `GATES` list, which does not name
      1.1.3 — were both checked first.
- [x] EP-M4 — companion documentation is coherent and discoverable. Steps 8 and
      9's first run are recorded above; the nine `make lint` findings it
      surfaced are fixed and committed as `5bae2c1`, and the rerun is with
      `scrutineer`.
- [x] CodeRabbit pass one (F1, F2, F9) — the live-document needle guard and the
      roadmap's template link are fixed and committed as `956c912`, on top of
      the plan's mdtablefix reflow and MD038 fix.
- [x] CodeRabbit pass one (F3, F4, F5/F7, F6/F8) — the blocking findings are
      actioned: the scan is fallible and names the path it could not read; the
      citation predicate parses the whole `<repo>@<sha>:<path>` shape; the
      788-line root is split into four scenario modules; and a contradictory
      note is rejected rather than resolved, with `INV-CONSISTENCY` and its
      controls added to this plan. All 43 contract scenarios pass. Recorded as
      D26.
- [ ] EP-M5 — delivery: full gates, review, roadmap ticked.

Timestamps are added as each item completes.

## Surprises & discoveries

Findings from the planning phase and from implementation are recorded here
because none is derivable from the repository alone, and each changed the
design.

- Observation: Whitaker's `no_std_fs_operations` lint denies `std::fs` in
  integration-test crates, and **cannot be suppressed by any Rust attribute**.
  Evidence, measured on 2026-09-19 against `whitaker` with
  `DYLINT_LIBRARY_PATH` set to the installed suite. A one-line probe test
  calling `std::fs::read_to_string` was denied in every attribute form tried:
  item-level `#[allow(no_std_fs_operations)]`, item-level `#[expect(...)]`
  (which *also* raised `unfulfilled_lint_expectations`, because the `expect`
  suppressed nothing), and crate-level `#![allow(no_std_fs_operations)]` as the
  first line of the file. Only a `dylint.toml` entry suppressed it, and both
  forms worked: `excluded_crates = ["probe_std_fs"]` and the narrower
  `excluded_paths = ["probe_std_fs::notes"]`. The lint's own
  `crates/no_std_fs_operations/ui/` fixtures contain no suppression test, and
  the `users-guide.md` claim that "a standard `#[allow(no_std_fs_operations)]`
  attribute on the item or module also works" did not hold for an integration
  test. This is why the claim is recorded as measured behaviour rather than as
  a reading of the documentation. Impact: this plan's notes-directory scan
  needs a file *content* read, and neither remaining mechanism is free.
  `camino` enumerates (`read_dir_utf8`, `Utf8DirEntry::file_name`) but has **no
  content-read API** — `grep` over `camino-1.2.5/src/lib.rs` finds no
  `read_to_string`. So the scan must either (a) take a `dylint.toml` exclusion,
  adding a configuration file the plan's file list does not include, or (b) add
  `cap-std` as a dev-dependency and read through `cap_std::fs::Dir`, which
  measured at 45 → 84 packages. Both breach a constraint. **Resolved
  2026-09-19: option (a), in the path-scoped form**, so the exemption names one
  module rather than a whole crate; see Q5 and D20. Corroborating measurement
  worth keeping: the attribute-suppression failure is **suite-wide, not
  specific to this lint**. A probe carrying
  `#[allow(module_must_have_inner_docs)]` — a different lint in the same suite,
  needing no filesystem access — was also denied, at both item and crate scope.
  And plain `cargo clippy` reports `no_std_fs_operations` as an unknown lint,
  indistinguishable from a bogus name in the same position, which confirms the
  lint is registered only under the dylint driver and that the denial is real
  behaviour rather than a name-resolution artefact. The practical consequence
  is that the `addressing-whitaker-findings` skill's statement that crate-level
  `excluded_crates` is the *only* working escape hatch is too narrow: path
  scope works and is strictly narrower.

- Observation: `mdtablefix` merges an empty delimiter pair onto one line.
  Evidence: on the first `make fmt` run, ADR 004's four delimiter pairs were
  each rewritten from two adjacent lines into a single line carrying both
  comments (`<!-- status-register:begin --> <!-- status-register:end -->`).
  Impact: the plan's Step 2 ("delimiter comments but no register tables") would
  have produced `EmptyRegister` rather than the `MissingDelimiters` that Step 4
  predicts, because the merged line is non-empty but yields no rows. Decision:
  introduce each delimiter pair together with its content, in Step 5, so that
  Step 4's predicted red state holds.

- Observation: the question is about *stability*, not about *numbers*.
  Evidence: `docs/roadmap.md:204` asks whether "a stable identifier is needed";
  design §6.1 says "stable numeric discriminant or other low-cardinality
  identifier"; design §9 declares the field names "public operational API".
  Impact: the verdict axis records a required *property*, not an operation.
  Without this the most likely real finding is unrecordable.

- Observation: `std::mem::discriminant` cannot supply a stable numeric
  identifier. Evidence: the standard library's stability clause, and the
  undefined behaviour of transmuting `Discriminant<T>` to a primitive. Impact:
  any identifier is hand-assigned, hence new public API; recorded in ADR 004's
  "Known risks and limitations".

- Observation: a numeric identifier cannot reduce observability cardinality.
  Evidence: both map the same state domain; Prometheus and OpenTelemetry frame
  cardinality as a property of the value set. Impact: cardinality gates
  admissibility and never decides the verdict.

- Observation: adding a row to `docs/design.md` §11.1 is *not* inert, despite
  `has_table_bet` being a presence check. Evidence:
  `tests/v0_1_exit_register_contract.rs:119-126` and `:274-290` embed the B1
  row byte-exactly with its trailing padding; `mdtablefix` repads columns to
  `max(content) + 2`. Impact: Q1 carries a width budget, and the plan's first
  draft asserted a safety property it had verified against the wrong function.

- Observation: a control that mutates a document is only as good as its needle,
  and a line break can split one. Evidence, in two forms. First, against a
  *live* document: `mdtablefix --wrap` reflows prose and moves its line breaks,
  so a `.replace` needle spanning a wrap point stops matching at the moment the
  document is formatted. The control then asserts a failure that the check no
  longer produces, and passes for the wrong reason or fails for a confusing
  one. This happened three times in Step 7's `quoted_passages_still_resolve`
  alone — the needle shrank from `"consumes something stronger"` to
  `"consumes something"` (which produced the nonsense clause "made-up clause
  stronger") to the single word `stronger` — and four of the seven
  `committed_state_name_notes_are_rejected` cases failed the same way, their
  needles split by the source's own line continuations inside a `format!`
  string. Impact: two remedies, both now in the contract. Every control over a
  live document routes through a `mutated(source, needle, replacement)` helper
  that asserts the needle is present before substituting, which turns a silent
  no-op into a loud failure naming the reflow that broke it. And the fixture
  tables are assembled from `[...].join("\n")` arrays of `#[rustfmt::skip]`'d
  row constants, so a note in a rejecting case is built from whole rows rather
  than by excising text from an assembled one, and no needle has to survive a
  line break at all.

- Observation: the contract's own total-cover check could not see past its first
  matching row. Evidence: `aggregation_register_is_total` selected its row with
  `rows.iter().find(|row| row.admissible_notes == notes).filter(|row| notes ==
  "None" || row.any_insufficient == insufficient)`.
  `find` returns the first row whose first column matches, and `filter`
  applied to a single `Option` can never reconsider that choice, so for
  `One or more` notes the second column was never examined and the assertion
  could only be satisfied by whichever row happened to come first. Impact: the
  two conditions are now one predicate. This was a defect in the *check*, not
  in the document — and a check that cannot fail for the reason it names is
  precisely the vacuity this plan's verification section exists to prevent,
  which is why it is recorded here rather than silently repaired.

- Observation: the quoted-clause resolver compared a clause's attribution word
  against a file path. Evidence: `SOURCES` held `("design", "docs/design.md")`,
  and the lookup was `position(|(_, path)| *path == document)` — comparing the
  name the ADR uses, `design`, against the path it stands for. Every
  well-formed clause was rejected. A second defect sat in the same function:
  `resolve_clause` split the attribution on its first space *before* stripping
  the code spans, so `` `design` `6.1 State naming` `` yielded the document
  `` `6.1 ``. Impact: the lookup is keyed on the attribution name, the spans
  are stripped before the split, and the failure message now names the *file*
  the reader must open rather than the attribution word, because the two differ
  by a path.

- Observation: the marker that identifies a validation note must be matched as a
  line, not as a substring. Evidence: `docs/validation-notes/README.md`
  documents the marker inside a code span, so a substring test read the README
  as a committed note and then rejected it for carrying no note register.
  Impact: the marker must be a line of its own. A residual defect of the same
  shape remains, and is noted here rather than fixed: `parse_table` names
  `Register::Note.document()` — the template — inside the message it produces
  while reading a note, so a malformed committed note composes as
  `2.2.1-mdtablefix.md: docs/phase-2-validation-note-template.md: no note
  register found …`.
  The caller's `{name}:` prefix still leads with the file to open, and no
  input reaches it today: `docs/validation-notes/` holds only its README, whose
  marker sits inside a code span rather than on a line of its own, so the scan
  yields no notes. The first note to arrive malformed will read oddly, and
  repairing that means giving the message a name for the document being read
  rather than a fixed path — a change to `parse_table`'s error construction,
  not to a document.

- Observation: a register's header row participates in row identification, so a
  header reproduced inexactly is reported far from the edit that caused it.
  Evidence: `parse.rs::expected_header` recognized `| ... | Outcome |` while
  the live aggregation table's third column is headed
  `Outcome if publication proceeds`. The mismatched header is not structural,
  so it is parsed as a data row and then rejected as malformed several checks
  later. Impact: the exact header is now carried in one place in the contract,
  with a doc comment stating why. The same hazard caught the status register,
  where the fixture's row said `Overridden` for a field the live document
  records as `Not a named type`.

- Observation: an embedded note's H1 collides with the host document's.
  Evidence: `make fmt` failed with `MD025/single-title/single-h1` on
  `docs/phase-2-validation-note-template.md`, which embedded a complete note —
  including its `# Validation note: ...` heading — as literal Markdown, and a
  document may carry only one H1. Impact: the copyable form is fenced as
  ```` ```markdown ````, which satisfies MD025 and has the incidental virtue of
  making the copy boundary visible to the reader.

- Observation: a cell recording that no consumer exists is still an observation,
  and it still needs to say where it was made. Evidence: the fixture gave
  `identifier-need` and `tracing-use` prose cells — "subscriber, metrics, model
  checker and generated documentation considered" and "emits
  `transition.state.before`" — and every note-derived assertion failed on the
  citation check before reaching the property under test. Impact: both cells now
  carry a `<repo>@<sha>:<path>` citation alongside their prose. ADR 004 requires
  a citation of every field, including one whose status is `None`, and the
  requirement is doing its job.

- Observation: `clippy.toml`'s `allow-expect-in-tests` reaches `#[test]` bodies
  and not the helper functions they call. Evidence: Step 9's first `make lint`
  run failed with nine findings, two of them `expect_used` on `.expect()` calls
  sitting in `fn blocked_by(...) -> Result<Resolution, String>` and
  `fn gate_table_fragment(gate: &str) -> String` — both helper functions called
  from tests, neither a test itself. The flag is documented as allowing `expect`
  in tests; its actual scope is the `#[test]`-annotated item. Impact: both
  helpers now return `Result` and report the parse failure in the caller's error
  channel, which is also the better behaviour — the failure they would have
  panicked on is a real answer, and it belongs where the caller can name the
  fixture it came from. Recorded because the flag's scope is invisible at the
  call site: a helper that `expect`s compiles, reads as test code, and fails
  only at the gate.

- Observation: `make lint` runs clippy before Whitaker, so a clippy failure
  leaves the `dylint.toml` exemption entirely unexercised. Evidence: the first
  Step 9 run aborted at clippy with nine errors, and the log contains zero
  matches for `whitaker`, `no_std_fs`, or `dylint` — the exemption added under
  D20 had never been validated by a gate run at any point before Step 9's
  second attempt. Impact: none for this plan, since the second run exercises it;
  recorded because it means "`make lint` is green" is *not* evidence that a
  lint exemption works, and the plan's quality criterion assumed it was.

- Observation: an unread note is an unguarded note, and nothing reports it.
  Evidence: the scan selected candidate files with
  `entry.file_name().ends_with(".md")`, which clippy flagged as a case-sensitive
  extension comparison. The flagged form is not merely untidy: a note committed
  as `2.2.1-mdtablefix.MD` would be skipped silently, and no check would notice,
  because a skipped note produces exactly the same result as no note at all —
  an empty vector and a passing scan. Impact: the extension is now compared
  case-insensitively, off the path rather than off the file name's tail.
  Recorded because the failure mode is silence, which is the one direction the
  scan's own design makes invisible.

- Observation: a negative control can be defeated by the fixture it mutates, and
  the defeat is silent in the same direction as the defect it tests for. Three
  of the fixes in D26 hit this. `is_citation_shaped`'s message promised
  `<repo>@<sha>:<path>` while the predicate accepted any token containing an
  `@`, so `#[case::citation_without_a_path]` adds no coverage the old predicate
  lacked — it would have passed for the wrong reason, asserting a message about
  a shape nothing checked. The contradiction control's first draft asserted a
  message naming the same field twice, because it attributed a second decisive
  status to a field whose contribution the note did not select. And the
  `blocked_notes_resolve_to_not_resolved` control's middle case used a status
  the register does not define, so it failed on the *vocabulary* check rather
  than the admissibility check it exists to exercise. Impact: the contradiction
  control now asserts its register is still accepted by every document-level
  check before asserting the rejection, and carries two controls proving the
  register still resolves an agreeing note and still blocks on an inadmissible
  cell — so a guard rejecting every multi-decisive note fails, and one reaching
  the contradiction branch before the admissibility branch also fails.
  Recorded because the plan's own non-vacuity rule is what caught all three,
  and because "the control passes" was, in each case, not evidence until the
  control's precondition had been shown to hold.

- Observation: the 400-line cap is not a style preference; it caught a real
  defect in the review. Evidence: the crate root had reached 788 lines against
  AGENTS.md's cap, and both CodeRabbit findings that flagged it (F5 and F7)
  named the same number. The plan's own constraint 7 repeats the cap, so the
  contract was in breach of a constraint it declared. Impact: the scenarios are
  now four child modules, and the root is 87 lines. Recorded because the breach
  was invisible in every gate: `make test` passed, `make check-fmt` passed, and
  the file's own module doc never mentioned its size. Only a review that counts
  found it, which is the argument for the review existing.

- Observation: `make check-fmt` is not satisfied by prose a human has wrapped
  *below* 80 columns; `mdtablefix --wrap` fills prose to 80 and will reflow
  anything narrower. Evidence: `make check-fmt` failed on this plan at the
  paragraph introducing the traced-items table —
  `docs/execplans/1-1-3-...md +4 -4`, `1 file would be reformatted` — and every
  offending line belonged to the paragraph added in the same commit. Measuring
  rather than guessing was the point here, because the first two explanations
  of the cause were both wrong. The line lengths were `71, 66, 70, 75, 77, 30`;
  a greedy fill at
  80 over the same words yields `78, 59, 79, 77, 80, 17`. So the formatter is
  not narrower than `MD013` — it is exactly as wide, and it pulls words *up*
  from the short lines rather than breaking any. A paragraph whose lines merely
  look conventional fails. Impact: none on the document's content; the reflow
  is whitespace-only and `make check-fmt` is idempotent afterwards. Recorded
  because the failure is trivial to misattribute in the other direction: since
  `make fmt` fixes it silently, the temptation is to treat the red as noise.
  But a docs-only diff that fails `check-fmt` is exactly what "the gates must
  succeed before a review is requested" exists to stop, and that instruction's
  warning against handing a reviewer a deterministic failure applies to prose
  wrapping as much as to a type error. The practical rule: write prose to a
  single short line per sentence and let `make fmt` set the wrap, rather than
  choosing a width by hand.

- Observation: a `mdtablefix` probe run with `--diff` alone is **vacuous**,
  because `--diff` does not enable `--wrap`; the rule flags are separate and
  must be repeated. Evidence: while investigating the entry above, three
  successive `mdtablefix --diff <file>` probes reported "1 file left unchanged"
  for input that the real gate rejects, which prompted a bisect by prefix
  length before the discrepancy was located in the invocation rather than in
  the file. Re-running with `$(MDTABLEFIX_RULES)` reproduced the gate's finding
  immediately and at every prefix length. The `--check` form behaves the same
  way: it checks only the rules it is given. Impact: none on any artefact; the
  document was already correct. Recorded because it is the same class of error
  as the vacuous controls this plan's `Verification plan` was written to
  prevent — a check that cannot fail for the reason it names — and it was made
  three times in a row against the tool being used to decide whether a gate
  result was real. The remedy is the same as for the controls:
  run the probe with the same arguments as the thing being investigated, not
  with arguments that merely look equivalent.

## Decision log

- D1: Define "validation note" as the record a validation task produces,
  instantiated from the template and committed to `docs/validation-notes/`.
  Rationale: the roadmap uses the phrase seven times without defining it, and
  without a location gates S1 to S3 are unenforceable. Date/Author: 2026-09-18,
  planning agent.

- D2: Two documents, with the template *generated* from the ADR's field
  register and checked by byte equality. Rationale: differing lifecycles argue
  for two files; generation removes the hand-maintained drift edge that argues
  against them. One parser fewer. Date/Author: 2026-09-18, planning agent.

- D3: Separate admissibility from verdict; the verdict register has one axis.
  Rationale: a two-axis register spent two of its four cells on "naming
  defect", which is a refusal to answer rather than a verdict, and left the
  procedure non-terminating over half its domain. With cardinality moved to
  admissibility, the exclusion rules become unrepresentable rather than
  policed. Date/Author: 2026-09-18, planning agent, after design review.

- D4: Read "metrics cardinality" as the bound on the set of names the code can
  emit, not as an observation from a metrics backend. Rationale: `mdtablefix`
  installs no recorder, so the literal reading is unanswerable and the field
  would read "not exercised" every time. The chosen reading is determinable
  from the state type and still detects the dangerous case of a name
  synthesized from data. Referred to the approval gate as Q0 because it
  reinterprets a roadmap-named field, and accepted there on 2026-09-18.
  Date/Author: 2026-09-18, planning agent.

- D5: Name the verdict axis by the requirement (`Property required`), not by
  the existence of a consumer. Rationale: `wireframe`'s Stateright model is a
  named consumer that does *not* establish a need. An axis labelled "named
  consumer" would be ticked there and produce the wrong verdict. The token must
  carry its own test. Date/Author: 2026-09-18, planning agent, after design
  review.

- D6: Use one token vocabulary across the whole record. **Superseded by D13**,
  which achieves this by merging the registers rather than by keeping two in
  step. Rationale: with `None observed / Named consumer` against
  `Absent / Present`, no document records which maps to which, so a "bijection"
  check degrades to a hard-coded translation table — a control tautological
  with its own constant. The second draft accepted this decision and then
  violated it anyway, which is the evidence that two registers cannot reliably
  be kept in step by intent. Date/Author: 2026-09-18, planning agent, after
  design review.

- D7: Bind gates by roadmap task *title fragment*, not task number.
  Rationale: the existing contract requires bound tasks to be present and
  unticked, which means completing a bound task breaks the build. Binding four
  more numbers would freeze seven roadmap numbers and collide with the
  project's own `mapsplice` tooling, whose purpose is renumbering. Title
  binding keeps the reference meaningful without freezing anything, and does
  not fire when a gate succeeds. Date/Author: 2026-09-18, planning agent, after
  design review.

- D8: Add an aggregation register for the multiset of notes.
  Rationale: three notes reach 3.2.1 and nothing combined them. Exhaustive
  totality over one note's inputs is a proof about a domain the deciding gate
  does not inhabit. Date/Author: 2026-09-18, planning agent, after design
  review.

- D9: No `insta`, `kani`, `verus`, `proptest`, `rstest-bdd`, or
  `cargo-mutants` for this task. Rationale: `insta` — the multivariant output
  is the repair-message set, and exact literals protect it as well while being
  un-blessable. `kani` — the bounded domains here have two and three elements
  and are already exhausted by case tables. `verus` — the load-bearing
  proposition behind the cardinality argument is a claim about observability
  backends and about a function this repository does not contain; encoding it
  would prove that an injective function has an image the size of its domain, a
  restatement of injectivity and precisely the vacuous proof the standard
  forbids. `proptest` and `rstest-bdd` — see Q3; measured at 45 to 185
  packages. Date/Author: 2026-09-18, planning agent.

- D10: One generic delimited-table parser, with typed mappers layered over it.
  Rationale: the existing contract conflates syntax and typing in one parser,
  which is where its repair commits landed. Three registers plus a note format
  share one syntax; only the mapping differs. Date/Author: 2026-09-18, planning
  agent, after design review.

- D11: Read the roadmap's "optional identifier need" adjectivally — the
  *need* is optional, the *field* is mandatory. Rationale: the alternative
  reading makes the field omissible, which defeats the instrument. Recorded
  explicitly because a sign-off reviewer reading "optional" as "may be omitted"
  will collide with the closed vocabulary. Date/Author: 2026-09-18, planning
  agent, after design review.

- D12: On Q1, add the `docs/design.md` §14 bullet in this task and raise bet
  B7 separately, using the row drafted in Q1. Rationale: §14 "Deferred
  decisions" is a better fit than §11.1 — it is literally the list of decisions
  to resolve before publishing v0.1, it already carries a `StateName` item, and
  the return shape is absent from it. The bullet is one line and verified
  inert, because the existing contract reads §14 only as a heading-string
  relocation anchor. B7 remains worth filing, and its safety objection is
  discharged by measurement, but it is upstream design prose this task did not
  ask for. Note that the wording proposed in this plan's first draft is stale:
  it said "Two validation notes" where the gate table names three recording
  gates, and it was written against the superseded named-consumer axis. Q1
  carries the corrected wording. Date/Author: 2026-09-18, planning agent, after
  design review.

- D13: Merge the field register, its admissibility column, and the verdict
  register into one status register keyed on `(Field, Status)`. Rationale: the
  second draft's field register recorded which *field* gates admissibility but
  never which *status* blocks, so `resolve_note` had to hardcode
  `Not a named type` and `Unbounded`. That is exactly the constant tautological
  with its own control that D6 removed from the verdict axis, reintroduced one
  level down. The second draft also violated D6 outright: its field register
  said `None / Property required` while its verdict axis said `No / Yes`, and
  its blocking status was written `Not a type` in one table and
  `Not a named type` in an invariant — mismatches that byte-exact fixtures
  would have frozen into the contract. One table with one vocabulary makes both
  admissibility and verdict derivable from the document. Date/Author:
  2026-09-18, planning agent, after design review.

- D14: `state-display-name` records the enumerated set of returned strings.
  Rationale: the second draft's statuses recorded only whether a named type
  existed, so the note never captured the actual labels. That left
  `metrics-cardinality: Bounded` as an unaudited assertion by the note's
  author, and meant a reviewer at task 3.2.1 would decide the fate of a
  `&'static str` without seeing a single string. It also failed the roadmap
  noun it was mapped to. Enumerating the names makes the cardinality bound
  derivable and gives Finding one's stability argument concrete labels.
  Date/Author: 2026-09-18, planning agent, after design review.

- D15: `INV-FILLED` keys on a `<!-- state-name-note -->` marker, not on a glob,
  and an empty `docs/validation-notes/` is not a failure. Rationale: the
  directory is shared. Roadmap task 1.2.3's benchmark note, task 2.2.3's exit
  note, and task 3.1.3's decision note are all validation notes and none is a
  `StateName` note; a glob would make each one's arrival a build failure.
  Separately, the second draft required a filled note to be committed now as
  the suite's accepting witness — but no honest note can exist before task
  2.2.1 has annotated anything, so its citations would have been fabricated
  while modelling the standard for Phase 2. The witness is a string fixture;
  the illustrative example lives in ADR 004 marked as illustration.
  Date/Author: 2026-09-18, planning agent, after design review.

- D16: `INV-TEMPLATE` compares parsed values, not bytes.
  Rationale: the second draft rendered the template and asserted byte equality,
  which would have required the renderer to reproduce `mdtablefix`'s
  `max(content) + 2` canonicalization at test time, with no "format then copy"
  escape. That welded the suite to a third-party padding algorithm to guard a
  property that does not depend on padding. Date/Author: 2026-09-18, planning
  agent, after design review.

- D17: The plan was approved on 2026-09-18 with all five referred decisions
  settled as recommended — Q0 accepted, Q1 taking the `docs/design.md` §14
  bullet in this task with bet B7 deferred to a separate change, Q2 keeping two
  files, Q3 declining both dev-dependencies, and Q4 adding the discoverability
  pointers. Rationale: recorded here so that a later reader can tell which
  parts of the design were chosen by the planning agent and which were ratified
  by the approving authority, and so that reopening any of them is visibly a
  change of decision rather than a fresh choice. Date/Author: 2026-09-18,
  approved by the project owner.

- D18: **Blocked on 2026-09-19; lifted by D20 the same day. Superseded in
  outcome by D20, retained as the record of why the work stopped.** The
  notes-directory scan needs a file *content* read, which `camino` cannot
  perform and Whitaker forbids in test crates, in a form no Rust attribute can
  suppress. Measured 2026-09-19; the evidence is in `Surprises & discoveries`
  and the options are in Q5. Rationale for stopping rather than choosing: both
  remedies breach a standing constraint — option (a) adds a file outside this
  plan's "Files this plan reads or writes" list, option (b) adds a
  dev-dependency against constraint 3 and the settled Q3 precedent, which
  declined two dependencies on a measured 45 → 185 graph increase. The plan's
  own tolerance rule 2 states that any dependency addition stops the work, and
  its exception procedure requires an explicit direction rather than a
  workaround. No artefact has been fabricated to route around the finding: the
  contract test is not yet written, so nothing depends on the choice.
  Date/Author: 2026-09-19, implementing agent.

- D19: Introduce each delimiter pair together with its register content, not
  before it. Rationale: `mdtablefix` merges adjacent delimiter comments onto
  one line, so Step 2's delimiters-without-tables would have produced
  `EmptyRegister` and falsified Step 4's `MissingDelimiters` prediction. See
  `Surprises & discoveries`. This is a mechanical change to the order of two
  steps; it alters no requirement and no architecture. Date/Author: 2026-09-19,
  implementing agent.

- D20: **Q5 is settled as option A — a path-scoped `dylint.toml` exemption —
  and D18's block is lifted.** The approving authority directed the
  `excluded_paths` form explicitly, in preference to the crate-wide
  `excluded_crates` form the first draft of Q5 named. Rationale:
  `excluded_paths` is the narrower instrument — it exempts one named module and
  its descendants rather than a whole test crate, so the parsers, policies,
  fixtures, and `INV-*` checks of this contract test all remain under the lint.
  That property is what makes the exemption acceptable despite the plan having
  been approved without it: the deviation introduces a new tracked file and a
  visible exemption in the enforcing configuration, but it does not withdraw
  lint coverage from the code that carries this task's logic. Measured working
  on 2026-09-19 in the form `excluded_paths = ["<crate>::<module>"]`; see
  `Surprises & discoveries`. Impact on the approved plan: `dylint.toml` joins
  "Files this plan reads or writes", a new constraint 10 confines the exemption
  to one module, the test module set grows by `notes.rs` (its own module,
  precisely so that the exempted path is one module and not the crate), and
  Q5's option B (`cap-std`) is declined, leaving constraint 3 and D9's
  dependency position exactly as approved. This also settles a second-order
  point worth recording: the `addressing-whitaker-findings` skill asserts that
  crate-level `excluded_crates` is the *only* working escape hatch, which the
  measurement contradicts. The skill is corrected as a separate change; this
  plan records the measurement rather than the skill's claim. Date/Author:
  2026-09-19, approved by the project owner; recorded by the implementing agent.

- D21: The contract test is six modules plus the crate root, not the five
  modules "Interfaces and dependencies" declares, and `Register` gains a fifth
  variant `Evidence`. Rationale: two boundaries the plan drew in the abstract
  proved wrong against the code. First, `policy.rs` reached 597 lines — a
  module owning the note verdict, the register's consistency *and* the quoted
  clauses — which breached tolerance 5's 300-line trigger for a named module
  had the split been kept. The re-planned split gives each module one invariant
  class rather than an even share of the text: `policy.rs` keeps admissibility
  and verdict, `clauses.rs` takes quoted-clause resolution, and `registers.rs`
  takes the roadmap binding and the cross-register checks. Second, the quoted
  clauses are a delimited register like the other four, so
  `check_quoted_clauses` needs a `Register` token to say which document and
  section it failed in; a fifth variant is cheaper and more honest than a
  second parallel error type. Both changes are internal to the test crate.
  Neither alters a requirement, a register's field set, a repair message's
  obligation, or any document this plan ships, and tolerance 5's *purpose* — no
  module growing past 300 lines unnoticed — is met with margin. A third,
  smaller narrowing is recorded here rather than as its own entry: the plan's
  `EmptyRegister` variant was dropped in favour of `MissingDelimiters`, because
  an empty block and an absent one demand the same repair and a distinct
  message would have been a third way to fail at the same thing. Date/Author:
  2026-09-19, implementing agent.

- D22: Step 4's red-state prediction is corrected: `INV-FILLED` passes an empty
  notes directory rather than failing it, and the plan as drafted said the
  opposite. Rationale: the drafted sentence contradicted `INV-FILLED`'s own
  non-vacuity clause, which states that the accepting witness is a string
  fixture *so that* the check cannot pass merely because the directory is
  empty, and that an empty directory is explicitly not a failure because no
  honest note can exist before task 2.2.1 has annotated anything. One of the
  two had to give: either the check demands a committed note, or the red step
  expects a pass. The check is the load-bearing half — demanding a note now
  would mean demanding a fabricated citation, which is the failure D15 already
  removed once from this design. Correcting the prose rather than the code
  keeps the acceptance criterion honest and leaves `INV-FILLED`'s non-vacuity
  resting on the rejecting controls, where D15 put it. This is a
  mechanical correction to a prediction, not a change to a requirement or to an
  architecture. Date/Author: 2026-09-19, implementing agent. Count corrected
  from seven to eight by D26, which added
  `#[case::citation_without_a_path]`; the reasoning above is unaffected.

- D23: Fixture tables are assembled from row constants, and every control over a
  live document routes through a `mutated()` helper that refuses to apply a
  needle the document no longer contains. Rationale: the two halve the same
  hazard from opposite ends. The `mutated()` guard makes a stale needle loud,
  and the row constants mean the controls that mutate *fixtures* never need a
  needle long enough to be fragile in the first place — a rejecting case builds
  a note from the rows it wants, rather than excising a phrase from a note that
  was assembled for a different purpose. Both were forced by measurement rather
  than foreseen: `mdtablefix --wrap` moved a line break under three successive
  needles in a single control. This is a test-internal restructuring. It changes
  no requirement, no register, no document this plan ships, and no repair
  message's obligation — the messages the controls assert are the messages the
  tests carry either way. Date/Author: 2026-09-19, implementing agent, after the
  Step 7 gate.

- D24: The status and aggregation fixtures track the live documents' exact
  spellings, and the note fixtures carry a citation in every evidence cell.
  Rationale: both were the contract disagreeing with the document it guards
  rather than a document defect, and in both cases the contract was wrong.
  `status_register()` said `Overridden` where the ADR says `Not a named type`,
  and the aggregation header omitted `if publication proceeds`. The first had
  been introduced deliberately, to dodge a mutation collision with the
  `| Enumerated |` row; the collision is real but the remedy was misdirected —
  the control that needed the dodge now mutates the row constant directly, so
  the collision is dodged without the fixture diverging from the document it
  mirrors. Recorded rather than silently corrected because a fixture edited to
  match its document is indistinguishable in the diff from a fixture edited to
  match a defect, and the distinction is the whole value of the contract.
  Date/Author: 2026-09-19, implementing agent, after the Step 7 gate.

- D25: The nine `make lint` findings from Step 9's first run are fixed in the
  contract's own source, with no `#[allow]` added and no exemption widened.
  Rationale: the findings were genuine defects of the code that carried them,
  and two of them named the dangerous direction rather than the untidy one — the
  case-sensitive extension comparison would have skipped an unread note
  silently, and the two `expect` calls sat in helpers whose failure belongs in
  the caller's error channel. The `dylint.toml` exemption is unchanged. Recorded
  because the first `make lint` run is also the first run in which Whitaker
  executed at all: clippy runs first in `make lint`, so the exemption's
  behaviour is not merely unvalidated until a clippy-clean run exists — it is
  *unexecuted*, and an earlier green claim from a partial gate would have been
  true of a gate that never reached the lint it names. Date/Author: 2026-09-19,
  implementing agent, after the Step 9 rerun was dispatched.

- D26: **The first CodeRabbit pass over the contract found nine issues; four
  were blocking, and the fixes are recorded here.** F1/F9 (a live-document
  needle that `mdtablefix --wrap` can split) and F2 (an undeclared
  `docs/phase-2-validation-note-template.md` link in the roadmap) were fixed
  and committed before this entry; the remaining four are these. **F3**:
  `committed_notes` discarded read and enumeration failures, so an unreadable
  note was skipped silently — the exact hazard `notes.rs`'s own doc comment
  names, one level down from the marker rule. Both the scan and the read now
  return `Result`, an absent directory still yields no notes, and
  `scan_scenarios.rs` forces the read failure by passing a directory where a
  file is expected. Failing on permissions instead would have required a write
  from a module constraint 10 does not exempt, and would pass vacuously where
  the suite runs as root. **F4**: `is_citation_shaped` accepted any token
  containing an `@`, while its message promises `<repo>@<sha>:<path>`; the
  predicate now parses the full shape with all three components non-empty, and
  `#[case::citation_without_a_path]` is the eighth rejection case.
  **F5/F7**: the crate root had reached 788 lines against AGENTS.md's 400-line
  cap. It is now 87 lines, with the scenarios in four child modules
  (`anchor_scenarios.rs` 223, `note_scenarios.rs` 271, `register_scenarios.rs`
  284, `scan_scenarios.rs` 135). The four classes were chosen so each owns a
  question rather than a slice of the file: what the *template and roadmap*
  say, what a *note* says, what the *register* says, and what the *scan* reads.
  **F6/F8**: `contribution` collapsed two different non-`nothing` contributions
  into the stronger one, resolving exactly the note ADR 004 says "is rejected
  rather than resolved" — and resolving it silently, toward the verdict that
  overturns the default. It now returns the rejection, and
  `contradictory_notes_are_rejected` proves both halves: a register with a
  second decisive field is still a working register when its cells agree, still
  blocks on an inadmissible cell, and *rejects* when they contradict.
  Tolerance 5's 300-line trigger was reached twice while fixing these —
  `policy.rs` at 319 and the root at 788 — and met both times by re-planning
  the split rather than by tolerating the growth, per D21's precedent. The
  register-consistency checks (`check_exclusions`, `check_vocabulary`) moved
  from `policy.rs` to `registers.rs` because both are claims about the
  *register* rather than about a note read through it, which also answers F6/F8
  at the right level: the guard against contradiction belongs with the note,
  and the guard that the register has one power to overturn the default belongs
  with the register. No requirement, register field, repair-message obligation,
  or shipped document changed; `docs/repository-layout.md` and
  `docs/developers-guide.md` were updated because both enumerate the child
  modules. Date/Author: 2026-09-20, implementing agent, actioning the review the
  scrutineer returned after D25.

- D27: The EP-M5 gate run failed `make check-fmt` on this plan, and the finding
  is a **prose-wrapping rule rather than a defect**, so the remedy is to state
  the rule rather than to change a requirement. `mdtablefix --wrap` fills prose
  to 80 columns, and this plan's paragraph introducing the traced-items table
  was hand-wrapped to 77 at its longest line. The gate report named
  `+4 -4` on that paragraph alone, and the measured line lengths before and
  after (`71, 66, 70, 75, 77, 30` → `78, 59, 79, 77, 80, 17`) show the
  formatter pulling words *up* rather than breaking lines down: it is exactly
  as wide as `MD013`, not narrower, and a paragraph whose lines merely look
  conventional fails. `make fmt` applied the reflow, and `make check-fmt` was
  then idempotent on a second run. Rationale for recording it at all: the
  failure is misattributable in the harmless-looking direction — since
  `make fmt` repairs it silently, the temptation is to file the red as noise —
  and the standing instruction requires the deterministic gates to be green
  before a review is requested, which a docs-only diff does not exempt itself
  from. Two by-products are recorded under `Surprises & discoveries`: the first
  two explanations of the cause were both wrong, and three `mdtablefix --diff`
  probes said "unchanged" for input the gate rejects, because `--diff` does not
  imply `--wrap` and the rule flags must be repeated on the probe. No
  requirement, register, invariant, repair message, or shipped document
  changed; the diff is whitespace inside one paragraph of this plan.
  Date/Author: 2026-09-20, implementing agent, after the EP-M5 re-gate.

## Outcomes & retrospective

### What was delivered

Roadmap task 1.1.3 is ticked and linked. ADR 004 defines the `StateName`
consumption evidence; `docs/phase-2-validation-note-template.md` is the form a
Phase 2 engineer copies; `tests/state_name_consumption_contract.rs` and its
eleven child modules guard both against drift. The task's own success criterion
is itself checked, so the instrument is bound to the sentence that grades it.

The task's stated purpose was to make task 3.2.1's instruction executable. A
Phase 2 engineer now has a form to fill, a rule that turns the filled form into
a verdict, and a committed worked example showing what an adequate evidence
cell looks like. What does **not** exist, deliberately, is the verdict itself:
ADR 004 records evidence and the rule for reading it, and leaves "is
`&'static str` sufficient?" to task 3.2.1. Constraint 2 forbids the answer, and
the aggregation register is shaped so that the answer arrives as a row
selection rather than as prose.

### Reconciliation of discoveries against the conformance basis

Every entry in `Surprises & discoveries` was checked against the artefacts
named in `Conformance basis`. All twenty were accounted for; the disposition
of each follows. The last two were recorded during the EP-M5 gate run itself
and are mechanical — one is a prose-wrapping rule, the other a correction to
how this plan had been probing the formatter — so neither bears on any upstream
artefact.

**Falsified an upstream premise; upstream amended in this task.**

- The `mdtablefix` empty-delimiter-merge, the §11.1 repadding hazard, the
  needle-versus-reflow finding, and the H1 collision all bear on
  `docs/design.md` §11.1 and on how the two new documents are formatted. None
  falsifies §6.1's premise — that `&'static str` is the default until a real
  example consumes something stronger — and §6.1 is therefore unchanged except
  for the pointer sentence Step 8 added. The §11.1 hazard was *avoided* rather
  than amended: constraint 5 forbids touching that table, and bet B7 is
  deferred to a separate change precisely so the repadding trap is met with the
  width budget Q1 recorded rather than by accident.

**Mechanical differences, recorded in `Decision log`.**

- The `no_std_fs_operations` suppression measurements (D18, D20) changed which
  mechanism the exemption uses, not what any document requires.
- The `allow-expect-in-tests` scope finding and the `make lint` ordering
  finding changed two helper signatures and the order of evidence collection
  (D25). No requirement moved.
- The contract-internal defects — `find`-then-`filter`, the attribution-versus-
  path comparison, the header-row identification, the missing-citation cells —
  were defects in checks the plan had already specified. Each was repaired
  toward the plan's stated intent, and each is recorded rather than silently
  fixed because a check that cannot fail for the reason it names is the vacuity
  the `Verification plan` exists to prevent.
- The 400-line breach and the CodeRabbit findings that surfaced it changed the
  *file layout* (D26), not the invariants, the register, any repair message's
  obligation, or any shipped document other than the two that enumerate child
  modules.

**No effect on the conformance basis.**

- The stability-not-numbers reading (D4), the `std::mem::discriminant` finding,
  and the cardinality finding (D11, D17) were settled at the approval gate as
  Q0. They shaped the register before it was written; the register as delivered
  matches the approved reading, so nothing upstream needs amending.

### Upstream artefacts left alone, and why

- **`docs/design.md` §11.1** is byte-identical to its pre-task state except for
  the added §14 bullet elsewhere in the file. The bet table gains no row.
- **ADR 001's outstanding decision** is *not* resolved by this work, which is
  the point: task 1.1.3 prepares the evidence 3.2.1 will read. ADR 001 needs no
  amendment, and none was made.
- **`docs/terms-of-reference.md`** is untouched. No TOR assumption was
  falsified — the findings above concern how the repository's own tooling
  behaves, not what the project is for.
- **ADR 002 and ADR 003** are untouched, per constraint 4. Finding four is
  recorded *inside* ADR 004, and the exit register's gate list does not name
  1.1.3, which was checked before Step 8 rather than after.

### Lessons

Four, each of which cost something measurable.

**A passing control is not evidence until its precondition is shown to hold.**
Three of D26's fixes failed on this, and the failure is silent in the same
direction as the defect under test: an unsatisfiable needle makes a negative
control assert a message for a defect it no longer introduces, and the test
goes on passing. The remedies now in the contract — the `mutated()` guard that
refuses a stale needle, and the contradiction control's three supporting
assertions that its register is still usable — are the general form: assert the
precondition, then assert the rejection.

**A check can be vacuous without being empty.** `find`-then-`filter` selected
the first matching row and then could not reconsider it, so the assertion could
only be satisfied by whichever row came first. The check had a name, a message
and a passing case, and could not fail for the reason it named. Reading a
predicate for whether it can fail is a different exercise from reading it for
what it asserts.

**Two counts that are easy to conflate, and both are needed.** Nextest reports
collected *cases*; the plan names *scenarios*. `rstest` expands three of them,
so 43 cases across 29 functions. A gate that silently stopped collecting a
scenario would move the case total without moving the scenario list, and only
the case total notices. This plan now records both.

**The 400-line cap earns its keep.** It was breached invisibly: `make test`
passed, `make check-fmt` passed, and the file's own module doc never mentioned
its size. Only the review counted. A size constraint is a proxy for a property
that the gates do not otherwise observe, which is exactly when a proxy is worth
having and exactly when it looks like bureaucracy.

### Residual gaps, stated rather than implied

- **`parse_table` names a fixed path in a message about a variable file.** A
  malformed committed note composes as
  `2.2.1-mdtablefix.md: docs/phase-2-validation-note-template.md: no note
  register found …`. No input reaches it today — the notes directory holds only
  a README whose marker sits inside a code span — and the caller's `{name}:`
  prefix still leads with the file to open. Repairing it means giving the
  message a name for the document being read; recorded in `Surprises &
  discoveries` rather than fixed, because it is a change to error construction
  rather than to a document.
- **The gate table binds by title fragment, not by task number.** Completing a
  bound task therefore does not break the build, which is intended; the cost is
  that a *reworded* title breaks it, and the ambiguity control is what makes
  that failure legible rather than mysterious.
- **The three `docs/validation-notes/` notes this contract expects** — 1.2.3's
  benchmark note, 2.2.3's exit note, 3.1.3's decision note — do not exist yet.
  The marker rule means their arrival is not a build failure, and the scan's
  accepting witness is a string fixture rather than a committed file, so none
  of them is required for this task. The first malformed `StateName` note is
  the first real exercise of the filler's side of the workflow.

## Verification plan

This change adds no runtime behaviour. It introduces ten non-trivial
propositions about documents and their relationships, and every one is
checkable.

**Design for falsifiability.** Documents with fixed paths are embedded with
`include_str!`. Notes under `docs/validation-notes/` are enumerated at test
time from `CARGO_MANIFEST_DIR` through `camino::Utf8PathBuf`, the idiom already
used by `tests/dev_fast_contract.rs:19`. Parsing is one pure function,
`parse_table(source, register) -> Result<Vec<Vec<String>>, ParseError>`, which
performs syntax only: locate the block between two HTML-comment delimiters,
reject structural header and divider rows by exact recognition, split on `|`,
trim. Typed mappers convert rows to register types; policy predicates operate
on typed rows. A failure is therefore attributable to one layer.

`ParseError` is keyed on a `Register` token rather than on a document string,
because ADR 004 carries more than one register and a message must say *which*.
The token yields document, begin marker, end marker, and section name through
`const fn` accessors, which is what makes every asserted repair message
derivable. Note also that the existing implementation's `line` field is
block-relative, not file-relative; this plan's messages say "row N of the
`<name>` register" rather than implying a file offset.

**Axioms**, assumed and not verified: the `std::mem::discriminant` clauses;
`tracing`'s value handling; Prometheus and OpenTelemetry cardinality guidance;
and `mdtablefix`'s canonical column width. The last is the only one
repository-owned logic depends on, and it is exercised against the real
formatter by running `make fmt` before fixtures are written and by
`make check-fmt` in every gate run.

### INV-CRITERION — the acceptance criterion still resolves

- **Obligation**: roadmap task 1.1.3's success bullet resolves verbatim
  (whitespace-folded) in `docs/roadmap.md`, and each of its four nouns — state
  display name, identifier need, metrics cardinality, tracing use — maps to
  exactly one field identifier in ADR 004's status register.
- **Method**: parameterized test, one case per noun.
- **Rationale**: this is the one thing the task is graded on, and no other
  invariant touches it. It is also the cheapest in the set.
- **Artefact**: test `anchor_scenarios::success_criterion_still_maps`.
- **Non-vacuity**: a fixture register with `tracing-use` removed must fail
  naming the unmapped noun; a roadmap whose bullet is reworded must fail naming
  the clause. Without the second control the check passes vacuously against an
  absent bullet.

### INV-TEMPLATE — the blank form matches the register it instantiates

- **Obligation**: the field identifiers in
  `docs/phase-2-validation-note-template.md`'s note register equal the distinct
  field identifiers of ADR 004's status register, in first-appearance order,
  and every status and evidence cell in the template holds the literal `TBD`.
- **Method**: parse both, compare typed values with
  `pretty_assertions::assert_eq!`.
- **Rationale**: this is the schema-versus-instance edge, and it is the thing
  most likely to rot, because the two files will be edited months apart. The
  comparison is deliberately *semantic* rather than byte-exact. The second
  draft rendered the template from the register and asserted byte equality,
  which would have required `render_blank_note` to reproduce `mdtablefix`'s
  `max(content) + 2` column canonicalization — an external formatting rule this
  plan lists as an axiom, with no "run `make fmt`, then copy" escape available
  because generation happens at test time. That coupled the suite to a
  third-party padding algorithm forever, to guard a property that does not
  depend on padding.
- **Artefact**: test `anchor_scenarios::template_matches_the_status_register`.
- **Non-vacuity**: four controls. A template with a field removed, with a field
  added, with fields reordered, and with `Bounded` pre-filled in a status cell
  must each fail with a diff naming the row. An emptied template register must
  yield `MissingDelimiters`, not a vacuously equal pair of empty vectors.

### INV-REGISTERS — both registers match their fixtures exactly

- **Obligation**: the status register and the aggregation register parsed from
  ADR 004 equal their fixture literals in `fixtures.rs`.
- **Method**: exact equality per register.
- **Rationale**: equality against a literal is a stronger and more readable
  check than a family of policy predicates, and a `pretty_assertions` diff
  names the changed cell better than any bespoke message. It subsumes totality
  and row cardinality.
- **Artefact**: tests `register_scenarios::status_register_matches_fixture`
  and `register_scenarios::aggregation_register_matches_fixture`.
- **Non-vacuity**: a register whose delimiters are absent, or whose block holds
  no data row, must yield `MissingDelimiters` naming the register and both
  markers, not a vacuously equal pair of empty vectors; an unrecognized token
  must yield `UnknownToken` naming the column.

### INV-EXCLUSION — the default holds without a required property, and can fall

- **Obligation**: in the status register, no row contributing `Insufficient`
  belongs to a field other than `identifier-need`; and at least one row
  contributes `Insufficient`.
- **Method**: two predicates over the typed verdict rows.
- **Rationale**: `INV-REGISTERS` pins the live document, so these are
  *implied* for it. They are not redundant against the real threat model: an
  editor who changes ADR 004 and updates the fixture in the same commit keeps
  `INV-REGISTERS` green and trips these. Stating that relationship here is
  deliberate, because otherwise a later reviewer reasonably "simplifies" them
  away. The second half is the guard against a degenerate register: this plan's
  own analysis expects the default to survive, which is exactly the pressure
  that produces an instrument incapable of the inconvenient answer.
- **Artefact**: tests
  `register_scenarios::default_survives_without_a_required_property` and
  `register_scenarios::register_can_select_insufficient`.
- **Non-vacuity**: a fixture-plus-document pair edited together so that a
  status other than `Property required` selects `Insufficient` must fail with
  `docs/adr-004-state-name-consumption-evidence.md: field identifier-need status
  None selects Insufficient. Repair: only the Property required status may
  overturn the &'static str default.`
  A pair in which the `Property required` row is softened to `Sufficient` —
  which violates nothing else — must fail with
  `docs/adr-004-state-name-consumption-evidence.md: no row selects Insufficient.
  Repair: a register that cannot overturn the default is not a decision
  procedure.`
  A third control separates the two branches of the same check: a status
  selectable as `Insufficient` from a *field* other than `identifier-need` must
  fail naming that field and status.

### INV-ADMISSIBILITY — an unusable note yields no verdict and names its blocker

- **Obligation**: a note selecting any status whose register row says
  `Admissible: no` resolves to `Not resolved` and reports the blocking field
  and status. The blocking set is read from the status register, never
  hardcoded in Rust.
- **Method**: parameterized test over fixture notes, one case per row the
  register marks inadmissible, enumerated from the register itself so a new
  blocking status cannot be added without a case, plus one admissible control.
- **Rationale**: this is where cardinality belongs. The first state Phase 2
  meets is `ProcessBuffer`'s `bool in_table`, which is not a named type at all,
  and ADR 002 confirms that is the present shape of `mdtablefix`. A vocabulary
  admitting only "recorded" or "not reached" cannot express it, and a register
  treating it as a verdict axis loops.
- **Artefact**: test `blocked_notes_resolve_to_not_resolved`.
- **Non-vacuity**: an admissible fixture note must resolve to a verdict, so the
  check cannot pass by blocking everything; and a blocked note must name the
  specific field, so it cannot pass by reporting a generic failure.

### INV-CONSISTENCY — a contradictory note is rejected, not resolved

- **Obligation**: a note selecting two different non-`nothing` contributions
  yields an error naming both fields and both contributions, rather than
  resolving to either one. ADR 004 states the rule in the paragraph before the
  status register: a note's contribution is "the single non-`nothing` value
  among the contributions its cells select", and "a note selecting two
  different contributions is contradictory and is rejected rather than
  resolved".
- **Method**: a fixture register carrying a second decisive status, plus a note
  selecting both it and the live register's own decisive status; the control
  asserts the exact rejection message.
- **Rationale**: the code collapsed the pair into `Insufficient`, which resolves
  exactly the note the ADR says to refuse — and resolves it silently, in the
  direction that overturns the default. The pair is unreachable through the live
  register, which is precisely why the guard must exist: nothing else in the
  suite would notice the rule being dropped. A register where a second field
  contributes `Sufficient` passes every document-level check there is — its
  vocabulary is still closed, exactly one row selects `Insufficient`, and the
  default can still fall — so only a note-level guard notices that reading a
  note through it has become contradictory.
- **Artefact**: test `contradictory_notes_are_rejected`.
- **Non-vacuity**: the control's register differs from the live one by one
  contribution cell, and its note from the accepting witness by one decisive
  cell, so a rejection for any other reason — an unknown field, an inadmissible
  status, a non-citation — fails the control's own preconditions rather than
  passing it.

### INV-FILLED — every committed StateName note is a usable note

- **Obligation**: every file under `docs/validation-notes/` that declares a
  `<!-- state-name-note -->` marker parses, contains no residual `TBD`, uses
  only statuses admissible for its field per the status register, carries a
  citation-shaped evidence cell for every field, and resolves. Files without
  the marker are ignored.
- **Method**: a directory scan keyed on the marker, asserting per matching
  file.
- **Rationale**: keying on a declared marker rather than on `*.md` matters more
  than it looks. `docs/validation-notes/` is not this task's namespace to
  claim: roadmap task 1.2.3 produces a *benchmark* note with four different
  fields, task 2.2.3's note chooses one of ADR 003's three exits, and task
  3.1.3's cites both validation examples. None is a `StateName` note. A glob
  would turn the arrival of any of them into a build failure — the same hazard
  D7 removed from the gates, one level down.
- **Artefact**: test `committed_state_name_notes_are_usable`.
- **Non-vacuity**: the accepting witness is a string fixture, not a committed
  file, so the test cannot pass merely because the directory is empty — and an
  empty directory is explicitly *not* a failure, because no note can honestly
  exist until task 2.2.1 has annotated something. Eight rejecting cases, all
  string fixtures: residual `TBD`; a status outside the field's register
  vocabulary; an evidence cell that is prose rather than a citation; a citation
  missing its path; an `identifier-need` cell naming no consumer; a status and
  evidence that contradict; a file carrying the marker but missing a field; and
  a file carrying it with its fields reordered. A ninth control sits outside
  the table: a status borrowed from a field the register defines under another.
  Four controls cover the scan itself: a benchmark-shaped note *without* the
  marker is ignored rather than rejected; a note without the marker still
  parses, so that control isolates the marker; a directory passed where a note
  is expected is an error naming the path rather than a silent skip; and a root
  with no notes directory yields no notes rather than failing.

### INV-AGGREGATE — the rule for reading several notes is total

- **Obligation**: the aggregation register maps each of the three reachable
  states of the note multiset — no admissible note; at least one admissible
  note and none `Insufficient`; at least one `Insufficient` — to exactly one
  outcome at task 3.2.1.
- **Method**: exhaustive parameterized test, one case per state.
- **Rationale**: this is the rule 3.2.1 actually executes. Writing it before
  the evidence is the point of the whole task.
- **Artefact**: test `aggregation_register_is_total`.
- **Non-vacuity**: a register missing the "no admissible note" row must fail
  naming that state; a register mapping it to "ratify" must fail, because
  ratifying on no evidence is the precise failure this rule exists to prevent.

### INV-ANCHORS — the three load-bearing clauses still say what is quoted

- **Obligation**: the clauses quoted in ADR 004's "Evidence the record
  preserves" resolve, whitespace-folded, within their named sections: design
  §6.1's "The default remains `&'static str` until a real example consumes
  something stronger"; roadmap 3.2.1's "backed by observed example consumption,
  not anticipation"; and ADR 002's `wireframe` sentence giving "production
  logs, tests, and the model the same" labels.
- **Method**: folded substring resolution, with each section bounded by
  `split_once` on the *next* heading's literal text.
- **Rationale**: three clauses, not six. Each additional anchor is a permanent
  edit-tax on a document nobody will remember is load-bearing, and these three
  are the ones the decision rests on. ADR 002's clause is included because
  Finding four depends on it and the first draft left it unguarded while
  claiming otherwise.
- **Artefact**: test `quoted_passages_still_resolve`.
- **Non-vacuity**: four controls. A rewritten clause must fail naming clause
  and file. A clause relocated out of its section must fail as relocated,
  proving the check is section-scoped. An ADR quoting a clause present in no
  source must fail, proving the ADR cannot satisfy the check by quoting itself.
  An ADR whose evidence section is empty must fail — without this, the check
  passes over zero clauses, which is exactly what would have happened at the
  red step of this plan's first draft.
- **Two traps, both verified during planning**: folding is mandatory, because
  `mdtablefix --wrap` rewraps ADR 004's prose at different break points from
  the source, and three of six clauses in the first draft did not resolve
  without it. And section bounds must not be found by scanning for a leading
  `#`: design §6.1 contains `#[derive(StateName)]` at column zero inside a
  fence, and the scanner in `support/split_case.rs:12-14` would truncate the
  section there, reporting the clause as missing.

### INV-GATES — every gate names exactly one live roadmap task

- **Obligation**: each gate's task-title fragment matches exactly one task in
  `docs/roadmap.md`, and the fragment is specific enough that no other task
  matches.
- **Method**: parameterized test, one case per gate.
- **Rationale**: a gate pointing nowhere is an instrument with no consumer.
  Binding by title rather than number is deliberate: it keeps the reference
  meaningful without freezing task numbers and without breaking the build on
  the day a bound task is legitimately completed.
- **Artefact**: test `gate_titles_resolve`.
- **Non-vacuity**: a fragment matching two tasks must fail as ambiguous; a
  fragment matching none must fail as unresolved. Note that gate S3's natural
  fragment overlaps task 3.1.2 on the single word "wireframe", so the fragment
  is the longer "Apply the conventions-only baseline"; the ambiguity control is
  what makes that choice checkable rather than assumed.

### Methods chosen and refused

Chosen: exact equality with `pretty_assertions` for structural comparisons;
exhaustive `rstest` case tables for the two- and three-element domains;
`googletest` matchers where an assertion is about shape; named policy
predicates only where their failure message carries an argument a reader needs;
a runtime directory scan for committed notes.

Refused, each with a reason in D9: `insta`, `kani`, `verus`, `proptest`,
`rstest-bdd`, `cargo-mutants`. End-to-end tests are also refused: there is no
binary and no externally observable workflow beyond `make test`, which is
itself the acceptance command.

Behavioural coverage is delivered as scenario-named `rstest` cases over the
committed worked example and its eight documented defects —
`committed_state_name_notes_are_usable` and the `INV-FILLED` controls
constitute the fill-and-gate workflow. The `docs/developers-guide.md` addition
carries the prose walkthrough. If the dependency cost declined under Q3 is later
judged acceptable, these convert to Gherkin mechanically.

## Plan of work

### Stage A — orient and confirm (no changes)

Read `tests/v0_1_exit_register_contract.rs` and all four children. Confirm the
branch, a clean tree, and a green `make test`. Confirm roadmap task 1.1.3 is
unticked and that the three quoted clauses still occur in their sections. Stage
A ends with no diff; if any confirmation fails, the conformance basis has moved
and the plan needs revision before code.

### Stage B — red (EP-M1)

Create both new documents with their full prose and their delimiter comments
but **no register tables**, and create `docs/validation-notes/README.md` with
an empty directory otherwise. The documents must exist before the test compiles
— `include_str!` of a missing file is a compile error, not a test failure.
Write the contract test in full. Run `make test` and observe the red state:
`MissingDelimiters` naming each register, and `INV-FILLED` failing because the
directory holds no note. Record the transcript.

Do not use an expected-failure marker. `AGENTS.md` requires every commit to
pass the gates, so the red state is observed within a session and not
committed; the first commit is the green one and carries the red transcript in
its body.

### Stage C — green (EP-M1, EP-M2, EP-M3)

Insert the status register and the aggregation register into ADR 004. Run
`make fmt` first so `mdtablefix` sets column widths, then copy the formatted
rows into `fixtures.rs`; writing fixtures before formatting is what makes them
drift. Then proceed to Step 6 for the template and the illustrative example,
and Step 7 for the gate table and the evidence section. Observe each invariant
go green in turn, committing at each coherent point.

### Stage D — sync and delivery (EP-M4, EP-M5)

Apply the documentation sync map, including the discoverability pointers under
Q4, which was approved. Run every gate sequentially, obtain review, tick the
roadmap, and set this plan to `COMPLETE`.

## Milestones and plateaus

### EP-M1 — the decision record exists and its registers are guarded

- **Outcome**: ADR 004 exists with its status and aggregation registers;
  `INV-CRITERION`, `INV-REGISTERS`, `INV-EXCLUSION`, and `INV-AGGREGATE` green.
- **Requirements**: the decision half of `ROADMAP-1.1.3`; `TDD-6.1-stable-id`;
  `TDD-6.2-no-speculative-api`.
- **Acceptance**: `make test` passes; every negative control asserts a specific
  message rather than `is_err()`.
- **Conformance check**: no runtime code; no verdict pronounced; ADR 002's
  boundary untouched; no roadmap renumbering; no dependency change.
- **Recovery**: additive — delete the ADR and the test.
- **Remaining gaps**: no template, no committed note.
- **Compatibility decision**: none. Nothing is released; `src/` is a stub.

### EP-M2 — the template matches the register and the example exists

- **Outcome**: the template's fields match the status register and every cell
  holds `TBD`; ADR 004 carries an illustrative worked example marked as
  non-evidence; `INV-TEMPLATE`, `INV-ADMISSIBILITY`, and `INV-FILLED` green.
- **Requirements**: the instrument half of `ROADMAP-1.1.3` — the roadmap's
  literal success criterion.
- **Acceptance**: `make test` passes; and a manual check that filling the
  template for `ContinuationMode` yields a verdict derivable from ADR 004
  alone, without a judgement call.
- **Conformance check**: template fields derived from the status register; the
  illustrative example uses only admissible statuses and is marked as
  illustration; `docs/validation-notes/` holds no fabricated evidence.
- **Recovery**: additive.
- **Remaining gaps**: gates and anchors unbound.
- **Compatibility decision**: none.

### EP-M3 — gates and anchors are guarded

- **Outcome**: ADR 004 carries its gate table and evidence section, both inside
  delimiters; `INV-GATES` and `INV-ANCHORS` green.
- **Requirements**: binds `ROADMAP-2.2.1`, `2.2.2`, `3.1.2`, and `3.2.1` by
  title.
- **Acceptance**: each named control fails for its own reason when applied.
- **Conformance check**: no task number frozen; folding applied; sections
  bounded by next-heading text.
- **Recovery**: two contiguous blocks; remove with their tests.
- **Remaining gaps**: companion documentation.
- **Compatibility decision**: none.

### EP-M4 — documentation is coherent and the template is discoverable

- **Outcome**: the sync map is applied; a Phase 2 engineer starting from
  roadmap task 2.2.1 reaches the template in one hop.
- **Requirements**: `AGENTS.md`'s documentation obligation; the Q4 pointers.
- **Acceptance**: `make markdownlint`, `make nixie`, `make check-fmt` pass, and
  `make test` still passes **including** the pre-existing exit-register
  contract, which reads three of the documents being edited.
- **Conformance check**: no requirement altered, only pointers added; "Last
  substantive revision" updated where prose changed.
- **Recovery**: each sync edit is an independent diff.
- **Remaining gaps**: none.
- **Compatibility decision**: none.

### EP-M5 — delivery

- **Outcome**: every gate green; review clean; roadmap task 1.1.3 ticked; plan
  `COMPLETE`.
- **Acceptance**: transcripts for `make check-fmt`, `make lint`, `make test`,
  `make markdownlint`, `make nixie`, `make audit`, and
  `make test-workflow-contracts` recorded in `Artefacts and notes`; a
  zero-finding independent review.
- **Conformance check**: the full checklist above, plus reconciliation of every
  entry in `Surprises & discoveries` against the conformance basis.
- **Recovery**: the branch reverts as a unit; nothing is published.
- **Remaining gaps**: none for 1.1.3. The verdict belongs to task 3.2.1.
- **Compatibility decision**: none.

## Concrete steps

All commands run from the repository root — the directory containing
`Cargo.toml` and `Makefile`.

### Step 1 — confirm the starting state

```bash
git branch --show-current
git status --short
make test 2>&1 | tee /tmp/test-statelet-$(git branch --show-current).out
```

Expected: the branch is `1-1-3-define-state-name-consumption-question`, the
tree is clean, and every test passes across the five existing binaries.

### Step 2 — create the documents and the notes directory

Create `docs/adr-004-state-name-consumption-evidence.md` and
`docs/phase-2-validation-note-template.md` with full prose, and
`docs/validation-notes/README.md`. This step precedes the test because
`include_str!` needs the files to exist.

**Done 2026-09-19, with one departure from the step as written.** The step
originally asked for "delimiter comments but no tables". `make fmt` merges an
adjacent, empty delimiter pair onto a single line, which would have made Step 4
fail with `EmptyRegister` rather than the predicted `MissingDelimiters`; see
D19 and `Surprises & discoveries`. ADR 004 was therefore committed with its
four delimiter pairs *empty and merged*, and the registers arrive with their
delimiters in Step 5.

### Step 3 — write the contract test in full

Create the seven-file contract described in `Interfaces and dependencies`,
including every negative control, before any register exists. Create
`dylint.toml` first, with the single path-scoped exemption defined in D20:
without it the `notes.rs` module fails `make lint`, and creating it now keeps
the exemption visible from the moment the code that needs it exists rather than
retro-fitted at delivery.

### Step 4 — observe red

```bash
make test 2>&1 | tee /tmp/test-statelet-red-$(git branch --show-current).out
```

Expected transcript fragment:

```plaintext
docs/adr-004-state-name-consumption-evidence.md: no status register found
between <!-- status-register:begin --> and <!-- status-register:end -->.
Repair: add the register block to the Status register section.
```

Every register-dependent test fails with `MissingDelimiters` naming its own
register, and `INV-ANCHORS` fails because the evidence section is empty — the
latter is the empty-clause-list control doing its job, and it is the reason
that control exists.

`INV-FILLED` does **not** fail, and this corrects the prediction above as first
drafted. An earlier draft of this step said the directory scan fails because
`docs/validation-notes/` holds no note, which contradicted `INV-FILLED`'s own
non-vacuity clause three hundred lines earlier: the accepting witness is a
string fixture precisely so that the check cannot pass merely because the
directory is empty, and an empty directory is explicitly *not* a failure,
because no honest note can exist before task 2.2.1. A red step that demanded
one would have demanded a fabricated observation. `INV-FILLED`'s red evidence
is the `committed_state_name_notes_are_rejected` cases — seven when this step
was written, eight as delivered (F4 added
`#[case::citation_without_a_path]`) — which fail at Step 4 for the same
`MissingDelimiters` reason as every other register-dependent scenario, plus
`unmarked_notes_are_ignored`, which passes throughout and is the accepting end
of its marker control. See D22 and D26.

### Step 5 — insert the registers, format, then fixture

```bash
make fmt
make test 2>&1 | tee /tmp/test-statelet-green-$(git branch --show-current).out
```

Insert the status register and the aggregation register, run `make fmt` so
`mdtablefix` canonicalizes column widths to `max(content) + 2`, then copy the
formatted rows into `fixtures.rs`. Running `make fmt` after writing fixtures
would change documents under a green test. Commit.

### Step 6 — write the template and the illustrative example

Write `docs/phase-2-validation-note-template.md` from the status register's
field list, with `TBD` in every cell. Add the illustrative worked example to
ADR 004 as a fenced block under a heading that marks it explicitly as
illustration and not evidence. Run `make fmt` and `make test`. Commit.

`docs/validation-notes/` stays empty except for its `README.md` until task
2.2.1 fills the first note. That is deliberate: a note committed now could only
cite work that has not happened, so it would either carry fabricated citations
while serving as the suite's accepting witness, or it would mean task 1.1.3 had
quietly done task 2.2.2's observation. The suite's witness is a string fixture
instead.

### Step 7 — gate table and quoted evidence

Add the gate table and the evidence section, each inside its own delimiter
pair. Run `make fmt && make check-fmt && make test`. Commit.

### Step 8 — companion sync

Apply every item in `Documentation sync map`, then:

```bash
make fmt
make check-fmt    2>&1 | tee /tmp/checkfmt-statelet-$(git branch --show-current).out
make markdownlint 2>&1 | tee /tmp/mdlint-statelet-$(git branch --show-current).out
make nixie        2>&1 | tee /tmp/nixie-statelet-$(git branch --show-current).out
make test         2>&1 | tee /tmp/test-statelet-sync-$(git branch --show-current).out
```

The `make test` run matters here specifically: `docs/design.md`,
`docs/context.md`, and `docs/roadmap.md` are all read by the pre-existing
exit-register contract. If it fails, revert the offending edit, as required by
the fourth constraint. Then commit.

### Step 9 — full gates and delivery

```bash
make check-fmt               2>&1 | tee /tmp/checkfmt-statelet-$(git branch --show-current).out
make lint                    2>&1 | tee /tmp/lint-statelet-$(git branch --show-current).out
make test                    2>&1 | tee /tmp/test-statelet-$(git branch --show-current).out
make markdownlint            2>&1 | tee /tmp/mdlint-statelet-$(git branch --show-current).out
make nixie                   2>&1 | tee /tmp/nixie-statelet-$(git branch --show-current).out
make audit                   2>&1 | tee /tmp/audit-statelet-$(git branch --show-current).out
make test-workflow-contracts 2>&1 | tee /tmp/wfc-statelet-$(git branch --show-current).out
```

Run these sequentially, never in parallel; the repository relies on build
caching and concurrent cargo jobs contend for the package-cache lock.

Then tick roadmap task 1.1.3, append the ADR link to its success bullet as
tasks 1.1.1 and 1.1.2 do, set this plan's status to `COMPLETE`, and record
outcomes.

## Documentation sync map

1. `docs/contents.md`: add ADR 004 under "Decision records" after ADR 003;
   add the template and `docs/validation-notes/` under "Project guides", not
   "Product and design" — a blank form is not design material. Also add an
   `execplans/` entry enumerating the three execution plans. That last item
   closes a pre-existing gap the style guide explicitly contemplates; it is
   flagged here rather than slipped in, and may be declined without affecting
   anything else.
2. `docs/design.md`: add both new documents to the companion list; add one
   sentence at the end of §6.1 pointing to ADR 004 and the template; add one
   sentence at the end of §12 pointing to the template, because §12 is what
   roadmap task 2.2.1 cites. Per Q1, add the `StateName` return-shape bullet to
   §14 "Deferred decisions" — verified inert, because the existing contract
   reads §14 only as a heading-string relocation anchor. Update "Last
   substantive revision". No §11.1 edit — see Q1 and D12.
3. `docs/terms-of-reference.md`: add both documents to the companion list and
   ADR 004 to §10.2. Update "Last substantive revision".
4. `docs/context.md`: add two glossary entries. **State identifier** — "a
   stable value distinguishing one state from another for machine consumption;
   distinct from a state name, which is a human-readable label." **Validation
   note** — "the record a validation task produces, instantiated from
   `docs/phase-2-validation-note-template.md` and committed to
   `docs/validation-notes/` carrying the marker its contract test keys on."
   Deliberately *not* made `INV-ANCHORS` targets: welding glossary entries to
   the test suite buys little and taxes every future edit.
5. `docs/roadmap.md`: tick task 1.1.3 and append the ADR link to its success
   bullet. Per Q4, add one `- See docs/phase-2-validation-note-template.md.`
   bullet to tasks 2.2.1, 2.2.2, and 3.1.2. Renumber nothing.
6. `docs/users-guide.md` "Current status": one sentence recording that the
   `StateName` return type is not yet settled and that no identifier will be
   added without recorded evidence, matching the existing paragraph that
   already tells consumers the project may ship nothing.
7. `docs/developers-guide.md`: a subsection recording what is machine-checked,
   that `tests/state_name_consumption_contract.rs` is the guard, which edits
   break it by design and how to repair them, that `notes.rs` is the one module
   carrying a `no_std_fs_operations` exemption and why, and — most importantly
   for a Phase 2 engineer — that filling a note means copying the template into
   `docs/validation-notes/<task>-<subject>.md` and committing it, never editing
   the template.
8. `docs/repository-layout.md`: note the new test files, `dylint.toml`, and
   `docs/validation-notes/`.

## Validation and acceptance

**Red evidence.** Before the registers exist, `make test` fails with
`MissingDelimiters` naming each register and an empty-clause-list failure on the
evidence section. Not a panic, not an index-out-of-bounds, not a bare
`assertion failed`. An empty notes directory passes, deliberately: see
`INV-FILLED` and D22.

**Green evidence.** After Step 7, `make test` passes and the binary
`state_name_consumption_contract` reports every scenario named in the
`Verification plan`. Named with their modules, because two of them differ by
one letter and the contract is eleven modules:
`anchor_scenarios::success_criterion_still_maps`,
`anchor_scenarios::template_matches_the_status_register`,
`anchor_scenarios::quoted_passages_still_resolve`,
`anchor_scenarios::gate_titles_resolve`,
`register_scenarios::status_register_matches_fixture`,
`register_scenarios::aggregation_register_matches_fixture`,
`register_scenarios::default_survives_without_a_required_property`,
`register_scenarios::register_can_select_insufficient`,
`register_scenarios::aggregation_register_is_total`,
`note_scenarios::blocked_notes_resolve_to_not_resolved`,
`note_scenarios::contradictory_notes_are_rejected`,
`note_scenarios::committed_state_name_note_is_usable`,
`scan_scenarios::committed_state_name_notes_are_usable`.

The acceptance command is unchanged: `make test` exits 0 and the counted test
total matches the suite's length, so a scenario that silently stopped being
collected fails here.

**Negative-control evidence.** Every control asserts an exact message with
`pretty_assertions::assert_eq!`. A control asserting only `is_err()` does not
discharge its obligation and must be rewritten.

**Manual acceptance.** Read ADR 004's illustrative worked example and its
status register, and nothing else. The verdict must be derivable without a
judgement call. If it is not, the instrument has failed regardless of the tests.

Quality criteria: every gate in Step 9 green; every invariant with a passing
test and a failing control; `make lint` reporting no clippy or Whitaker finding
and no `#[allow]` added; `make audit` reporting no advisory. The single
Whitaker exemption is the `dylint.toml` entry itself, is path-scoped to
`notes.rs`, and is required to be visible in that file with its rationale — an
exemption recorded in configuration is auditable in a way an in-source
`#[allow]` is not, which is the one thing this deviation buys in exchange for
the constraint it breaches. Performance is not applicable — there is no runtime
code.

## Idempotence and recovery

Every step is re-runnable. `make fmt` is idempotent. The contract test writes
nothing; it reads through `include_str!` and one read-only directory scan.

Work is additive through Step 7. Recovery before Step 8 is `git checkout -- .`
plus deleting the new files. Step 8 is a separate commit from the artefact
commits precisely so it can be reverted alone.

The one hazard is Step 8's edits to `docs/design.md`, `docs/context.md`, and
`docs/roadmap.md`, all read by the pre-existing exit-register contract. Confirm
`make test` is green before those edits and re-run immediately after. If the
pre-existing contract fails, revert the single offending edit rather than
adapting that contract.

The contract test writes nothing: it reads through `include_str!` and one
read-only directory scan, so no step that runs it can dirty the tree. The plan
itself writes exactly what "Files this plan reads or writes" lists, and nothing
else: new files under `docs/` and `tests/state_name_consumption_contract/`,
`dylint.toml`, and one-line edits to the eight documents named there. Nothing
outside the repository is written except under `/tmp`, which holds gate logs and
scratch, and `target/`, which holds build output.

## Artefacts and notes

Transcripts are appended here as steps complete. The register drafts below are
narrower than 120 columns so that they do not breach `MD013`'s
`code_block_line_length` when previewed inside a fence.

### The status register

This one table replaces the separate field register, its admissibility column,
and the two-row verdict register that the second draft carried. Keying on
`(Field, Status)` makes both admissibility and verdict *derivable from the
document*, where the earlier shape left the blocking statuses hardcoded in Rust
— the same "constant tautological with its own control" defect that D6 removed
from the verdict axis, reintroduced one level down.

Between `<!-- status-register:begin -->` and `<!-- status-register:end -->`:

```markdown
| Field               | Status            | Admissible | Contributes  |
| ------------------- | ----------------- | ---------- | ------------ |
| state-display-name  | Enumerated        | yes        | nothing      |
| state-display-name  | Not a named type  | no         | nothing      |
| identifier-need     | None              | yes        | Sufficient   |
| identifier-need     | Property required | yes        | Insufficient |
| metrics-cardinality | Bounded           | yes        | nothing      |
| metrics-cardinality | Unbounded         | no         | nothing      |
| tracing-use         | Full              | yes        | nothing      |
| tracing-use         | Partial           | yes        | nothing      |
| tracing-use         | None              | yes        | nothing      |
```

*Table 2: Every admissible status for every field, whether it blocks the note,
and what it contributes to the verdict. A note is admissible when no cell it
selects is `Admissible: no`; its verdict is the single non-`nothing`
contribution.*

`state-display-name` is `Enumerated` only when the note lists the actual
strings each annotated state can return. That is what the roadmap noun "state
display name" asks for, and the second draft did not deliver it: its statuses
recorded only whether a named type existed, so `metrics-cardinality: Bounded`
was an unaudited assertion and a reviewer at task 3.2.1 would decide the fate
of a `&'static str` without ever seeing the strings. With the names enumerated,
the cardinality bound is derivable from the note rather than asserted by its
author, and Finding one's stability case can be argued against concrete labels.

The evidence cell for `identifier-need` must list the consumers considered —
tracing subscriber, metrics recorder or its documented absence, any model
checker, and generated documentation — and, where a property is required, name
which of equality, stability across releases, ordering, or compact encoding the
variant-name `&'static str` fails to supply.

### The aggregation register

Between `<!-- aggregation-register:begin -->` and
`<!-- aggregation-register:end -->`:

```markdown
| Admissible notes | Any insufficient | Outcome if publication proceeds  |
| ---------------- | ---------------- | -------------------------------- |
| None             | n/a              | Blocked: no admissible evidence  |
| One or more      | No               | Ratify the current return type   |
| One or more      | Yes              | Amend design 6.1 before publish  |
```

*Table 3: How task 3.2.1 reads the committed notes together. Every outcome is
conditional on publication: if ADR 003's gate G2 has already selected exit E1,
Statelet ships nothing and no return type is ratified.*

### The gate table

Between `<!-- gate-table:begin -->` and `<!-- gate-table:end -->`:

```markdown
| Gate | Roadmap task title fragment           | Role            |
| ---- | ------------------------------------- | --------------- |
| S1   | Annotate `mdtablefix` `ProcessBuffer`  | Records a note  |
| S2   | Annotate `mdtablefix` continuation     | Records a note  |
| S3   | Apply the conventions-only baseline    | Records a note  |
| S4   | Finalize the `StateName` return shape  | Decides         |
```

*Table 4: Gates, bound by task title rather than task number so that completing
a gate does not break the build.*

### ADR 004's section order

Per `docs/documentation-style-guide.md`, with custom sections in the slot ADR
003 uses for `## Exit register`:

```plaintext
# Architectural decision record (ADR) 004: Define the StateName consumption evidence
## Status / ## Date / ## Context and problem statement
## Decision drivers
## Options considered          -> with a numbered comparison table
## Decision outcome / proposed direction
## Status register             -> delimited; every (field, status) pair, its
                                  admissibility and its contribution
## Admissibility               -> prose reading of the register, naming the
                                  owner of an upstream repair
## Aggregation register        -> delimited
## Worked example              -> illustrative fenced note, marked explicitly
                                  as illustration and not as evidence
## Gates                       -> delimited
## Evidence the record preserves -> delimited; three clauses
## Goals and non-goals
## Known risks and limitations -> the discriminant finding; the hand-assigned
                                  identifier consequence
## Outstanding decisions       -> the verdict itself, left to task 3.2.1
## Architectural rationale     -> the cardinality argument in full
```

The full prose of both new documents is written during Stage B and appended to
this section as it is drafted, so that this plan ends the task self-contained.

### Step 9 gate transcripts

Run at revision `aebe29d`, sequentially, one gate at a time. Each command is
the one Step 9 names, and each log is the one that command wrote.

`make check-fmt` — exit 0:

```plaintext
cargo fmt --all -- --check
mdtablefix --check --git --include-untracked --wrap --renumber --breaks --ellipsis --fences
28 files left unchanged.
```

`make lint` — exit 0. The `whitaker` line matters as much as the exit code:
until it appears, the run has only proved clippy clean, and D25 was recorded
precisely because a clippy failure once stopped the lint suite before Whitaker
ran:

```plaintext
RUSTFLAGS="-D warnings " whitaker --all -- --all-targets --all-features
Checking with toolchain `nightly-2026-05-28`
    Checking statelet v0.1.0 (...worktrees/99bdf268-...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s
```

`make test` — exit 0, `cargo nextest run`:

```plaintext
Summary [   0.047s] 83 tests run: 83 passed, 0 skipped
```

Doctests, in the same run:

```plaintext
test src/lib.rs - greet (line 8) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Eighty-three includes this contract's twenty-nine scenarios, alongside the
existing suites. Twenty-nine functions occupy forty-three collected cases,
because `rstest` expands three of them: `gate_titles_resolve` into six
(one per gate plus the ambiguity control), `aggregation_register_is_total`
into three (one per reachable state of the note multiset), and
`committed_state_name_notes_are_rejected` into eight (one per documented note
defect). The two totals are both worth recording: forty-three is what the suite
must report, and twenty-nine is how many scenarios the `Verification plan`
names.

`make markdownlint` — exit 0:

```plaintext
Linting: 29 file(s)
Summary: 0 error(s)
```

`make nixie` — exit 0:

```plaintext
🧜‍♀️✨ All diagrams validated successfully!
```

`make audit` — exit 0:

```plaintext
    Loaded 1251 security advisories (from /home/leynos/.cargo/advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (45 crate dependencies)
```

No advisory line follows, which is the passing shape: `cargo audit` prints
each finding it has and prints nothing when it finds none.

`make test-workflow-contracts` — exit 0:

```plaintext
6 passed in 0.02s
```

Two warnings appear in `make lint` and `make test` output and neither is a
finding: `cargo::redundant_homepage` and its companion note, both about
`Cargo.toml`'s `homepage` key. They are generated by the manifest-carrying
commands rather than by this change — `Cargo.toml` is untouched by it — and
they are warnings under a `-D warnings` flag only for `rustc`/`clippy`
invocations, not for the cargo manifest parse that emits them.

`make check-fmt`'s "28 files left unchanged" is likewise a count of tracked
Markdown files, reported by `mdtablefix`; it is the passing form of the check,
not a partial run.

## Interfaces and dependencies

No dependency change. The existing dev-dependencies — `camino`, `googletest`,
`pretty_assertions`, `rstest`, `toml` — are sufficient. `camino` supplies the
notes-directory scan *enumeration*, following `tests/dev_fast_contract.rs:19`.

One module, and only one, steps outside that dependency set. `notes.rs` reads
the contents of each enumerated note through `std::fs`, and is exempted by name
in the root `dylint.toml`:

```toml
[no_std_fs_operations]
# `notes.rs` enumerates `docs/validation-notes/` and reads each note's text at
# run time. The note set is deliberately open -- a Phase 2 engineer adds a note
# months from now and its arrival must not require a Rust edit -- so the
# contents cannot be embedded with `include_str!`. `camino` enumerates but has
# no content-read API, and this plan's constraint 3 forbids adding `cap-std` as
# a dev-dependency. The exemption is path-scoped to this one module so that the
# parsers, policies, and fixtures of the same test crate remain under the lint.
excluded_paths = ["state_name_consumption_contract::notes"]
```

### Test module shape

The split is pre-declared rather than discovered, because the existing
`support.rs` needed 385 lines for one parser and four checks, and this contract
has more of both. `self_named_module_files` is denied, so children are included
with `#[path]`, as `tests/v0_1_exit_register_contract/support.rs:5-6` does.
`notes.rs` is a sibling module for the same reason, and its distinct name is
also what makes the `excluded_paths` entry one module wide rather than
crate-wide.

The split as delivered is eleven modules: the five below, plus `clauses.rs` for
quoted-clause resolution and `registers.rs` for the roadmap binding and the
cross-register checks (D21), and four scenario modules — `anchor_scenarios.rs`,
`note_scenarios.rs`, `register_scenarios.rs`, `scan_scenarios.rs` — that hold
the contract's tests rather than a share of the crate root (D26). Each module
owns one invariant class. The first two additions keep `policy.rs` from
carrying three unrelated ones; the scenario modules exist because the root file
had reached 788 lines against AGENTS.md's 400-line cap, and because a scenario
module per invariant class keeps every file small enough to stay there.

```rust,ignore
#[path = "state_name_consumption_contract/anchor_scenarios.rs"]
mod anchor_scenarios;
#[path = "state_name_consumption_contract/clauses.rs"]
mod clauses;
#[path = "state_name_consumption_contract/fixtures.rs"]
mod fixtures;
#[path = "state_name_consumption_contract/note_scenarios.rs"]
mod note_scenarios;
#[path = "state_name_consumption_contract/notes.rs"]
mod notes;
#[path = "state_name_consumption_contract/parse.rs"]
mod parse;
#[path = "state_name_consumption_contract/policy.rs"]
mod policy;
#[path = "state_name_consumption_contract/register_scenarios.rs"]
mod register_scenarios;
#[path = "state_name_consumption_contract/registers.rs"]
mod registers;
#[path = "state_name_consumption_contract/scan_scenarios.rs"]
mod scan_scenarios;
#[path = "state_name_consumption_contract/types.rs"]
mod types;
```

`notes.rs` is the only module calling `std::fs`, and it does exactly two things:

```rust,ignore
/// Every committed note declaring the `state-name-note` marker, with its text.
pub(crate) fn committed_notes(root: &Utf8Path) -> Result<Vec<CommittedNote>, String>;
```

It returns the file name and the file's contents, and decides nothing. Whether
a note is *admissible* is `policy.rs`'s question, and that module stays under
the lint. The exemption therefore covers reading, not judging.

The result is fallible, and that is the point of F3's fix: an *absent*
directory yields an empty vector, because no honest note can exist before task
2.2.1 annotates one, but a present directory that cannot be enumerated or whose
file cannot be read is an error naming the path. Discarding that error would
skip a note silently, and a skipped note is indistinguishable from no note at
all — the defect the marker's line-of-its-own rule exists to prevent, one level
down. `scan_scenarios.rs` carries the control, passing a directory where a file
is expected, so the failure is forced without any module outside the exemption
having to write to disk.

`types.rs` owns the tokens and the error. `Register` is the key that makes
every repair message derivable:

```rust,ignore
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Register { Status, Aggregation, Gates, Note, Evidence }

impl Register {
    pub(crate) const fn document(self) -> &'static str;
    pub(crate) const fn begin(self) -> &'static str;
    pub(crate) const fn end(self) -> &'static str;
    pub(crate) const fn section(self) -> &'static str;
    pub(crate) const fn label(self) -> &'static str;
}

pub(crate) enum ParseError {
    MissingDelimiters { register: Register },
    MissingSection { register: Register },
    MalformedRow { register: Register, row: usize },
    UnknownToken { register: Register, column: &'static str, found: String },
}
```

`parse.rs` owns one syntax function plus typed mappers:

```rust,ignore
pub(crate) fn parse_table(source: &str, register: Register)
    -> Result<Vec<Vec<String>>, ParseError>;
pub(crate) fn status_rows(adr: &str) -> Result<Vec<StatusRow>, ParseError>;
pub(crate) fn aggregation_rows(adr: &str) -> Result<Vec<AggRow>, ParseError>;
pub(crate) fn note_rows(note: &str) -> Result<Vec<NoteRow>, ParseError>;
pub(crate) fn field_order(rows: &[StatusRow]) -> Vec<String>;
```

`policy.rs` owns the predicates. Sources are bundled into one struct because
`clippy.toml` sets `too-many-arguments-threshold = 4`, and
`needless_pass_by_value` is denied, so it is passed by reference:

```rust,ignore
pub(crate) struct Sources<'a> {
    pub(crate) adr: &'a str,
    pub(crate) design: &'a str,
    pub(crate) roadmap: &'a str,
    pub(crate) adr_002: &'a str,
}

pub(crate) fn check_success_criterion(rows: &[StatusRow], roadmap: &str)
    -> Result<(), String>;
pub(crate) fn check_exclusions(rows: &[StatusRow]) -> Result<(), String>;
pub(crate) fn check_aggregation_total(rows: &[AggRow]) -> Result<(), String>;
pub(crate) fn resolve_note(rows: &[StatusRow], note: &[NoteRow])
    -> Result<Resolution, String>;
pub(crate) fn check_quoted_clauses(sources: &Sources<'_>) -> Result<(), String>;
pub(crate) fn check_gate_titles(adr: &str, roadmap: &str) -> Result<(), String>;
```

`Resolution` is `Sufficient`, `Insufficient`, or
`NotResolved { field, status }`, and carries a `Display` impl, because `Debug`
would print `NotResolved` where the documents say `Not resolved`. Note that
`resolve_note` takes the status register as an argument: admissibility and
verdict are both *derived* from the document, so adding a blocking status to
ADR 004 changes behaviour without a Rust edit. Nothing about the vocabulary is
hardcoded.

Every policy function returns `Result<(), String>` whose `Err` is the exact
repair message a control asserts. Nothing panics on a document defect;
`unwrap_used` and `indexing_slicing` are denied, so parsing uses `split_once`,
slice patterns, `get`, and `let ... else`. `option_if_let_else` is denied, so
prefer `map_or_else`. Test bodies may use `.expect(...)`, which `clippy.toml`
re-permits in tests.

## Revision note

- 2026-09-18, first draft. Two documents, a two-axis sufficiency register, ten
  invariants, and three approval-gate questions including the addition of
  `proptest` and `rstest-bdd`.
- 2026-09-18, second draft after five of six review lenses reported.
  Substantive changes, each traceable to a finding:
  - The verdict axis now records a required *property*, not an operation a
    string cannot perform. The first draft could not have recorded the most
    likely real finding — that variant-name `&'static str` is not stable across
    a rename, which design §9 makes semver-relevant. See D5 and Finding one.
  - Admissibility is separated from verdict, so cardinality gates usability and
    the verdict register has one axis and two rows. The first draft spent two
    of four cells on a non-terminating "repair the names" loop. See D3.
  - An aggregation register was added: three notes reach task 3.2.1 and the
    first draft resolved only one. See D8.
  - Filled notes gained a home (`docs/validation-notes/`), a committed worked
    example, and `INV-FILLED`. The first draft checked the blank form and never
    a filled one. See D1.
  - The template is now generated from the field register and checked by byte
    equality, deleting a parser and collapsing three invariants into one.
  - Gates bind by task title, not number. The first draft would have broken the
    build on the day roadmap task 2.2.1 was legitimately completed, and would
    have frozen seven task numbers against the project's own `mapsplice`
    tooling. See D7.
  - Q1 is now declined. Its safety argument was verified against the wrong
    function: `tests/v0_1_exit_register_contract.rs:119-126` embeds the §11.1
    B1 row byte-exactly with its padding, and `mdtablefix` repads the table
    when a wider cell is added. See D12.
  - Q3 is declined on both halves, on a measured 45-to-185 package increase.
    Behavioural coverage is delivered as scenario-named `rstest` cases over the
    committed worked example. See D9.
  - `INV-ANCHORS` narrowed from six clauses to three, gained mandatory
    whitespace folding, gained an empty-evidence-section control, and gained
    ADR 002 — whose clause the first draft called load-bearing while leaving it
    out of the guarded set.
  - Corrected premises: `clippy::panic` is not denied, only
    `panic_in_result_fn`; `expect_used` is re-permitted in tests;
    `too-many-arguments-threshold` is 4,
    which the first draft's five-parameter signature would have breached;
    `MD013.tables` is already `false`, so the first draft's table-bracketing
    mitigation addressed a problem that does not exist, while its own
    216-column fenced previews would have failed `make markdownlint`.
  - Corrected facts: `src/lib.rs` is twelve lines; the repository root is no
    longer given as an absolute worktree path.
- 2026-09-18, third draft after the sixth review lens — alternatives —
  reported. It argued for collapsing to a single document; that carrier change
  was declined, because the template's copy ergonomics are worth one file and
  the drift it would have removed is removed instead by D16. Every other
  finding was adopted:
  - The field and verdict registers merged into one status register, so
    admissibility and verdict are derived from the document rather than
    hardcoded. This also repaired two token mismatches the second draft had
    introduced against its own D6. See D13.
  - `state-display-name` now enumerates the returned strings, which the second
    draft never captured, leaving the cardinality bound unaudited. See D14.
  - `INV-FILLED` keys on a marker rather than a glob, so the shared
    `docs/validation-notes/` directory does not turn a later benchmark or exit
    note into a build failure. See D15.
  - The committed worked example was withdrawn: no honest note can cite work
    that has not happened, so it would have been fabricated evidence serving as
    the suite's accepting witness. The illustration moved into ADR 004 and the
    witness became a fixture. See D15.
  - `INV-TEMPLATE` compares parsed values rather than bytes, removing a
    dependency on `mdtablefix`'s padding algorithm. See D16.
  - Q1 was reframed: `docs/design.md` §14 is a better mechanism than §11.1, and
    its bullet is added in this task. Bet B7 is drafted, measured inert, and
    deferred to a separate change. See D12.
  - The aggregation register's outcomes are now explicitly conditional on
    publication proceeding, so they cannot read as contradicting ADR 003's
    exit E1.
- 2026-09-18, approved. Status moved from `DRAFT` to `APPROVED`. All five
  referred decisions were settled as recommended and the "Open questions"
  section became "Approval-gate decisions"; its reasoning is retained because
  it explains the shape of the artefacts, but nothing in it remains open. The
  constraint forbidding a `docs/design.md` §11.1 edit is now unconditional, and
  the §14 bullet is in scope. No implementation has started: `Progress` is
  unchanged and Stage A has not run.
