# Implement StateName and handwritten examples

Status: COMPLETE (2026-11-15).

This ExecPlan is a living document. Keep its constraints, tolerances, risks,
progress, discoveries, decisions, verification, and outcomes current. Approval
was granted explicitly on 2026-11-15; this change implements roadmap task
2.1.1, records its acceptance evidence, and marks the task done.

## Purpose / big picture

A consumer should be able to implement `statelet::StateName` for an ordinary
Rust enum and obtain a deliberately chosen static label without implementing
`Debug`. Handwritten examples will show the entire implementation rather than
hide it behind a derive macro. Running the public API tests and doctests will
show the exact labels and demonstrate that naming does not consume the state.

This is a small input to the later `mdtablefix` experiment, not evidence that
Statelet already improves that parser. The project can still choose to publish
nothing if local conventions prove sufficient.

## Constraints

- Await explicit approval of this plan before changing implementation files.
- Export exactly `pub trait StateName { fn state_name(&self) -> &'static str; }`
  with no supertraits, default implementation, or required associated items.
- Add no runtime dependencies, features, workspace members, proc macros,
  transition machinery, identifiers, tracing adapters, or outcome vocabulary.
- Keep the trait open to downstream implementations. Do not seal it or derive
  its labels from `Debug`, `Display`, type names, or numeric discriminants.
- Do not modify `mdtablefix` or claim its consumption evidence from local tests.
  Preserve ADR 0004's pending return-shape decision at task 3.2.1.
- Proposed ADRs 0006–0007 and observation RFCs are not accepted requirements for
  this task. Do not import their wider contracts or derive test matrix.
- Preserve strict lints, the shared Cargo cache, compiler admission wrappers,
  and sequential gates. Do not use `/tmp` as a build directory.
- Keep roadmap 2.1.1 unchecked until implementation and verification finish.
  Mark it done with `[x]` only at feature completion.

## Tolerances (exception triggers)

Stop implementation and set this plan to `BLOCKED` if the trait signature or
scope above must change, a runtime dependency is needed, a prerequisite is no
longer satisfied, or an accepted design decision would be contradicted. Record
options and impacts in the decision log and obtain explicit approval.

Allow up to 12 non-ExecPlan files and 600 net added lines of code and other
documentation. Exclude ExecPlan files from both counts. Existing test libraries
are sufficient; adding any dependency requires revisiting this plan. There is
no elapsed-time limit. A gate failure blocks committing; diagnose it rather
than bypassing it or starting duplicate builds.

## Risks

- Medium impact, medium likelihood: readers mistake a static lifetime for
  enforced semantic stability. Document that implementors choose and preserve
  names; Rust checks lifetime and types, not purity or label policy.
- Medium impact, low likelihood: examples accidentally create a public parser
  model. Keep example enums local and explicitly illustrative.
- Medium impact, medium likelihood: proposed observability refinements expand
  this task. Check authority and scope at each milestone boundary.
- Low impact, medium likelihood: broad formatters change unrelated documents.
  Inspect their diff and retain only intended changes; never discard another
  contributor's changes.

## Progress

- [x] (2026-11-03) Rename branch to `2-1-1-state-name-trait-examples`.
- [x] (2026-11-03) Read governing documents and obtain two Wyvern reports.
- [x] (2026-11-03) Draft contracts and proportionate verification strategy.
- [x] (2026-11-03) Complete six-perspective expert review and reconcile
      findings.
- [x] (2026-11-15) Validate and publish the planning-only draft pull request
      as PR 172.
