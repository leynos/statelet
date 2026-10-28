# Add a benchmark note template for macro cost budgets

Status: APPROVED — the user approved scope and tolerances on 2026-10-25, before
implementation; EP-M1 and EP-M2 acceptance completed on 2026-10-26. A
post-publication CodeScene follow-up passed deterministic validation and expert
review; commit `85e64a0` is published and PR #153 remains draft. A local
CodeRabbit review of repair commit `5bad4e6` found two overlapping placeholder
issues and one fixture-literal style issue; their corrections passed the full
local validation sequence. Commit `05a2360` passed its fresh local review with
five findings. Community review resolved conflicting fixture-location and
generic-syntax recommendations. Their corrections passed the full local gate
sequence. Commit `3be47f5` passed its local review with one minor checkbox
spacing finding that matches the explicit V2 verification requirement; that
correction passed the full validation sequence. The follow-up review of
`1a3581b` found two further contract gaps. Proposed remediation is recorded
below. The user approved the one-path variance on 2026-10-28, before the review
corrections resumed. Post-rebase validation passed; a fresh review remains.

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

This approved plan authorizes the template, documentation and private contract
work. It does not authorize creating the macro, running its future benchmarks,
selecting a v0.1 exit, or marking task 1.2.3 complete before EP-M2 acceptance.

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
- [x] (2026-10-25) Obtain explicit approval and record APPROVED before
  implementation; the approved scope and tolerances remain unchanged.
- [x] EP-M1: deliver the template, gate bindings and private contract together.
  - Implementation is delivered. The focused red run passed 18 cases and
    failed only on the missing live template path, as intended.
  - The focused green run now passes 22 contract cases, one doctest, 432
    workflow contract tests plus five subtests, and three topology-command
    tests. The first full gate attempt passed `make check-fmt`, then stopped at
    `make lint`: Clippy reported 51 findings in the new integration contract.
    A second run passed formatting and reduced the lint findings to six.
    A third run passed all gates but reported two module-size and two branch
    count advisories. A fourth run found two unused imports and one missing
    documentation code span after the module split. Those issues were fixed;
    the fifth full sequence passed without Clippy or Whitaker findings. Logs
    use the `ep-m1-5` suffix under `/tmp`.
- [x] EP-M2: validate acceptance evidence and mark roadmap task 1.2.3 done.
  - Contributor and reviewer dry runs used two temporary copies under `target/`
    and were removed after inspection. The real comparison used N = 3 and its
    exact stress-fixture exception; the synthetic copy used N = 20. All 18
    fields were filled with clearly illustrative values and revision-shaped
    citations. No measurement was recorded as real evidence.
  - Decimal checks reproduced the synthetic 0.49% pass and 0.5% failure
    boundaries, plus the 2% size pass and 2.001% failure. Roadmap 1.2.3 is
    marked done; the closed implementation candidate passed its full gates.
  - At EP-M2 acceptance, the integration root had exactly two direct private
    child modules, `contract` and `fixtures`, each below 400 lines. That
    milestone's diff contained 11 paths (10 excluding this plan), within the
    approved tolerance.
  - On 2026-10-26, PR #153's target `main` resolved to
    `0ca827880da2dcc27e7e89340f96878999243ac9`, already the branch's exact
    base. The explicit `zdiff3` rebase completed as a no-op with no conflicts;
    `range-diff` matched all three branch commits unchanged. Global Git
    attributes select Weave for Rust, so the unattended rebase disabled that
    ambient attribute file and used Git's built-in merge driver. The saved
    worktree patch was verified against the stash before rebase and restored
    afterwards.
  - Final validation passed: `make check-fmt`, `make lint`, `make test`,
    `make typecheck`, `make markdownlint`, `make spelling`, `make nixie`, and
    staged/unstaged diff checks. Rust tests passed 230 cases (three skipped),
    one doctest, 432 workflow tests plus five subtests, and three topology
    command tests. Markdown lint reported zero issues. The first Markdown-lint
    attempt stopped in spelling-file discovery because the removed test files
    were still tracked in the index; staging the reviewed deletions fixed that
    discovery, and the sequential documentation gates then passed.
  - No macro benchmark was run and no illustrative value was committed as
    evidence. Roadmap tasks 4.2.2 and 4.3.1 remain open; no Phase 4 budget or
    publication verdict is implied.
