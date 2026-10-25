# Add a benchmark note template for macro cost budgets

Status: DRAFT — requires explicit approval before implementation.

This ExecPlan is a living document. Keep Constraints, Tolerances, Risks,
Progress, Surprises & discoveries, Decision log, Outcomes & retrospective,
Conformance basis, and Verification plan current at every milestone.

## Purpose / big picture

Give Phase 4 reviewers a reproducible record of what the proposed macro costs
against handwritten conventions. Roadmap task 1.2.3 succeeds when contributors
can copy a blank benchmark form, record both the real `mdtablefix` comparison
and a synthetic fixture containing 20 annotated transitions, and trace those
records into the macro publication decision. Missing evidence must block a
claim that the cost gate is complete. Filling a form does not itself establish
that the macro deserves publication.

This plan authorizes future template, documentation and private test work only
after approval. It does not authorize creating the macro, running its future
benchmarks, selecting a v0.1 exit, or marking task 1.2.3 complete now.

## Constraints

Preserve design section 11.7's strict build-time limit and inclusive size
limit: less than 0.5% of the non-macro baseline per annotated transition, and
no more than 2% release artefact growth. Preserve ADR 003's E1/E2/E3 exit
register and G3 at task 4.3.1. Budget changes require a separate accepted
design revision before the gate is relaxed.

Do not change runtime code, public interfaces, Cargo dependencies, feature
policy, CI workflows or the topology checker. Do not add a benchmark runner or
a second crate. Keep the existing `state-name-note` marker and schema intact.
No compatibility layer or shared parsing library is required.

Use the shared Cargo cache and compiler admission. Never clean another
worktree's build directory, disable compiler admission, or build under `/tmp`.
Stop and report exhausted storage. Run candidate gates sequentially, delegate
them to `scrutineer`, retain logs, and commit only a passing candidate.

## Tolerances (exception triggers)

Stop and record a proposed deviation if implementation needs more than 12
changed files excluding this plan, more than 400 lines in any code file, a new
dependency, a public API, a workflow change, or changes to the accepted cost
thresholds or exit policy. Stop after three unsuccessful source-fix attempts
for the same gate failure. Healthy compiler queues have no elapsed-time limit.

If the evidence-presence contract requires interpreting free-form publication
decisions or building a general Markdown parser, stop and propose a narrower
contract. Record the affected requirements and options, set status to BLOCKED,
and obtain explicit approval before proceeding.

## Risks

- High impact, medium likelihood: compiler caches or admission delays dominate
  the tiny marginal cost. Record cache state, wrappers, wait diagnostics and
  host load; do not declare a pass from incomparable or noisy samples.
- High impact, medium likelihood: a complete note is mistaken for a passing
  macro gate. Keep completeness, measured budget verdicts and the G3 decision
  separate; topology and usefulness evidence remain additional prerequisites.
- Medium impact, medium likelihood: a documentation test becomes a brittle
  prose parser. Bound parsing to explicit markers and task identifiers, allow
  formatting whitespace, and test diagnostic reasons with negative controls.
- Medium impact, low likelihood: benchmark notes enter the StateName scanner.
  Use a distinct marker and prove that unrelated notes remain ignored.

## Progress

- [x] (2026-10-25) Read repository instructions and requested skills; rename
  branch to `1-2-3-macro-cost-benchmark-note-template`.
- [x] (2026-10-25) Complete two Wyvern reconnaissance assignments and consult
  Cargo documentation through Firecrawl.
- [x] (2026-10-25) Complete the six-lens community-of-experts review and
  reconcile dependency, lifecycle and measurement findings.
- [x] (2026-10-25) Validate the reviewed plan for draft publication: formatting,
  lint, tests, type checking, Markdown, spelling and diagrams passed.
- [ ] Obtain explicit approval and record APPROVED before implementation.
- [ ] EP-M1: deliver the template, gate bindings and private contract together.
- [ ] EP-M2: validate acceptance evidence and mark roadmap task 1.2.3 done.

