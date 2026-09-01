# Record the three possible v0.1 exits (roadmap 1.1.2)

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`, `Decision log`,
`Outcomes & retrospective`, `Conformance basis`, and `Verification plan` must
be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Statelet is a pre-1.0 Rust crate that does not yet know whether it should
exist. Its technical design records two low-confidence bets: that a real
segment of Rust developers wants a shared convention for handwritten state
machines (bet B1), and that a `#[transition]` attribute macro beats
`#[tracing::instrument]` plus a couple of helper functions (bet B2). The design
already describes, in scattered prose, what happens when each bet fails. It
does not anywhere state the resulting outcomes as a single, total, mutually
exclusive set, and it does not say which roadmap gate decides each one.

After this change, a reader who opens one document can answer three questions
without reconstructing them from four separate design sections:

1. What are the only three things Statelet can ship as v0.1?
2. What observation, at which named roadmap gate, selects each one?
3. Which of the named bets does that observation falsify or confirm?

Success is observable in two ways. First, a new decision record,
`docs/adr-003-v0-1-exit-register.md`, states the three exits and binds each to
concrete evidence in `docs/design.md` and to a gate in `docs/roadmap.md`.
Second, that binding is machine-checked: `make test` runs a contract test that
reads the decision record and fails if the exits stop covering every
combination of bet verdicts, if a design passage the record quotes stops
existing, or if a roadmap gate the record names is renumbered away. Renames and
deletions in the upstream documents are caught in the same commit that causes
them.

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
  ISC, with an **empty** `[dependencies]` table and only `camino = "1.2.5"` and
  `rstest = "0.26.1"` under `[dev-dependencies]`.
- `tests/stub.rs` is a disposable placeholder.
- `tests/dev_fast_contract.rs` is the one real test in the repository. It is a
  *contract test*: it asserts a property of the repository's own configuration
  rather than of runtime behaviour. Read it before starting; this plan follows
  its style closely.

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
  1.1.2 (implemented by this plan). Downstream, tasks 2.2.3, 3.1.3, and 4.3.1
  are the gates at which an exit is actually chosen.
- `docs/documentation-style-guide.md` — the authoritative ADR template and the
  prose rules.
- `docs/contents.md` — the master documentation index.
- `docs/whitaker-users-guide.md` — the Dylint lint suite that `make lint` runs.

### The verbatim evidence this plan is built on

These passages are the raw material. Each is reproduced here **exactly as it
appears in `docs/design.md`**, save that long lines are shown as they are
stored (the design wraps prose at 80 columns). The contract test asserts the
*emphasized clause* of each, whitespace-normalized, still appears in
`docs/design.md`; those clauses are the load-bearing part and are listed in
`Verification plan` under `INV-ANCHORS`.

`docs/design.md` §11.1, the two rows that matter, reproduced as a table because
they are stored as single long table rows in the design:

| Bet | Claim                                                                         | Confidence | Required evidence                                                                            |
| --- | ----------------------------------------------------------------------------- | ---------- | -------------------------------------------------------------------------------------------- |
| B1  | A real segment prefers handwritten state machines and wants shared convention | Low-medium | `mdtablefix` plus one second non-toy example both improve without framework adoption         |
| B2  | `#[transition]` beats `#[tracing::instrument]` plus helper functions          | Low        | A head-to-head `mdtablefix` baseline comparison states the concrete value added by the macro |

*Table 1: Bets B1 and B2 as recorded in `docs/design.md` §11.1.*

`docs/design.md` §11.2, closing sentences:

```plaintext
The macro gate passes only if the design note can state, in one paragraph, what
`#[transition]` adds over that baseline. Acceptable added value includes
removing repeated field capture, preventing field-name drift, preserving
consistent entry/exit logging, or reducing boilerplate enough that the
annotated code is easier to review. If the baseline is the more honest result,
the macro crate does not ship in v0.1.
```

`docs/design.md` §13.6, in full:

```plaintext
If the macro cannot beat the baseline in §11.2, the project ships the
conventions/runtime crate and defers `statelet-macros`. This is a successful
validation outcome, not a failed implementation.
```

`docs/design.md` §13.7, in full:

```plaintext
If `StateName`, documented transition fields, and local helpers add little over
plain `#[tracing::instrument(fields(state = %self.mode))]` in both validation
examples, the project should ship nothing and keep the pattern local. That is
the B1 failure case and must be treated as a valid discovery outcome.
```

`docs/terms-of-reference.md` §7.1, final bullet:

```plaintext
If both validation examples show that `StateName`, documented field names,
and helper functions add little over local `#[tracing::instrument]`
annotations, the correct outcome is to ship nothing and keep the convention
project-local.
```

### The gap this plan closes

The design states two of the three exits explicitly (§13.6 gives "ship
conventions only"; §13.7 gives "ship nothing"). The third, "ship the macro", is
only implied — it is the negation of §13.6, defined positively by the §11.2
gate. Nowhere does any document state:

1. that the three outcomes are exhaustive over the bet verdicts;
2. what happens when B1 is falsified but B2 looks promising;
3. what happens when **exactly one** of the two validation examples improves;
   and
4. at which roadmap gate each verdict is taken.

Items 1, 2, and 4 are the intellectual content of task 1.1.2. Item 3 is a gap
in the upstream documents that this plan discovered and cannot close by itself
— see `Open question for the approval gate` below. Everything else is
transcription.

### Open question for the approval gate

Roadmap 1.1.2 asks for exits keyed on B1 and B2. Working the mapping out
exposes a combination that neither upstream document maps.

`docs/design.md` §11.1 says B1 holds only when "`mdtablefix` plus one second
non-toy example **both improve**". So B1 is falsified when *either* example
fails. But §13.7 fires only when conventions add little "**in both** validation
examples", and `docs/terms-of-reference.md` §7.1 likewise says "**If both**
validation examples show … ship nothing". The split case — `mdtablefix`
improves, `wireframe` does not, or vice versa — therefore falsifies B1 without
triggering either document's ship-nothing rule. It is unmapped.

Note also that this case is really about **B6** ("`mdtablefix` is
representative enough to generalize the wedge", confidence low-medium), which
roadmap 1.1.2 does not mention.

There are two honest resolutions, and the choice is the user's:

- **R1 (recommended).** Treat the split case as falsifying B1, selecting E1
  ("ship nothing"). A convention that helps one codebase and not another is a
  project-local pattern, which is exactly what §13.7 concludes for the
  both-fail case; B6 is the bet that fails. This requires a one-sentence
  amendment to `docs/design.md` §13.7 broadening "in both validation examples"
  to "in either validation example", and the matching change to
  `docs/terms-of-reference.md` §7.1.
- **R2.** Treat the split case as a fourth outcome — ship conventions scoped to
  the domain that improved — and revise roadmap 1.1.2, 2.2.3, and 3.1.3, which
  all say "three exits".

This plan is written for R1. If R2 is preferred, say so at the approval gate;
`Tolerances` treats a fourth exit as an escalation trigger precisely because it
changes three roadmap tasks. Under either resolution the amendment to
`docs/design.md` §13.7 exceeds a cross-reference, so it is called out here
rather than buried in the sync map.

### Files this plan reads or writes

Reads only: `docs/design.md`, `docs/terms-of-reference.md`,
`docs/documentation-style-guide.md`, `docs/whitaker-users-guide.md`,
`tests/dev_fast_contract.rs`, `AGENTS.md`, `Makefile`, `clippy.toml`,
`.markdownlint-cli2.jsonc`.

Creates: `docs/adr-003-v0-1-exit-register.md`;
`tests/v0_1_exit_register_contract.rs`.

Edits: `Cargo.toml` (dev-dependencies only), `docs/context.md`,
`docs/contents.md`, `docs/design.md`, `docs/terms-of-reference.md`,
`docs/roadmap.md`, `docs/users-guide.md`, `docs/developers-guide.md`,
`docs/repository-layout.md`.

### Documentation to read before starting

In-repository guidance that bears directly on this work:

- `docs/documentation-style-guide.md` — the ADR template and prose rules. This
  is the authority for the deliverable's shape.
- `docs/rust-testing-with-rstest-fixtures.md` — fixture design and
  parameterized cases; the contract test uses `#[rstest]` with `#[case]` tables
  throughout.
- `docs/reliable-testing-in-rust-via-dependency-injection.md` — why the parser
  is a pure function over `&str` rather than a filesystem reader. That choice
  is what makes every negative control a string literal.
- `docs/complexity-antipatterns-and-refactoring-strategies.md` — load-bearing
  here, because `clippy.toml` caps cognitive complexity at 9 and functions at
  70 lines, and a table parser is exactly the code that breaches both.
