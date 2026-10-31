# Add the current-crate feature matrix (1.2.2)

Status: COMPLETE

This ExecPlan is a living document. Keep its constraints, tolerances, risks,
progress, discoveries, decisions, verification evidence, and outcomes current.
The planning pull request delivered this document without completing roadmap
task 1.2.2; the implementation described below completed it, and the Progress
and Outcomes sections record the delivered evidence. The user approved
implementation within its constraints and tolerances on 2026-10-25. A
pinned-tool experiment then exposed a row-selection gap before implementation;
the user approved the correction on 2026-10-28.

## Purpose and big picture

Provide one contributor command, `make feature-matrix`, that checks the current
Statelet library without defaults, with defaults, with each declared feature in
isolation, and with all features. In particular, `tracing` and `serde` must be
checked independently as soon as they exist. The command must already work on
today's crate, which declares neither feature. Document absent features as not
yet applicable, never as successful feature checks.

This establishes the validation spine before optional functionality appears. It
leaves the choices to ship nothing, conventions only, or a macro intact.
Success is observable through the command's exit status, Cargo's reported
invocations, deliberately broken fixture crates, and routine local and
Continuous Integration (CI) wiring.

## Context and orientation

The baseline is commit `0ca827880da2dcc27e7e89340f96878999243ac9`. `Cargo.toml`
defines one `statelet` library and no `[features]` table. `docs/roadmap.md`
marks prerequisite 1.2.1 complete and 1.2.2 incomplete. `Makefile` owns
contributor entrypoints; `all` orders its gates explicitly. `CARGO_FLAGS`
currently includes `--all-targets --all-features` and therefore must not be
reused to construct this matrix.

`tests/topology_check.rs` checks declared dependency topology.
`tests/topology_command.rs` and its private child modules exercise real Make
subprocesses using isolated Cargo fixtures. Their ignored tests run through
`make test-topology-command`, after ordinary tests and outside coverage.
`tests/workflow_contracts/mock_wiring.py` supplies evaluated Make recipes;
`topology_wiring_contract.py` and `topology_wiring_test.py` demonstrate wiring
contracts with negative controls. Consult these before adding helpers.
`.github/workflows/ci.yml` runs topology validation and a separate command
boundary test after coverage.

A feature matrix here means a set of library compilation configurations. It is
neither an exhaustive feature powerset nor proof of resolved dependency policy.
It complements the existing topology gate, whose future multi-crate policy
remains subject to issue 112.

## Conformance basis

All repository references below use the baseline revision above. Read
`docs/contents.md` and `docs/repository-layout.md` for orientation, then:

- R1: `docs/roadmap.md` task 1.2.2 and `docs/design.md` section 11.5 require
  one entrypoint for the five named configurations and an automated tool. EP-M1
  supplies it; V1 and V2 establish its observable behaviour.
- R2: `docs/design.md` sections 5 and 10 preserve the current crate boundary,
  additive features, and the unresolved `tracing` default decision. EP-M1
  preserves these constraints; V3 and diff review provide evidence.
- R3: `docs/design.md` sections 11.1, 11.5–11.7 connect feature validation to
  bet B5 and routine checks. EP-M2 wires validation; V4 guards the wiring.
  Compile-time budgets and multi-crate policy are not changed here.
- R4: `docs/terms-of-reference.md` and accepted ADR 002's marker-only scope,
  ADR 003's exit register, and ADR 004's evidence gate prohibit speculative
  runtime scope. Both milestones retain those decisions. No new product
  contract or architectural decision record is currently needed.

Use the `execplans`, `codegraph-mcp`, `hexagonal-architecture`, `rust-router`,
and routed `arch-crate-design` skills. Use `firecrawl-mcp` for further external
research, `logisphere-experts` for design review, and `mapsplice` if completion
requires structural roadmap editing. For Rust test details, load the routed
`rust-unit-testing` skill before implementation.

Required reading includes `docs/rust-testing-with-rstest-fixtures.md` for
parameterized cases,
`docs/reliable-testing-in-rust-via-dependency-injection.md` for process-local
inputs, `docs/rust-doctest-dry-guide.md` for executable API examples if needed,
`docs/complexity-antipatterns-and-refactoring-strategies.md` for cohesive
helpers, and `docs/rstest-bdd-users-guide.md` for behavioural specifications if
the command grows beyond simple process wiring. Follow
`docs/documentation-style-guide.md` and `docs/scripting-standards.md`.