## Surprises & discoveries

The repository already has a marker-scoped validation-note system. Its
StateName contract is extensive and private to that evidence domain; copying
its entire parser would exceed this task. `docs/validation-notes/README.md`
currently describes machine-checked StateName notes, so it must distinguish the
new benchmark contract explicitly.

There is no macro crate or stress fixture to measure at this stage. Empty
benchmark-note directories must therefore pass before Phase 4 claims
completion. The blank template intentionally contains placeholders; copied
notes used as evidence must not.

## Decision log

- (2026-10-25, proposed) Use a dedicated Markdown form and a small private Rust
  integration contract. A form alone does not enforce the roadmap's evidence
  prerequisite; a general benchmark framework would exceed the task.
- (2026-10-25, proposed) Check synthetic evidence when task 4.2.2 is
  marked complete, and both comparisons when task 4.3.1 is complete. This
  provides a concrete repository-owned claim boundary. Human reviewers still
  decide whether measurements support publication; tests do not parse prose
  into an E3 verdict or certify benchmark truth.
- (2026-10-25, proposed) Record the evidence contract in design section 11.7
  and contributor guidance. No new ADR is needed because existing budget and
  exit decisions remain unchanged. Escalate any substantive policy change.
- (2026-10-25, proposed) No Verus milestone: this work introduces no runtime
  invariant, lemma or mathematical business-policy evaluator. Finite document
  schema checks and deliberate negative controls match the obligation.

## Outcomes & retrospective

The reviewed draft is ready for approval. No implementation or cost measurement
has occurred. Local validation passed: 206 Rust tests, one doctest, 432
workflow contract tests plus five subtests, and three topology-command tests.
Formatting, lint, type checking, Markdown, spelling and diagram gates also
passed. Shared Cargo locks and compiler admission lengthened validation without
causing a failure. The implementation retrospective must distinguish task 1.2.3
completion from Phase 4 budget success and the independent G3 decision.

## Context and orientation

The repository currently has one `statelet` library package. Documentation
starts at `docs/contents.md` and `docs/repository-layout.md`. `docs/design.md`
section 11.7 defines the cost budgets; section 12 requires a real-world
comparison. `docs/roadmap.md` task 1.1.2 is complete, satisfying this task's
prerequisite. Task 4.2.2 produces stress evidence, while task 4.3.1 is the
publication decision. ADR 003 maps that decision to G3 and the B2 bet that a
macro adds value over handwritten conventions.

A baseline is the same downstream workload expressed without the proposed
macro. Marginal cost here means the total clean-build time difference divided
by the number of annotated transitions; it includes fixed macro overhead
amortized across those annotations. It is not a measured derivative or an
assertion that every annotation has identical cost.

`tests/v0_1_exit_register_contract.rs` checks ADR 003.
`tests/state_name_consumption_contract.rs` and its child modules check the
separate StateName evidence. Consult these for conventions, not as a mandate to
generalize their helpers. `Makefile` owns the validation commands.

Required reference signposts:

- `docs/rust-testing-with-rstest-fixtures.md`: fixtures and finite named cases
  for the new document contract.
- `docs/reliable-testing-in-rust-via-dependency-injection.md`: pass document
  strings and note collections into checks; avoid global environment mutation.
- `docs/rust-doctest-dry-guide.md`: keep illustrative Markdown forms out of
  executable Rust examples; no public Rust API is introduced here.
- `docs/complexity-antipatterns-and-refactoring-strategies.md`: keep schema
  reading, directory acquisition and completeness checks separate and small.
- `docs/rstest-bdd-users-guide.md`: available if behavioural steps become
  necessary; ordinary integration tests cover this document-only interface.
- `docs/documentation-style-guide.md`: headings, tables, British Oxford
  spelling, ADR criteria and roadmap completion syntax.