- [x] (2026-10-26) Reconcile post-publication CodeScene feedback in PR #153.
  - The review refactor keeps `MarkdownFile` as the path-plus-text boundary,
    moves register syntax and field-value checks under `contract::evidence`,
    and shares fixture-row mutation. All five Rust files remain below 400
    lines, and the current diff has 7 paths (6 excluding this plan), within
    the approved maximum.
  - The final `cs delta origin/main` reports two String Heavy advisories:
    53.3% of integration-root arguments across 15 functions and 41.4% of
    `evidence.rs` arguments across 14 functions, against a 39% threshold.
    Three expert reviewers found that raw Markdown, unknown field names,
    evidence prose and diagnostics are intentional until validation. The
    existing `MarkdownFile`, `Kind`, `Row` and `Register` types capture the
    useful invariants. Retain both advisories without metric-only wrappers or
    suppressions. Complexity and fixture-duplication findings are cleared.
  - The fourth sequential gate run passed `make check-fmt`, `make lint`,
    `make typecheck`, `make test`, `make markdownlint`, `make spelling`,
    `make nixie` and staged/unstaged diff checks. Tests passed 230 cases with
    three skipped, one doctest, 432 workflow cases plus five subtests, and
    three topology-command cases. The PR remains draft pending maintainer
    disposition of the retained advisories.
  - The first post-publication gate pass cleared `make check-fmt`, then
    `make lint` found nine compile errors in the extracted test modules. The
    sequence stopped there. A second lint attempt confirmed the row ownership
    fixes and narrowed the remaining errors to five private re-exports. The
    final source correction puts `Kind` and `full_revision` at the private
    contract boundary instead of widening their visibility through a child
    module. A third attempt compiled the contract and reported four Clippy
    shadowing findings; those bindings were renamed. The fourth gate run passed.