## Constraints

Do not implement before approval. Record approval and set status to APPROVED
before starting EP-M1. Do not add public features, runtime dependencies, a
second crate, runtime APIs, or a decision about feature defaults. Preserve
existing topology policy and its separate acceptance evidence.

Keep orchestration in Make and test-only process helpers under
`tests/feature_matrix_command/`. No new domain port, utility crate, generic
feature planner, or custom Cargo metadata parser is justified. Hexagonal
architecture applies here by keeping process infrastructure outside the
library. Before extracting any helper, search for an equivalent and document
its owner, permitted callers, and reuse policy in `docs/developers-guide.md`.

Do not use cargo-hack's `--no-dev-deps`: it temporarily rewrites real
manifests. Do not use feature-exclusion options that remove required rows, a
powerset, `--clean-per-run`, an isolated Cargo cache, or a `/tmp` build target.
Keep `RUSTC_WRAPPER` and `RUSTDOC` admission intact. Gates run sequentially
with logs captured using `tee` and shell `pipefail`; a scrutineer owns full
gates.

## Tolerances (exception triggers)

Stop and record a proposed deviation if a second package, production Rust
change, new runtime dependency, manifest rewriting, custom discovery parser, or
change to the five-row meaning becomes necessary. The proposed tool pin is
cargo-hack 0.6.45, observed installed during planning; changing it requires
recording compatibility evidence and the reason before proceeding.

Stop if the library-only isolation experiment described in V3 fails. Do not
silently weaken the check or add `--no-dev-deps`. Set status to BLOCKED and
present alternatives for explicit approval. The same applies to an upstream
design contradiction. Host disk exhaustion requires stopping and reporting;
unhealthy compiler admission requires reporting its status, without recovery
commands or bypasses. Healthy queueing is not a failure or time limit.

## Risks

High impact, medium likelihood: development dependency features could hide a
missing normal dependency feature. V3 is a decisive go/no-go experiment before
the approach is accepted as implemented.

Medium impact, medium likelihood: cargo-hack's automatic all-feature execution
is conditional on feature count. An explicit all-feature invocation removes
that ambiguity; zero- and one-feature fixtures test it.

Medium impact, medium likelihood: test subprocesses could inherit coverage
flags or contend for the outer build directory. Run them serially after
ordinary tests or coverage with separate fixture target directories under the
worktree's `target/`, preserving compiler admission.

Medium impact, low likelihood: an absent feature could be mistaken for a
checked feature, or defaults could contaminate isolated rows. Documentation
must distinguish availability from success, and V1/V2 use distinguishable
feature configurations rather than empty-default fixtures alone.

**Approved correction:** the pinned cargo-hack experiment showed that
`--each-feature` does not select implicit optional dependency features unless
`--optional-deps` is supplied. Add that option to the isolated-row command so
the existing V1 obligation for implicit optional dependencies is met. The user
approved this correction on 2026-10-28.

## Proposed command and interfaces

Add `.PHONY` targets `install-cargo-hack`, `check-cargo-hack`,
`feature-matrix`, and `test-feature-matrix-command` in `Makefile`. Use
`CARGO_HACK_VERSION` with default `0.6.45` as the shared local/CI pin. The
installer uses
`$(CARGO) install --locked --version $(CARGO_HACK_VERSION) cargo-hack`; retain
development build-tool prerequisites and admission. The check target compares
the installed version exactly and fails before matrix work if missing or
mismatched, naming expected and actual versions and `make install-cargo-hack`
as the repair command. Matrix execution must not install tools implicitly.

The four sequential commands, shown without Make's compiler environment, are:

```sh
cargo check --no-default-features --lib --manifest-path "$PWD/Cargo.toml" -p statelet
cargo check --lib --manifest-path "$PWD/Cargo.toml" -p statelet
cargo hack check --each-feature --optional-deps --exclude-all-features --lib \
  --manifest-path "$PWD/Cargo.toml" -p statelet
cargo check --all-features --lib --manifest-path "$PWD/Cargo.toml" -p statelet
```

