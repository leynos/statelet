# Record the three possible v0.1 exits (roadmap 1.1.2)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`,
`Decision log`, `Outcomes & retrospective`, `Conformance basis`, and
`Verification plan` must be kept up to date as work proceeds.

Status: DRAFT

## Purpose / big picture

Statelet is a pre-1.0 Rust crate that does not yet know whether it should
exist. Its technical design records two low-confidence bets: that a real
segment of Rust developers wants a shared convention for handwritten state
machines (bet B1), and that a `#[transition]` attribute macro beats
`#[tracing::instrument]` plus a couple of helper functions (bet B2). The design
already describes, in scattered prose, what happens when each bet fails. It
does not anywhere state the resulting outcomes as a single, total, mutually
exclusive set.

After this change, a reader who opens one document can answer three questions
without reconstructing them from four separate design sections:

1. What are the only three things Statelet can ship as v0.1?
2. What observation, at which point in the roadmap, selects each one?
3. Which of the two named bets does that observation falsify or confirm?

Success is observable in two ways. First, a new decision record,
`docs/adr-003-v0-1-exit-register.md`, states the three exits and binds each to
concrete evidence in `docs/design.md`. Second, and more importantly, that
binding is machine-checked: running `make test` executes a contract test that
parses the decision record and fails if the three exits stop covering every
combination of bet verdicts, or if the design-document anchors the record cites
stop existing. The document cannot silently rot away from the design it claims
to summarize.

This is a documentation and test change. It adds no runtime code and no public
API. That is deliberate: roadmap phase 1 exists precisely to settle exit
criteria *before* speculative public API is written.

## Context and orientation

Read this section first if you have never seen this repository.

### What Statelet is

Statelet is a "transition-boundary toolkit" for ordinary Rust state machines. A
**transition boundary** is a method or function where stateful logic decides to
stay in its current state, move to another, emit output, ignore input, or fail.
Statelet's accepted scope, fixed by
[ADR 002](../adr-002-transition-boundary-scope.md), is to *mark* those
boundaries and nothing more: it does not own dispatch, events, storage,
transition tables, or graph safety. The glossary for all such terms is
`docs/context.md`; treat it as normative and cite it rather than redefining
terms locally.

### The current state of the repository

This is, today, a documentation-heavy and code-empty skeleton:

- `src/lib.rs` contains a single nine-line stub, `greet()`, marked with a
  `TODO` to delete it once real functionality exists.
- `Cargo.toml` declares package `statelet` version 0.1.0, edition 2024, licence
  ISC, with an **empty** `[dependencies]` table and only
  `camino = "1.2.5"` and `rstest = "0.26.1"` under `[dev-dependencies]`.
- `tests/stub.rs` is a disposable placeholder.
- `tests/dev_fast_contract.rs` is the one real test in the repository. It is a
  *contract test*: it asserts a property of the repository's own build
  configuration rather than of runtime behaviour. This plan follows its style
  closely, so read it before starting.

There is no workspace, no proc-macro crate, and no runtime API. Everything this
plan touches is documentation plus one new integration test.

### The documents this plan depends on

- `docs/design.md` — the technical design. Sections that matter here:
  - §11.1 "Bet register": a six-row table (B1 to B6) of claims, confidence
    levels, and required evidence.
  - §11.2 "Baseline comparison": defines the non-macro `mdtablefix` baseline
    and the macro gate.
  - §13.6 "Macro is only `instrument` with extra steps".
  - §13.7 "Conventions baseline is also too weak".
  - §14 "Deferred decisions", which currently phrases the ship/no-ship question
    as an open item.
- `docs/terms-of-reference.md` — §7.1 states the "ship nothing" outcome in
  prose; §10.2 lists ADR candidates.
- `docs/roadmap.md` — phase 1, step 1.1 "Ratify the v0.1 product exits", task
  1.1.2 (the task this plan implements). Downstream, tasks 2.2.3 and 3.1.3 are
  the points at which an exit is actually chosen; task 2.2.3 says in terms that
  the note "chooses one of the three exits from 1.1.2".
- `docs/documentation-style-guide.md` — the authoritative ADR template and the
  prose rules (en-GB Oxford spelling, sentence-case headings, 80-column prose
  wrap, 120-column code wrap, captioned tables, GitHub-flavoured footnotes).
- `docs/contents.md` — the master documentation index; every new document must
  be registered there.
- `docs/whitaker-users-guide.md` — the Dylint lint suite that `make lint` runs.

### The verbatim evidence this plan is built on

These quotations are the raw material. They are reproduced here so the
implementer does not have to re-derive them.

From `docs/design.md` §11.1, rows B1 and B2:

```plaintext
| B1 | A real segment prefers handwritten state machines and wants shared
       convention | Low-medium | `mdtablefix` plus one second non-toy example
       both improve without framework adoption |
| B2 | `#[transition]` beats `#[tracing::instrument]` plus helper functions
     | Low | A head-to-head `mdtablefix` baseline comparison states the
       concrete value added by the macro |