- [ ] (2026-10-26) Complete local CodeRabbit review and draft PR closeout.
  - Remediation commit `85e64a02e2557d645d5afd4c641dedebf50eb9f9` was pushed
    over SSH; the remote-tracking branch and GitHub head matched after
    readback. PR #153's body records the CodeScene disposition, gate results
    and both required links. A generated Sourcery summary currently follows
    References; final section order still needs readback and correction. The PR
    remains an open draft.
  - On published head `edc2239cb878132f9b4cae3dca76023bdf1edf24`,
    `act-validation` and `build-test` passed; hosted CodeScene failed its
    new-code health and advisory gates for the two reviewed String Heavy
    findings. Hosted CodeRabbit reports that review was skipped because the PR
    is draft. Keep both advisories unsuppressed and retain the draft state.
  - The local CodeRabbit retry completed with five minor findings: the
    annotation-count wording disagreed with the integer parser; the roadmap's
    inline comment example could be counted as a real candidate declaration;
    descriptive template prose could pass as note content; and synthetic notes
    could use the real-comparison stress exception. The two template-prose
    findings overlap. Independent expert reviews confirmed the gaps and the
    current fixes, including negative controls for the inline-only and duplicate
    declaration cases and a prose-tainted annotation count.
  - The first gate attempt on the repair worktree failed formatting, then lint
    reported a five-argument helper. The worktree changed during that lint run,
    so neither result is exact-candidate evidence. `make fmt` now passes; the
    helper was simplified before a fresh run. That stable repair diff passed
    `make check-fmt`, `make lint`, `make typecheck`, `make test`,
    `make markdownlint`, `make spelling`, `make nixie`, and staged/unstaged
    diff checks. Tests passed 235 cases (three skipped), one doctest, 432
    workflow cases plus five subtests, and three topology-command tests.
    Markdown lint reported zero issues. Logs use the `repair2-edc2239` suffix
    under `/tmp`. The changes and plan closeout still need commit and a new
    local CodeRabbit review before they are pushed.
  - The review of commit `9efd1be25c75cdad4dbd9fd6a7794982a0b53d59` found
    two further issues. An `Unavailable:` measurement with an empty or
    angle-bracket reason could satisfy completeness when the verdict was
    inconclusive; the copied form also lacked an H1 heading. The current change
    rejects unfinished reasons, preserves cited failure reasons, and adds a
    title to the copied note. The second full sequence passed formatting, lint,
    type checking, tests, Markdown lint, spelling, diagrams and diff checks.
    It passed 238 Rust tests (three skipped), one doctest, 432 workflow cases
    plus five subtests, and three topology-command tests. Logs use the
    `review2fix-9efd1be` suffix under `/tmp`; commit and local review remain.
  - The review of commit `5bad4e6c7867bdd98dedbf149a3cc2f09dc65648` completed
    with two overlapping findings that embedded angle-bracket placeholders
    could pass, plus one finding against escaped newline continuations in
    fixture strings. The current fix recognizes placeholder-shaped tokens
    without treating ordinary cost inequalities as placeholders, adds both
    negative and positive controls, handles malformed angle markup before a
    placeholder, and switches the fixture literals to `concat!()`. The first
    gate attempt found one Clippy `shadow_reuse` finding; renaming the trimmed
    value resolved it. The exact corrected candidate passed all gates:
    `make check-fmt`, `make lint`, `make typecheck`, `make test`,
    `make markdownlint`, `make spelling`, `make nixie`, and staged/unstaged
    diff checks. Tests passed 241 cases (three skipped), one doctest, 432
    workflow cases plus five subtests, and three topology-command tests. Logs
    use the `73d2f8d6` suffix under `/tmp`. Commit `05a2360` contains that
    repair and plan update.
  - Its fresh local review found five issues involving fixture isolation and
    cleanup, angle-bracket generics, and marker prefixes. Two community
    reviewers confirmed that fixtures should follow the existing
    `CARGO_TARGET_TMPDIR` harness and that a lexical path-generic heuristic
    should keep standalone placeholders detectable. The current correction
    uses a `Drop` guard, keeps marker boundaries, tests the review-row
    exception, and documents the lexical ambiguity. An initial gate run found
    seven Clippy diagnostics and one empty-unavailable test failure. The fixes
    passed the full gate sequence: formatting, lint, typecheck, tests, Markdown
    lint, spelling, Nixie, and staged/unstaged diff checks. Nextest passed 256
    cases with three skipped; one doctest, 432 workflow cases plus five
    subtests, and three topology-command tests passed. Logs use the
    `2571631c` suffix under `/tmp`. Commit `3be47f5` contains these changes.
    Its local review found one minor issue: `task_status` must accept spacing
    changes after the roadmap checkbox, as required by V2. The local
    `cs delta origin/main` reports 41.2% String Heavy arguments in the root
    and 50.0% in `evidence.rs`, against the 39% threshold. Architecture and
    measurement reviewers recommend retaining both advisories because inputs
    remain raw Markdown until precise contract diagnostics are produced. They
    distinguish this judgment from hosted-gate approval.
  - V2 explicitly requires changed checkbox spacing to be tolerated. The
    correction parses the marker separately, requires whitespace after it, and
    tests one space, multiple spaces, a tab, and malformed spacing. Full gates
    passed on the exact candidate: 261 Nextest cases passed, three skipped;
    one doctest passed; 432 workflow cases and five subtests passed; and three
    ignored topology-command cases passed. `make check-fmt`, `make lint`,
    `make typecheck`, `make test`, `make markdownlint`, `make spelling`,
    `make nixie`, and staged/unstaged diff checks all passed. Logs use the
    `17e46d52` suffix under `/tmp`. Commit `1a3581b` records this correction.
    Its follow-up local review found that selected note links are accepted
    anywhere in a task block and that lowercase `unavailable:` values can
    avoid the inconclusive-verdict requirement. Wyvern verified both gaps;
    architecture and measurement reviewers confirmed the narrow remedies.
    Before remediation, the branch diff contains 14 paths including this plan,
    exceeding the approved limit of 12 implementation paths by one. A proposed
    correction confined to already changed paths is recorded below. The user
    approved this variance on 2026-10-28. The correction confines selected
    links to a single `Success:` criterion, handles case variants of
    `Unavailable:`, and adds negative and positive controls. The exact staged
    candidate passed `make check-fmt`, `make lint`, `make typecheck`,
    `make test`, `make markdownlint`, `make spelling`, `make nixie`, and diff
    checks. Nextest passed 272 cases with three skipped; one doctest, 432
    workflow cases plus five subtests, and three topology-command cases passed.
    Commit `d414986` recorded the correction before rebase.
  - On 2026-10-28, rebase onto `origin/main` at
    `7f5a3f0dd7614902dadc4f89a915d6a0edf6d403` replayed all 15 branch commits
    without conflicts. Range-diff matched every patch; the tree difference from
    the saved branch contains only the target's `rstest` and lockfile updates.
    Post-rebase formatting, lint, tests, typecheck, Markdown lint, spelling,
    Nixie, and diff checks passed. Nextest passed 272 cases with three skipped;
    one doctest, 432 workflow cases plus five subtests, and three
    topology-command cases passed. The local review of `0c493a0` returned five
    findings in three groups: candidate-comment wording, maintenance guidance,
    and malformed-row diagnostics. Wyvern confirmed the gaps. The first gate
    attempt stopped at a Clippy implicit-clone finding in the row parser; the
    correction moves the owned field into the register. The corrected candidate
    passed formatting, lint, tests, typecheck, Markdown lint, spelling, Nixie,
    and diff checks. Nextest passed 274 cases with three skipped; one doctest,
    432 workflow cases plus five subtests, and three topology-command cases
    passed. Commit `5eb889f` recorded these fixes. Its local review suggested a
    simpler per-opener placeholder scan; Wyvern and architecture reviewers
    confirmed equivalent traversal. CodeScene also flagged `parse_register`
    at complexity nine; the existing cell-decoding helper now validates row
    width before register sequencing. The local CodeScene delta clears the
    complexity finding and retains String Heavy advisories: 44.4% across 18
    root functions and 45.0% across 22 evidence functions against 39%. Expert
    review retains those string inputs for the private Markdown boundary;
    hosted-gate disposition remains separate. The refactor passed formatting,
    lint, tests, typecheck, Markdown lint, spelling, Nixie, and diff checks.
    Nextest passed 278 cases with three skipped; one doctest, 432 workflow
    cases plus five subtests, and three topology-command cases passed. Commit
    `d9c3eb5` recorded the refactor. Its local review was rate-limited; the
    planned retry was cancelled when final expert review found that an outdented
    paragraph after a `Success:` bullet could supply a selected link. Bound
    continuation to indented content and add a negative control. The corrected
    candidate passed formatting, lint, tests, typecheck, Markdown lint,
    spelling, Nixie, and diff checks. Nextest passed 279 cases with three
    skipped; 432 workflow cases plus five subtests and three topology-command
    cases passed. Expert verification confirmed that fix, then identified a
    related list-boundary cases: a non-checkbox sibling could extend the task
    block, and a nested `Success:` could impersonate the task criterion. Bound
    the task to the next top-level dash item and select only its direct-child
    criterion; add negative and positive controls. The architecture reviewer
    confirmed both boundaries, including wrapped direct-child text. Formatting,
    lint, tests, typecheck, Markdown lint, spelling, Nixie, and diff checks all
    passed. Nextest passed 282 cases with three skipped; one doctest, 432
    workflow cases plus five subtests, and three topology-command cases passed.
    Commit `fc4ef3f` recorded the fix. Its local review found a wrapped log
    reference, stale validation-note scope, and duplicated stress-exception
    tests. Wyvern confirmed the corrections while preserving Phase 2 and 3
    guidance and both distinct stress assertions. Formatting, lint, tests,
    typecheck, Markdown lint, spelling, Nixie, and diff checks passed. Nextest
    passed 282 cases with three skipped; one doctest, 432 workflow cases plus
    five subtests, and three topology-command cases passed. Commit `b6cec8f`
    recorded those corrections. Its local review requested stronger candidate
    and note-kind controls, a shared stress-exception constant, and another
    synthetic fixture wrapper. Wyvern confirmed the missing controls; the
    measurement reviewer recommended a single protocol constant but found the
    existing note factory sufficient. The new revision-shape, unexpected-kind,
    and duplicate-kind controls passed all gates: Nextest passed 286 cases with
    three skipped; one doctest, 432 workflow cases plus five subtests, and three
    topology-command cases passed. Commit `d47188a` records those controls.
    The separate stress-rule refactor centralizes the exact exception while
    retaining literal test values as independent oracles. Formatting, lint,
    tests, typecheck, Markdown lint, spelling, Nixie, and diff checks passed;
    Nextest passed 286 cases with three skipped, one doctest, 432 workflow
    cases plus five subtests, and three topology-command cases. The current
    local CodeScene delta retains String Heavy advisories of 50.0% across 19
    root functions and 45.0% across 22 evidence functions against 39%; expert
    review supports retaining the raw text inputs. A commit and fresh local
    review remain.