- `docs/rust-doctest-dry-guide.md` — for any doc examples added.
- `docs/rstest-bdd-users-guide.md` — read for context on the project's
  behavioural-testing direction, even though Decision D7 defers `rstest-bdd` to
  the first slice with runtime behaviour.
- `docs/whitaker-users-guide.md` — `no_std_fs_operations`,
  `module_must_have_inner_docs`, `module_max_lines`, `no_unwrap_or_else_panic`,
  and `bumpy_road_function` all bear on the test file.
- `docs/scripting-standards.md` — if any shell helper is added.
- `docs/developers-guide.md` and `docs/repository-layout.md` — both are edited
  by this plan.

### Skills to load before starting

- `execplans` — this document's format and the living-section discipline.
- `leta` — semantic code navigation. Run `leta workspace add .` once.
- `rust-router`, and from it `rust-unit-testing` — fixtures, table tests, and
  the `googletest` and `pretty_assertions` assertion guidance this plan uses.
- `en-gb-oxendict` — the repository enforces British English with Oxford
  spelling through `typos`; `make markdownlint` fails otherwise.
- `addressing-whitaker-findings` — if `make lint` reports Dylint findings.

Do **not** load `arch-decision-records` for the ADR's shape. That skill
prescribes a `docs/adr/` directory and a bare six-clause Y-Statement. This
repository uses its own template from `docs/documentation-style-guide.md`. The
repository's guide wins.

Do **not** load `kani` or `verus`. The artefact under test is a Markdown
document; neither tool can reach it. See `Verification plan`.

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

Traced items:

```plaintext
TOR-7.1-ship-nothing -> TDD-11.1-B1 -> EP-M1 -> ADR-003 exit E1 -> tests::totality_holds
TDD-13.6             -> TDD-11.1-B2 -> EP-M1 -> ADR-003 exit E2 -> tests::dominance_holds
TDD-11.2-macro-gate  -> TDD-11.1-B2 -> EP-M1 -> ADR-003 exit E3 -> tests::gate_bindings_resolve
TDD-11.1-B6          -> split case  -> EP-M1 -> ADR-003 §Exit register -> Open question R1
ROADMAP-1.1.2        -> EP-M1..M3   -> ADR-003 + roadmap checkbox ticked
ROADMAP-2.2.3/3.1.3/4.3.1 -> ADR-003 gates G1, G2, G3 -> tests::gate_bindings_resolve
```

ADR 002 explicitly declines to decide macro-versus-conventions; its non-goals
say those questions "are tracked separately". This plan therefore does not
deviate from ADR 002, it discharges a sibling obligation.

No upstream artefact defines a document type called an "exit note". The
roadmap's phrase "the exit note" is interpreted as the decision record produced
here; see `Decision log` D1.

## Constraints

Hard invariants. If satisfying the objective requires violating one, stop and
escalate rather than working around it.

1. **No runtime code.** Nothing is added to `src/`. Roadmap phase 1 forbids
   speculative public API before the exit criteria are settled; a
   `select_exit()` function in the library would be exactly the mistake the
   phase exists to prevent. All new code lives under `tests/`.
2. **No public API surface.** `Cargo.toml` `[dependencies]` stays empty. Only
   `[dev-dependencies]` may gain entries, and only with caret requirements
   naming a full version, per `AGENTS.md`.
3. **The ADR does not choose an exit.** It records the three exits and their
   trigger conditions. Choosing among them is roadmap tasks 2.2.3, 3.1.3, and
   4.3.1, which do not yet have evidence. Any draft that picks a winner is
   wrong.
4. **No commit may fail a gate.** `AGENTS.md` forbids committing changes that
   fail any quality gate. Red-Green-Refactor is therefore observed *within* a
   milestone and its transcript recorded; a red state is never committed. There
   is no `todo!()` in committed code — `panic_in_result_fn` is denied and
   `RUSTFLAGS="-D warnings"` turns an unused error variant into a build error.
5. **Repository ADR template only.** File named `docs/adr-003-<slug>.md`;
   sections in the order the style guide mandates; every table captioned.
6. **en-GB Oxford spelling**, sentence-case headings, prose wrapped at 80
   columns, code at 120, fenced blocks given a language identifier.
7. **`std::fs` is forbidden in test code.** Whitaker's `no_std_fs_operations`
   denies it. This plan uses `include_str!`, which reads at compile time and
   needs no filesystem crate.
8. **`unwrap()` is denied even in tests.** `clippy.toml` sets
   `allow-expect-in-tests = true` but not `allow-unwrap-in-tests`. Use
   `.expect("...")` or `let ... else { panic!(...) }`, as
   `tests/dev_fast_contract.rs` does.
9. **Complexity ceilings apply to test code.**
   `cognitive-complexity-threshold = 9`, `too-many-arguments-threshold = 4`,
   `too-many-lines-threshold = 70`, `excessive-nesting-threshold = 4`.
   `indexing_slicing`, `string_slice`, and `unreachable` are denied, so the
   parser must use `split`, `trim`, iterator adaptors, and `get()`, never
   `&line[a..b]` or `parts[2]`.
10. **The parser must survive `make fmt`.** That target runs `mdformat-all`,
    which invokes `mdtablefix --wrap --renumber --breaks --ellipsis --fences
    --in-place` over every Markdown file, then `markdownlint-cli2 --fix`. Table
    cell padding is re-derived and prose is reflowed. Parsing is therefore
    whitespace-tolerant, and the register's cells contain no `...` (which
    `--ellipsis` rewrites to `…`).

## Tolerances (exception triggers)

- **Scope**: more than 12 files touched, or more than roughly 600 net added
  lines excluding the ADR body itself.
- **Dependencies**: more than the two dev-dependencies named in
  `Interfaces and dependencies`. If a third appears necessary — including a
  transitively required companion crate — stop and escalate before adding it.
- **Runtime code**: any change to `src/`, or any addition to `[dependencies]`.
  Stop immediately.
- **Design amendment beyond the agreed one**: this plan authorizes exactly one
  substantive edit to `docs/design.md` §13.7 and its mirror in
  `docs/terms-of-reference.md` §7.1, under resolution R1. Any further
  substantive design change is an escalation.
- **A fourth exit**: if evidence supports an outcome that cannot be folded into
  three, stop and present it. Changing the number changes roadmap tasks 1.1.2,
  2.2.3, and 3.1.3.
- **Gate cost**: if adding the test increases a clean `make test` by more than
  roughly 30 seconds, or if CodeScene reports a code-health regression on the
  new file, stop and report. The parser must not become the repository's
  dominant code artefact.
- **`make audit`**: if a new advisory appears, stop. Do not add a
  `CARGO_AUDIT_IGNORES` entry without approval.
- **Iterations**: if `make test` still fails after three focused attempts on
  the same assertion, stop and report the transcript.
- **Ambiguity**: if the ADR's status should plausibly be `Proposed` rather than
  `Accepted`, present both readings rather than choosing. See D8, which makes
  the call this plan implements.

## Risks

- Risk: the ADR becomes a paraphrase of `design.md` and adds no information.
  Severity: high. Likelihood: medium. Mitigation: the four items under "The gap
  this plan closes" are the deliverable. `INV-TOTAL`, `INV-DOMINANCE`, and
  `INV-GATES` each assert one of them.

- Risk: the contract test is circular — it parses the document and then asserts
  a rule the parser already enforced. Severity: high. Likelihood: high if not
  designed against; this was the principal finding of the design review.
  Mitigation: parsing is strictly syntactic. `parse_register` extracts rows and
  knows nothing about totality, dominance, or gates. Every policy check is a
  separate pure predicate over the parsed rows, so each obligation has its own
  attributable failure. See `Verification plan`.

- Risk: the test becomes a tax a future contributor deletes rather than fixes.
  Severity: high. Likelihood: medium. Mitigation: every failure message names
  the two files involved and the exact one-line repair. The test module carries
  a `//!` header explaining what it guards and how to fix a break. This is a
  stated acceptance criterion, not a nicety.

- Risk: a false negative — `design.md` is rewritten to mean the opposite while
  the headings survive, and the suite stays green. Severity: medium.
  Likelihood: medium. Mitigation: `INV-ANCHORS` asserts the load-bearing
  *clause* of each quoted passage, whitespace-normalized, not merely that a
  heading exists. A semantic rewrite of §13.6 or §13.7 changes those clauses.

- Risk: `mdtablefix` reformats the register table and breaks parsing for a
  non-semantic reason. Severity: medium. Likelihood: high. Mitigation:
  Constraint 10; the register is delimited by HTML comments, which the
  formatter leaves alone; a negative control is the formatter-normalized
  rendering of the table; and `make fmt` runs at every go/no-go.