Use `execplans` throughout; `rust-router` routes contract-test work to
`rust-unit-testing` and benchmark methodology to `rust-performance-and-layout`.
Use `codegraph-mcp` for semantic navigation and `leta` where required by the
Rust router. Use `firecrawl-mcp` for remote documentation gaps,
`logisphere-experts` for design review, `mapsplice` for structural roadmap
edits, and `commit-message` / `pr-creation` for delivery. Load each skill
before using it. Use `context_pack` to share exact evidence with agent teams.

## Conformance basis

The inspected upstream revision is `0ca827880da2dcc27e7e89340f96878999243ac9`.
At implementation start, compare changes since this revision in the following
documents and reconcile drift. `docs/terms-of-reference.md` and
`docs/design.md` provide the product/design basis; no additional external
standard governs this note format.

- R1: roadmap 1.2.3 and completed prerequisite 1.1.2 → EP-M1 → copyable form
  containing all four requested measurement classes.
- R2: design 11.7 → EP-M1 → unchanged inequalities, explicit calculations,
  20-transition evidence and two expansion-review records.
- R3: roadmap 4.2.2 / 4.3.1 and ADR 003 G3 → EP-M1 → completion claims reject
  missing benchmark evidence without altering E1/E2/E3.
- R4: design 12 → EP-M1 → distinct real `mdtablefix` and synthetic comparisons.
- R5: repository contributor/documentation rules → EP-M2 → sequential gates,
  indexed guidance, completion evidence and task 1.2.3 marked done.

At each milestone confirm these links, unchanged accepted policies, no new
public interface/dependency/trust boundary, and no unapproved persistent format
beyond the proposed benchmark-note form. Record any deviation before proceeding.

## Plan of work

### EP-M1: Deliver the evidence instrument and its contract

First sweep existing helpers using CodeGraph and targeted path inspection.
Record why existing StateName-specific helpers cannot be reused directly. Keep
new helper ownership inside `tests/macro_cost_note_contract/`, callable only
from that integration target; document this reuse boundary in the developer
guide. Do not extract a shared library.

Add `docs/macro-cost-benchmark-note-template.md` containing a fenced, copyable
form with marker `<!-- macro-cost-note -->`. Copy one form per comparison to
`docs/validation-notes/<task>-<subject>.md`. Proposed subjects are
`4.2.1-mdtablefix-macro-cost.md` and `4.2.2-synthetic-macro-cost.md`. A bounded
table between `<!-- macro-cost-register:begin -->` and
`<!-- macro-cost-register:end -->` uses columns `Field`, `Value`, `Evidence`.
Define the following required rows and their semantics in the template:

- `comparison-kind`: exactly `mdtablefix` or `synthetic-20`.
- `candidate-id`: the macro commit under evaluation, shared by the pair of
  notes cited by G3, together with the governing design revision; old notes
  cannot stand in for a different candidate.
- `revisions`: baseline and macro repository commits, fixture paths, lockfile
  identity and a comparison diff identifying the annotation changes.
- `environment`: hardware, operating system, target triple, compiler/Cargo and
  tooling versions, profiles, features, backend, linker, flags and job count.
- `clean-build-protocol`: exact commands, build-directory ownership/reset,
  cache and incremental settings, wrappers, admission and host-load evidence,
  run order, repetition count and exclusions with reasons.
- `clean-debug-baseline-seconds` and `clean-debug-macro-seconds`: all raw sample
  durations plus the chosen summary statistic and spread, with log citations.
- `annotation-count`: actual count and source locations; exactly 20 for the
  synthetic comparison, positive and explicit for the real comparison.
- `marginal-seconds` and `marginal-percent`: calculation inputs and result.
- `release-baseline-bytes` and `release-macro-bytes`: exact corresponding
  binary or library artefact paths, byte counts and build commands.
- `release-growth-percent`: inputs and result; never compare whole target
  directory sizes or different artefact kinds.
- `stress-fixture-results`: synthetic source citation, proof that all 20
  transitions are represented, correctness checks and observations. For a real
  comparison use exactly `Not applicable: real comparison`; the real and
  synthetic experiments can be committed independently.