## Surprises & discoveries

The repository already has a marker-scoped validation-note system. Its
StateName contract is extensive and private to that evidence domain; copying
its entire parser would exceed this task. `docs/validation-notes/README.md`
currently describes machine-checked StateName notes, so it must distinguish the
new benchmark contract explicitly.

CodeGraph also found the ADR 003 exit-register parser. It consumes a distinct
five-column verdict table, so the macro-cost `Field | Value | Evidence` parser
remains private under the benchmark contract instead of sharing either schema.

There is no macro crate or stress fixture to measure at this stage. Empty
benchmark-note directories must therefore pass before Phase 4 claims
completion. The blank template intentionally contains placeholders; copied
notes used as evidence must not.

The `cap-std` dependency is present without its optional `fs_utf8` feature. The
test boundary therefore uses `cap_std::fs::Dir` with `camino::Utf8Path` inputs
and reports entry-name, enumeration and content-read failures explicitly; it
does not change Cargo features. `mapsplice` preview and application worked for
task 1.2.3 and task 4.2.2. Its task-level replacement of 4.3.1 failed with a
`dependency anchor 4.3.1 was not found` error even for the unchanged fragment,
so that one task is updated by a narrowly scoped text patch and inspected in
the diff.

## Decision log

- (2026-10-25, approved) Use a dedicated Markdown form and a small private Rust
  integration contract. A form alone does not enforce the roadmap's evidence
  prerequisite; a general benchmark framework would exceed the task.