- [x] (2026-11-15) Obtain explicit plan approval.
- [x] EP-M1: implement and validate the minimal public contract and examples.
  - [x] (2026-11-15) Re-run unchanged baseline gates: `make fmt`,
        `make check-fmt`, `make lint`, and `make test` all exited zero; logs
        are `/tmp/fmt-statelet-2-1-1-baseline.out`,
        `/tmp/check-fmt-statelet-2-1-1-baseline.out`,
        `/tmp/lint-statelet-2-1-1-baseline.out`, and
        `/tmp/test-statelet-2-1-1-baseline.out`.
  - [x] (2026-11-15) Write the `StateName` contract into design §6.1 and the
        reuse policy into the developer guide before production code.
  - [x] (2026-11-15) Add `tests/state_name.rs`; the red stage failed with
        `E0432: unresolved import statelet::StateName` (exit 101, log
        `/tmp/test-state-name-red.out`).
  - [x] (2026-11-15) Replace the greeting stub with the exact trait and two
        handwritten doctests; focused tests and doctests pass (10 integration
        tests, 2 doctests; logs `/tmp/test-state-name-green.out` and
        `/tmp/test-state-name-doc.out`).
  - [x] (2026-11-15) Negative control: a temporary `Reading(0) => "Idle"`
        branch failed `naming_preserves_boundary_payloads::case_1_zero` with
        actual `"Idle"` against expected `"Reading"` (log
        `/tmp/test-state-name-negative-control.out`); the mutation was removed
        before the milestone gates.
  - [x] (2026-11-15) Complete EP-M1 gate sequence passed sequentially through
        the scrutineer against the final candidate: `make check-fmt`,
        `make lint`, `make test`, `make feature-matrix`, `make typecheck`,
        `make markdownlint`, and `make nixie` all exited zero. `make test`
        reported 352 nextest tests passed (21 skipped), 2 doctests passed,
        515 Python contract tests passed, and 3 topology plus 18 feature-matrix
        command-boundary tests passed. Logs are
        `/tmp/check-fmt-statelet-2-1-1.out`,
        `/tmp/lint-statelet-2-1-1.out`,
        `/tmp/test-statelet-2-1-1.out`,
        `/tmp/feature-matrix-statelet-2-1-1.out`,
        `/tmp/typecheck-statelet-2-1-1.out`,
        `/tmp/markdownlint-statelet-2-1-1.out`, and
        `/tmp/nixie-statelet-2-1-1.out`.
  - [x] (2026-11-15) Review R1–R4 against the diff: exact label tests, the
        no-`Debug` witness, and both doctests discharge R1; the single
        dependency-free trait and green topology/feature-matrix gates preserve
        R2; the provisional return shape and unchanged ADR 0004 gate preserve
        R3; the repository test frameworks and sequential gates satisfy R4.
        Commit EP-M1 as the coherent feature commit (`d1b293a`).
- [x] EP-M2: finish documentation, reconcile evidence, and mark 2.1.1 done.
  - [x] (2026-11-15) Document `tests/state_name.rs` in the repository layout
        and confirm every updated document is reachable from
        `docs/contents.md`. Update the contents entry for this plan from a
        proposal to the delivered record.
  - [x] (2026-11-15) Review the examples (design sketch, two Rustdoc examples,
        and the user-guide reference to them) for consistent labels and
        wording. They describe local behaviour only; neither `mdtablefix`
        consumption nor a return-shape verdict is claimed.
  - [x] (2026-11-15) Mark roadmap 2.1.1 `[x]` because its implementation and
        acceptance evidence now exist; leave 2.1.2, 2.1.3, and the isolated
        representation roadmap's unrelated 2.1.1 untouched.
  - [x] (2026-11-15) Complete EP-M2 gate sequence passed sequentially through
        the scrutineer over `d1b293a` plus these documentation edits:
        `make check-fmt`, `make lint`, `make test`, `make feature-matrix`,
        `make typecheck`, `make markdownlint`, and `make nixie` all exited
        zero. `make test` ran 373 nextest tests, all passed (33 skipped by
        design), and the Python workflow contracts and doctests were green.
        Logs are `/tmp/check-fmt-statelet-2-1-1-m2.out`,
        `/tmp/lint-statelet-2-1-1-m2.out`,
        `/tmp/test-statelet-2-1-1-m2.out`,
        `/tmp/feature-matrix-statelet-2-1-1-m2.out`,
        `/tmp/typecheck-statelet-2-1-1-m2.out`,
        `/tmp/markdownlint-statelet-2-1-1-m2.out`, and
        `/tmp/nixie-statelet-2-1-1-m2.out`.
  - [x] (2026-11-15) Commit the documentation and completion bookkeeping;
        re-run the Markdown and format checks over the recorded evidence, which
        is the only content changed after the full gate run.