- Risk: five to six new dev-dependencies on a crate with one real test is
  disproportionate. Severity: medium. Likelihood: was high in the first draft.
  Mitigation: reduced to two. See D7.

## Progress

- [x] (2026-08-22 00:29Z) EP-M0: plan approved, including the R1-versus-R2
      choice in `Open question for the approval gate`; orientation and
      conformance-basis checks passed.
- [x] (2026-08-22 00:29Z) EP-M1: ADR 003 written; contract test complete and
      green; the red transcript observed and recorded in `Artefacts and notes`.
- [x] (2026-08-22 00:29Z) EP-M2: companion-document sync complete; roadmap
      1.1.2 ticked.
- [x] (2026-08-22 00:29Z) EP-M3: full gates green; branch pushed; pull
      request #50 remains open as a draft.
- [x] (2026-08-23 23:40Z) Maintenance: split gate-task and parsed-row checks
      into private helpers while preserving the wrapper signature, validation
      order, and existing error text; added focused negative controls.
- [x] (2026-08-26 21:53Z) EP-M4: restored the missing reachability,
      ADR-citation,
      and handwritten-case negative controls identified during review; focused
      red/green evidence, the full delivery suite, and CodeRabbit are green.
- [x] (2026-09-01 21:53Z) EP-M5: enforced each register row's exit-to-gate
      binding and replaced the remaining broad negative-control assertions with
      exact error contracts; focused red/green evidence, full delivery gates,
      and CodeRabbit are green.

Add a UTC timestamp as each completes: `- [x] (2026-08-22 14:05Z) EP-M1: ...`.

## Surprises & discoveries

Findings from the planning pass, carried forward:

- Observation: `clippy.toml` sets `allow-expect-in-tests = true` but not
  `allow-unwrap-in-tests`. Evidence: `clippy.toml` line 7; `Cargo.toml` denies
  both `unwrap_used` and `expect_used`. Impact: test code uses `.expect()` or
  `panic!`, never `.unwrap()`.

- Observation: `make fmt` rewrites Markdown aggressively.
  Evidence: `mdformat-all` runs
  `mdtablefix --wrap --renumber --breaks --ellipsis --fences --in-place` across
  every `.md` file. Impact: Constraint 10. Note also that `make check-fmt` is
  `cargo fmt --check` only, so Markdown formatting is *not* gated — a drifted
  document passes CI until someone runs `make fmt`.

- Observation: a backslash-escaped backtick inside inline code defeats
  `mdtablefix`, which then reflows the whole paragraph onto one 400-character
  line and fails `make markdownlint` with MD013. Evidence: hit while gating
  this plan; writing ``` `- \`path\`` ``` in a list item produced a
  403-character line on the next `make fmt`. Impact: never escape a backtick
  inside inline code in this repository. Describe the literal instead, as
  sync-map item 2 now does.

- Observation: `rstest-bdd` 0.5.0 does not ship its macros; `#[scenario]`,
  `#[given]`, `#[when]`, and `#[then]` live in the separate crate
  `rstest-bdd-macros` 0.5.0, which depends on `proc-macro-error ^1`. Evidence:
  every example in `docs/rstest-bdd-users-guide.md` imports from
  `rstest_bdd_macros`; the crates.io dependency list for `rstest-bdd-macros`
  0.5.0 includes `proc-macro-error ^1`, `gherkin ^0.14`, and `cap-std ^3.4.4`.
  Impact: adopting it means two dependencies, roughly forty transitive crates, a
  `CARGO_AUDIT_IGNORES` entry for the unmaintained `proc-macro-error`, and a
  `dylint.toml` teaching Whitaker that `#[given]`/`#[when]`/`#[then]` are test
  attributes. See D7.

- Observation: no crate in the Rust ecosystem was found that publicly
  pre-commits to kill criteria or possible abandonment. Evidence: a web survey
  of the `std` feature lifecycle, the Rust RFC template, and `tokio`'s
  `--cfg tokio_unstable` idiom found staged *gating* but no published *quit*
  conditions. Impact: the ADR must not claim ecosystem precedent. It may cite
  the Stage-Gate Go/Kill/Hold vocabulary and Duke's kill-criteria "state and
  date" framing as the actual antecedents, which are managerial, not
  Rust-specific.

- Observation: Leta could register the workspace but `rust-analyzer` could not
  start because its language-server component is unavailable. Evidence: the
  Stage A `leta` query returned `LSP error -1: Connection closed`. Impact:
  direct, read-only inspection is the navigation fallback for this change; no
  toolchain component is installed because this documentation-and-test change
  does not require altering the developer environment.

- Observation: Whitaker rejects a three-term syntactic filter even in a small
  parser. Evidence: the first EP-M1 `make lint` run reported
  `conditional-max-n-branches` for the row filter. Impact: the parser now uses
  the named `is_register_header_or_divider` predicate, preserving the syntax
  boundary while meeting the two-branch house rule.

- Observation: a quote checker can appear to validate ADR citations while only
  checking a parallel hard-coded list. Evidence: an ADR-only fabricated clause
  initially failed because the original citation was absent, not because the
  fabricated clause could not resolve. Impact: `INV-ANCHORS` now extracts
  quotes from the ADR evidence section before checking `docs/design.md`.

- Observation: validating a gate-table identifier and validating a register
  row's selected decision gate are distinct obligations. Evidence: changing an
  E3 row from G3 to known G1 initially passed every gate check. Impact:
  `check_row_gates` now rejects a known-but-wrong selected gate with a targeted
  repair.

## Decision log

- Decision D1: the deliverable is an ADR, `docs/adr-003-v0-1-exit-register.md`.
  Rationale: roadmap step 1.1 is titled "Ratify the v0.1 product exits", and
  ratification is a decision act. Task 1.1.1 set the ADR precedent for this
  step. The style guide supplies an ADR template and no "note" template.
  Roadmap 2.2.3 says a later note "chooses one of the three exits from 1.1.2",
  and a chooser needs a stable, dated, citable target rather than a living
  design section. Date/Author: 2026-08-22, planning agent.

- Decision D2: the ADR records a single decision, not three.
  Rationale: no established decision-record format — MADR 4.0, Nygard, or the
  Y-Statement — supports recording several alternative *future* outcomes; all
  assume one chosen option. The honest single decision is: *defer the
  ship/no-ship choice and bind it to named evidence at named gates.* The three
  exits then sit under "Options considered". Date/Author: 2026-08-22, planning
  agent.

- Decision D3: keep the word "exit", and define it in the glossary.
  Rationale: "exit criteria" already means "definition of done for testing" in
  test-management usage, so a third sense invites misreading. But the term is
  load-bearing across this repository — roadmap step 1.1's title, tasks 2.2.3
  and 3.1.3, and `README.md`'s documentation index all use it. Renaming forks
  the vocabulary. The fix is a precise definition in the normative glossary
  `docs/context.md`, which the contract test then treats as a citation target
  so it cannot be silently deleted. Date/Author: 2026-08-22, planning agent.

- Decision D4: bind exits to three roadmap gates, not two, and not to dates.
  Rationale: kill criteria bind only when they state both a *state* and a
  *when*. Calendar dates are meaningless for a hobby-cadence project, but the
  roadmap supplies natural decision points. The first draft of this plan bound
  E2 and E3 both to task 3.1.3; design review showed that is wrong. B2's
  required evidence is "a head-to-head `mdtablefix` baseline comparison", which
  roadmap task **4.2.1** produces and task **4.3.1** records. Task 3.1.3 only
  "unlocks Phase 4", which is not the same as shipping the macro. The gates are
  therefore G1 = task 2.2.3, G2 = task 3.1.3, G3 = task 4.3.1. Date/Author:
  2026-08-22, planning agent, after design review.

- Decision D5: reject a fourth exit shipping the macro behind an unstable
  `cfg`. Rationale: `tokio` gates unstable APIs behind a bare
  `--cfg tokio_unstable` RUSTFLAG, excluding them from semantic-versioning
  guarantees. It is a real option, rejected because it would let the project
  ship the macro *without* passing the §11.2 gate — precisely the discipline
  phase 1 exists to impose — and because the roadmap fixes the count at three.
  Recorded in the ADR's "Options considered" so the reasoning is not lost.
  Date/Author: 2026-08-22, planning agent.