```

From `docs/design.md` §11.2:

```plaintext
The macro gate passes only if the design note can state, in one paragraph, what
`#[transition]` adds over that baseline. ... If the baseline is the more honest
result, the macro crate does not ship in v0.1.
```

From `docs/design.md` §13.6:

```plaintext
If the macro cannot beat the baseline in §11.2, the project ships the
conventions/runtime crate and defers `statelet-macros`. This is a successful
validation outcome, not a failed implementation.
```

From `docs/design.md` §13.7:

```plaintext
If `StateName`, documented transition fields, and local helpers add little over
plain `#[tracing::instrument(fields(state = %self.mode))]` in both validation
examples, the project should ship nothing and keep the pattern local. That is
the B1 failure case and must be treated as a valid discovery outcome.
```

### The gap this plan closes

The design states two of the three exits explicitly (§13.6 gives "ship
conventions only"; §13.7 gives "ship nothing"). The third, "ship the macro", is
only implied — it is the negation of §13.6, defined positively by the §11.2
gate. Nowhere does any document state:

- that the three outcomes are exhaustive;
- what happens in the fourth logical combination, where B1 is falsified but B2
  looks good; or
- at which roadmap gate each verdict is taken.

Those three items are the intellectual content of task 1.1.2. Everything else
is transcription.

### Files this plan reads or writes

Reads only:

- `docs/design.md`
- `docs/terms-of-reference.md`
- `docs/documentation-style-guide.md`
- `docs/whitaker-users-guide.md`
- `tests/dev_fast_contract.rs`
- `AGENTS.md`, `Makefile`, `clippy.toml`, `.markdownlint-cli2.jsonc`

Creates:

- `docs/adr-003-v0-1-exit-register.md`
- `tests/v0_1_exit_register_contract.rs`
- `tests/features/v0_1_exit_selection.feature`
- `tests/snapshots/` (one `insta` snapshot file, generated)

Edits:

- `Cargo.toml` (dev-dependencies only)
- `docs/context.md`
- `docs/contents.md`
- `docs/design.md`
- `docs/terms-of-reference.md`
- `docs/roadmap.md`
- `docs/users-guide.md`
- `docs/developers-guide.md`
- `docs/repository-layout.md`
- `README.md`

### Skills to load before starting

Load these, in this order:

- `execplans` — this document's own format and the living-section discipline.
- `leta` — semantic code navigation. Run `leta workspace add .` once. Prefer
  `leta show`, `leta refs`, and `leta grep` over reading whole files.
- `rust-router` — routes to the smallest useful Rust skill. From it, load
  `rust-unit-testing` for the `rstest` fixture, table-test, and assertion
  guidance this plan relies on.
- `proptest` — for the single ordering property in the verification plan.
- `en-gb-oxendict` — the repository enforces British English with Oxford
  spelling through `typos`; `make markdownlint` will fail otherwise.
- `addressing-whitaker-findings` — if `make lint` reports Dylint findings.

Do **not** load `arch-decision-records` for the ADR's shape. That skill
prescribes a `docs/adr/` directory and a bare six-clause Y-Statement. This
repository uses its own template from `docs/documentation-style-guide.md`
(files named `docs/adr-NNN-short-description.md`, sections Status / Date /
Context and problem statement plus conditional sections). The repository's guide
wins.

Do **not** load `kani` or `verus`. The verification plan below explains why
neither is warranted here.

## Conformance basis

Upstream artefacts and their revisions at the time of writing:

- Terms of reference: `docs/terms-of-reference.md`, "Draft v0.2 after design
  review", last substantive revision 2026-06-14.
- Technical design: `docs/design.md`, "Draft v0.2 after design review", last
  substantive revision 2026-06-15.
- Decision records: `docs/adr-001-proving-ground-candidates.md`;
  `docs/adr-002-transition-boundary-scope.md` (Accepted, 2026-07-22).
- Roadmap: `docs/roadmap.md` at commit `78c30a1`.
- Governing standard: `docs/documentation-style-guide.md`.

Traced items and their chain:

```plaintext
TOR-7.1-ship-nothing  -> TDD-11.1-B1 -> EP-M2 -> ADR-003 exit E1 -> tests::exit_register::total_mapping
TDD-11.1-B2           -> TDD-13.6    -> EP-M2 -> ADR-003 exit E2 -> tests::exit_register::b1_dominates
TDD-11.2-macro-gate   -> TDD-11.1-B2 -> EP-M2 -> ADR-003 exit E3 -> features::v0_1_exit_selection
ROADMAP-1.1.2         -> EP-M1..M4   -> ADR-003 + roadmap checkbox ticked
ROADMAP-2.2.3, 3.1.3  -> ADR-003 gate bindings G1, G2
TOR-10.2-adr-candidate("macro-first versus trait-first API shape") -> ADR-003 (partial: ADR 003 fixes the *decision procedure*, not the shape)
```

ADR 002 explicitly declines to decide macro-versus-conventions; its non-goals
say those questions "are tracked separately". This plan therefore does not
deviate from ADR 002, it discharges a sibling obligation.

No upstream artefact defines a document type called an "exit note". The
roadmap's phrase "the exit note" (line 44) is interpreted in this plan as the
decision record produced here. See `Decision log` entry D1.

## Constraints

Hard invariants. If satisfying the objective requires violating one, stop and
escalate rather than working around it.

1. **No runtime code.** Nothing is added to `src/`. Roadmap phase 1 forbids
   speculative public API before the exit criteria are settled; adding an exit
   type or a decision function to the library would be exactly the mistake the
   phase exists to prevent. All new code lives under `tests/`.
2. **No public API surface.** `Cargo.toml` `[dependencies]` stays empty. Only
   `[dev-dependencies]` may gain entries.
3. **The ADR does not choose an exit.** It records the three exits and their
   trigger conditions. Choosing among them is roadmap task 2.2.3 and 3.1.3,
   which do not yet have evidence. Any draft that picks a winner is wrong.
4. **The ADR must not restate design.md's substance divergently.** Where the
   design already states a rule, the ADR quotes or cites it. Where the ADR adds
   something the design does not say (the exhaustiveness claim, the dominance
   rule, and the gate bindings), it must say so explicitly and the design must
   gain a pointer back.
5. **Repository ADR template only.** File named `docs/adr-003-<slug>.md`;
   sections in the order the style guide mandates.
6. **en-GB Oxford spelling**, sentence-case headings, prose wrapped at 80
   columns, code at 120, every table captioned, every fenced block given a
   language identifier.
7. **`std::fs` is forbidden in test code.** Whitaker's `no_std_fs_operations`
   lint denies it. This plan uses `include_str!`, which reads at compile time
   and needs no filesystem crate at all.
8. **`unwrap()` is denied even in tests.** `clippy.toml` sets
   `allow-expect-in-tests = true` but does **not** set
   `allow-unwrap-in-tests`. Use `.expect("...")` with a message, or `let ...
   else { panic!(...) }`, as `tests/dev_fast_contract.rs` does.
9. **Complexity ceilings apply to test code.** `clippy.toml` sets
   `cognitive-complexity-threshold = 9`, `too-many-arguments-threshold = 4`,
   `too-many-lines-threshold = 70`, and `excessive-nesting-threshold = 4`.
   Keep parser helpers small and flat.
10. **`make check-fmt`, `make lint`, and `make test` must all pass** at every
    milestone boundary, not only at the end.

## Tolerances (exception triggers)

Stop and escalate rather than improvising when any of these is reached.

- **Scope**: more than 12 files touched, or more than roughly 700 net added
  lines across all files. Both figures are generous for this task; exceeding
  them means the plan has misjudged something.
- **Dependencies**: more than the five dev-dependencies named in `Interfaces
  and dependencies`. Each was explicitly authorized; a sixth was not.
- **Runtime code**: any change to `src/`, or any addition to `Cargo.toml`
  `[dependencies]`. Stop immediately; this violates Constraint 1 or 2.
- **Design conflict**: if drafting the exit register reveals that `design.md`
  §§11.1, 11.2, 13.6, and 13.7 are mutually inconsistent, or that the three
  exits genuinely cannot be made exhaustive, stop. That is a design defect, not
  a documentation defect, and it requires a change to `docs/design.md` agreed
  with the user.
- **A fourth exit**: if the evidence supports a fourth outcome that cannot be
  folded into the three (for example, shipping the macro behind a
  `--cfg statelet_unstable` flag as `tokio` does for unstable APIs), stop and
  present it. The roadmap says three; changing that number changes roadmap
  tasks 2.2.3 and 3.1.3 as well.
- **Iterations**: if `make test` still fails after three focused attempts on
  the same assertion, stop and report the transcript.
- **Gate failures**: if `make lint` or `make markdownlint` fails for a reason
  not covered by this plan's `Idempotence and recovery` section after two
  attempts, stop and report.
- **Ambiguity**: if the ADR's status should plausibly be `Proposed` rather than
  `Accepted` at completion, present both readings rather than choosing.

## Risks

- Risk: the ADR becomes a paraphrase of `design.md` and adds no information,
  making the whole task ceremonial.
  Severity: high. Likelihood: medium.
  Mitigation: the three additions in `Context and orientation` → "The gap this
  plan closes" (exhaustiveness, the dominance rule, the gate bindings) are the
  deliverable. The contract test asserts all three. If those three are removed,
  the test goes red.

- Risk: the contract test is written so loosely that it passes on any document
  containing the right words, i.e. verification theatre.
  Severity: high. Likelihood: medium.
  Mitigation: every obligation in `Verification plan` carries a named negative
  control — a mutated fixture the parser must reject for a stated reason. The
  negative controls are written and observed failing before the live document
  is asserted against.

- Risk: the exit register drifts from `design.md` after a later design
  revision, silently.
  Severity: medium. Likelihood: high over the project's lifetime.
  Mitigation: obligation `INV-ANCHORS` re-derives the bet identifiers and the
  §13.6/§13.7 headings from `docs/design.md` itself at compile time via
  `include_str!`. A design edit that renames or removes them breaks `make
  test`.

- Risk: five new dev-dependencies on a crate with one real test is
  disproportionate.
  Severity: low. Likelihood: high (a reviewer will raise it).
  Mitigation: each is separately justified in `Interfaces and dependencies`,
  all five are named in `AGENTS.md` as the project's standard test stack, and
  all are dev-only so none reaches a consumer. If the reviewer disagrees,
  `insta` and `proptest` are the two that can be dropped with the least loss;
  `rstest`, `rstest-bdd`, and `googletest` carry the core obligations.

- Risk: `rstest-bdd` 0.5 does not integrate cleanly on this toolchain, since
  the repository has never used it.
  Severity: medium. Likelihood: medium.
  Mitigation: EP-M1 adds it and runs a single trivial scenario first, before
  any real scenario is written. If it does not work, record the finding and
  fall back to `rstest` table tests for the same four cases, noting the
  substitution in `Decision log`. Do not spend more than one hour on
  integration.

- Risk: the roadmap's "three exits" framing is subtly wrong, because "ship
  conventions only" and "ship macro" are not symmetric — the macro exit
  presupposes the conventions crate ships too.
  Severity: medium. Likelihood: medium.
  Mitigation: the ADR states this nesting explicitly rather than pretending the
  three are disjoint products. They are disjoint *release scopes*: E3 ships a
  strict superset of E2. The decision table remains a partition because it
  partitions verdict combinations, not artefacts.

## Progress

- [ ] EP-M0: plan approved by the user.
- [ ] EP-M1: dev-dependencies added; test harness and negative controls in
      place; ADR 003 present as `Proposed` with only the two exits `design.md`
      states verbatim; `make test` red on `INV-TOTAL` for the stated reason.
- [ ] EP-M2: exit register completed (third exit, dominance rule, gate
      bindings); ADR status `Accepted`; `make test` green.
- [ ] EP-M3: companion-document sync complete; roadmap 1.1.2 ticked.
- [ ] EP-M4: full gates green; branch pushed; pull request opened.

Add a UTC timestamp to each line as it completes, in the form
`- [x] (2026-08-22 14:05Z) EP-M1: ...`.

## Surprises & discoveries

None yet. Record observations here as `Observation / Evidence / Impact`
triples during implementation.

Two findings from the planning pass are worth carrying forward:

- Observation: the repository has no `allow-unwrap-in-tests` setting even
  though it sets `allow-expect-in-tests = true`.
  Evidence: `clippy.toml` line 7 sets only the latter; `Cargo.toml` denies
  `unwrap_used` and `expect_used`.
  Impact: test code must use `.expect()` or explicit `panic!`, never
  `.unwrap()`.

- Observation: no crate in the Rust ecosystem was found that publicly
  pre-commits to kill criteria or possible abandonment.
  Evidence: a web survey of the `std` feature lifecycle, the Rust RFC template,
  and `tokio`'s `--cfg tokio_unstable` idiom found staged *gating* but no
  published *quit* conditions.
  Impact: the ADR must not claim ecosystem precedent for this practice. It may
  cite the Stage-Gate Go/Kill/Hold vocabulary and Duke's kill-criteria framing
  as the actual antecedents, which are managerial rather than Rust-specific.

## Decision log

- Decision D1: the deliverable is an ADR, `docs/adr-003-v0-1-exit-register.md`,
  not an untemplated "note".
  Rationale: `docs/roadmap.md` step 1.1 is titled "Ratify the v0.1 product
  exits", and ratification is a decision act. Task 1.1.1 set the ADR precedent
  for exactly this step. `docs/documentation-style-guide.md` supplies an ADR
  template and no "note" template, so an untemplated file would have no house
  form to follow. `docs/terms-of-reference.md` §10.2 already anticipates ADRs
  in this area. The roadmap's phrase "the exit note" is read as informal
  reference to that record. **This is the single most consequential
  interpretation in the plan; if the user intends a lighter artefact, say so at
  the approval gate and the plan collapses to EP-M2 plus EP-M3.**
  Date/Author: 2026-08-22, planning agent.

- Decision D2: the ADR is a single decision, not three.
  Rationale: no established decision-record format — MADR 4.0, Nygard, or the
  Y-Statement — supports recording several alternative *future* outcomes; all
  assume one chosen option. The honest single decision is: *defer the
  ship/no-ship choice and bind it to named evidence at named gates.* The three
  exits then sit naturally under "Options considered", and the trigger
  conditions under an added "Exit register" section. This keeps the template's
  shape intact without distorting it.
  Date/Author: 2026-08-22, planning agent.

- Decision D3: keep the word "exit" despite its ambiguity.
  Rationale: research flagged that "exit criteria" already means "definition of
  done for testing" in ISTQB usage and "phase-completion conditions" in
  acquisition usage, so a third sense invites misreading. However, the term is
  already load-bearing across this repository — `docs/roadmap.md` step 1.1
  title, tasks 2.2.3 and 3.1.3, and `README.md`'s documentation index all use
  it. Renaming would fork the vocabulary. The correct fix is to define the term
  precisely once, in the normative glossary `docs/context.md`, which is what
  that file exists for. Mitigation is a glossary entry, not a rename.
  Date/Author: 2026-08-22, planning agent.

- Decision D4: bind each exit to a roadmap gate rather than a calendar date.
  Rationale: kill criteria are only binding when they specify both a *state*
  and a *when*. This is a hobby-cadence open-source project, so calendar dates
  are meaningless, but the roadmap already supplies natural decision points:
  task 2.2.3 ("Decide the Phase 2 exit") and task 3.1.3 ("Decide whether the
  runtime/conventions crate earns publication"). Using them as the "when"
  substitutes a resource-and-progress budget for a date, which is the
  recognized alternative.
  Date/Author: 2026-08-22, planning agent.

- Decision D5: reject a fourth exit ("ship the macro behind an unstable cfg").
  Rationale: `tokio` gates unstable APIs behind a bare `--cfg tokio_unstable`
  RUSTFLAG, deliberately excluding them from semantic-versioning guarantees.
  This is a real and attractive option. It is nevertheless rejected for v0.1
  because it would let the project ship the macro *without* passing the §11.2
  gate, which is precisely the discipline roadmap phase 1 exists to impose, and
  because the roadmap fixes the count at three. It is recorded in the ADR's
  "Options considered" as considered-and-rejected so the reasoning is not lost.
  Date/Author: 2026-08-22, planning agent.

- Decision D6: verification is by contract test over the document text, not by
  a runtime decision function.
  Rationale: a `fn select_exit(b1: Verdict, b2: Verdict) -> Exit` in `src/`
  would make the invariants trivially checkable, and would be exactly the
  speculative public API that Constraint 1 forbids. Parsing the ADR's own
  tables keeps the document as the single source of truth and makes drift, not
  just logic error, detectable.
  Date/Author: 2026-08-22, planning agent.

## Verification plan

The change adds no runtime behaviour, so there is nothing to verify about
program execution. It does introduce three non-trivial propositions about the
*document*, and those are checkable. Each is stated below with a method, a
domain, an artefact, evidence, and a non-vacuity argument.

The parsing and validation logic lives in a private module inside
`tests/v0_1_exit_register_contract.rs`. It is a pure function from `&str` to
`Result<ExitRegister, RegisterError>`. Purity is a deliberate design choice: it
makes every negative control a one-line string literal, needs no filesystem,
and keeps the obligations independent of where the document lives.

### Obligation INV-TOTAL — the exit mapping is total

- **Obligation**: for every combination of verdicts on bets B1 and B2 — each
  being `Falsified` or `Held` — the ADR's decision table names exactly one
  exit. Four combinations, no gaps, no duplicates.
- **Method**: exhaustive parameterized test (`rstest` `#[case]` per
  combination) over the parsed table, plus a totality check in the parser.