- [x] EP-M3: rebase onto advanced `origin/main` and revalidate.
  - [x] (2026-11-15) Rebase the three-commit series from baseline `d979204`
        onto target `37e0c70` using the merge backend with `zdiff3` and the
        ambient Weave driver bypassed for an unattended replay. The two
        implementation commits applied cleanly; only
        `docs/repository-layout.md` conflicted, between the target's
        `tests/documented_examples` inventory and ADR renumbering and this
        branch's new entries. The resolution keeps both. The replayed series
        is `32545ee`, `c8b8ae3`, `7c7dba6`; recovery refs remain under
        `refs/recovery/2-1-1/`.
  - [x] (2026-11-15) Refresh inbound ADR links to the target's
        `docs/adrs/adr-nnnn-...` paths and normalize citations to four-digit
        numbers. Replace the user-guide Rust fence with a reference to the
        compiled Rustdoc example, as required by the target's tested
        user-facing example policy (D6).
  - [x] (2026-11-15) Full gate sequence passed sequentially through the
        scrutineer over `7c7dba6` plus these edits: `make check-fmt`,
        `make test`, `make typecheck`, `make lint`, `make feature-matrix`,
        `make markdownlint`, and `make nixie` all exited zero. `make test`
        reported 392 nextest tests passed (21 skipped), 2 doctests, 515 Python
        contract tests, 3 topology command-boundary tests, and 18 feature-matrix
        command-boundary tests. Logs are
        `/tmp/check-fmt-statelet-2-1-1-rebased.out`,
        `/tmp/test-statelet-2-1-1-rebased.out`,
        `/tmp/typecheck-statelet-2-1-1-rebased.out`,
        `/tmp/lint-statelet-2-1-1-rebased.out`,
        `/tmp/feature-matrix-statelet-2-1-1-rebased.out`,
        `/tmp/markdownlint-statelet-2-1-1-rebased.out`, and
        `/tmp/nixie-statelet-2-1-1-rebased.out`.
  - [x] (2026-11-15) Hosted completeness assessment
        EP-2-1-1-COMP-20261115-01 found no runtime defect and two documentation
        issues: the user guide claimed the returned label is not a fresh
        allocation, which the return type cannot guarantee (D7), and this
        retrospective called the 373-test EP-M2 run final while the planning
        steps still said implementation was unauthorized. The retrospective
        now distinguishes the EP-M2 and EP-M3 runs, and the planning steps are
        marked historical. Follow-up posted at
        <https://github.com/leynos/statelet/pull/172#issuecomment-6886883078>.

## Surprises & discoveries

At planning baseline `d9792043ce6a3625c722b8bce8947d4f575605d2`, `src/lib.rs`
contains only `greet()` and a TODO to remove that stub when real functionality
exists. No equivalent runtime naming abstraction was found. `Cargo.toml` has no
runtime dependencies; the needed test libraries already exist. Roadmap
1.1.1–1.1.3 and 1.2.1–1.2.3 are checked complete.

CodeGraph tools were absent from the available tool inventory and discovery
results. The requested skill was loaded; direct file reads supplied the
fallback evidence. Context pack `pk_ephobhax` carries the reconnaissance for
agent collaboration; this document contains the durable execution context.

At implementation on 2026-11-15, the compiled tree matched the planning
baseline: `src/lib.rs` held only `greet()`, and no runtime `StateName` existed.
The red stage failed for the intended reason — an unresolved import — rather
than a syntax error in the new test. The temporary payload-zero mutation was
observed failing the named boundary test before the correct body was restored.
No new dependency or feature was needed.