- Decision D6: verification parses the document; parsing is strictly syntactic.
  Rationale: a runtime `select_exit()` would make the invariants trivially
  checkable and would be the speculative API Constraint 1 forbids. Parsing the
  ADR keeps the document as the single source of truth. Critically, the first
  draft had `parse_register` reject dominance violations, which made the
  dominance test tautological — any register that parsed satisfied it by
  construction. Parsing now extracts rows and nothing else; totality,
  dominance, citation resolution, and gate resolution are separate predicates
  over those rows, each independently falsifiable. Date/Author: 2026-08-22,
  planning agent, after design review.

- Decision D7: use `rstest` with `googletest` and `pretty_assertions`; defer
  `rstest-bdd`, `insta`, and `proptest`. Rationale: the standing instruction is
  to use behavioural, snapshot, and property tests *where applicable*,
  exercising judgement on rigour. Applying that judgement here: `rstest-bdd`
  would add two crates, roughly forty transitive dependencies, an audit-ignore
  entry for the unmaintained `proc-macro-error`, and a new `dylint.toml`, to a
  repository with two dev-dependencies and no runtime behaviour — and the same
  independent cross-check is obtained by hand-writing the verdict triples as
  `#[case]` rows that are authored separately from the parser (`INV-CASES`). It
  is the right tool for the first slice with real runtime behaviour, phase 2,
  not for a document. `proptest` was proposed for permutation-invariance of the
  parser, which is a property of this plan's own test code rather than of the
  artefact, over a 24-element domain that `#[case]` enumerates exactly; it is
  ceremony. `insta` would snapshot a four-row table every other obligation
  already pins. Its one unique contribution — pinning the evidence and gate
  columns — is covered directly by `INV-ANCHORS` and `INV-GATES`. **This trims
  the user's stated default test stack, so it is flagged for the approval
  gate.** Date/Author: 2026-08-22, planning agent, after design review.

- Decision D8: the ADR ships with status `Accepted`.
  Rationale: the decision being ratified is D2's — that these are the three
  exits, with these triggers and these gates. That decision is made now and is
  not contingent on future evidence. The *outcome* of the gates is separately
  tracked under the ADR's "Outstanding decisions" section. ADR 002 is
  `Accepted` on the same reading. The first draft left this ambiguous between
  its Stage C and its Tolerances; it is resolved here. Date/Author: 2026-08-22,
  planning agent, after design review.

- Decision D9: apply the glossary entry and R1 source amendments in EP-M1.
  Rationale: `INV-ANCHORS` is a complete, green contract at the end of EP-M1;
  it cannot check `docs/context.md` and the R1 rule while those documents wait
  for EP-M2. This moves three already-authorized documentation edits earlier,
  without changing their content, scope, architecture or acceptance evidence.
  EP-M2 retains every remaining companion reference and the roadmap checkbox.
  Date/Author: 2026-08-22, implementing agent.

- Decision D10: split the contract test into a parent and private child module.
  Rationale: the original test file reached 436 lines, exceeding the
  repository's 400-line code-file limit. The parent owns externally visible
  test scenarios; `tests/v0_1_exit_register_contract/support.rs` owns only pure
  parser and policy predicates. The repository has no existing equivalent
  helper, as confirmed by the Stage A symbol search and a fallback text search
  after Leta's language server failed. The developer's guide records that this
  test-only boundary is not reusable from runtime code or other contracts.
  Date/Author: 2026-08-22, implementing agent.

- Decision D11: pause before EP-M2 because the scope tolerance is reached.
  Evidence: EP-M1 touches ten files; the mandatory remaining sync requires
  `docs/contents.md`, `docs/roadmap.md`, `docs/users-guide.md`, and
  `docs/repository-layout.md`, increasing the total to fourteen. The stated
  tolerance is more than twelve files touched. Options are to authorize this
  bounded fourteen-file completion, or to revise the implementation approach.
  The latter cannot preserve every required sync-map edit. Date/Author:
  2026-08-22, implementing agent.

  Outcome: the user authorized the bounded fourteen-file completion on
  2026-08-22. EP-M2 may proceed; its required sync map remains unchanged.

- Decision D12: retain `check_gate_bindings` as the ordered public contract and
  move its independent loops into private helpers. Rationale: the two checks
  have distinct responsibilities, but the task requires gate-to-task and live
  roadmap validation to fail before any parsed-row gate error. The helpers
  preserve that sequence and every existing error string, without changing the
  Markdown parser, register model, gate definitions, or contract scope.
  Date/Author: 2026-08-23, implementing agent.

- Decision D13: make `check_dominance` the policy owner for register
  reachability, derive quoted design clauses from ADR 003's evidence section,
  and reuse one handwritten-case predicate for live and invalid rows.
  Rationale: reachability is parsed syntax that needs an independent policy
  assertion; a hard-coded citation list cannot detect a fabricated ADR quote;
  and the handwritten predicate must reject the same dominance-invalid fixture
  as the dominance check. These changes preserve the parser's syntax-only
  boundary and add no runtime code or dependencies. Date/Author: 2026-08-26,
  implementing agent.

- Decision D14: make `check_row_gates` validate the decision gate selected by
  each parsed row as well as gate vocabulary. Rationale: a known gate can still
  select the wrong decision point; E1 rows must use G2 and E2/E3 rows must use
  G3. The policy stays separate from the syntax parser and runs only after a
  row's gate name is known. Date/Author: 2026-09-01, implementing agent.

## Outcomes & retrospective

EP-M1 delivered an accepted ADR with a syntactically parsed, independently
checked register. EP-M2 connected that record to every required companion
document and completed roadmap task 1.1.2. EP-M3's complete deterministic
delivery suite and final CodeRabbit review are green.

The implementation confirmed that a document contract needs syntax and policy
to stay separate: enforcing dominance in the parser would have made its test
vacuous. It also confirmed that strict module-size and branch-count limits
improve the test's boundary: the parent names scenarios, while its private child
owns pure parser and policy work.

EP-M4 closed three review-identified gaps without changing the document model:
reachability is now a dominance policy, citations come from ADR 003 itself, and
the handwritten case predicate tests both the real and dominance-invalid rows.

EP-M5 completed the row-level gate contract and converted the remaining broad
negative controls into exact error contracts, so a passing test now proves both
the rejection reason and the repair guidance.

## Verification plan

The change adds no runtime behaviour, so there is nothing to verify about
program execution. It introduces four non-trivial propositions about the
*documents*, and those are checkable.

**Design for falsifiability.** All checked documents are embedded at compile
time with `include_str!`. `parse_register` is a pure function from `&str` to
`Result<Vec<Row>, ParseError>` that performs *syntax only*: it locates the
register between two HTML-comment delimiters, splits rows on `|`, trims cells,
and maps cell text to `Verdict` and `Exit` tokens. It applies no policy. Each
obligation below is a separate pure predicate over the resulting rows, so a
failure is attributable to one obligation rather than swallowed by a parse
error. Purity means every negative control is a string literal.

`ParseError` distinguishes `MissingDelimiters`, `MalformedRow { line }`,
`UnknownVerdict { found }`, and `UnknownExit { found }`. Reporting a formatting
break as `MissingDelimiters` rather than as a content failure is deliberate:
mislabelling a formatting problem as missing content is the exact silent-rot
failure the suite exists to prevent.

### INV-TOTAL — the exit mapping is total and unambiguous

- **Obligation**: for every combination of final verdicts on B1 and B2 — each
  `Falsified` or `Held` — the register names exactly one exit. Four
  combinations, no gaps, no duplicates.
- **Method**: exhaustive parameterized test (`#[rstest]` with one `#[case]` per
  combination) over the parsed rows.
- **Rationale**: the domain has four elements; exhaustive enumeration is the
  proof. The obligation's real content is not "the table has four rows" but
  "the parser faithfully reflects the document", which is where the risk lies
  and which the controls below target.
- **Domain**: `{Falsified, Held} × {Falsified, Held}`.
- **Artefact**: `tests/v0_1_exit_register_contract.rs`, tests
  `totality_holds` and `parser_rejects_malformed_registers`.
- **Evidence**: `make test`.
- **Non-vacuity**: six controls, each asserting a *specific* error rather than
  `is_err()`. (a) The empty string must yield `MissingDelimiters`, not a
  vacuously complete empty map — this is the classic vacuity hole. (b) A
  document with delimiters but no rows must yield `MissingDelimiters`. (c) A
  register missing one row must fail totality naming that combination. (d) A
  register with a duplicated combination must fail as ambiguous. (e) A register
  with a spurious fifth combination must fail. (f) A register with an
  unrecognized verdict token must yield `UnknownVerdict`. A witness — the real
  ADR — must be accepted. If `parse_register` were replaced by a stub returning
  an empty `Ok`, controls (a) through (f) fail.

### INV-DOMINANCE — a falsified B1 forces the off-ramp