- **Rationale**: the domain has four elements. Exhaustive enumeration *is* the
  proof; a property test would generate the same four cases with extra
  machinery, and a bounded model checker or a `verus` proof would add ceremony
  without adding rigour over a complete case split. Recording that judgement
  here is required by the plan format; see also the note on `kani`/`verus`
  below.
- **Domain**: `{Falsified, Held} × {Falsified, Held}`, all four elements.
- **Artefact**: `tests/v0_1_exit_register_contract.rs`, tests
  `every_verdict_combination_maps_to_one_exit` and
  `parser_rejects_incomplete_table`.
- **Evidence**: `make test`. At EP-M1 this fails with a message naming the
  uncovered combination, for example `uncovered verdict combination: (B1=Held,
  B2=Held)`. At EP-M2 it passes.
- **Non-vacuity**: the parser is exercised against four fixture strings — a
  complete table (witness that the check can pass), a table with one row
  deleted (must fail with `Incomplete`), a table with a row duplicated under a
  different exit (must fail with `Ambiguous`), and a table with an unknown exit
  identifier (must fail with `UnknownExit`). Each expected error variant is
  asserted specifically, not merely `is_err()`. If the parser were a no-op
  returning `Ok`, three of these four fixtures would fail the suite.

### Obligation INV-DOMINANCE — a falsified B1 forces the off-ramp