After publication, `origin/main` advanced from the planning baseline to
`37e0c70`, which adopted the latest documentation style guide (#169). That
change moved ADRs under `docs/adrs/` with four-digit numbers, added
`tests/documented_examples.rs`, and made every fence in a user-facing document
a marked, executable Bash example. The rebase applied both implementation
commits cleanly and conflicted only in `docs/repository-layout.md`, where the
new `tests/` inventory and the ADR renumbering met this branch's new entries.
The guide's Rust fence also violated the adopted example policy: the loader
rejects any unmarked or non-Bash fence in `docs/users-guide.md`. The user-guide
copy therefore became a link to the compiled Rustdoc example rather than a
duplicated fence, so the copyable implementation remains available and is
executed as a doctest.

## Decision log

- D1 (2026-11-03): use the exact §6.1 trait, directly in `src/lib.rs`. A second
  module or abstraction is unnecessary for a single method. Remove the greeting
  stub and its doctest when replacing it; no compatibility wrapper is needed
  for this pre-1.0 scaffold.
- D2 (2026-11-03): provide one fieldless enum and one small payload-bearing
  handwritten example. Payload-independent labels demonstrate the benefit over
  debug formatting without implementing a parser or state machine. The property
  test checks this example convention, not a guarantee supplied by the trait.
- D3 (2026-11-03): use finite case tests for labels and a lightweight property
  test for payload independence. There is no repository-owned business rule or
  unbounded transition algorithm requiring Verus. Do not manufacture a proof
  milestone for a trait declaration.
- D4 (2026-11-03): describe stable labels as the implementor's contract, not a
  guarantee the trait can enforce for arbitrary downstream Rust code. Retain
  ADR 0004's evidence gate and publication uncertainty.
- D5 (2026-11-15): mark the trait method `#[must_use]`. The repository denies
  `clippy::must_use_candidate`, and the attribute states the existing contract
  that a returned label should be used. It adds no supertrait, default method,
  associated item, or sealing, so the approved signature is unchanged.
- D6 (2026-11-15): after rebasing onto `37e0c70`, satisfy the user-guide
  deliverable by linking to the compiled Rustdoc example instead of repeating a
  Rust fence. The adopted documentation style guide requires every fence in a
  user-facing document to be a marked, executable Bash example, and its loader
  fails the build on any other fence. Repeating the implementation as a fence
  would therefore break a repository gate; the doctest already compiles and
  runs the full example. ADR references use the target's `docs/adrs/`
  four-digit paths because the old filenames no longer exist.
- D7 (2026-11-15): describe only the guarantees the signature actually gives.
  A `&'static str` return keeps the label alive for the rest of the program and
  does not consume the state, but it does not prove that no allocation
  happened: an implementation can build and leak a string before returning it.
  The guide now states the lifetime guarantee and explicitly declines the
  allocation claim, so it cannot mislead a reader into assuming an
  allocation-free implementation.

## Outcomes & retrospective

The Wyvern team and the six-perspective expert panel completed planning review.
The revised plan documents reuse before implementation, checks the actual
feature matrix separately, stops gate scripts on failure, and uses a generic
lifetime witness. The planning-only PR delivered no runtime feature; approval
on 2026-11-15 authorized the implementation.

EP-M1 delivered the exact `StateName` trait in `src/lib.rs` with two
handwritten doctests, added `tests/state_name.rs` with no-`Debug` fixtures, and
updated the design, user, and developer documentation. The red stage failed
with an unresolved import; the payload-zero negative control failed the named
boundary test before the correct match was restored. EP-M2 documented the new
integration test, updated the contents entry, and marked roadmap 2.1.1 done.

Each milestone gate sequence passed all seven applicable checks. EP-M2's run
covered 373 nextest tests plus doctests, Python workflow contracts, and the
topology and feature-matrix command boundaries. After the rebase, EP-M3 re-ran
every gate, and that final run covered 392 nextest tests, 2 doctests, 515
Python contract tests, and the same command boundaries. The approved runtime
scope did not change and no dependency changed. One documentation adaptation
followed the rebase: D6 replaces the user-guide's duplicated Rust fence with a
link to the compiled Rustdoc example, because the target branch now requires
every user-facing fence to be an executable Bash example. No `mdtablefix`
evidence or return-shape verdict is claimed. The remaining limitations are
intentional: the trait's label stability is an implementor obligation, not an
enforced property, and the `&'static str` return shape remains ADR 0004's open
decision at roadmap task 3.2.1.

## Context and orientation

`src/lib.rs` owns the library's public exports. `tests/` contains integration
and behavioural tests, including private document-contract checks; their
`StateName` terminology describes evidence records, not a runtime trait.
`tests/features/` contains existing Gherkin behavioural specifications.
`Cargo.toml` already provides `rstest`, `googletest`, `pretty_assertions`,
`proptest`, and `rstest-bdd` as development dependencies. Reuse them without
changing versions or adding features for this task.

`docs/contents.md` indexes documentation. `docs/users-guide.md` currently
explains the design-phase status. `docs/developers-guide.md` owns internal
conventions and reuse guidance; `docs/repository-layout.md` owns file placement.
`docs/design.md` §6.1 owns the public naming contract. ADR 0004 and
`docs/phase-2-validation-note-template.md` govern later consumption evidence.
Do not fill a validation note with synthetic evidence from these examples.

## Conformance basis

All references below were the versions at planning baseline
`d9792043ce6a3625c722b8bce8947d4f575605d2` and were refreshed after the rebase
onto target `37e0c70a5bc1c73aeabeac8bd795b98260be4202`. The target moved ADRs
under `docs/adrs/` with four-digit numbers, so the links and citations below
use those canonical paths.

- R1: [roadmap 2.1.1](../roadmap.md) and [design §6.1](../design.md) require
  stable enum labels without `Debug`. EP-M1 discharges this through exact label
  tests, a no-`Debug` compile witness, and runnable documentation.
- R2: design §§3.1–3.2, 9–10 and
  [ADR 0002](../adrs/adr-0002-scope-statelet-to-transition-boundary-marking.md)
  constrain ownership and runtime size. EP-M1 keeps one dependency-free trait;
  existing topology and feature-matrix gates plus diff review check the
  boundary.
- R3: [terms of reference §§7–9](../terms-of-reference.md),
  [ADR 0003](../adrs/adr-0003-record-the-v0-1-exit-register.md), and
  [ADR 0004](../adrs/adr-0004-define-the-statename-consumption-evidence.md)
  preserve the product exit and evidence gates. EP-M2 updates status without
  claiming B1 validation or a settled identifier decision.
- R4: repository `AGENTS.md` and the user's testing requirements map to the
  verification plan and the sequential acceptance commands in both milestones.

There is no separate accepted architecture record needed to restate §6.1.
Record the implementation clarification there. If approval introduces a
substantive architecture change, create an ADR following the documentation
style guide and link it from the design before implementing that deviation.

## Verification plan

### Abstract model and contract sketch

Before writing implementations, define a finite set of variant identities `V`
and a table `L: V -> static labels` independent of any Rust match body. For the
fieldless example use `Normalize -> "Normalize"`,
`TightCodeSpan -> "TightCodeSpan"`, and `VerbatimFlush -> "VerbatimFlush"`,
matching the design vocabulary. For the payload example use `Idle -> "Idle"` and
`Reading(u64) -> "Reading"`. The representation relationship maps each
concrete enum value to its variant, ignoring its payload. The expected name is
`L(variant(value))`.

The precondition is simply a valid value of the example enum; there is no
parsing or validation step. Postconditions for the examples are an exact static
label, unchanged state, and repeatable results. Equal variants with different
payloads have equal names. Labels are explicit source literals: Rust variant
renames need not change them. These are example implementation obligations, not
a claim that Rust enforces the same behaviour for every trait implementor. The
trait permits implementors to violate the convention through interior
mutability or other effects; do not claim automatic non-interference.

The reasoning is direct exhaustive case analysis over the finite variants;
there are no additional semantic lemmas or non-trivial axioms to introduce.
Rust's type/lifetime checking, exhaustive matching, and the documented
assertion and property-test interfaces are trusted tool boundaries. A static
reference alone neither proves zero allocation nor prevents work inside an
implementation. Review the handwritten bodies for literal-only matches rather
than promising a universal performance property.

### Obligations and executable evidence

- V1, exact mapping: in `tests/state_name.rs`, use `rstest` cases for every
  fieldless variant and both payload-example variants. Keep expected literals
  in test data, never derive the oracle with the implementation's own match. Use
  `googletest` matcher assertions for individual contracts and
  `pretty_assertions::assert_eq!` for an ordered complete label collection. A
  swapped or misspelled label must fail for the affected variant.
- V2, public API and borrowing: the test enum must deliberately lack `Debug`.
  Implement the public trait from the integration test crate, call it twice
  through `fn name<T: StateName>(state: &T) -> &'static str`, whose body calls
  `state.state_name()`. Retain the returned value as `&'static str` after the
  enum's scope ends, and inspect the original state after a borrowed call. No
  auxiliary trait bound should be needed. An added `Debug` bound or a result
  lifetime tied to `self` must break these witnesses.
- V3, payload independence: add a `proptest` case over `any::<u64>()` for the
  local `Reading` fixture and compare its result with the independently stated
  `"Reading"` label. No rejection filters are needed. Include explicit 0 and
  `u64::MAX` cases and verify the payload remains unchanged. This samples an
  open input space; it is not an exhaustive proof. A temporary wrong branch for
  payload zero must be rejected by the named boundary test; remove the mutation
  before gates and record that negative-control result.
- V4, executable teaching examples: replace the greeting doctest with a
  complete handwritten fieldless implementation and give a second payload
  example in Rustdoc. Check the expected labels in each doctest. Doctests must
  import `statelet::StateName`, compile without a derive or `Debug`, and retain
  the implementation in visible source rather than hidden boilerplate.
- V5, scope: inspect the public diff and run existing topology and feature
  checks through the normal gates. No features, runtime dependencies, parser
  models, or macro exports may appear.

The normal test command below runs the new integration tests and doctests. For
a focused red/green check use `cargo test --test state_name`; for doctests use
`cargo test --doc`. Use repository Make targets for complete gates and preserve
all configured compiler wrappers and flags. Record expected initial failure as
unresolved import `statelet::StateName` before the trait exists; then verify
that the full successful suite exercises all variants. Do not commit the
intentional red stage.

Use `googletest::assert_that!` for ordinary `rstest` tests when no accumulated
non-fatal assertions are needed. If using `verify_that!`, propagate its result;
never discard failures. Avoid introducing global test-result state merely to
combine frameworks. The upstream [googletest documentation][googletest] was
consulted through Firecrawl; compile the chosen pattern against this checkout's
pinned dependency rather than relying on a documentation version's macros.

No Gherkin feature is needed: this task adds no transitions, user workflow,
error path, or interaction sequence beyond the trait call. The public-crate
integration test and doctests are the consumer boundary checks. `rstest-bdd`
remains appropriate for the later parser workflow. Unhappy-path assurance here
is rejection of missing APIs, unintended bounds, and wrong labels, not an
invented `Result` API. Snapshot tests with `insta` add no value because there
are no multivariant output formats. Kani and Verus add no useful assurance to
this finite naming surface; no solver-specific command or proof milestone is
planned. If implementation introduces substantive business logic or new lemmas,
stop and revise the specification first. Any subsequently approved Verus
milestone must state its actual verifier command and prove the semantic claim
without `assume`, `admit`, or trusted external bodies; executable integration
coverage remains required alongside it.

## Plan of work

### EP-M1: public trait and handwritten examples

After approval, refresh Git status and confirm prerequisites and the absence of
an equivalent runtime abstraction. Read `AGENTS.md` and the references below.
Run the unchanged baseline gates and retain their logs. Document the
abstraction contract first: update `docs/design.md` §6.1 and the reuse policy in
`docs/developers-guide.md` as described below, before adding production code.
Then add `tests/state_name.rs` with locally owned fixtures for V1–V3, module
comments, and the independent oracle. Observe the missing-export failure first.

Replace the stub in `src/lib.rs` with the exact trait and method Rustdoc,
including the two handwritten examples for V4. Keep the implementation
unconditional and no larger than necessary; no private forwarding abstraction
or blanket implementation is needed. Make the focused tests pass, perform the
negative control, restore the correct body, and rerun the focused tests.

Update `docs/design.md` §6.1 to distinguish the implemented handwritten trait
from deferred derive support and explain the implementor-owned stability
contract. Update `docs/users-guide.md` with a copyable implementation and the
provisional publication status. Update `docs/developers-guide.md` with the
reuse strategy: applications own enums and explicit matches, any
application-owned state may implement the trait, consumers depend only on this
projection, and Statelet does not own state storage or transitions. Example
fixtures stay local to their tests; do not export them as shared parser
utilities.

Acceptance: V1–V5 pass, documentation describes the actual minimal API, and all
sequential milestone gates below pass. Review R1–R4 against the diff and commit
this coherent feature. If the milestone fails, retain the red evidence, repair
the smallest responsible change, and rerun the gates after edits; no partial
public API should be committed. The remaining gap is final navigation,
completion bookkeeping, and evidence reconciliation in EP-M2.

### EP-M2: reconcile documentation and completion evidence

Update `docs/repository-layout.md` for the new public API integration test and
confirm all new documentation is reachable from `docs/contents.md`. Review
examples for consistency and distinguish local success from later `mdtablefix`
validation. Check all conformance links and record actual test counts, log
paths, and negative-control observations in this plan. Mark roadmap 2.1.1 done
with `[x]` only when its acceptance evidence exists; leave adjacent tasks
alone. Use the `mapsplice` skill for structural roadmap edits if needed.

Run the complete sequential gates again for this final milestone, inspect the
diff, and commit documentation and completion bookkeeping. Failure leaves the
feature at the EP-M1 plateau; do not claim task completion until repaired and
validated. No publication or macro acceptance follows automatically from this
milestone. Before setting the plan to `COMPLETE`, confirm there are no
unaccepted deviations or stale upstream claims.

## Concrete steps and validation

Run from the repository root on `2-1-1-state-name-trait-examples`. A scrutineer
is the sole delegated full-gate runner for this candidate. Run each command to
completion, sequentially, with Bash `pipefail` and a separate `/tmp` log:

```bash
set -euo pipefail
make fmt 2>&1 | tee /tmp/fmt-statelet-2-1-1.out
make check-fmt 2>&1 | tee /tmp/check-fmt-statelet-2-1-1.out
make lint 2>&1 | tee /tmp/lint-statelet-2-1-1.out
make test 2>&1 | tee /tmp/test-statelet-2-1-1.out
make feature-matrix 2>&1 | tee /tmp/feature-matrix-statelet-2-1-1.out
make typecheck 2>&1 | tee /tmp/typecheck-statelet-2-1-1.out
make markdownlint 2>&1 | tee /tmp/markdownlint-statelet-2-1-1.out
make nixie 2>&1 | tee /tmp/nixie-statelet-2-1-1.out
```

Stop at the first failure. All commands must exit zero. The `make test`
entrypoint includes ordinary tests, doctests, topology, and command-boundary
checks. Its feature-matrix command tests use fixtures; the separate
`make feature-matrix` invocation checks the actual Statelet candidate.
`make markdownlint` includes spelling. Use milestone-specific suffixes when
retaining multiple evidence runs. Install missing pinned tooling via the
Makefile's documented installer targets; do not weaken gates. If spelling
regenerates unrelated `typos.toml` content, follow the repository's
separate-base-change policy rather than folding it in.

Historical planning step, completed on 2026-11-03 and superseded by the
2026-11-15 approval: for the then-planning-only PR, run formatting and
documentation gates; runtime milestones and their code gates were not executed
until approval. That step inspected the full branch diff, committed the
validated plan, pushed over verified SSH, refreshed
`origin/2-1-1-state-name-trait-examples`, and set it as upstream. It created a
draft PR titled `Plan: Implement StateName and handwritten examples (2.1.1)`
with a branch link to this document and a terminal References section linking
the Lody session. That publication was authorized; implementation was not
authorized until the approval recorded at the top of this plan.

## Idempotence and recovery

Reads and gates can be repeated. Inspect status before retrying edits; do not
replace someone else's work. Maintain small validated commits as rollback
points. On interrupted builds, inspect the existing process and log before
starting another. A queued compiler is not a failed test. Report unhealthy
admission or exhausted disk space without bypassing host controls. After any
candidate edit, run the required sequential checks against the final content.

## References and skills

Read these references before implementation:

- [Documentation style](../documentation-style-guide.md).
- [rstest fixtures](../rust-testing-with-rstest-fixtures.md).
- [Dependency injection](../reliable-testing-in-rust-via-dependency-injection.md).
- [Doctest DRY guidance](../rust-doctest-dry-guide.md).
- [Complexity and refactoring](../complexity-antipatterns-and-refactoring-strategies.md).
- [rstest-bdd guide](../rstest-bdd-users-guide.md).

There is no environment, clock, or filesystem dependency in the trait, so no
injected service or mutable environment fixture is necessary.

Use `execplans` to maintain this document, `rust-router` then
`rust-types-and-apis` for the trait boundary, and `rust-unit-testing` or
`proptest` when implementing the tests. Use `codegraph-mcp` for semantic
navigation if available and `leta` where its LSP tools are available. Use
`firecrawl-mcp` for external documentation gaps, `logisphere-experts` for
review, `commit-message` for commits, and `pr-creation` for publication. The
Wyvern reconnaissance and expert review inform planning; neither authorizes
execution.

[googletest]: https://docs.rs/googletest/0.14.3/googletest/

Revision note (2026-11-03): initial plan records the naming contract before
implementation, the test/proof boundary, and the explicit approval gate.

Revision note (2026-11-03, expert review): apply the architecture, contract,
and delivery corrections above. The planned API and approval gate are
unchanged; acceptance now explicitly includes the candidate feature matrix.

Revision note (2026-11-15): explicit approval recorded; PR 172 published the
planning-only draft and carries this implementation. Status moved from DRAFT to
APPROVED. Implementation begins at EP-M1; no scope, tolerance, or API decision
changed at approval.

Revision note (2026-11-15, completion): EP-M1 and EP-M2 are delivered and
verified, roadmap 2.1.1 is checked, and status moved from APPROVED to COMPLETE.
The recorded negative control, gate counts, and log paths are the acceptance
evidence; the return-shape decision and any `mdtablefix` consumption evidence
remain outside this plan.