Use `$(CARGO)` and `$(BUILD_JOBS)` in all four recipes. Each binds the same
absolute `$(CURDIR)/Cargo.toml` and `-p statelet`. Apply the existing
warning-denying `RUSTFLAGS` composition and `STANDARD_RUSTFLAGS` once per
subprocess, preserving caller flags. Do not forward `CARGO_FLAGS`,
`TEST_FLAGS`, or arbitrary matrix feature arguments. Add the two matrix targets
to `.NOTPARALLEL`.

cargo-hack discovers declared features, including future `tracing` and `serde`.
The explicit Cargo commands own the no-default, default, and all-feature modes
even when default features are absent. Absent optional features have no
invented invocation; the guide explicitly states that they are not validated
until declared. Retain upstream command diagnostics instead of creating a
stable output format or parsing human-readable logs in production. A success
summary, if added, must describe only declared configurations and must never
appear after a failing child.

Fixtures run a copy of the real Makefile in isolated package roots, with the
existing build-tool preflight inputs available there. Do not introduce a
production manifest override merely for testing. Test process environments may
set executable paths and target directories on each child; never mutate the
parent process environment. Follow the topology harness's caller-flags pattern
to avoid composing `RUSTFLAGS` twice.

## Verification plan

The abstract specification is a set of required configurations for package P:
no defaults; the manifest's defaults; each declared feature alone with defaults
disabled; and every declared feature together. `tracing` and `serde` are
members of the isolated set exactly when declared. Compilation success means
every selected invocation succeeded; a missing tool or any failed invocation
makes the command fail. Duplicate semantic configurations when defaults are
empty do not remove the requirement to exercise the corresponding modes.

Preconditions are a valid current-package manifest, the pinned toolchain and
cargo-hack, and available build tools. Postconditions concern process status,
configuration selection, source preservation, and routine invocation. Cargo's
feature semantics, cargo-hack's documented interface, and Make's ordinary
failure propagation are external assumptions, not properties to prove anew. No
new business logic, semantic lemma, or unbounded state invariant is introduced.
Verus, Kani, and proptest are not warranted for this finite command
integration. If implementation introduces a non-trivial selection algorithm,
return to this specification and reconsider verification before coding it. Do
not add speculative proof bodies, `assume`, `admit`, or trusted shortcuts.

### V1: Coverage and default isolation

Use `rstest` fixtures and parameterized end-to-end cases in
`tests/feature_matrix_command.rs`, with private helpers in its sibling folder.
Cover no features, only tracing, only serde, both, a non-empty default, and an
implicit optional dependency feature. Also cover an explicit `dep:` feature,
which suppresses the dependency's implicit feature. Include fixture-only
`default = ["baseline"]`, `baseline = []`, `tracing = []`, and `serde = []` to
make the five required rows distinguishable. These fixture features must never
be added to Statelet's manifest.

Include a fixture root containing spaces to test path quoting. Capture child
invocations through a test-owned forwarding executable or use Cargo's
diagnostic invocation records, asserting semantic argument sets rather than
complete output snapshots. The forwarding executable must delegate to real
Cargo for compilation; pure command doubles alone cannot discharge V1. Assert
no-default, default, tracing-only, serde-only, and all-feature modes, including
disabled defaults on isolated rows. Witness explicit no-default and default
commands for each fixture and validate zero/one-feature all-feature execution
explicitly. This finite witness set and V2's failures prevent success from an
empty matrix. Run `make test-feature-matrix-command`; every expected row must
be witnessed and all healthy fixtures must pass.

### V2: Failure propagation and non-vacuity

For the distinguishable fixture, inject one `compile_error!` predicate at a
time, true only in each required row. For example, a default-only fault needs
`baseline` enabled with both optional features disabled; an isolated tracing
fault needs tracing enabled with baseline and serde disabled. Demonstrate each
healthy fixture first, then require its poisoned variant to fail for the
intended compiler message. A syntax error that fails every row is insufficient.

Also test absent cargo-hack, a mismatched version, malformed Cargo input, and
an all-features-only failure. Require a non-zero status, actionable
diagnostics, and no global success message. Do not require GNU Make to preserve
the child's numeric exit code. `googletest` assertions and `pretty_assertions`
provide clear boolean and structured-result diagnostics; use existing
dependencies first. These tests are executable behavioural and command-boundary
evidence. A second `rstest-bdd` layer would duplicate the same simple cases and
is not planned. There is no multivariant presentation contract needing `insta`
snapshots.

### V3: Library isolation and source preservation