- **Obligation**: whenever B1 is falsified, the selected exit is E1 ("ship
  nothing"), regardless of B2's verdict.
- **Method**: parameterized test over the two `B1=Falsified` rows, plus a
  behavioural scenario in `tests/features/v0_1_exit_selection.feature`.
- **Rationale**: this is the one rule the design does not currently state, and
  the one a careless reader would get wrong — it is tempting to think a
  brilliant macro could rescue a wedge nobody wants. B1's required evidence in
  §11.1 is that *both* validation examples improve "without framework
  adoption"; if that fails, there is no baseline for the macro to beat, so B2's
  verdict is not merely outweighed but undefined. Encoding it behaviourally
  makes the reasoning legible to a human reviewer as well as to the compiler.
- **Domain**: the two rows where B1 is `Falsified`.
- **Artefact**: `tests/v0_1_exit_register_contract.rs`, test
  `falsified_b1_always_selects_ship_nothing`; and the scenarios
  "B1 falsified at the Phase 2 gate" and "B1 falsified despite a promising
  macro" in the feature file.
- **Evidence**: `make test`. Red at EP-M1 (the rows do not exist yet), green at
  EP-M2.
- **Non-vacuity**: a negative-control fixture maps `(Falsified, Held)` to E3
  ("ship macro"). The test must reject that fixture with
  `DominanceViolated { .. }`. Without this control, an assertion that merely
  read the table back would pass on any table whatsoever. A witness fixture
  with the correct mapping must be accepted, proving the check is not
  unconditionally failing.

### Obligation INV-ANCHORS — every design citation resolves

- **Obligation**: every design-document anchor the ADR cites still exists in
  `docs/design.md`: the bet identifiers `B1` and `B2` as rows of the §11.1
  table, and the headings `### 13.6 Macro is only \`instrument\` with extra
  steps`, `### 13.7 Conventions baseline is also too weak`, and
  `### 11.2 Baseline comparison`.
- **Method**: a cross-document contract test. Both documents are embedded at
  compile time with `include_str!`; the test extracts the citation set from the
  ADR and checks each against the design text.
- **Rationale**: this is the obligation that earns the test its keep. The other
  two protect against a logic slip today; this one protects against silent rot
  over the project's whole life. A `design.md` revision that renumbers §13.6 or
  drops bet B1 will break `make test` in the same commit that causes the drift,
  which is the only moment at which it is cheap to fix.
- **Domain**: the citation set extracted from the ADR — at least
  `{B1, B2, §11.2, §13.6, §13.7}`.
- **Artefact**: `tests/v0_1_exit_register_contract.rs`, test
  `every_design_citation_resolves`.
- **Evidence**: `make test`.
- **Non-vacuity**: two controls. First, a fixture design text with the §13.7
  heading removed must produce `DanglingCitation { anchor: "13.7", .. }`.
  Second — and this is the control that catches the worse failure — a fixture
  ADR citing a section that has never existed, `§99.9`, must also be rejected;
  this proves the check reads the ADR's citations rather than a hard-coded
  list. A witness pair (the real ADR against the real design) must be accepted.

### Snapshot: normalized exit register

- **Obligation**: not an invariant, a review aid. The parsed register is
  rendered to a stable normalized form and snapshotted with `insta`.
- **Rationale**: it makes semantic changes to the exit register visible as a
  small readable diff in review, rather than as a prose diff a reviewer might
  skim. Included because this document's whole value is that its meaning does
  not drift quietly.
- **Artefact**: `tests/v0_1_exit_register_contract.rs`, test
  `exit_register_snapshot`; snapshot under `tests/snapshots/`.
- **Non-vacuity**: the snapshot is generated at EP-M2 from the completed
  register and reviewed by eye before acceptance. It is not created at EP-M1,
  where the register is deliberately incomplete.

### Property: row ordering does not change meaning

- **Obligation**: parsing is invariant under permutation of the decision-table
  rows. Formally, for any permutation `p` of the four rows, `parse(render(p))`
  yields the same `ExitRegister`.
- **Method**: `proptest`, generating permutations of the row set.
- **Rationale**: this is the one genuine invariant over an *ordering* rather
  than a finite partition, which is the criterion for reaching for a property
  test. It matters because a future editor will reorder rows for readability,
  and the test must not encode incidental order as meaning.
- **Domain**: all 24 permutations of four rows; `proptest` samples them, and a
  regression file records any failing case.
- **Artefact**: `tests/v0_1_exit_register_contract.rs`, test
  `parsing_is_order_independent`.
- **Non-vacuity**: the generator's output is asserted non-empty and to include
  at least one non-identity permutation, so a generator that only ever emitted
  the identity would fail. The negative control is a deliberately
  order-sensitive parser variant during development: if `parse` is changed to
  return rows as a `Vec` compared by position, the property must fail.

### Methods deliberately not used

- **`kani`** (bounded model checking): there is no unsafe code, no arithmetic,
  no memory-safety question, and no state machine with a transition depth worth
  bounding. The state space is four elements.
- **`verus`** (deductive proof): the only lemma in sight — that the four-row
  mapping is a total function — is discharged completely by enumerating four
  cases. A `verus` proof would restate the assumption in a different syntax,
  which the plan format explicitly rejects as vacuous.
- **End-to-end tests**: there is no externally observable workflow, network
  boundary, persistence format, or command-line surface. The `make test`
  invocation *is* the observable workflow, and it is exercised directly.

## Plan of work

### Stage A — orient and confirm (no changes)

Read `docs/design.md` §§11.1, 11.2, 13.6, 13.7, and 14; `docs/roadmap.md`
step 1.1 and tasks 2.2.3 and 3.1.3; `docs/adr-002-transition-boundary-scope.md`
in full, for house ADR voice; and `tests/dev_fast_contract.rs`, for house test
voice. Confirm no file matching `docs/adr-003-*` or `docs/*exit*` already
exists.

Validation: the four quotations in `Context and orientation` above still match
the current text of `docs/design.md` verbatim. If any has changed, stop — the
conformance basis is stale and the plan must be revised before proceeding.

### Stage B — red (EP-M1)

Add the five dev-dependencies. Write
`tests/v0_1_exit_register_contract.rs` complete with its parser, its error
enum, and all negative-control fixtures. Write
`tests/features/v0_1_exit_selection.feature` with its scenarios. Create
`docs/adr-003-v0-1-exit-register.md` with `Status: Proposed`, full context and
options prose, and a decision table containing **only** the two rows that
`docs/design.md` states verbatim:

```plaintext
| B1 falsified | B2 falsified | E1 ship nothing        | design.md §13.7 |
| B1 held      | B2 falsified | E2 ship conventions    | design.md §13.6 |
```

This is an honest intermediate state: it is precisely what the design says
today, and its incompleteness is the gap task 1.1.2 exists to close.

Validation (go/no-go): `make test` fails, and the failure names the uncovered
combinations `(B1=Held, B2=Held)` and `(B1=Falsified, B2=Held)`. All
negative-control tests pass. `make check-fmt`, `make lint`, and
`make markdownlint` pass. If `make test` passes at this point, the test is
vacuous — stop and fix the test before writing any more of the ADR.

### Stage C — green (EP-M2)

Complete the exit register. Add the two missing rows, the dominance rule, exit
E3, the gate bindings for G1 and G2, and the considered-and-rejected fourth
option from Decision D5. Change `Status` to `Accepted` with a one-line summary,
following ADR 002's convention of stating the decision once and not restating
it in the status line. Generate and review the `insta` snapshot.

Validation (go/no-go): `make test` passes, including the four BDD scenarios and
the ordering property. `make check-fmt`, `make lint`, `make markdownlint`, and
`make nixie` pass.

### Stage D — companion sync and wider validation (EP-M3, EP-M4)

Apply the documentation sync map below, tick roadmap 1.1.2, then run the full
gate set. Commit at each milestone.

## Milestones and plateaus

### EP-M1 — red plateau

- **Outcome**: the repository contains a proposed ADR that faithfully reflects
  what `design.md` says today, plus a test suite that demonstrates the gap.
  This is coherent: a `Proposed` ADR with an acknowledged gap is a legitimate
  state, and the failing test documents exactly what is missing.
- **Requirements advanced**: `ROADMAP-1.1.2` (partially); `INV-TOTAL` and
  `INV-DOMINANCE` shown red for the intended reason.
- **Acceptance evidence**: transcript of `make test` showing the named
  uncovered combinations, with all negative-control tests green.
- **Conformance check**: no `src/` change; `[dependencies]` still empty; ADR
  filename and section order match the style guide; the four design quotations
  still match `design.md`.
- **Recovery**: `git checkout -- .` restores the tree. Nothing is destructive.
- **Remaining gaps**: the register is incomplete by design; no companion
  document has been updated.
- **Compatibility decision**: none required. Nothing is released, nothing is
  public, and there is no consumer.

### EP-M2 — green plateau

- **Outcome**: the exit register is complete, total, and accepted; every
  obligation in the verification plan is discharged.
- **Requirements discharged**: `TDD-11.1-B1`, `TDD-11.1-B2`, `TDD-11.2`,
  `TDD-13.6`, `TDD-13.7` are each cited by at least one exit and checked by
  `INV-ANCHORS`.
- **Acceptance evidence**: `make test` green; the `insta` snapshot reviewed and
  committed; the four BDD scenarios reported as passing by name.
- **Conformance check**: the ADR chooses no exit (Constraint 3); every claim
  the ADR makes beyond `design.md` is flagged as an addition (Constraint 4);
  no unapproved dependency, interface, or persisted format.
- **Recovery**: revert to the EP-M1 commit; the red state is a valid plateau.
- **Remaining gaps**: companion documents still point at nothing.
- **Compatibility decision**: none required.

### EP-M3 — documentation-sync plateau

- **Outcome**: every companion document references ADR 003; the glossary
  defines "v0.1 exit"; roadmap 1.1.2 is ticked.
- **Acceptance evidence**: `make markdownlint` and `make nixie` green; a link
  check confirms every new cross-reference resolves; `grep -c "adr-003"` over
  `docs/` returns at least the expected count.
- **Conformance check**: `docs/contents.md` lists the new record;
  `docs/design.md` and `docs/terms-of-reference.md` companion lists include it;
  no scope prose contradicts ADR 002.
- **Recovery**: each edit is a small, independently revertible insertion.
- **Remaining gaps**: none.
- **Compatibility decision**: none required.

### EP-M4 — delivery

- **Outcome**: branch pushed, draft pull request opened, gates green.
- **Acceptance evidence**: transcripts of `make check-fmt`, `make lint`, and
  `make test`, each captured to `/tmp`.
- **Recovery**: the pull request is a draft; it can be closed without effect.

## Concrete steps

Run everything from the repository root,
`/home/leynos/.lody/repos/github---leynos---statelet/worktrees/8a7120cf-1d08-4a72-9031-10a3d69a87de`.
Set a log prefix once per shell:

```bash
export LOG=/tmp/statelet-$(git branch --show-current)
```

### Step 1 — confirm the starting state

```bash
git branch --show-current
ls docs/adr-*.md
grep -c "^### 13.7 Conventions baseline is also too weak" docs/design.md
```

Expected:

```plaintext
1-1-2-record-the-three-possible-v0-1-exits
docs/adr-001-proving-ground-candidates.md
docs/adr-002-transition-boundary-scope.md
1
```

A `grep` result of `0` means the design has been revised and the conformance
basis is stale. Stop.

### Step 2 — add dev-dependencies

Edit `Cargo.toml`, `[dev-dependencies]` only, leaving `[dependencies]` empty:

```toml
[dev-dependencies]
camino = "1.2.5"
googletest = "0.14.3"
insta = "1.48.0"
pretty_assertions = "1.4.1"
proptest = "1"
rstest = "0.26.1"
rstest-bdd = "0.5.0"
```

Then:

```bash
cargo fetch 2>&1 | tee "$LOG-fetch.out"
```

If another Cargo job holds the package-cache lock, wait for it. Do not create
an isolated Cargo cache.

### Step 3 — verify the `rstest-bdd` integration before relying on it

Write one throwaway scenario and run it. If it does not work within an hour,
record the finding under `Surprises & discoveries`, fall back to `rstest` table
tests for the same four cases, note the substitution in `Decision log`, and
remove `rstest-bdd` from `Cargo.toml`.

### Step 4 — write the contract test (red)

Create `tests/v0_1_exit_register_contract.rs`. Its shape is specified in
`Interfaces and dependencies` below. Then:

```bash
make test 2>&1 | tee "$LOG-test-red.out"
```

Expected: a failure naming the uncovered combinations, alongside passing
negative controls. Something of this form:

```plaintext
---- every_verdict_combination_maps_to_one_exit stdout ----
Value of: register.exit_for(Verdict::Held, Verdict::Held)
Expected: is ok
Actual: Err(Incomplete { missing: [(Held, Held), (Falsified, Held)] })
```

### Step 5 — write the ADR (green)

Complete `docs/adr-003-v0-1-exit-register.md`, then:

```bash
make test 2>&1 | tee "$LOG-test-green.out"
cargo insta review
make check-fmt lint 2>&1 | tee "$LOG-lint.out"
make markdownlint 2>&1 | tee "$LOG-mdlint.out"
```

### Step 6 — companion sync

Apply every edit in the sync map below. Each insertion must be idempotent:
`grep` for the target string before inserting, so re-running the step is safe.

```bash
grep -rn "adr-003-v0-1-exit-register" docs/ README.md
```

Expected: one hit in each of `docs/contents.md`, `docs/design.md`,
`docs/terms-of-reference.md`, `docs/roadmap.md`, `docs/users-guide.md`,
`docs/developers-guide.md`, and `docs/context.md`.

### Step 7 — full gates and delivery

```bash
make check-fmt 2>&1 | tee "$LOG-checkfmt.out"
make lint      2>&1 | tee "$LOG-lint.out"
make test      2>&1 | tee "$LOG-test.out"
make markdownlint 2>&1 | tee "$LOG-mdlint.out"
make nixie     2>&1 | tee "$LOG-nixie.out"
```

Read each log rather than scrolling the terminal; long output is truncated by
the environment. Commit after each milestone, then push and open the pull
request.

## Documentation sync map

Follow the pattern established for ADR 002. Each edit is a small insertion; do
not rewrite surrounding prose.

1. `docs/contents.md`, "Decision records" section: add a bullet after the ADR
   002 entry, matching the existing two-line link-plus-description form.
2. `docs/design.md`, "Companion documents" list near the top: add
   `- \`docs/adr-003-v0-1-exit-register.md\``. Then, at the end of §11.1, add
   one sentence pointing to ADR 003 as the record that maps B1 and B2 to the
   three v0.1 exits, and at the end of §13.7 a sentence noting that §§13.6 and
   13.7 are registered there as exits E2 and E1. Update "Last substantive
   revision".
3. `docs/terms-of-reference.md`: add the same companion-document entry, and in
   §10.2 replace the plain-text ADR candidate line for macro-versus-conventions
   with a link to ADR 003 if it is now discharged, or add ADR 003 as a new
   linked bullet if it is not. Update "Last substantive revision".
4. `docs/context.md`: add a glossary entry defining **v0.1 exit** — "one of the
   three mutually exclusive release scopes Statelet may deliver as v0.1, each
   selected by a stated verdict on bets B1 and B2 at a named roadmap gate; not
   to be confused with test exit criteria." This entry is the mitigation for
   Decision D3.
5. `docs/roadmap.md`: tick task 1.1.2 to `- [x]`, and append the ADR link to
   its "Success:" bullet, matching how task 1.1.1 links ADR 002.
6. `docs/users-guide.md`, "Current status": add two sentences telling a
   prospective consumer that the project may deliberately ship nothing, and
   linking ADR 003. A consumer choosing whether to wait for this crate needs
   that information.
7. `docs/developers-guide.md`: add a short subsection recording the internal
   convention that the exit register is machine-checked, that
   `tests/v0_1_exit_register_contract.rs` is the guard, and that a `design.md`
   revision touching §§11.1, 11.2, 13.6, or 13.7 will break `make test` by
   design.
8. `docs/repository-layout.md`: note the new test file alongside the existing
   `tests/` entries.
9. `README.md`: the documentation index already describes the roadmap as
   carrying "exit criteria". Add ADR 003 to that list only if the README lists
   individual ADRs; if it does not, leave it alone rather than starting a new
   convention.

## Validation and acceptance

The change is accepted when all of the following hold.

- Running `make test` from the repository root passes. The tests
  `every_verdict_combination_maps_to_one_exit`,
  `falsified_b1_always_selects_ship_nothing`, `every_design_citation_resolves`,
  `parsing_is_order_independent`, and `exit_register_snapshot` are reported by
  name as passing, together with the four negative-control cases.
- Each of those tests was observed **failing** at EP-M1, for the reason stated
  in the verification plan, and the transcript is recorded under `Artefacts and
  notes`.
- Opening `docs/adr-003-v0-1-exit-register.md` shows: `Status: Accepted` with a
  date; three exits named E1, E2, E3; a four-row decision table over the bet
  verdicts; a gate binding to roadmap tasks 2.2.3 and 3.1.3 for each exit; and
  citations to `design.md` §§11.1, 11.2, 13.6, and 13.7.
- Deleting any one row from that decision table and re-running `make test`
  produces a failure naming the removed combination. Restore the row
  afterwards. This is the acceptance check a reviewer can run in thirty seconds
  to confirm the test is not theatre.
- `make check-fmt`, `make lint`, `make markdownlint`, and `make nixie` all pass.
- `docs/roadmap.md` task 1.1.2 reads `- [x]` and links the ADR.
- Every document named in the sync map contains exactly one reference to the
  new record, and every such link resolves.

Quality criteria:

- Tests: `make test` green; every obligation in the verification plan
  discharged with its negative control observed rejecting.
- Verification: `INV-TOTAL`, `INV-DOMINANCE`, and `INV-ANCHORS` discharged by
  exhaustive parameterized test, behavioural scenario, and cross-document
  contract test respectively.
- Lint: `make lint` green, including the Whitaker Dylint suite.
- Performance: not applicable; no runtime code.
- Security: not applicable; no runtime code, no new runtime dependency.

Quality method: run the five gate commands in Step 7, capturing each to a log
under `/tmp`, and read the logs.

## Idempotence and recovery

Every step is safely repeatable. The documentation edits are grep-guarded
insertions, so re-running Step 6 makes no second change. `cargo insta review`
is interactive and idempotent once accepted. Nothing is deleted and nothing is
destructive; `git checkout -- .` returns the tree to the last commit at any
point.

Two failure modes have specific recoveries. If `rstest-bdd` will not integrate,
follow the fallback in Step 3 rather than fighting it. If `make lint` reports
Whitaker findings, load the `addressing-whitaker-findings` skill; the likely
offenders are `no_std_fs_operations` (use `include_str!`, never `std::fs`),
`module_must_have_inner_docs` (add `//!` docs to the test file),
`module_max_lines`, and `bumpy_road_function` (split the parser into small
helpers, respecting the cognitive-complexity threshold of 9).

## Artefacts and notes

Record here, as work proceeds: the EP-M1 red transcript; the EP-M2 green
transcript; the generated `insta` snapshot; and the output of the
row-deletion acceptance check from `Validation and acceptance`.

## Interfaces and dependencies

### New dev-dependencies and why each is needed

- `googletest = "0.14.3"` — matcher-based assertions with structured failure
  output. It is documented as working with `rstest`. Used so that a failing
  totality check reports *which* combination is missing rather than
  `assertion failed: false`.
- `pretty_assertions = "1.4.1"` — readable diffs when comparing parsed
  register structures.
- `insta = "1.48.0"` — the normalized-register snapshot.
- `proptest = "1"` — the single ordering property.
- `rstest-bdd = "0.5.0"` — the behavioural scenarios that encode the exit
  selection procedure. `rstest = "0.26.1"` and `camino = "1.2.5"` are already
  present.

`[dependencies]` remains empty. No dependency reaches a consumer.

### Test module shape

In `tests/v0_1_exit_register_contract.rs`, define the following. Keep every
function under the complexity ceilings in Constraint 9.

```rust
//! Contract tests for the v0.1 exit register recorded in
//! `docs/adr-003-v0-1-exit-register.md`.
//!
//! These tests parse the decision record itself and check that its three
//! exits cover every combination of verdicts on bets B1 and B2, that a
//! falsified B1 always selects the off-ramp, and that every `design.md`
//! anchor the record cites still exists. Both documents are embedded with
//! `include_str!`, so the tests need no filesystem access and rebuild
//! whenever either document changes.

/// A verdict on one of the design's named bets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Verdict {
    Falsified,
    Held,
}

/// One of the three release scopes Statelet may deliver as v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Exit {
    ShipNothing,
    ShipConventions,
    ShipMacro,
}

/// Why a candidate exit register was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
enum RegisterError {
    Incomplete { missing: Vec<(Verdict, Verdict)> },
    Ambiguous { combination: (Verdict, Verdict) },
    UnknownExit { found: String },
    DominanceViolated { combination: (Verdict, Verdict), found: Exit },
    DanglingCitation { anchor: String },
}

/// The parsed decision table.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ExitRegister {
    /* total map from verdict pair to exit, plus the citation set */
}

/// Parses the decision table out of an ADR body.
///
/// Returns `Err` when the table does not define exactly one exit for each of
/// the four verdict combinations, names an unknown exit, or violates the
/// dominance rule that a falsified B1 forces `Exit::ShipNothing`.
fn parse_register(adr_body: &str) -> Result<ExitRegister, RegisterError> {
    todo!("EP-M1")
}

/// Checks every design citation in the register against the design text.
fn check_citations(register: &ExitRegister, design: &str) -> Result<(), RegisterError> {
    todo!("EP-M1")
}
```

`ExitRegister::exit_for(b1, b2) -> Exit` is total by construction: the type can
only be built through `parse_register`, which rejects incomplete tables. That
is why the totality obligation is checkable at all.

### Behavioural specification

Create `tests/features/v0_1_exit_selection.feature`:

```gherkin
Feature: Selecting the v0.1 exit from bet verdicts

  The exit register in ADR 003 must let a maintainer standing at a roadmap
  gate determine, without judgement, which of the three v0.1 exits applies.

  Scenario: The conventions baseline is too weak
    Given the exit register from ADR 003
    When bet B1 is falsified
    And bet B2 is falsified
    Then the selected exit is "ship nothing"

  Scenario: B1 falsified despite a promising macro
    Given the exit register from ADR 003
    When bet B1 is falsified
    And bet B2 is held
    Then the selected exit is "ship nothing"

  Scenario: The conventions wedge exists but the macro adds nothing
    Given the exit register from ADR 003
    When bet B1 is held
    And bet B2 is falsified
    Then the selected exit is "ship conventions only"

  Scenario: Both bets survive validation
    Given the exit register from ADR 003
    When bet B1 is held
    And bet B2 is held
    Then the selected exit is "ship macro"
```

Keep the feature file and the exit register synchronized; if the register
gains a row, the feature gains a scenario.

### The exit register itself

The content EP-M2 must produce. Reproduced here so the implementer transcribes
rather than improvises. Exit names and identifiers are normative.

- **E1 — Ship nothing (the off-ramp).** Statelet is not published. The naming
  and tracing-field conventions stay project-local in `mdtablefix`. Trigger:
  bet B1 falsified, that is, `StateName`, documented transition fields, and
  local helpers add little over plain
  `#[tracing::instrument(fields(state = %self.mode))]`. Evidence:
  `design.md` §11.1 row B1 and §13.7. Gate: provisionally at roadmap task
  2.2.3 if the `mdtablefix` baseline alone is weak — B1 requires *both*
  examples to improve, so one clear failure already falsifies it — and
  finally at roadmap task 3.1.3.
- **E2 — Ship conventions only.** The runtime crate publishes `StateName` and
  the documented `transition.*` tracing field contract. `statelet-macros` is
  deferred, not cancelled. Trigger: B1 held and B2 falsified, that is, no
  paragraph can state what `#[transition]` adds over the §11.2 baseline.
  Evidence: `design.md` §§11.2 and 13.6. Gate: roadmap task 3.1.3.
- **E3 — Ship the macro.** Everything in E2, plus `statelet-macros`. Trigger:
  B1 held and B2 held, that is, the head-to-head comparison states the concrete
  added value in one paragraph. Evidence: `design.md` §11.2. Gate: roadmap task
  3.1.3 unlocking phase 4.

The dominance rule, which no existing document states: **a falsified B1 selects
E1 regardless of B2.** The reasoning belongs in the ADR, not only in the table.
B1's required evidence is that both validation examples improve *without*
framework adoption. If they do not, there is no conventions baseline for a
macro to beat, so B2's verdict is not outweighed — it is undefined, because
§11.2 defines the macro gate relative to that baseline. Recording this closes
the one combination a careless reader would resolve the wrong way.

Note the nesting, per the last entry in `Risks`: E3 ships a strict superset of
E2. The three are disjoint as *release scopes* and the table is a partition of
*verdict combinations*, not of artefacts. The ADR must say this plainly rather
than implying three unrelated products.

## Revision note

- 2026-08-22: initial draft. Establishes the ADR-versus-note interpretation
  (Decision D1) as the plan's principal open question for the approval gate,
  fixes the artefact as `docs/adr-003-v0-1-exit-register.md`, and specifies a
  red-green cycle in which the red state is the honest transcription of what
  `docs/design.md` says today and the green state adds the exhaustiveness
  claim, the dominance rule, and the gate bindings.