- (2026-10-25, approved) Check synthetic evidence when task 4.2.2 is
  marked complete, and both comparisons when task 4.3.1 is complete. This
  provides a concrete repository-owned claim boundary. Human reviewers still
  decide whether measurements support publication; tests do not parse prose
  into an E3 verdict or certify benchmark truth.
- (2026-10-25, approved) Record the evidence contract in design section 11.7
  and contributor guidance. No new ADR is needed because existing budget and
  exit decisions remain unchanged. Escalate any substantive policy change.
- (2026-10-25, approved) No Verus milestone: this work introduces no runtime
  invariant, lemma or mathematical business-policy evaluator. Finite document
  schema checks and deliberate negative controls match the obligation.
- (2026-10-25, implementation) Keep the filesystem adapter on the enabled
  `cap_std::fs::Dir` API. Enabling `fs_utf8` would alter the dependency feature
  set without adding value for the relative note scan.
- (2026-10-26, implementation) Keep raw Markdown, table values and diagnostics
  as strings until the private contract validates them. Expert review found
  that unknown field names and arbitrary evidence text are needed for precise
  errors, while wrappers would add conversions without carrying new invariants.
  Retain both CodeScene String Heavy findings rather than suppressing or hiding
  them with type-only changes. At that point a `Utf8PathBuf` return from
  `fixture_root` was deferred; a later review correction adopts it for the
  Cargo temporary-directory guard.
- (2026-10-26, review correction) An unavailable measurement needs a concrete
  reason even when the verdict is inconclusive. Keep the cited failure-record
  path valid, but reject an empty reason or an angle-bracket placeholder. Start
  copied notes with a level-one heading while retaining the marker contract.
- (2026-10-26, review correction) Treat only standalone candidate-comment lines
  as declarations, reject copied template values in marked notes, and allow the
  stress-results exception only for real comparisons. The local review prompted
  these contract corrections; the shared compile queue delayed the sequential
  gates but remained healthy.
- (2026-10-26, review correction) Reject embedded angle-bracket placeholders
  while preserving valid inequalities in evidence values. Use `concat!()` for
  the split Markdown fixture strings to follow the repository's Rust guidance.
- (2026-10-26, review correction) Keep filesystem fixtures under Cargo's
  `CARGO_TARGET_TMPDIR` and own each tree with a `Drop` guard. This follows the
  existing topology-command harness and removes stale fixture trees on panic.
- (2026-10-26, review correction) Treat placeholder checks as a lexical
  heuristic: reject standalone angle tokens even in backticks, but allow angle
  pairs attached to a path identifier as generic syntax. Contributors must not
  encode missing values as ambiguous `Type<sha>` text. Reject case-insensitive
  placeholder marker words at the start of values while preserving the exact
  `review`-row exception.