Create a real local path dependency exposing a symbol only behind a feature.
The fixture's normal dependency does not enable that feature; its development
dependency on the same package does. A library reference to that symbol must
fail under the matrix's `--lib` check. First show a healthy counterpart passes,
then require the broken counterpart's missing-symbol diagnostic. This tests the
external assumption that development features cannot mask the defect. If it
unexpectedly passes, block EP-M1 and revise the design with approval.

Compare all fixture source manifests byte-for-byte before and after healthy and
failing matrix runs. Keep lockfile generation separate from the manifest
assertion. Routine repository runs must leave tracked manifests unchanged. This
is integration evidence, not a claim to prove all future feature leakage.

### V4: Routine wiring and argument integrity

Add `tests/workflow_contracts/feature_matrix_wiring_contract.py` and
`feature_matrix_wiring_test.py`, reusing the existing evaluated-recipe and
strict workflow-reading helpers. Assert that `make all` invokes the matrix once
after `check-topology`, CI provisions the pinned tool and invokes the same
target, and `make test` invokes the command-boundary suite after ordinary
routes. CI runs the latter after coverage without inherited instrumentation.

Negative controls must remove each gate, alter either package/manifest binding,
inject `--all-features` into the isolated command, remove warning/build flags,
remove serialization, disconnect the ignored suite, or weaken version checks.
Each control must fail for its intended clause. Preserve all topology wiring
contracts. `make test-workflow-contracts` must pass the real configuration and
reject every control. Include new recipe names in existing development-build
flag checks where those checks enumerate targets.

## Milestones and plan of work

### EP-M1: A tested local matrix command

After approval, re-read the current manifest, Makefile, and conformance
sources. Run the baseline gates below and record their results. Sweep existing
helpers before extraction. Add the smallest command-boundary tests first and
observe failure because the target is absent. Keep red results as evidence, not
as a commit. Implement the pinned installer/preflight and the four-command
recipe, then pass V1–V3. Add a serial ignored-test entrypoint patterned on
`test-topology-command`, using `--run-ignored only --no-tests fail` and one
test thread. Wire it into `make test` in this same milestone so tests cannot
remain unexecuted locally. Do not run nested command tests in ordinary nextest
or coverage runs.

Document the local command, installation, absent-feature semantics,
library-only scope, and test-helper reuse policy in `docs/developers-guide.md`.
Record the matrix decision and limits in design section 11.5, linking this
plan. Update `docs/repository-layout.md` for new test ownership. The user's
guide receives a short command/link update only where its existing public build
instructions would otherwise be incomplete; no consumer API change is claimed.

Acceptance is V1–V3 plus all milestone gates. Recheck R1/R2, unchanged feature
defaults and library API, source preservation, and current trace links. Commit
this coherent local capability once green. CI integration remains for EP-M2. If
the milestone fails, keep the uncommitted evidence and fix or block within
tolerances; no destructive cleanup or partial passing claim is permitted.

### EP-M2: Routine validation and completion

Write V4's failing wiring tests, then add the matrix to `all` after
`check-topology` and before spelling. Add `make install-cargo-hack` and
`make feature-matrix` to CI after build provisioning and topology validation.
Add `make test-feature-matrix-command` after coverage and the existing topology
command boundary step. Preserve all existing topology order and coverage
boundaries. Run V4 and the full milestone gates, then reconcile design and
operator documentation with the actual command.

Only after all acceptance evidence exists, mark roadmap item 1.2.2 done by
changing its checkbox to `[x]`, update this plan's progress and outcomes, and
record the implementation result. Repeat affected documentation checks and
commit the coherent completed change. R3/R4 and V4 must be discharged without
changing product gates or claiming resolved-graph proofs. If interrupted, EP-M1
remains a usable local command; 1.2.2 stays unchecked until EP-M2 passes.

## Concrete validation and recovery

Run from the candidate worktree root. A scrutineer runs gates sequentially and
records exit statuses and logs under `/tmp`; pipeline status must be preserved:

```bash
set -eo pipefail
make fmt 2>&1 | tee /tmp/fmt-statelet-1-2-2-feature-matrix-command.out
make check-fmt 2>&1 | tee /tmp/check-fmt-statelet-1-2-2-feature-matrix-command.out
make lint 2>&1 | tee /tmp/lint-statelet-1-2-2-feature-matrix-command.out
make typecheck 2>&1 | tee /tmp/typecheck-statelet-1-2-2-feature-matrix-command.out
make test 2>&1 | tee /tmp/test-statelet-1-2-2-feature-matrix-command.out
make markdownlint 2>&1 | tee /tmp/markdownlint-statelet-1-2-2-feature-matrix-command.out
make nixie 2>&1 | tee /tmp/nixie-statelet-1-2-2-feature-matrix-command.out
```

Stop the sequence on failure and inspect its log. `make markdownlint` includes
spelling. Inspect formatter and generated spelling changes before staging;
unrelated generated dictionary changes require a separate base change under
repository policy. After EP-M1 and EP-M2 also run `make feature-matrix` and
`make check-topology`, with the same logging discipline. Validate a changed
Makefile with `mbake validate Makefile`. Gate success means completed zero exit
statuses, never merely progress output. Tests run both before and after code
changes; the red phase fails for the expected missing or broken behaviour.

Use worktree-local fixture directories and target directories; never delete
shared caches or another agent's files. Re-running gates is safe after fixing a
candidate defect. A healthy compiler queue needs patience, not retries. Revert
an owned milestone commit to roll back if needed; do not rewrite shared history
or revert unrelated edits. Rolling back an implemented milestone means
reverting its commit on a branch, never rewriting the published history of the
pull request.

## Progress

- [x] (2026-10-25) Renamed branch and inspected the baseline and prerequisite.
- [x] (2026-10-25) Completed two Wyvern investigations and external tool
      research.
- [x] (2026-10-25) Incorporated six-expert findings; final review found no
  approval-blocking design defects.
- [x] (2026-10-25) Completed plan review and all publication validation gates.
- [x] (2026-10-25) User approved implementation; the selected decisions below
      are binding for EP-M1 and EP-M2.
- [x] (2026-10-25) Baseline `make check-fmt`, `make lint`, `make typecheck`,
      `make test`, `make markdownlint`, and `make nixie` passed at approved
      checkpoint `0582f9ed2206ceaf24be5a55bbbcbdf15523c98a`.
- [x] (2026-10-25) Before implementation, cargo-hack 0.6.45 fixture evidence
      exposed that implicit optional-dependency features are omitted without
      `--optional-deps`; the plan is blocked pending approval of the proposed
      correction.
- [x] (2026-10-28) User approved the `--optional-deps` correction and
      authorized implementation to resume.
- [x] (2026-10-28) The initial command-boundary test failed because Make had no
      `feature-matrix` target. The V3 isolation experiment passed its healthy
      and missing-symbol controls with unchanged fixture manifests.
- [x] (2026-10-28) The local command and 16 boundary cases passed their focused
      run. The workflow contracts initially rejected a wrapped topology
      `.NOTPARALLEL` declaration; preserving that declaration and adding a
      separate matrix declaration restored all 432 cases and five subtests.
- [x] (2026-10-28) The full EP-M1 gates passed at the tree committed as
      `bf5545c`. CodeRabbit reviewed all 11 changed files and found Act
      provisioning, fixture cleanup and path-convention gaps. The direct
      `make all` matrix gate is already scheduled for EP-M2.
- [x] (2026-10-28) The EP-M1 review repairs and subsequent feature-mode
      corrections passed the full gate sequence before reassessment.
- [x] (2026-10-28) The EP-M1 reassessment reviewed all 15 paths and identified
      three maintainability refinements: share matrix recipe arguments, reuse
      Cargo subcommand dispatch, and split the row assertions into helpers.
- [x] (2026-10-28) The EP-M1 refactoring passed all gates and a fresh
      CodeRabbit review of all 16 changed paths with no findings.
- [x] (2026-10-28) EP-M1 implemented and validated the local command.
- [x] (2026-10-28) EP-M2 integrated routine gates, passed all milestone gates,
      and marked 1.2.2 done. Its final review covered all eight changed paths.
- [x] (2026-10-28) Hosted CI and Act exposed forced-colour cargo-hack output
      that the test parser did not recognize. Reproduced the failure, corrected
      parsing, strengthened wiring controls, and passed all local gates and a
      fresh review of the three repair paths with no findings.
- [x] (2026-10-28) A second hosted run exposed cargo-hack's distinct GitHub
      Actions group output. Reproduced it with child-only CI variables, added
      a real fixture regression, and passed all gates and repair reassessment.