- **Obligation**: both rows where B1 is `Falsified` name exit E1 ("ship
  nothing"); `(Falsified, Held)` is marked unreachable; and the three other
  verdict combinations are marked reachable.
- **Method**: parameterized test over the parsed rows, plus `INV-CASES` below.
- **Rationale**: this is the one rule no existing document states, and the one
  a careless reader gets wrong — it is tempting to think a good macro could
  rescue a wedge nobody wants. The first draft justified it by claiming B2 is
  "undefined" when B1 fails; design review showed that reasoning is wrong,
  because §11.2's baseline is the `mdtablefix` baseline specifically and
  remains evaluable if B1 falls on `wireframe`. The correct justification is
  structural: gate G2 (roadmap 3.1.3) selects E1 when B1 is falsified, so gate
  G3 is never reached and B2 can never be recorded as `Held`. The
  `(Falsified, Held)` row is therefore unreachable by construction. It exists
  to make the mapping total and to forbid a later reader from reasoning their
  way to E3 from a dead wedge. The ADR must state this, and must mark the row
  unreachable.
- **Domain**: all four register rows, with the exit rule applying to the two
  rows where B1 is `Falsified`.
- **Artefact**: `tests/v0_1_exit_register_contract.rs`, test
  `dominance_holds`.
- **Evidence**: `make test`.
- **Non-vacuity**: a control fixture mapping `(Falsified, Held)` to E3 parses
  successfully — parsing applies no policy — and must then fail
  `dominance_holds` naming that row. A separate fixture changing that row's
  `Reachable` cell from `no` to `yes` must fail with the repair to mark it
  unreachable. Because the parser does not enforce either policy, these tests
  can fail on a real document, which is exactly what the first draft got wrong.
  A witness fixture with the correct mapping must pass.

### INV-ANCHORS — every quoted design passage still says what is quoted

- **Obligation**: for each passage the ADR quotes from `docs/design.md` and
  `docs/terms-of-reference.md`, the load-bearing clause still appears in the
  source, compared after whitespace normalization. The clauses are:
  - `both improve without framework adoption` (§11.1, row B1);
  - `states the concrete value added by the macro` (§11.1, row B2);
  - `the macro crate does not ship in v0.1` (§11.2);
  - `the project ships the conventions/runtime crate and defers` (§13.6);
  - `the project should ship nothing and keep the pattern local` (§13.7).

  Additionally the row identifiers `B1` and `B2` must appear as *table rows* of
  §11.1, and the glossary entry anchor for "v0.1 exit" must exist in
  `docs/context.md`.
- **Method**: extract quoted clauses from ADR 003's evidence section, then
  compare each with `include_str!`-embedded source text after whitespace is
  folded to single spaces.
- **Rationale**: this obligation earns the test its keep. Checking that a
  *heading* exists is a weak proxy: §13.7 could keep its title and reverse its
  meaning. Checking the load-bearing clause converts existence into agreement.
  Whitespace folding is required because `mdtablefix --wrap` reflows prose to
  80 columns, so a clause may be split across lines; the 1.1.1 ExecPlan hit
  exactly this and solved it the same way.
- **Domain**: the seven anchors listed above.
- **Artefact**: test `quoted_passages_still_resolve`.
- **Evidence**: `make test`.
- **Non-vacuity**: three controls. (a) A design fixture with the §13.7 sentence
  reworded must fail, naming the clause. (b) A design fixture with the **B1 row
  deleted** but the token `B1` still present in surrounding prose must fail —
  without this control a naive `contains("B1")` passes and the check is
  worthless. (c) An ADR fixture citing a clause that has never existed must
  fail, proving the check reads the ADR's citations rather than a hard-coded
  list. The real pair must be accepted.

### INV-GATES — every named gate resolves to a live roadmap task

- **Obligation**: each exit's gate identifier (G1 → task 2.2.3, G2 → task
  3.1.3, G3 → task 4.3.1) appears in `docs/roadmap.md` as a task, each such
  task is still unticked, and every register row binds E1 to G2 or E2/E3 to G3.
- **Method**: cross-document contract test over `include_str!`-embedded
  `docs/roadmap.md`.
- **Rationale**: D4 says kill criteria bind only when they state both a state
  and a *when*. The "when" is the gate, and in the first draft it was entirely
  unverified — and wrong. This repository ships `mapsplice`, a tool whose
  purpose is renumbering roadmap tasks, so gate drift is among the highest
  probability failures in the whole change and had zero guard. The unticked
  check matters because a gate that has already been passed cannot select an
  exit; if it has been ticked, the register needs revisiting.
- **Domain**: the three gate-table identifiers and all four parsed register
  rows.
- **Artefact**: test `gate_bindings_resolve`.
- **Evidence**: `make test`.
- **Non-vacuity**: three controls. (a) A roadmap fixture in which task 3.1.3
  has been renumbered to 3.1.4 must fail, naming the missing gate. (b) A
  roadmap fixture in which task 4.3.1 is ticked `- [x]` must fail with a
  distinct message. (c) A syntactically valid E3 row changed from its required
  G3 to known but incorrect G1 must fail with the repair to bind it to G3. A
  control citing a gate identifier that never existed must also fail, proving
  extraction is from the ADR.

### INV-CASES — handwritten expectations agree with the document

- **Obligation**: four `#[case]` triples of
  `(B1 verdict, B2 verdict, expected exit)`, written by hand from the design's
  prose and *not* derived from the register, agree with the parsed register.
- **Method**: parameterized test.
- **Rationale**: this is the least circular check available. Every other
  obligation reads the document and asserts a property of what it read; this
  one asserts that an independently authored expectation matches. It is a
  two-key change guard: to break the register silently, an editor must change
  both the ADR and the test's case table.
- **Artefact**: test `hand_written_cases_match_register`.
- **Non-vacuity**: mutating any single `#[case]` row must fail the test, and
  the negative-control fixture used for `INV-DOMINANCE` must also fail here.

### Failure-message requirement

Every assertion failure must name the two files involved and the one-line
repair, for example:

```plaintext
docs/adr-003-v0-1-exit-register.md cites design.md §13.7 clause
  "the project should ship nothing and keep the pattern local"
but no such text appears in docs/design.md (whitespace-normalized).
Repair: restore the clause in docs/design.md §13.7, or update the quotation in
docs/adr-003-v0-1-exit-register.md and re-check the exit it justifies.
```

This is an acceptance criterion, not a nicety. A guard whose failure a stranger
cannot repair gets deleted rather than fixed.

### Methods deliberately not used

- **`kani`** and **`verus`**: the artefact under test is a Markdown document,
  which neither tool can reach. Even setting that aside, the only lemma present
  — that a four-row mapping is a total function — is discharged completely by
  enumerating four cases, and a proof would restate the assumption in another
  syntax.
- **`proptest`**, **`insta`**, **`rstest-bdd`**: see Decision D7.
- **End-to-end tests**: there is no externally observable workflow, network
  boundary, persistence format, or command-line surface. `make test` is the
  observable workflow and is exercised directly.

## Plan of work

### Stage A — orient and confirm (no changes)

Read `docs/design.md` §§11.1, 11.2, 13.6, 13.7, and 14; `docs/roadmap.md` step
1.1 and tasks 2.2.3, 3.1.3, 4.2.1, and 4.3.1;
`docs/adr-002-transition-boundary-scope.md` in full, for house ADR voice; and
`tests/dev_fast_contract.rs`, for house test voice. Confirm no file matching
`docs/adr-003-*` exists.

Validation: run the whitespace-folded clause check from Step 1 below. Do
**not** attempt a line-for-line comparison of the quotations in this plan
against `docs/design.md`; this plan re-wraps them for its own 80-column limit,
so a literal match fails spuriously. Compare the folded clauses only.

### Stage B — author and verify together (EP-M1)

Add the two dev-dependencies. Write the complete contract test — parser, error
enum, all predicates, all negative controls. Write
`docs/adr-003-v0-1-exit-register.md` from the artefact embedded below, but
initially **without** the register block, so the live-document tests fail with
`MissingDelimiters`. Apply the glossary entry and the authorized R1 source
amendments before this red run, so unrelated anchors stay green. Capture that
transcript; this is the red step. Then insert the register block and re-run;
this is the green step. Commit only the green state, per Constraint 4.

Validation (go/no-go): `make fmt` then `make check-fmt`, `make lint`,
`make test`, `make markdownlint`, `make audit`. All green. The red transcript
is pasted into `Artefacts and notes`.

### Stage C — companion sync (EP-M2)

Apply the remaining sync-map references and tick roadmap 1.1.2. The R1
amendment and glossary entry were moved to EP-M1 by D9.