- (2026-10-26, review correction) Parse the roadmap checkbox marker separately
  from the task identifier and skip formatting whitespace between them. V2
  explicitly requires task lookup to tolerate changed checkbox spacing.
- (2026-10-28, approved deviation) Bind selected-note links to each task's
  `Success:` criterion and recognize `Unavailable:` case-insensitively for both
  placeholder and verdict checks. These review fixes require no new paths,
  dependencies, public API or budget changes, but the current branch has 13
  changed paths excluding this plan, one over the approved limit of 12. The
  user approved this one-path variance before these corrections resumed.

## Outcomes & retrospective

The reviewed plan was approved before implementation; EP-M1 and EP-M2 are
complete. Approval validation passed: 206 Rust tests, one doctest, 432 workflow
contract tests plus five subtests, and three topology-command tests. Approval
formatting, linting, type checking, Markdown, spelling and diagram gates also
passed. The focused EP-M1 green run passed 22 contract cases, one doctest, 432
workflow tests plus five subtests, and three topology-command tests. The first
full attempt passed formatting before Clippy found 51 findings. The second
reduced those to six. A third passed all gates with four Whitaker advisories; a
fourth passed formatting before lint found two unused imports and a missing
code span. The fifth full sequence passed without Clippy or Whitaker findings:
228 Rust tests (three skipped), one doctest, 432 workflow tests plus five
subtests, three topology tests, type checking, Markdown lint, spelling, Mermaid
validation, and diff checks. Shared Cargo locks and compiler admission
lengthened validation. EP-M2 acceptance dry runs used temporary illustrative
notes outside `docs/validation-notes`; they verified independent real and
synthetic copies, the real comparison exception, and the calculation
boundaries. No measurements were committed. All final local gates passed on the
M2 implementation and roadmap candidate; the plan and contents index then
recorded completion and passed documentation gates. Roadmap 1.2.3 is done,
while tasks 4.2.2 and 4.3.1 remain open. Task completion does not establish
Phase 4 budget success or settle the independent G3 decision.

The later local CodeRabbit review found five minor issues in the published
candidate, with two overlapping reports about copied template prose. The repair
clarifies the count field, distinguishes the inline roadmap example from a
standalone candidate declaration, rejects values copied unchanged from the
template, and prevents synthetic notes from using the real-comparison
exception. Negative controls cover those failure cases. All requested local
gates pass on the repair diff; the draft PR remains subject to hosted checks
and maintainer disposition of the two existing CodeScene advisories.

A review of the resulting commit found that the documented unavailable-value
syntax still admitted an empty reason and copied notes lacked a title. Those
gaps are addressed with negative controls for empty and bracketed reasons, a
positive case for a cited failure reason, and a level-one heading in the form.
The follow-up full sequential gate run passed; commit and a new local review
remain pending.

The local review of commit `5bad4e6` then found incomplete placeholder
detection for values such as `Measured on <target>` and split fixture literals
that should use `concat!()`. The new scanner distinguishes placeholder-shaped
tokens from comparison operators and continues scanning after malformed angle
markup. Negative and positive controls cover those boundaries, and the fixtures
now use `concat!()`. The first lint run exposed a shadowed binding; after
renaming it, the full sequential gate set passed. A local review of that commit
then found five further issues. The revised contract handles marker prefixes,
keeps Rust path generics distinct from standalone placeholders, moves scratch
fixtures under Cargo's test temporary directory, and cleans them with a `Drop`
guard. Contributor guidance documents the lexical heuristic. An initial gate
attempt reported Clippy findings and an empty-unavailable regression; the
corrected exact candidate passed the complete local sequence. Commit and a new
local review remain pending.

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

Add `tests/macro_cost_note_contract.rs` with two private child modules for
contract checks and test fixtures. Keep each implementation file within the
repository's 400-line limit. The checker accepts document text and supplied
note collections; filesystem acquisition stays a thin boundary using repository
capability-filesystem conventions. Enumeration and read errors must fail with
the affected path, never masquerade as an empty note collection. Include a
filesystem-boundary negative control for a selected missing or unreadable file.
Use `rstest` cases with explicit repair diagnostics. The contract checks live
template row names, unique markers, required non-placeholder
measurement/evidence cells, allowed comparison kind, and the Phase 4
documentation bindings. It does not calculate performance verdicts or attempt
to prove the truth of cited measurements.

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