- [x] (2026-10-30) A final gate run on this host exposed a pre-existing
      locale dependency in the shared Python discovery recipe. `find | sort`
      under `en_US.UTF-8` orders `.github` differently from the code-point
      order the workflow contract expects, failing two contract cases on
      every branch. Pinned `LC_ALL=C` for the inventory; the module passed
      under C collation before the change. Recorded as a separate
      prerequisite repair, updated the discovery contract assertion that
      matched the literal `| sort` text, then re-ran the gates.
- [x] (2026-10-31) Committed the prerequisite repair as `f930e22` and pushed
      it (`git@github.com:leynos/statelet.git`), confirming the remote head
      matches. Gate run 4 on this content passed `fmt`, `check-fmt`, `lint`,
      `typecheck`, workflow contracts (515 tests plus 5 subtests), nextest
      (218), doctests (1), topology command (3/3), `markdownlint`, `nixie`,
      `feature-matrix`, `check-topology`, `mbake`, and `git diff --check`.
      The ignored command-boundary suite is the only outstanding gate: each of
      its 18 serial cases cold-builds a private Cargo fixture, and shared-host
      admission made a single `make test` invocation exceed the harness's
      two-hour cap after three passes. Re-ran that one target detached on the
      committed tree; all 18 cases passed (5781.7 s, exit 0), so every local
      gate is green on the published head. Log
      `/tmp/test-statelet-1-2-2-feature-matrix-command-5.out`.
- [x] (2026-10-31) CodeRabbit's first formal review of PR #152 raised three
      actionable findings: the layout document omitted the `.codescene/`
      directory this branch adds; the discovery recipe pinned `LC_COLLATE=C`,
      which is void whenever a caller exports `LC_ALL`; and the README's
      `make all` examples did not provision `cargo-hack`. Repaired all three,
      restated the plan header status as `COMPLETE`, and re-ran the affected
      gates. The review's two additional (unverified) comments were
      dispositioned: the compound-assertion split was declined because the
      repository configures no Ruff gate and the other contracts in that file
      use the same single-message form.

## Surprises & discoveries

cargo-hack's all-feature pass is conditional; its documented `--no-dev-deps`
option rewrites actual manifests while running. These findings motivated the
explicit final row, avoidance of rewriting, and mandatory isolation experiment.
The existing Make defaults would contaminate isolated feature checks if reused.
The pinned 0.6.45 fixture run selected no-default, default, and each explicit
feature row but omitted the fixture's implicit optional dependency feature. The
tagged upstream README documents `--optional-deps` for including those
dependencies; the user accepted the correction on 2026-10-28. The V3 healthy
fixture also showed that cargo-hack runs `cargo check --lib` only once when no
features are declared. An explicit no-default invocation is therefore necessary
to exercise both requested modes on today's crate.

## Decision log

2026-10-25: Propose pinned cargo-hack rather than a custom feature parser.
Automatic discovery reduces repository-owned policy and covers future features.
Use `--exclude-all-features` only to delegate that row to the explicit final
command; retain both baseline modes. The user approved these decisions on
2026-10-25; this entry records design intent, not implementation evidence.

2026-10-25: The expert panel raised default contamination, conditional all-row
coverage, development-dependency unification, tool mismatch diagnostics,
manifest pinning, and nested build isolation. V1–V4 and both milestones now
address these concerns. The trade-off is a small external tooling dependency
instead of a bespoke discovery implementation. No public abstraction is added.

2026-10-25: The user explicitly approved implementation of this plan. The
selected implementation is the pinned cargo-hack approach above, with the
explicit all-features invocation, the library-isolation go/no-go experiment,
and no runtime API or placeholder features. EP-M1 may begin within the listed
tolerances; deviations still require a recorded proposal and approval.

2026-10-25: Before implementation, a real cargo-hack 0.6.45 fixture run showed
that `--each-feature --exclude-all-features` omits implicit optional dependency
features. Firecrawl's retrieval of the [tagged cargo-hack README] confirms
`--optional-deps` includes them. Proposed correction: add `--optional-deps` to
the isolated-row command and retain the V1 optional dependency witness. This is
a plan deviation, which was blocked pending user acceptance at that time.

2026-10-28: The user approved adding `--optional-deps` to the isolated-row
command. Resume EP-M1 under the existing verification and isolation gates.