Validation: `make markdownlint`, `make nixie`, and `make test` — the last
because `INV-ANCHORS` now covers `docs/context.md` and the amended §13.7.

### Stage D — delivery (EP-M3)

Run the full gate set, push, open a draft pull request.

## Milestones and plateaus

### EP-M1 — the exit register exists and is guarded

- **Outcome**: `docs/adr-003-v0-1-exit-register.md` is present and accepted;
  `tests/v0_1_exit_register_contract.rs` passes and would fail if the register
  or its citations drifted.
- **Requirements discharged**: `ROADMAP-1.1.2` substantively; `TDD-11.1-B1`,
  `TDD-11.1-B2`, `TDD-11.2`, `TDD-13.6`, `TDD-13.7` each cited by at least one
  exit and checked by `INV-ANCHORS`.
- **Acceptance evidence**: `make test` green with the six named tests reported
  passing; the recorded red transcript showing `MissingDelimiters`.
- **Conformance check**: no `src/` change; `[dependencies]` still empty; the
  ADR chooses no exit; the ADR's additions beyond `design.md` are flagged as
  additions; gates G1/G2/G3 resolve.
- **Recovery**: `git checkout -- .`; nothing is destructive.
- **Remaining gaps**: companion documents do not yet reference the record.
- **Compatibility decision**: none required. Nothing is released, nothing is
  public, there is no consumer.

### EP-M2 — documentation coherent

- **Outcome**: every remaining companion document references ADR 003 and
  roadmap 1.1.2 is ticked. `docs/context.md` and the R1 amendments are already
  present from EP-M1 under D9.
- **Acceptance evidence**: `make markdownlint`, `make nixie`, `make test`
  green; every new cross-reference resolves.
- **Conformance check**: no scope prose contradicts ADR 002; the design
  amendment is the only substantive upstream change and matches R1 exactly.
- **Recovery**: each edit is a small, independently revertible insertion.
- **Compatibility decision**: none required.

### EP-M3 — delivery

- **Outcome**: branch pushed; draft pull request opened; gates green.
- **Acceptance evidence**: transcripts of every gate command, captured to
  `/tmp`.
- **Recovery**: the pull request is a draft and can be closed without effect.

## Concrete steps

Run everything from the repository root. Set a log prefix once per shell:

```bash
export LOG=/tmp/statelet-$(git branch --show-current)
```

### Step 1 — confirm the starting state and the conformance basis

```bash
git branch --show-current
ls docs/adr-*.md
fold_check() {
  tr '\n' ' ' < "$1" | tr -s ' ' | grep -qF "$2" \
    && echo "OK   $2" || echo "MISS $2"
}
for clause in \
  "both improve without framework adoption" \
  "the macro crate does not ship in v0.1" \
  "the project ships the conventions/runtime crate and defers" \
  "the project should ship nothing and keep the pattern local"; do
  fold_check docs/design.md "$clause"
done
```

Expected: the branch name, the two existing ADRs, and four `OK` lines. Any
`MISS` means the design has been revised and the conformance basis is stale.
Stop and revise this plan.

### Step 2 — add dev-dependencies

Edit `Cargo.toml`, `[dev-dependencies]` only, leaving `[dependencies]` empty.
`AGENTS.md` mandates full caret requirements:

```toml
[dev-dependencies]
camino = "1.2.5"
googletest = "0.14.3"
pretty_assertions = "1.4.1"
rstest = "0.26.1"
```

```bash
cargo fetch 2>&1 | tee "$LOG-fetch.out"
make audit 2>&1 | tee "$LOG-audit.out"
```

If another Cargo job holds the package-cache lock, wait. Do not create an
isolated Cargo cache. If `make audit` reports a new advisory, stop.

### Step 3 — write the contract test in full

Create `tests/v0_1_exit_register_contract.rs` per
`Interfaces and dependencies`. It must be complete — no `todo!()`, no unused
variants — because `panic_in_result_fn` is denied and `RUSTFLAGS="-D warnings"`
makes `dead_code` an error. At this point the ADR does not exist, so
`include_str!` will not compile; create the ADR file first with its prose and
no register block.

### Step 4 — observe red

```bash
make test 2>&1 | tee "$LOG-test-red.out"
```

Expected: the live-document tests fail with `MissingDelimiters`, while every
negative-control test passes:

```plaintext
---- totality_holds stdout ----
docs/adr-003-v0-1-exit-register.md: no exit register found between
  <!-- exit-register:begin --> and <!-- exit-register:end -->.
Repair: add the register block to the "Exit register" section.
```

If `make test` passes here, the suite is vacuous. Stop and fix the test.

### Step 5 — insert the register and go green

Add the register block, then:

```bash
make fmt        2>&1 | tee "$LOG-fmt.out"
git diff --stat
make test       2>&1 | tee "$LOG-test-green.out"
make check-fmt lint 2>&1 | tee "$LOG-lint.out"
make markdownlint   2>&1 | tee "$LOG-mdlint.out"
```

Run `make fmt` *before* `make test` deliberately: the parser must tolerate the
formatter's output, not merely the handwritten form. `make fmt` is known to
touch unrelated imported guides; restore any such churn before committing.

Commit only when every gate is green.

### Step 6 — companion sync

Apply every edit in the sync map. Each insertion is grep-guarded so re-running
is safe.

```bash
grep -rln "adr-003-v0-1-exit-register" docs/ | sort
```

Expected, exactly these seven files: `docs/contents.md`, `docs/context.md`,
`docs/design.md`, `docs/developers-guide.md`, `docs/roadmap.md`,
`docs/terms-of-reference.md`, `docs/users-guide.md`.
`docs/repository-layout.md` gains a test-file entry, not an ADR link, and
`README.md` is left alone — it links document *categories*, not individual
ADRs, and starting a new convention there is out of scope.

### Step 7 — full gates and delivery

```bash
make check-fmt              2>&1 | tee "$LOG-checkfmt.out"
make lint                   2>&1 | tee "$LOG-lint.out"
make test                   2>&1 | tee "$LOG-test.out"
make markdownlint           2>&1 | tee "$LOG-mdlint.out"
make nixie                  2>&1 | tee "$LOG-nixie.out"
make audit                  2>&1 | tee "$LOG-audit.out"
make test-workflow-contracts 2>&1 | tee "$LOG-wfc.out"
```

Read each log rather than scrolling the terminal; long output is truncated by
the environment. The last two are run by continuous integration and are easy to
forget locally. Commit after each milestone, then push and open the pull
request.

## Documentation sync map

1. `docs/contents.md`, "Decision records": add a bullet after the ADR 002
   entry, in the same two-line link-plus-description form.
2. `docs/design.md`: add a bullet naming `docs/adr-003-v0-1-exit-register.md`
   to the companion-documents list, matching the form of the ADR 002 entry
   already there. At the end of §11.1 add one sentence pointing to ADR 003 as
   the record that maps B1 and B2 to the v0.1 exits. **Under resolution R1**,
   amend §13.7's opening condition from "in both validation examples" to "in
   either validation example", and add a sentence noting that §§13.6 and 13.7
   are registered in ADR 003 as exits E2 and E1. Update "Last substantive
   revision".
3. `docs/terms-of-reference.md`: add the companion-documents entry; apply the
   matching R1 amendment to §7.1's final bullet; add ADR 003 as a linked bullet
   in §10.2. Update "Last substantive revision".
4. `docs/context.md`: add a glossary entry defining **v0.1 exit** — "one of the
   three mutually exclusive release scopes Statelet may deliver as v0.1, each
   selected by a stated verdict on bets B1 and B2 at a named roadmap gate; not
   to be confused with test exit criteria." This is D3's mitigation, and
   `INV-ANCHORS` treats it as a citation target so it cannot be silently
   dropped.
5. `docs/roadmap.md`: tick task 1.1.2 to `- [x]` and append the ADR link to its
   "Success:" bullet, matching how task 1.1.1 links ADR 002. Do **not**
   renumber anything; `INV-GATES` depends on 2.2.3, 3.1.3, and 4.3.1 keeping
   their numbers.
6. `docs/users-guide.md`, "Current status": two sentences telling a prospective
   consumer that the project may deliberately ship nothing, linking ADR 003. A
   consumer deciding whether to wait for this crate needs that.
7. `docs/developers-guide.md`: a short subsection recording that the exit
   register is machine-checked, that `tests/v0_1_exit_register_contract.rs` is
   the guard, that a revision touching `design.md` §§11.1, 11.2, 13.6, or 13.7
   or renumbering roadmap tasks 2.2.3, 3.1.3, or 4.3.1 will break `make test`
   by design, and how to repair such a break.
