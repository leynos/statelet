# Implement StateName and handwritten examples

Status: DRAFT — awaiting explicit approval before implementation.

This ExecPlan is a living document. Keep its constraints, tolerances, risks,
progress, discoveries, decisions, verification, and outcomes current. This
change prepares roadmap task 2.1.1; it does not implement or complete it.

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
  Preserve ADR 004's pending return-shape decision at task 3.2.1.
- Proposed ADRs 006–007 and observation RFCs are not accepted requirements for
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
- [ ] Validate and publish the planning-only draft pull request.
- [ ] Obtain explicit plan approval.
- [ ] EP-M1: implement and validate the minimal public contract and examples.
- [ ] EP-M2: finish documentation, reconcile evidence, and mark 2.1.1 done.

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
  ADR 004's evidence gate and publication uncertainty.

## Outcomes & retrospective

The Wyvern team and the six-perspective expert panel completed planning review.
The revised plan documents reuse before implementation, checks the actual
feature matrix separately, stops gate scripts on failure, and uses a generic
lifetime witness. No runtime feature, release, consumption verdict, or roadmap
completion is delivered by this PR. Record actual gate logs, commits, and
observed behaviour here during execution. Before declaring `COMPLETE`,
reconcile discoveries against every conformance document below and record any
accepted deviations; unresolved deviations prevent completion.

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
`docs/design.md` §6.1 owns the public naming contract. ADR 004 and
`docs/phase-2-validation-note-template.md` govern later consumption evidence.
Do not fill a validation note with synthetic evidence from these examples.

## Conformance basis

All references below are the versions at planning baseline
`d9792043ce6a3625c722b8bce8947d4f575605d2`. Refresh this basis if upstream
changes before approval or execution.

- R1: [roadmap 2.1.1](../roadmap.md) and [design §6.1](../design.md) require
  stable enum labels without `Debug`. EP-M1 discharges this through exact label
  tests, a no-`Debug` compile witness, and runnable documentation.
- R2: design §§3.1–3.2, 9–10 and
  [ADR 002](../adr-002-transition-boundary-scope.md) constrain ownership and
  runtime size. EP-M1 keeps one dependency-free trait; existing topology and
  feature-matrix gates plus diff review check the boundary.
- R3: [terms of reference §§7–9](../terms-of-reference.md),
  [ADR 003](../adr-003-v0-1-exit-register.md), and
  [ADR 004](../adr-004-state-name-consumption-evidence.md) preserve the product
  exit and evidence gates. EP-M2 updates status without claiming B1 validation
  or a settled identifier decision.
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

For this planning-only PR, run formatting and documentation gates; runtime
milestones and their code gates remain unexecuted until approval. Inspect the
full branch diff, commit the validated plan, push over verified SSH, refresh
`origin/2-1-1-state-name-trait-examples`, and set it as upstream. Create a
draft PR titled `Plan: Implement StateName and handwritten examples (2.1.1)`
with a branch link to this document and a terminal References section linking
the Lody session. This publication is authorized; implementation is not.

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