2026-10-28: The feature-free V3 fixture showed that cargo-hack does not invoke
the no-default mode separately when no features exist. Add an explicit
no-default library check before cargo-hack, preserving the already approved
configuration contract without a feature discovery parser. V1 must witness this
command even for the feature-free fixture.

2026-10-28: The pinned cargo-hack source also shows that a package with
features but no declared `default` feature receives no separate default-mode
invocation. Add an explicit default library check to satisfy the same approved
mode contract for every fixture, without introducing a feature discovery parser.

2026-10-28: The milestone review identified that Act CI reaches `make test` and
therefore needs cargo-hack before that gate. Provision it there and guard the
order with negative controls. Classify the installer as provisioning rather
than a suite, clean up owned test roots, and use `camino` paths with capability
directories. The review's two direct `make all` gate findings describe the
already scheduled EP-M2 wiring and are not an EP-M1 regression.

2026-10-28: A cleanup test that caught a deliberate unwind failed under the
repository's test build; a joined-thread variant aborted. Fixture tests now
capture results, drop the owned root, and then assert, so expected assertion
failures occur after cleanup. The harness also attempts cleanup on drop during
unwinding when the toolchain supports it.

2026-10-30: The final gate run failed two workflow-contract cases that assert
the Python discovery inventory is sorted. The shared recipe used `find | sort`
without pinning collation, and GNU `sort` under `en_US.UTF-8` ignores leading
punctuation, placing `.github` after the benchmark roots. The test compares
against Python's `sorted`, which uses code-point order. Neither the recipe nor
the test is changed by this branch, and the failing module passed with
`LC_ALL=C`, so this is a pre-existing host-locale defect rather than a feature
matrix regression. Pin `LC_ALL=C` on the discovery sort. `LC_ALL` rather than
`LC_COLLATE` is deliberate: `LC_ALL` outranks every `LC_*` category, so a
`LC_COLLATE`-only pin is void whenever the caller exports `LC_ALL`, which
containers and CI images commonly do. The repair is a separate atomic commit
because it is outside the feature-matrix product scope; it makes the existing
gate reproducible on non-C locales and does not change any matrix row, feature
default, or runtime API. The discovery contract test matched the literal
`| sort` text, so it now requires the locale-pinned form; a bare sort is the
locale-dependent shape the behavioural test rejects.

## Outcomes & retrospective

The user approved the plan and the `--optional-deps` correction. The matrix
command, command-boundary tests, and routine local and CI wiring are complete.
Runtime APIs and feature defaults remain unchanged. The final implementation
gates passed: formatting, lint, typecheck, tests, Markdown, diagrams, spelling,
the feature matrix, topology, and Makefile validation. The test gate included
515 workflow-contract tests and five subtests, 18 matrix boundary cases, three
topology cases, 218 ordinary Rust tests, and one doctest. Darwin and
cross-target linker cases use Make overrides rather than a native Darwin run.

EP-M2 review identified and prompted fixes to the ordering negative control,
documentation, and platform-specific linker assertion. The final reassessment
proposed replacing the pinned shared installer with `cargo-binstall`, but the
existing exact-version installer already meets the approved contract and keeps
local and CI provisioning consistent. Three further suggestions to split test
helpers add indirection to bounded checks without addressing a demonstrated
defect; the existing focused mutation controls exercise those checks. These
advisory suggestions were therefore not adopted.

The first hosted CI and Act runs failed because the parser did not recognize
the cargo-hack output. Forced colour exposed one parsing gap, but a second
hosted failure showed that `GITHUB_ACTIONS` also changes the status format to
group markers. Focused reproductions failed before each repair and passed
afterwards, including a real fixture requiring a group-form feature row.
CodeScene also identified complexity in the Python contract; extraction reduced
its local score to 10.0 without suppressions. Its string-heavy findings on Rust
command fixtures refer to textual arguments intrinsic to those tests;
introducing wrappers solely to change that metric would obscure their purpose.

The user subsequently authorized CodeScene suppression. The remaining
module-level string-argument findings are disabled only for the two affected
test files in `.codescene/code-health-rules.json`; the rule remains active
elsewhere. These tests intentionally inspect textual Cargo arguments and
output. Local CodeScene 1.0.52 reports both files at 10.0 with no findings
under the scoped configuration. Hosted validation remains the authoritative
check.