- `fallible-expansion` and `infallible-expansion`: `cargo expand` commands,
  revision-bound output citations and reviewer explanation of readability.
- `budget-verdict`: separate build-time, size and expansion conclusions, any
  inconclusive evidence and the governing design revision.
- `review`: author, reviewer, date, evidence limitations and links to the
  Phase 4 decision and other required evidence when available.

Placeholders are allowed in the blank form only. After trimming whitespace,
empty cells, case-insensitive `TBD`, `TODO`, `pending`, `N/A` and unfilled
angle-bracket placeholders are incomplete in copied notes. Only the `review`
row may contain `Pending: G3 decision not yet recorded`. The real comparison
exception for `stress-fixture-results` is the only ordinary not-applicable
value. A failed measurement may instead say `Unavailable:` followed by a
reason, a revision-bound failure-log citation and an explicit `Inconclusive`
budget verdict. Such an honest adverse record is complete as evidence of
failure, but cannot support macro publication. Keep raw samples and long
outputs in cited artefacts, with summaries in the table. Require citations of
the form `<repo>@<full-sha>:<path>` for measurement, source and failure
evidence; human review verifies that the referenced artefact exists and
supports the claim. Preserve raw negative deltas rather than clamping or
silently rounding them. Define, with positive baseline time T0, macro time T1,
annotation count N, and artefact sizes S0 > 0 and S1:

```plaintext
marginal seconds = (T1 - T0) / N
marginal percent = 100 * (T1 - T0) / (N * T0)
release growth percent = 100 * (S1 - S0) / S0
synthetic time acceptance: marginal percent < 0.5, N = 20
size acceptance for each comparison: release growth percent <= 2
```

Require matched workload semantics, observation fields, dependencies except for
the macro implementation, and features across each comparison. Identify the
executable entrypoints or library exports that retain all measured transitions
so dead-code removal cannot erase the intended size comparison. Record
stripping, link-time optimization and code-generation settings.

Define T0 and T1 as medians of their respective raw sample sets. Propose at
least five paired clean-debug runs with alternating baseline/macro order;
record every sample and median plus range. A result dominated by queue time,
cache asymmetry or unexplained variance is inconclusive and cannot support
publication. This protocol operationalizes the budget without inventing a
statistical confidence guarantee. Distinguish clean build outputs from the
shared dependency-download and compiler caches: a new target directory alone
does not prove an uncached compiler run. Preserve admission wrappers and record
their effect; if valid evidence is impossible, report that limitation.

Add `tests/macro_cost_note_contract.rs` and at most two private child modules
for schema/note handling and test fixtures. The checker accepts document text
and supplied note collections; filesystem acquisition stays a thin boundary
using repository capability-filesystem conventions. Enumeration and read errors
must fail with the affected path, never masquerade as an empty note collection.
Include a filesystem-boundary negative control for a selected missing or
unreadable file. Use `rstest` cases with explicit repair diagnostics. The
contract checks live template row names, unique markers, required
non-placeholder measurement/evidence cells, allowed comparison kind, and the
Phase 4 documentation bindings. It does not calculate performance verdicts or
attempt to prove the truth of cited measurements.

Read completed checkboxes for the exact roadmap task identifiers 4.2.2 and
4.3.1, bounded to their task lines. When neither task is complete, an empty
benchmark-note collection is valid. Task 4.2.2 completion requires its
synthetic note only; task 4.3.1 completion requires both kinds. Preserve the
independent sequencing of tasks 4.2.1 and 4.2.2. Add a bounded Markdown link on
each completion criterion to its selected evidence note(s), with a
`macro-cost-candidate` HTML comment recording the evaluated macro SHA beside
the completed claim. Require the selected notes to exist and match that SHA and
governing design revision; unrelated historical notes must not satisfy a new
claim. Do not infer which candidate is current by scanning for the newest file.
Reject incomplete marked benchmark notes at any stage. Wrong or absent markers
cannot satisfy a required note; unrelated StateName notes remain ignored.
Require the publication task's criterion to reference completed cost evidence
and preserve its other gates. This does not prevent an informal prose claim
outside the checked roadmap; reviewers must enforce the documented decision
rule too.