The plan-only draft PR and its approval checkpoint are already published. For
each implementation milestone, run the focused contract test and the same
ordinary gates before committing. Push via SSH, refresh
`origin/1-2-3-macro-cost-benchmark-note-template`, set the named upstream and
verify the remote head. Keep the PR in draft during implementation. Its title,
summary and References section must describe the delivered work and retain the
branch-wise plan link and Lody session reference. Do not merge without explicit
authorization.

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

Firecrawl also checked the pinned capability-filesystem API:
[`cap_std::fs::Dir`](https://docs.rs/cap-std/4.0.3/cap_std/fs/struct.Dir.html)
and
[`cap_std::fs::DirEntry`](https://docs.rs/cap-std/4.0.3/cap_std/fs/struct.DirEntry.html).
The enabled API supports relative directory opening, enumeration and reads;
missing directories can be distinguished from other I/O errors. The optional
UTF-8 module is not enabled in this repository.

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

2026-10-25: Recorded the successful plan-publication gates and the user's
explicit approval of this scope and its tolerances. Implementation may proceed
through EP-M1 and EP-M2; no performance verdict is implied.

2026-10-25: Started EP-M1 after approval. The focused red test run confirmed
that the live-template check fails with the intended missing-file path while
the other 18 contract cases pass. After adding the form and bindings, the
focused target passed 22 cases; the `make test` routes also passed one doctest,
432 workflow contract tests plus five subtests, and three topology-command
tests. One nested topology fixture took 97.627 seconds under shared compiler
admission. Full EP-M1 gates remain to be run on the complete candidate.

2026-10-25: The first full EP-M1 attempt passed `make check-fmt` and stopped at
`make lint`, which reported 51 Clippy findings in the newly added contract
tests. The fixes replace unchecked slices and indexing, split note validation
into smaller helpers, and remove fallible fixture assumptions. No later gate
ran on that candidate; the complete sequence must be repeated after cleanup.

2026-10-25: The second full attempt again passed `make check-fmt` and stopped at
`make lint`. It reduced the diagnostics to two shadowed local names and four
ignored formatting results. The latest revision gives those locals distinct
names and builds fixture rows with direct string appends. No later gate ran;
the complete sequence must be repeated on this revision.

2026-10-25: The third complete gate sequence passed `make check-fmt`,
`make lint`, `make test`, `make typecheck`, Markdown lint, spelling, Mermaid
checks, and staged/unstaged diff checks. Tests included 228 Rust cases (three
skipped), one doctest, 432 workflow tests plus five subtests, and three
topology tests. Whitaker reported that the contract and fixture modules exceed
its 400-line limit and flagged two conditional branch counts. Split the modules
and simplify those conditions before accepting EP-M1; rerun every gate on the
resulting candidate.

2026-10-25: Split candidate-bound roadmap parsing into `claims.rs`, and moved
two focused schema-isolation tests into their own child modules. Reduced the
measurement-count and heading checks to explicit predicates. All Rust source
files are now at most 366 lines; the complete gate sequence must run on this
revision before EP-M1 can be committed.

2026-10-25: The fourth full sequence passed `make check-fmt` and stopped at
`make lint`. It found two unused imports introduced by the split and a missing
code span in a module comment. The latest revision removes both imports and
marks `StateName` as code; rerun the full sequence before acceptance.

2026-10-25: The fifth full EP-M1 sequence passed every gate on the 14-path
candidate. Clippy and Whitaker reported no findings; Python lint rated 10/10.
`make test` passed 228 Rust tests (three skipped), one doctest, 432 workflow
tests plus five subtests, and three topology tests. Type checking, Markdown,
spelling, Mermaid and staged/unstaged diff checks passed. EP-M1 is complete;
EP-M2 remains before the roadmap entry can be marked done.

2026-10-26: Preserved the uncommitted EP-M2 changes and rebased the branch onto
the fetched PR target. The target SHA matched the branch's exclusive base, so
Git reported the branch up to date; the three-commit range-diff is unchanged
and no conflicts occurred. Restored the verified patch and reviewed its
semantic diff. The roadmap checkbox is set, the dry-run arithmetic passed, and
the final sequential gates passed. The plan and contents index now record
completion; their documentation gates passed.