Planning validation passed on 2026-10-25: `make fmt`, `make check-fmt`,
`make lint`, `make typecheck`, `make test`, `make markdownlint` (including
spelling), and `make nixie`. Tests passed with 206 nextest cases, one doctest,
432 workflow tests plus five subtests, and three topology command cases. Three
command-boundary cases were intentionally skipped by the ordinary nextest run
and executed by their dedicated gate. Logs are stored under
`/tmp/{action}-statelet-1-2-2-feature-matrix-command.out`. These results
validated the planning candidate and the pre-existing repository, and the final
validation recorded below superseded them once the feature matrix was
implemented.

Final validation on the published candidate passed every applicable gate.
`make fmt`, `make check-fmt`, `make lint`, `make typecheck`,
`make markdownlint` (including spelling), `make nixie`, `make feature-matrix`,
`make check-topology`, `mbake validate Makefile`, and `git diff --check` were
clean. `make test` passed 218 nextest cases, one doctest, 515 workflow-contract
tests plus five subtests, and three topology cases; its ignored
command-boundary suite was re-run on its own and all 18 cases passed. The run
also exposed and repaired a pre-existing host-locale dependency in the shared
Python discovery recipe, committed separately as `f930e22` because it is
outside the feature-matrix product scope.

## Research and revision notes

Firecrawl checked the upstream [cargo-hack documentation][hack] and
[command help][help] on 2026-10-25. The installed version reported
`cargo-hack 0.6.45`; EP-M1 must verify its actual fixture behaviour rather than
assuming newer upstream help proves the pinned binary's contract.

[hack]: https://github.com/taiki-e/cargo-hack
[help]: https://github.com/taiki-e/cargo-hack/blob/main/tests/long-help.txt
[tagged cargo-hack README]: https://github.com/taiki-e/cargo-hack/blob/v0.6.45/README.md

Revision 1 (2026-10-25): Created the approval-gated plan from repository
evidence, Wyvern reconnaissance, tool research, and the initial community
review. The remaining work is final plan validation/publication, approval, then
EP-M1/EP-M2.

Revision 2 (2026-10-25): Final expert review confirmed the design and corrected
the validation block to stop on failed pipelines. Added a path-quoting fixture.
Implementation remains pending approval.

Revision 3 (2026-10-25): Recorded successful sequential planning gates and test
counts. Implementation approval and both implementation milestones remained
open.

Revision 4 (2026-10-25): Recorded the user's explicit approval and selected
implementation decisions before beginning EP-M1.

Revision 5 (2026-10-25): Recorded the baseline gate results and a pinned-tool
row-selection mismatch before implementation. Proposed `--optional-deps` to
meet the existing implicit optional-dependency test obligation; implementation
is blocked pending explicit user acceptance.

Revision 6 (2026-10-28): Recorded the user's approval of `--optional-deps`
before resuming implementation.

Revision 7 (2026-10-28): Recorded the red command test and successful V3
isolation experiment. Added the explicit no-default check required when
cargo-hack encounters a feature-free package.

Revision 8 (2026-10-28): Recorded the passing focused command cases and the
topology serialization contract repair. Full EP-M1 gates remain pending.

Revision 9 (2026-10-28): Recorded the passing EP-M1 gates, milestone review,
and the in-scope Act provisioning and fixture-harness repairs. Their validation
and review remain pending.

Revision 10 (2026-10-28): Recorded the reassessment and its three bounded
refactoring findings. The refactored candidate requires fresh validation.

Revision 11 (2026-10-31): Recorded EP-M1 and EP-M2 completion, the final
validation evidence, and the review repairs. The plan status is now `COMPLETE`.
`make feature-matrix` and its 18-case command-boundary suite pass, the routine
wiring is in `make all`, CI, and Act validation, and roadmap item 1.2.2 is
ticked. The final gate set passed on the published candidate: `fmt`,
`check-fmt`, `lint`, `typecheck`, `test` (218 nextest cases, one doctest, 515
workflow-contract tests plus five subtests, three topology cases, and 18
boundary cases), `markdownlint` including spelling, `nixie`, `feature-matrix`,
`check-topology`, `mbake validate Makefile`, and `git diff --check`. Review
repairs then covered the `.codescene/` layout entry, the `LC_ALL=C` discovery
pin, the README `cargo-hack` provisioning step, and the status restatement.