Test first with in-memory fixtures and a failing live-template existence check
that reports a missing file, rather than an `include_str!` compiler failure.
Observe the intended red result before adding the template and guidance. Then
make the live checks pass. Do not commit the red intermediate state.

Update design 11.7 to name the template and completeness boundary. Update
roadmap 4.2.2 and 4.3.1 with the exact evidence prerequisites above, without
marking either complete. Update `docs/developers-guide.md`,
`docs/validation-notes/README.md`, `docs/contents.md` and
`docs/repository-layout.md` for note creation, ownership and the new test. No
user-guide change is required for an internal evidence form; if actual
user-facing behaviour enters scope, escalate and include that guide.

This milestone ends with the complete instrument and passing gates. Confirm
R1–R4, the preserved budgets and the private helper boundary; commit the
coherent change. Remaining work is final acceptance and roadmap completion.
Recovery is to repair this owned change or revert its atomic commit; no data
migration or compatibility mechanism exists.

### EP-M2: Record acceptance and close task 1.2.3

Review the instrument as a contributor copying it and a reviewer consuming it.
Use temporary illustrative data outside the tracked validation-note directory;
never commit fabricated measurements as real evidence. Record the verification
results below and reconcile all upstream changes. Mark only roadmap task 1.2.3
`[x]` (done), update this plan's status and retrospective, run the full
sequential gates again, and commit. R5 is discharged only by these recorded
results. No Phase 4 task is complete and no macro verdict is implied.

## Verification plan

The introduced invariants concern document completeness and gate wiring, not
runtime state transitions. There are no new lemmas or unbounded business
invariants requiring Verus. The external assumptions are Cargo's documented
build-output/profile interfaces, trustworthy revision-bound evidence, and human
review of benchmark comparability and expansion readability. Tests cannot prove
those last two assumptions; record that residual gap explicitly.

V1 (R1/R2): a live template has exactly the intended required rows and marker
boundaries. Named tests reject a removed row, duplicate row, wrong marker and
missing boundary with the specific cause. A valid in-memory form and the live
form are positive witnesses. Formatting whitespace is not a semantic change.

V2 (R3/R4): a completed task 4.2.2 or 4.3.1 cannot coexist with missing or
incomplete benchmark comparisons. Parameterize no claim, each individual claim,
and both claims against zero notes, each comparison alone, both complete
comparisons, and a placeholder-bearing note. Synthetic-only evidence satisfies
4.2.2 but not 4.3.1. A real note can be committed before its synthetic sibling.
Reject dangling selected links, mismatched candidate SHAs or design revisions
and historical notes not selected by the claim. Accept a documented failed
measurement as adverse evidence while retaining its Inconclusive verdict.
Confirm all combinations are exercised; unrelated notes cannot supply evidence.
Test changed checkbox spacing and unrelated task text so lookup remains bounded
to the right task.

V3 (R2): a reviewer can reproduce calculations and boundary decisions from the
form. Use clearly labelled illustrative inputs T0 = 100 seconds, N = 20: T1 =
109.8 gives 0.49% (below the time limit); T1 = 110 gives 0.5% (fails). S0 =
100000 bytes and S1 = 102000 gives 2% (passes); S1 = 102001 fails. Missing
samples, non-positive denominators, the wrong annotation count, unmatched
artefact kinds or incomparable cache conditions make evidence inconclusive.
These manual controls independently catch an inverted inequality or the wrong
denominator; they are not benchmark results or proof claims.

V4 (R3): the live design and roadmap retain the evidence requirement at the
specified sections/tasks. Negative controls remove each binding in memory and
must produce a targeted repair message. Existing ADR 003 and StateName tests
must continue to pass. No test silently promotes E3 from a filled form.