8. `docs/repository-layout.md`: note the new test file alongside the existing
   `tests/` entries.

## Validation and acceptance

Accepted when all of the following hold.

- `make test` passes, with `totality_holds`, `dominance_holds`,
  `quoted_passages_still_resolve`, `gate_bindings_resolve`,
  `hand_written_cases_match_register`, and `parser_rejects_malformed_registers`
  reported passing by name, alongside every negative-control case.
- The red transcript from Step 4 is recorded in `Artefacts and notes`, showing
  `MissingDelimiters` for the live-document tests while the controls passed.
- `docs/adr-003-v0-1-exit-register.md` shows: `Status: Accepted` with a date;
  three exits E1, E2, E3; a four-row decision table over the final bet
  verdicts, with `(Falsified, Held)` marked unreachable and explained; a gate
  table binding G1, G2, G3 to roadmap tasks 2.2.3, 3.1.3, 4.3.1; and quotations
  from `design.md` §§11.1, 11.2, 13.6, 13.7.
- **The reviewer's thirty-second check**: delete any one row from the register
  and re-run `make test`; it must fail naming the removed combination. Change a
  `Falsified` cell in a `B1` row to point at E3; it must fail
  `dominance_holds`, *not* a parse error — that distinction is what proves the
  parser applies no policy. Restore both afterwards.
- A second check: change `3.1.3` to `3.1.4` in the register's gate table and
  re-run; `gate_bindings_resolve` must fail. Restore.
- Every failure message names both files and the one-line repair.
- `make check-fmt`, `make lint`, `make markdownlint`, `make nixie`,
  `make audit`, and `make test-workflow-contracts` all pass, and `make fmt` is
  a no-op on the newly written files at commit time.
- `docs/roadmap.md` task 1.1.2 reads `- [x]` and links the ADR; tasks 2.2.3,
  3.1.3, and 4.3.1 are unchanged.
- The seven files listed in Step 6 each contain exactly one reference to the
  new record, and every such link resolves.

Quality criteria:

- Tests: `make test` green; every obligation discharged with its negative
  control observed rejecting for the stated reason.
- Verification: `INV-TOTAL`, `INV-DOMINANCE`, `INV-ANCHORS`, `INV-GATES`, and
  `INV-CASES` discharged.
- Lint: `make lint` green, including the Whitaker Dylint suite.
- Performance: `make test` wall-clock increase under roughly 30 seconds.
- Security: `make audit` green with no new ignore entries.

Quality method: run the seven gate commands in Step 7, capturing each to a log
under `/tmp`, and read the logs.

## Idempotence and recovery

Every step is safely repeatable. The documentation edits are grep-guarded
insertions. Nothing is deleted and nothing is destructive; `git checkout -- .`
returns the tree to the last commit at any point.

Specific recoveries. If `make lint` reports Whitaker findings, load
`addressing-whitaker-findings`; the likely offenders are `no_std_fs_operations`
(use `include_str!`, never `std::fs`), `module_must_have_inner_docs` (add `//!`
docs), `module_max_lines`, and `bumpy_road_function`. If clippy reports
`indexing_slicing` or `string_slice`, rewrite the parser with `split`, `trim`,
and `get()` rather than adding an allow. If `make fmt` breaks the parser, that
is Constraint 10 doing its job: make the parser tolerant rather than reverting
the formatting.

## Artefacts and notes

- Step 4 red, 2026-08-22: `make test` compiled the contract and reported the
  expected `MissingDelimiters` repair for every live-register assertion; all
  syntax and policy negative controls passed. The first run also exposed an
  overly strict B1/B2 table-row check, which was corrected to parse a trimmed
  first cell before the green run.
- Step 5 green, 2026-08-22: `make fmt` then `make test` passed 20 tests and
  one doctest. The test adds under 30 seconds to a clean run.
- EP-M1 gates, 2026-08-22: `make check-fmt`, `make lint`, `make test`,
  `make markdownlint`, and `make audit` all passed. The gate runner's logs are
  `/tmp/*-8a7120cf-1d08-4a72-9031-10a3d69a87de-1-1-2-record-the-three-possible-v0-1-exits-6.out`.
- EP-M1 CodeRabbit, 2026-08-22: `coderabbit review --agent` completed with
  zero findings after commit `7e8f957` was pushed.
- EP-M2 gates, 2026-08-22: `make markdownlint`, `make nixie`, and `make test`
  passed. Logs are
  `/tmp/{markdownlint,nixie,test}-8a7120cf-1d08-4a72-9031-10a3d69a87de-1-1-2-record-the-three-possible-v0-1-exits.out`.
- EP-M2 CodeRabbit, 2026-08-22: `coderabbit review --agent` completed with
  zero findings after commit `b2bb823` was pushed.
- EP-M3 gates, 2026-08-22: `make check-fmt`, `make lint`, `make test`,
  `make markdownlint`, `make nixie`, `make audit`, and
  `make test-workflow-contracts` passed. Logs are
  `/tmp/{check-fmt,lint,test,markdownlint,nixie,audit,test-workflow-contracts}-8a7120cf-1d08-4a72-9031-10a3d69a87de-1-1-2-record-the-three-possible-v0-1-exits.out`.
- EP-M3 CodeRabbit, 2026-08-22: `coderabbit review --agent` completed with
  zero findings and no rate limit after commit `1dcce4d` was pushed.

## Interfaces and dependencies

### New dev-dependencies

- `googletest = "0.14.3"` — matcher-based assertions with structured failure
  output, documented as working with `rstest`. Used so a totality failure
  reports *which* combination is missing.
- `pretty_assertions = "1.4.1"` — readable diffs when comparing parsed rows.

`camino` and `rstest` are already present. `[dependencies]` remains empty.

### Test module shape

In `tests/v0_1_exit_register_contract.rs`. Keep every function under the
Constraint 9 ceilings; the parser is expected to be four or five small helpers,
not one function.

```rust
//! Contract tests for the v0.1 exit register in
//! `docs/adr-003-v0-1-exit-register.md`.
//!
//! # What this guards
//!
//! The exit register says which of three v0.1 outcomes applies for each
//! combination of verdicts on the design's bets B1 and B2, and which roadmap
//! task decides it. These tests check that the register stays total, that a
//! falsified B1 always selects the off-ramp, that every passage the register
//! quotes from `docs/design.md` still says what is quoted, and that every
//! roadmap gate it names still exists and is untaken.
//!
//! # If this test fails
//!
//! It is telling you two documents have drifted apart, not that code is
//! broken. Each failure message names both files and the one-line repair.
//! Do not `#[ignore]` it: the drift guard is the only thing that makes the
//! register more than prose.
//!
//! Documents are embedded with `include_str!`, so the tests need no
//! filesystem access and rebuild whenever a source document changes.

/// A verdict on one of the design's named bets, at a gate evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

/// One parsed row of the register: a verdict pair, its exit, and its gate.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    b1: Verdict,
    b2: Verdict,
    exit: Exit,
    gate: String,
    reachable: bool,
}

/// Why a candidate register could not be read. Syntax only; no policy.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseError {
    MissingDelimiters,
    MalformedRow { line: usize },
    UnknownVerdict { found: String },
    UnknownExit { found: String },
}

/// Extracts the register rows from an ADR body.
///
/// Locates the block between `<!-- exit-register:begin -->` and
/// `<!-- exit-register:end -->`, splits each table row on `|`, and trims every
/// cell, so the formatter's column re-padding does not change the result.
///
/// This function applies **no policy**: it does not check totality, dominance,
/// or gate validity. Those are separate predicates, so that each obligation
/// fails independently and none is satisfied by construction.
fn parse_register(adr_body: &str) -> Result<Vec<Row>, ParseError> { /* ... */ }

/// Folds all whitespace runs to single spaces, for quotation comparison
/// across the formatter's 80-column reflow.
fn fold_whitespace(text: &str) -> String { /* ... */ }
```

The policy predicates are separate free functions over `&[Row]`:
`check_totality`, `check_dominance`, `check_quoted_clauses`, and
`check_gate_bindings`, each returning a `Result<(), String>` whose error is the
repair-instruction message required above.

Normative vocabulary, so two implementers write the same parser. Verdict cells
read exactly `Falsified` or `Held`. Exit cells read exactly `E1 ship nothing`,
`E2 ship conventions only`, `E3 ship macro`. Gate cells read exactly `G1`,
`G2`, or `G3`, and the gate table maps those to roadmap task numbers. Column
order is B1, B2, Exit, Gate, Reachable. Register cells contain no `...`, because
`mdtablefix --ellipsis` would rewrite it.

### The exit register itself

The normative content EP-M1 must produce, delimited so the formatter cannot
disturb it:

```markdown
<!-- exit-register:begin -->