Use the existing Make test selector for focused red/green runs:

```bash
set -o pipefail
make test TEST_FLAGS='--all-features --test macro_cost_note_contract' \
  2>&1 | tee /tmp/test-statelet-macro-cost-focused.out
```

Check the Makefile selector still reaches the new integration target before
using it; its other test routes also run. Red must be the intended missing
form/binding assertion, green must pass all cases, and refactoring must retain
all negative controls. No expected-failure marker remains in the final suite.

## Concrete steps and acceptance

Work from the repository root. Before implementation, read this plan,
`AGENTS.md`, the cited design sections and the current Makefile, record user
approval, then set APPROVED. Capture the clean starting diff. Run baseline
checks before introducing contract code, and record pre-existing failures
separately from changes made here.

At each major milestone use a `scrutineer` agent to run the following commands
in this order with `set -o pipefail` and individual `/tmp` tee logs. Do not run
them concurrently. In the implementation phase include the focused red/green
run above before these final gates.

```bash
make fmt
make check-fmt
make lint
make test
make typecheck
make markdownlint
make nixie
git diff --check
```

Success means every command exits zero and each V1–V4 acceptance item has
recorded evidence. Inspect formatting changes before staging. If spelling
regenerates unrelated `typos.toml`, follow repository instructions for a
separate base change; never edit the generated file manually or bury it here.
If a gate fails, read its log, fix the owned cause and rerun affected gates on
the revised candidate. Do not weaken lints or treat queued builds as failures.

For this plan-only pull request, run the same ordinary gates before committing
and publishing; there is no red/green implementation run yet. Push via SSH,
refresh `origin/1-2-3-macro-cost-benchmark-note-template`, set the named
upstream and verify the remote head. Create a draft PR titled
`Plan: Add macro cost benchmark note template (1.2.3)`, link this document on
the branch, and end the body with the Lody session reference. Publication does
not count as approval.

## Idempotence and recovery

Copying the blank form creates a new task-scoped note; never replace the blank
form with observations. Repeat checks safely on the same revision. Fix
incomplete evidence rather than deleting adverse findings. Keep build outputs
in owned directories outside `/tmp`; retain logs separately. Preserve unrelated
work and revert only owned atomic commits when rollback is necessary.

## Research and review evidence

Firecrawl retrieved the official Cargo build documentation on 2026-10-25:
[Cargo build](https://doc.rust-lang.org/cargo/commands/cargo-build.html). It
documents `--target-dir` for output placement, `--release` for the optimized
profile, `--locked` for lockfile stability and `--timings` for compilation
reports. Record exact commands and timing boundaries; compilation reports alone
do not establish an uncontended, uncached wall-time comparison. No additional
benchmark dependency is proposed.

Two Wyvern agents supplied contract and verification reconnaissance through
context pack `pk_u2bhz5hg`. CodeGraph indexing and symbol navigation confirmed
the existing private documentation-contract pattern. The six expert lenses were
reviewed by three agents. Pandalump and Wafflecat identified accidental
dependencies between independent Phase 4 experiments; the plan now requires
only synthetic evidence at 4.2.2 and both at G3. Doggylump and Dinolump
identified the note-ordering conflict; real notes now use an explicit
synthetic-field exception. Buzzy Bee and Telefono requested candidate-bound
evidence, matched workloads and explicit placeholder rules; these are now part
of the schema and negative controls. The main trade-off is automated
completeness versus human assessment of measurement validity. Adverse evidence
remains recordable without becoming publication approval.

## Revision note

2026-10-25: Draft revised after expert review to preserve independent
experiments, bind selected evidence to the evaluated candidate, define honest
failed measurements and clarify measurement comparability. These revisions
change the future contract tests; implementation remains unstarted.

2026-10-25: Recorded the successful plan-publication gates. The draft remains
unapproved; the implementor must obtain approval before EP-M1.