| B1 verdict | B2 verdict | Exit                     | Gate | Reachable |
| ---------- | ---------- | ------------------------ | ---- | --------- |
| Falsified  | Falsified  | E1 ship nothing          | G2   | yes       |
| Falsified  | Held       | E1 ship nothing          | G2   | no        |
| Held       | Falsified  | E2 ship conventions only | G3   | yes       |
| Held       | Held       | E3 ship macro            | G3   | yes       |

<!-- exit-register:end -->
```

*Table 2: The v0.1 exit register. Each combination of final bet verdicts
selects exactly one exit at exactly one gate.*

The gate table, in the same section:

| Gate | Roadmap task | Decides                                                        |
| ---- | ------------ | -------------------------------------------------------------- |
| G1   | 2.2.3        | Whether the `mdtablefix` baseline is strong enough to continue |
| G2   | 3.1.3        | B1 finally, across both validation examples                    |
| G3   | 4.3.1        | B2, from the head-to-head macro comparison                     |

*Table 3: Gates at which each verdict is taken.*

Prose the ADR must carry alongside those tables:

- **E1 — ship nothing (the off-ramp).** Statelet is not published; the naming
  and tracing-field conventions stay project-local. Trigger: B1 falsified.
  Evidence: `design.md` §11.1 row B1 and §13.7. Decided at G2, with an early
  exit available at G1 if the `mdtablefix` baseline alone is weak — B1 requires
  *both* examples to improve, so one clear failure already falsifies it.
- **E2 — ship conventions only.** The runtime crate publishes `StateName` and
  the documented `transition.*` tracing field contract; `statelet-macros` is
  deferred, not cancelled. Trigger: B1 held, B2 falsified. Evidence:
  `design.md` §§11.2 and 13.6. Decided at G3.
- **E3 — ship the macro.** Everything in E2, plus `statelet-macros`. Trigger:
  B1 held, B2 held. Evidence: `design.md` §11.2. Decided at G3.
- **The dominance rule.** A falsified B1 selects E1 regardless of B2. The
  `(Falsified, Held)` row is unreachable by construction: G2 selects E1 when B1
  is falsified, so G3 is never reached and B2 can never be recorded as `Held`.
  The row is registered anyway, to keep the mapping total and to forbid a later
  reader reasoning from a promising macro to E3 over a dead wedge.
- **The nesting.** E3 ships a strict superset of E2. The three are disjoint
  *release scopes*, and the register partitions *verdict combinations*, not
  artefacts. Say this plainly; do not imply three unrelated products.
- **The split case.** Under resolution R1, B1 is falsified when *either*
  validation example fails to improve, not only when both do. This is bet B6
  ("`mdtablefix` is representative enough to generalize the wedge") failing.
  The ADR must state that this broadens `design.md` §13.7 and
  `terms-of-reference.md` §7.1, both of which are amended in the same change.

### The ADR's remaining sections

Follow the style guide's order. `Status: Accepted` with the date and a one-line
summary that does *not* restate the decision, following ADR 002's convention so
the two cannot drift. `Context and problem statement` frames the question:
under what evidence does Statelet ship the macro, ship conventions only, or
ship nothing? `Decision drivers` names B1, B2, and B6 as forces with their
recorded confidence levels, and states the rule that opinion evidence does not
raise confidence. `Options considered` presents the three exits plus the
rejected `--cfg statelet_unstable` fourth option from D5, with a captioned
comparison table. `Decision outcome` states the single decision from D2: defer
the ship/no-ship choice and bind it to the register above.
`Goals and non-goals` states that the ADR does not choose an exit.
`Known risks and limitations` records that no Rust-ecosystem precedent for
published kill criteria was found, and that the antecedents are managerial
rather than Rust-specific. `Outstanding decisions` lists the exit itself as
unresolved until G2 and G3, and cross-references `design.md` §14.

## Revision note

- 2026-08-22, revision 1: initial draft.
- 2026-08-22, revision 2: rewritten after a six-lens design review. Substantive
  changes, all of which alter the work rather than its description.
  - **Corrected the gate bindings.** Revision 1 bound E2 and E3 both to roadmap
    task 3.1.3. B2's evidence comes from task 4.2.1 and its decision from 4.3.1;
    3.1.3 only unlocks phase 4. Added `INV-GATES` to check the bindings
    mechanically, since `mapsplice` renumbers roadmap tasks and nothing
    guarded them.
  - **Removed the circularity.** Revision 1 had the parser reject
    dominance-violating tables, which made the dominance test tautological.
    Parsing is now strictly syntactic and every policy check is a separate
    falsifiable predicate.
  - **Corrected the dominance justification.** Revision 1 argued B2 is
    "undefined" when B1 fails; that is wrong, because §11.2's baseline survives
    a `wireframe`-only failure. The correct argument is that G3 is never
    reached, so the row is unreachable by construction.
  - **Surfaced the split case** as an unmapped combination in both upstream
    documents, with two resolutions, and made the choice an approval-gate
    question rather than an assumption.
  - **Strengthened `INV-ANCHORS`** from heading existence to load-bearing
    clause matching, whitespace-normalized, which catches a semantic rewrite
    that keeps the heading.
  - **Dropped `insta`, `proptest`, and `rstest-bdd`**, cutting new
    dev-dependencies from five to two. `rstest-bdd` 0.5.0 additionally needs
    the separate `rstest-bdd-macros`, an audit-ignore entry for the
    unmaintained `proc-macro-error`, and a `dylint.toml`. Recorded as D7 and
    flagged for approval, since it trims the standing test stack.
  - **Removed the red commit.** Revision 1 proposed committing a deliberately
    failing milestone, which contradicts `AGENTS.md` and its own Constraint 10.
    Red-Green now happens within EP-M1 and only green is committed; the plan
    drops from four milestones to three.
  - **Added the formatter constraint.** `make fmt` runs `mdtablefix --wrap
    --ellipsis`, so the register is delimited by HTML comments, parsing is
    whitespace-tolerant, and `make fmt` runs before the go/no-go.
  - **Added missing gates** `make audit` and `make test-workflow-contracts`,
    both of which continuous integration runs.
  - **Removed `todo!()` from the specified code**, which would have failed
    `panic_in_result_fn`, and specified the parser vocabulary, column order,
    and error variants that revision 1 left as a comment.
  - **Added the failure-message requirement** and the reviewer's
    thirty-second checks, so the guard is repairable by a stranger.
  - **Added the in-repository documentation signposts** the task asked for and
    revision 1 omitted.
- 2026-08-22, revision 3: implementation began. Recorded the unavailable Leta
  language server, the red/green evidence, and D9's sequencing correction so
  that `INV-ANCHORS` remains a complete green contract at EP-M1.
- 2026-08-22, revision 4: completed EP-M1 with a guarded ADR, total register,
  R1 amendments, and independently green deterministic gates.
- 2026-08-22, revision 5: reopened EP-M1 before commit because the contract
  test exceeded the 400-line file limit. Split its pure test-only support
  module and documented the ownership and non-reuse rule in the developer's
  guide; the milestone requires fresh gates before it can complete.
- 2026-08-22, revision 6: completed EP-M1 after the split test modules passed
  the complete deterministic gate suite independently.
- 2026-08-22, revision 7: blocked before EP-M2 at the plan's twelve-file scope
  tolerance. The remaining required sync would bring the total to fourteen; D11
  records the options requiring approval.
- 2026-08-22, revision 8: resumed after the user authorized the bounded
  fourteen-file completion recorded in D11.
- 2026-08-22, revision 9: completed EP-M2, including every required
  cross-document reference and the roadmap completion marker.
- 2026-08-22, revision 10: recorded the full EP-M3 deterministic-gate evidence
  and the required outcomes and retrospective section; final review is pending.
- 2026-08-22, revision 11: final CodeRabbit review returned zero findings;
  marked the plan complete.
- 2026-08-26, revision 13: reopened after review identified that reachability
  had no policy check, ADR quotations were hard-coded, and the handwritten
  expectation did not exercise the dominance-invalid fixture. Added EP-M4 and
  D13; the parser remains syntax-only and full delivery gates remain pending.
- 2026-08-26, revision 14: completed EP-M4. The focused red run rejected the
  unvalidated reachability and hard-coded citation controls; the green suite
  passed after the predicates were added. All deterministic delivery gates and
  the required CodeRabbit review are green.
- 2026-09-01, revision 15: completed EP-M5. A known but wrong row gate failed
  in the focused red run; the exit-to-gate policy made it pass in green. All
  deterministic delivery gates and the required CodeRabbit review are green.
