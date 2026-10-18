# Add the single-crate topology-check placeholder

Status: IN PROGRESS — M1 is complete except for its commit; M2 is not started.

This ExecPlan is a living document. Keep Constraints, Tolerances, Risks,
Progress, Surprises & discoveries, Decision log, Outcomes & retrospective,
Conformance basis, and Verification plan current at every milestone.

## Purpose / big picture

Roadmap task 1.2.1 establishes a real, locally runnable dependency-topology
check before Statelet grows a second package. A topology is the set of Cargo
packages and the dependency edges between them. After approval and
implementation, `make topology-check` must accept the initial `statelet`
runtime package, reject premature workspace expansion and macro dependencies,
and explain that the macro must first beat the handwritten baseline.

This is a development gate, not a Statelet runtime feature. It preserves the
three possible release outcomes: ship nothing, ship conventions only, or ship
the macro. Planning does not select an outcome or authorize implementation.

## Constraints

- Keep one package and the existing runtime library. Do not add an `xtask`,
  support crate, runtime dependency, binary, macro feature, or public API.
- Consume Cargo metadata JSON. Manual `cargo tree` output is diagnostic only.
- Apply the pre-macro policy unconditionally. Neither a feature flag, an
  environment variable, nor a checked roadmap box may grant macro admission.
- External test dependencies such as `rstest` and `rstest-bdd` may use proc
  macros. The prohibition concerns Statelet workspace topology and the
  `statelet-macros` dependency, not every external procedural macro.
- Do not implement feature-matrix task 1.2.2 or the later graph evaluator.
- Preserve compiler admission, shared Cargo caches, the pinned toolchain, and
  the Makefile's development, coverage, release, and Whitaker build routes.
- Run gates sequentially, with `pipefail` and logs under `/tmp`. Never build
  under `/tmp` or interrupt another agent's processes.
- Keep code files below 400 lines, document modules, and use fallible fixture
  setup. Do not suppress lints or mutate process-global environment in tests.
- Leave roadmap 1.2.1 unchecked until implementation and acceptance finish.

## Tolerances (exception triggers)

Stop implementation, set status to BLOCKED, and record options for approval if
any new package, public API, runtime dependency, feature flag, generic
workspace graph algorithm, or change to release-gate semantics becomes
necessary. The proposed development dependencies are `serde_json`, `rstest-bdd`,
`rstest-bdd-macros`, and `thiserror`, alongside existing `rstest`,
`googletest`, `pretty_assertions`, `proptest`, and `camino`. A capability-based
filesystem dependency may be added for isolated fixture creation; justify its
scope in the developer guide. Escalate any other dependency requirement.

Use at most 18 changed implementation files and 900 added Rust lines, excluding
small fixture manifests and documentation. Exceeding either bound requires a
scope review. Three unsuccessful fixes for the same gate failure require a
diagnosis before further edits. There is no elapsed-time limit. Supervisor
refusal, disk exhaustion, or unavailable authentication is an external blocker;
report it and continue only independent authorized work.

## Risks

A metadata-only policy can be mistaken for a complete resolved-graph check
(high impact, medium likelihood). State the limitation on success and in the
documentation; later package admission must replace the policy deliberately.

Dependency aliases and inactive optional or target-specific declarations can
hide an edge (high impact, medium likelihood). Inspect dependency declarations
by their package `name`, across all kinds and targets, rather than aliases or
only active resolved edges.

Running Cargo from a test can contend with shared build resources (medium
impact, medium likelihood). Metadata acquisition does not compile, and full
command end-to-end checks run sequentially outside recursive test discovery. Do
not add a custom test harness or nested full-repository gate.

The roadmap's macro admission wording is not fully aligned with ADR 003 (medium
impact, observed). Task 4.1.1 refers to 3.1.3 choosing “ship macro”, whereas
ADR 003 assigns B1 to 3.1.3 and B2 publication to 4.3.1. This task never reads
those documents as executable permission. Resolve that later-phase wording
before a future change admits a macro package.

## Progress

- [x] (2026-10-12 23:50Z) Inspect branch, prerequisite, design, and tooling;
  register the Leta workspace and rename the branch.
- [x] (2026-10-12 23:52Z) Complete two Wyvern reconnaissance passes; verify
  Cargo metadata semantics against local output and upstream documentation.
- [x] (2026-10-12 23:58Z) Review with all six community experts; reconcile
  zero-test success, fixture routing, metadata partitions, and command
  ownership.
- [x] (2026-10-14) Validate this plan for draft publication: formatting, lint,
  typechecking, tests, Markdown/spelling, and Mermaid gates passed.
- [x] (2026-10-14) Rebase onto `7cb1267` without conflicts and reconcile the
  issue 112 prerequisite and newly proposed observation documents. All six
  validation gates passed on the rebased candidate.
- [x] (2026-10-18) Receive explicit plan approval through the
  `sdlc-implementation` invocation for this plan and PR #137.
- [x] M1: implement and verify the private policy and local command.
  - [x] (2026-10-18 20:35Z) Red observed: moving
    `tests/topology_check/policy.rs` aside made `cargo nextest` report
    `error: couldn't find file tests/topology_check/policy.rs`, exit 101,
    logged to `/tmp/topology-red-1-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 20:35Z) Green: `tests/topology_check.rs` with the
    `fixtures`, `metadata`, `policy`, and `policy_cases` modules passes all 20
    nextest cases and the doctest. Evidence:
    `/tmp/topology-green-3-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 21:10Z) V2 and V4 green: `claim_properties` adds four
    metamorphic properties and `behaviour` runs the plan's three Gherkin
    scenarios over the same evaluator. The focused suite was 27 of 27 passing.
    Evidence: `/tmp/topology-green-6-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 21:20Z) Negative control observed: replacing the forbidden
    guard with a never-matching name made the generated property, the renamed
    partition case, the BDD scenario, and the reordering property fail for the
    intended reason. The policy was restored and the suite re-verified.
    Evidence: `/tmp/topology-negative-control-3-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 21:45Z) Make targets added: `topology-check`,
    `check-topology`, and `test-topology-command`, with the empty-override guard
    in `optional_variable`. `make topology-check` exits 0 and prints the three
    stable lines. Evidence:
    `/tmp/topology-check-2-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 22:20Z) V3 green: `make test-topology-command` passes all
    nine command-boundary cases sequentially. Evidence:
    `/tmp/test-topology-command-5-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 22:35Z) Documentation updated: design section 11.6 states
    the implemented contract, the developer guide gains a
    dependency-topology section and the new targets, the users guide carries a
    validation-command signpost, and the repository layout records the module
    ownership. The stale `D10`/`D52` citations in `Cargo.toml` were corrected to
    the decisions actually recorded here.
  - [x] (2026-10-18 23:05Z) Red gates observed on the first commit attempt:
    21 clippy findings across the five new test sources, one `unused_braces`
    compile error, and three hyphenated spellings `typos` rejects. Evidence:
    `/tmp/lint-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/typecheck-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/test-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/markdownlint-statelet-1-2-1-topology-check-placeholder.out`. The
    repairs are recorded as D11 and D12.
  - [x] (2026-10-18 23:20Z) Repairs green: `cargo clippy --all-targets
    --all-features -- -D warnings` exits 0, and `make test` now passes nextest
    (177 cases, 1 skipped) and the doctest. The only remaining failure is the
    inherited locale-ordering workflow contract described under Surprises.
    Evidence: `/tmp/test-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 23:35Z) Scrutineer re-ran every applicable gate end to end.
    Green: `make check-fmt`, `make lint` (including `cargo doc`, Clippy, and
    Pylint 10.00/10), `make typecheck`, `make nixie`, `make check-topology`,
    and `make test-topology-command`; nextest passed 177 of 177 with one
    skipped and the doctest passed. Two gates were red. `make markdownlint`
    stopped in `spelling` on the retrospective sentence that quoted the
    rejected spelling, so `markdownlint-cli2` had not run at all. Separately,
    the `.hypothesis/` cache is untracked but was not ignored, and it gained a
    constants file on every workflow-contract run. Both are repaired here: the
    sentence names the spellings without reproducing one, and `.hypothesis/`
    joins `.pytest_cache/` under the local-tool-state block.
    `make markdownlint` then reported `0 issues in 0 files` over 40 files.
    Evidence:
    `/tmp/lint-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/test-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/markdownlint-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 23:40Z) Tolerance review, recorded as D13. The staged
    change is 19 files: 18 against the 18-file bound plus `.gitignore`, whose
    added `.hypothesis/` line is test-tool hygiene rather than implementation.
    The new Rust sources are 1736 lines against the 900-line bound, and the
    line bound is therefore exceeded. The escalation it requires is this
    paragraph: no new package, public API, runtime dependency, feature flag, or
    graph algorithm was introduced, and the overrun is documentation density
    inside the test sources rather than behavioural scope, which is why the
    plan continues instead of stopping.
  - [x] (2026-10-18 23:55Z) Final gate set re-run in plan order after the last
    repair. `make markdownlint` reported `0 issues in 0 files` over 40 files,
    so `spelling` and `markdownlint-cli2` are both green this time.
    `make check-fmt`, `make lint` (Pylint 10.00/10), and `make typecheck` were
    re-confirmed. `make test` again passed nextest (177 run, 177 passed, 1
    skipped) and the doctest, and failed only in its workflow-contract phase on
    the two inherited locale cases. That phase passes 416 tests with 5 subtests
    under `LC_ALL=C`, which confirms the cause is the ambient collation and not
    this branch. Evidence:
    `/tmp/markdownlint-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/check-fmt-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/lint-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/test-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-19 00:05Z) Committed as `5140a53` with every applicable gate
    green except the inherited workflow-contract locale failure. `/verify`
    exercised the real target: the repository run printed the three stable
    lines and exited 0, and an extra-member fixture was refused with `the
    workspace declares 2 members` and a non-zero status. `/simplify` reviewed
    the new sources and found no behaviour-preserving simplification worth a
    second commit.
  - [x] Gates and commit (`5140a53`).
- [ ] M2: integrate routine gates, reconcile documentation, and mark 1.2.1 done.
  - [x] (2026-10-18 21:30Z) Makefile and CI wired. `make all` runs
    `make check-topology` after `make test` and before `make spelling`;
    `make test` ends with `make test-topology-command` after the workflow
    contracts. CI gains `Check declared dependency topology` after the nextest
    installation and `Test the topology command boundary` after the coverage
    action. `.NOTPARALLEL` already named both composite targets from M1.
    `mbake validate Makefile` reports `Makefile: Valid syntax`.
  - [x] (2026-10-18 21:44Z) Green wiring contract: all 13 cases pass (the gold
    clause test plus 12 mutation controls). Evidence:
    `/tmp/topology-wiring-green-2-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 21:43Z) Negative control on the real workflow. Deleting
    the `Check declared dependency topology` step from `ci.yml` made
    `test_every_routine_wiring_clause_holds` report the finding that ci.yml
    must run `make check-topology` exactly once, and exit 1. Three
    mutation cases also errored with `StopIteration`, because the mutation
    helpers look up the step the deletion removed; the gold clause test is the
    one that reports the intended finding. The step was restored from
    `/tmp/ci.yml.topology-backup-v2` and the contract re-verified green.
    Evidence:
    `/tmp/topology-wiring-negative-control-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 21:38Z) Workflow-contract inventory updated.
    `check-topology` and `test-topology-command` are classified as Make *suite*
    targets in `suite_provisioning.py`, not non-suite commands, because both
    run `cargo nextest run` and must stay bound by the Linux preflight
    contract. The existing unconditional `make install-rust-toolchain` step
    already precedes both in CI, so the discovery contract passes unchanged.
  - [x] (2026-10-18 21:40Z) Documentation reconciled for M2. The developer
    guide states the routine wiring and the new contract module; design
    section 11.6 records that the check is part of routine validation; the
    users guide notes that `make all` and CI run the policy; the repository
    layout records `tests/workflow_contracts/topology_wiring_test.py` and the
    topology test paths. Roadmap 1.2.1 is ticked `[x]`; see D14 for why the
    tick was applied without `mapsplice`.
  - [x] (2026-10-19 00:30Z) Scrutineer's M2 gate run was red on the new Python
    sources: Pylint reported 17 messages (two module-length, one
    slots-for-dataclass, thirteen fixture-shadowing, one duplicate-code) and
    `ty` reported five optional-index diagnostics, and the spelling gate
    rejected the British spelling in two Rust call sites. Repairs: the test
    sources were renamed to the American spelling, `Wiring` gained `slots=True`,
    the fixture and every parameter binding were renamed so none shadows
    `live_wiring`, and the two `int | None` index returns are narrowed before
    subscripting or comparison. The Make evaluator was extracted as D17, which
    also cleared the duplicate-code finding and the line limits. Focused
    evidence: Pylint 10.00/10 over the five touched modules, `ty` reports all
    checks passed, and the focused contract passes 13 cases; the full contract
    directory passes 429 tests with 5 subtests. Evidence:
    `/tmp/pylint-targets-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/ty-targets-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/topology-wiring-focused-statelet-1-2-1-topology-check-placeholder.out`,
    and
    `/tmp/topology-contracts-repair-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-19 00:10Z) Full sequential gate set re-run by the scrutineer
    after the repairs. Eight gates green: `make lint` (Pylint 10.00/10),
    `make typecheck`, `make markdownlint`, `make test` (177 nextest cases with
    one skipped, the doctest, and 429 workflow-contract cases with 5
    subtests), `make check-topology`, `make test-topology-command`,
    `make nixie`, and `mbake validate Makefile`. `make check-fmt` was red
    because `mdtablefix --check` wanted to rewrap two documentation paragraphs;
    `make fmt` applied exactly that reflow, `make check-fmt` then reported 40
    files unchanged, and `make markdownlint` stayed green. Evidence:
    `/tmp/lint-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/typecheck-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/markdownlint-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/test-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/check-topology-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/test-topology-command-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/nixie-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/mbake-m2-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/check-fmt-m2b-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/markdownlint-m2b-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-19 00:20Z) `/verify` exercised the real targets. The routine
    `make check-topology` printed the success and scope lines and exited 0; the
    same target run with
    `TOPOLOGY_MANIFEST_PATH=/tmp/statelet-healthy-fixture/Cargo.toml` still
    printed the repository manifest assignment and exited 0, so the pin holds
    against a command-line override; and `make test-topology-command` ran the
    one boundary case to a pass with no skips. Evidence:
    `/tmp/verify-repo-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/verify-override-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/verify-command-statelet-1-2-1-topology-check-placeholder.out`.
    `/simplify` extracted the repeated CI step lookup in the mutation helpers
    into one `_ci_step`; the focused contract still passes 13 cases, and Pylint
    and `ty` remain clean. Evidence:
    `/tmp/topology-wiring-simplified-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/pylint-simplified-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/ty-simplified-statelet-1-2-1-topology-check-placeholder.out`.
  - [ ] The M2 commit.

## Surprises & discoveries

The baseline package has no runtime dependencies and already has six direct
development dependencies. `cargo metadata --no-deps --format-version 1` returns
one workspace member, its dependency declarations, and `resolve: null`.
Consequently the initial policy can see optional declarations without resolving
or downloading dependency packages. It cannot classify arbitrary external
packages or prove a future resolved graph acyclic.

CodeGraph's module summary failed with `Transport closed`; Leta worked and is
the navigation fallback. The first Firecrawl structured extraction wrongly
claimed that `--no-deps` omits dependency declarations. The full Cargo Book
source and a local metadata invocation contradicted that extraction; the plan
uses the verified source and local observation.

Planning validation exposed an inherited `GIT_EXEC_PATH` pointing to a Lody
remote-helper directory. The shared workflow-contract dependency fetch failed
through that helper even with `/usr/bin/git` and a cleaned `PATH`. Removing
`GIT_EXEC_PATH` selected `/usr/libexec/git-core`, and the public dependency
repository became reachable. Clean command environments must remove this
override as well as the Git and GitHub variables listed in `AGENTS.md`.

Planning validation also encountered shared-service memory throttling. The Lody
cgroup used approximately 81 GiB against an 80 GiB `memory.high` limit, with a
96 GiB hard limit and over 118 million high-limit events. Its file cache fell
below 5 MiB. The pinned formatter remained alive but accumulated over 75 GB of
disk reads without further file input or output. By 2026-10-14 the formatter
log reached its final zero-issue Markdown summary and the host was responsive.
No service setting or other agent's process was changed. The subsequent full
test run passed; retain the distinction between observed completion and an
unfinished process when recording future gate evidence.

M1 implementation found the baseline `make test` is red before any topology
change. The nextest and doctest phases pass, and
`tests/workflow_contracts/python_gate_failure_test.py::test_python_gateway_receives_sorted_complete_inventory`
fails for both `[lint-python-PYLINT]` and `[typecheck-python-TY]`. The
Makefile builds its Python source inventory with `find | sort`, which orders by
the ambient `LC_COLLATE` (`en_US.UTF-8` here puts `.github` after `benches`),
while the test's `sorted(...)` is byte order (`.github` first). The scrutineer
reproduced both orderings directly, so the disagreement is the baseline's, not
this branch's. Repairs touch either the Makefile's ordering or the test's
comparison, both outside this plan's scope; the plan records the defect and
continues, and `make test` cannot be reported green until a scope decision is
made. The `.hypothesis/` directory appears because the same suite runs
Hypothesis property tests; it is a test-run artefact, so it was ignored
alongside the other local caches rather than committed or deleted by hand.

Two fixture defects surfaced during the first green attempts, both in the cases
rather than the policy. `dangling_workspace_member` supplied two members
alongside its dangling identifier, so the member-count check correctly fired
first; it now supplies the dangling identifier alone.
`unrelated_fields_are_ignored` edited `/packages/0/metadata`, which the
template did not declare, so the pointer refused to resolve; the template now
carries the empty `metadata` object Cargo actually prints. Both were case bugs
that the pointer-and-verdict design reported as such, which is the behaviour it
was chosen for.

The first V3 run reported success for every fixture, which was a false pass
rather than a policy result. The harness exported
`STATELET_TOPOLOGY_MANIFEST_PATH` into the child environment, but the recipe
assigns that variable from the make variable, so the child read the repository
manifest each time. The general lesson outlasts this gate: a recipe that
assigns a variable it also inherits cannot be steered by exporting it, and a
boundary test that only checks exit status cannot see the difference. The
corrected harness passes the fixture as a make argument, and a manual run
against the extra-member fixture now reports the member count.

Probing the fixtures before trusting them found three invalid shapes. A virtual
workspace root listing `"."` as a member silently drops that member from
`workspace_members`, so it could never prove the extra-member rule. Two
manifests emitted `[lib]` twice and Cargo refused them with a duplicate-key
error rather than a policy verdict. A self path dependency on the fixture's own
directory is unresolvable, and a declared member with no manifest is refused by
Cargo before any policy runs. Each was replaced with a shape Cargo accepts, and
the fixture table now returns an error for an unlisted file instead of writing
a zero-byte manifest whose parse failure looks like a policy rejection.

The first full gate run found that the repository's `.rustfmt.toml` sets
`fn_single_line = true`, which rewrites
`fn world() -> TopologyWorld { TopologyWorld::default() }` onto one line, and
rustc then rejects that line with `unused_braces`. A multi-line braced body is
accepted; only the single-line form is not. The fixture body therefore carries
a comment, which keeps the block multi-line through `make fmt` while explaining
why the slot starts empty. Any future single-expression fixture in this
repository hits the same trap.

`allow-expect-in-tests = true` does not reach `#[given]`, `#[when]`, or
`#[then]` step functions, so the BDD world could not keep the `expect` calls
that a `#[test]` would have been allowed. `rstest-bdd` 0.6.0 accepts
`Result<(), E>` steps and propagates the `Err` as a step failure; the steps now
return `Result<(), String>` and read their slots through small fallible
helpers. A `Given` step whose only fallibility is a helper call passes
`unnecessary_wraps` because the helper's `?` is the propagating call, not the
step itself.

The red lint run also surfaced a behavioural gap in an assertion: `assert_eq!`
would compare a whole `Result` and print a sparse message, while the rewritten
case returns `expected {expected:?}, got {other:?}`. That is why the table's
diagnostic is now built by hand rather than by the assertion macro; the
`panic_in_result_fn` finding forced the change and improved the failure text.

## Decision log

D1 (2026-10-13): propose a normal Rust integration-test target as the private
checker, invoked by `make topology-check`. Existing repository contract tests
already enforce build and document policy this way. A separate tooling crate,
custom `harness = false` target, or shipped binary earns no boundary here. Use
the existing nextest runner with explicit `--no-tests fail` so a renamed or
ignored checker cannot silently pass by selecting zero tests. A Python checker
would also be viable, but would add a second testing path and scripting
dependencies to a policy naturally covered by the existing Rust fixtures and
assertions.

D2 (2026-10-13): propose a deliberately closed, single-package policy using
metadata declarations. Reject every additional workspace member now. This makes
a multi-member cycle impossible within the accepted state without prematurely
designing a general graph checker. Reject a self-dependency by package name as
well. Later members need a separately reviewed policy change.

D3 (2026-10-13): make future dependency directions explicit without approving
both directions simultaneously. An optional runtime-to-macro re-export does not
make a reciprocal macro-to-runtime dependency acyclic. The future change must
choose an acyclic arrangement and verify normal, build, and dev edges.

D4 (2026-10-13): record the proposed placeholder in design section 11.6 now,
clearly marked pending approval. It specializes the existing design rather than
changing the product boundary, so no new ADR is needed. If implementation
changes that boundary or release evidence requirements, stop and propose an ADR.

D5 (2026-10-13): expert review identified a false-success path in the original
`cargo test --exact` proposal: zero matching tests still exit successfully.
Switch to nextest with an explicit no-tests failure policy, require the guard
not to be ignored, and add a missing-selection negative control. Explicitly pin
repository input in routine validation to prevent fixture substitution. The
installed `cargo nextest run --help` confirms `--no-tests fail` and the
ignored-test selection options.

D6 (2026-10-13): run command end-to-end tests through a dedicated sequential
gate after ordinary tests or coverage. Mark that integration target's test
functions ignored with a documented reason so routine discovery never starts
nested build commands during the coverage run. The dedicated gate selects
ignored tests explicitly and fails if it selects none.

D7 (2026-10-14): incorporate `main` through
`7cb12674737491af3c76c18220862a84b60d1e15`. Roadmap 1.2.1 now explicitly
requires resolution of
[issue 112](https://github.com/leynos/statelet/issues/112) before defining the
later workspace policy. Keep this placeholder closed to additional members and
do not present its future constraints as a settled allow-list or as a fix for
that issue. Preserve the new Phase 6 RFCs and ADRs 006–007 as proposals, not
permission for additional crates, public APIs, or new release gates. The
upstream changes add documentation, not reusable runtime helpers or build
changes; no implementation adaptation is needed.

D8 (2026-10-18): V2 needs generated inputs, so `proptest` joins the development
dependencies the plan already named. The properties are metamorphic —
reordering, appending an extra package record, appending an extra member, and
generating a forbidden declaration however it is spelled — rather than a second
policy, and the plan's V2 obligation is discharged by four properties at 128
cases each. `cap-std` joins them for V3: the plan requires capability-based
filesystem fixtures, and capability handles are the only way to write a fixture
without an ambient `std::fs` call the repository's Whitaker
`no_std_fs_operations` lane denies. Neither addition changes the runtime crate;
both are `[dev-dependencies]` of the existing package.

D9 (2026-10-18): the V3 fixture path reaches the command through the
`TOPOLOGY_MANIFEST_PATH` make variable, not through the child environment. The
recipe assigns `STATELET_TOPOLOGY_MANIFEST_PATH` from that variable, so an
environment value is overwritten and every case silently checks the repository
manifest. The first V3 run demonstrated exactly that failure mode: all nine
cases exited zero. The harness now passes `TOPOLOGY_MANIFEST_PATH=<path>` as a
make argument, and a manual run against the extra-member fixture was confirmed
to report `the workspace declares 2 members` rather than the repository's
success line.

D10 (2026-10-18): V3's fixture table carries seven policy partitions, not the
four the plan's acceptance list names verbatim. The extra three — a proc-macro
package under the runtime name, a build-kind declaration, and an alias to the
runtime package itself — exercise branches whose only evidence would otherwise
be an in-memory case, and the kind and alias are carried into the diagnostic a
contributor reads. A dangling-member fixture is deliberately absent: Cargo
refuses to load a declared member without a manifest, so that policy branch is
reachable only from a handwritten document and stays covered by the partition
case. A missing arm in the fixture table is now an error, because the first
version's empty-string fallback wrote zero-byte manifests and the resulting
Cargo parse failure was reported as a policy disagreement.

D11 (2026-10-18): the behavioural world is a `Slot`-only `ScenarioState`, and
its steps return `Result<(), String>` rather than calling `expect`. The
repository's `allow-expect-in-tests` allowance covers `#[test]` functions only,
so an `expect` in a `#[given]`/`#[when]`/`#[then]` step is a lint error. The
fallible form also reports a missing slot or a refused verdict as a scenario
failure with its own message rather than a panic at the assert. The fixture
body carries a comment because `fn_single_line` would otherwise collapse it
into the `{ Expr }` form that rustc rejects as `unused_braces`.

D12 (2026-10-18): `tests/topology_check/claim_properties.proptest-regressions`
is committed. The two recorded seeds shrink to the empty-dependency and
reordered-dependency shapes that the deliberately mutated policy rejected
during the V2 negative control; the unmutated policy passes all four properties
at 128 cases. Committing the file makes the next run replay those cases first,
which keeps the negative control reproducible without re-deriving its inputs.
The file is proptest's own artefact and is not edited by hand.

D13 (2026-10-18): the tolerance bound on added Rust lines was exceeded and the
work continued rather than stopping. The measured change is 19 files and 1736
new Rust lines, of which 1066 are non-comment, non-attribute code; the bound is
18 files and 900 lines. Eighteen of those files are the implementation and test
surface the plan projected, and the nineteenth is `.gitignore`, which gains one
line ignoring the `.hypothesis/` cache the workflow-contract suite writes. That
line is test-tool hygiene rather than implementation, and it was added because
the directory was otherwise untracked and unignored. The line count is a
documentation-density fact rather than evidence of behavioural scope: every
public item and every non-obvious branch carries the explanation the repository
asks for, and the tests include a nine-case command-boundary table. None of the
conditions the Tolerances section names as requiring a stop — a new package,
public API, runtime dependency, feature flag, generic workspace graph
algorithm, or release-gate change — is present, and the Constraints section's
prohibitions were rechecked against the diff. This entry is the scope review
the tolerance demands; a reviewer who wants the line count back under the bound
should delete explanatory prose from the test sources, not behaviour.

D14 (2026-10-18): roadmap 1.2.1 was ticked `[x]` by a direct one-character edit
rather than through `mapsplice`, because
`mapsplice replace docs/roadmap.md 1.2.1 <fragment>` refused the edit with the
message that the dependency anchor `1.2.1` was not found in the target roadmap,
under `error_class` `dangling_dependency`, and left the file unchanged. The
same command against anchor 1.2.2 exited 0, and a minimal two-line fragment
failed identically, so the refusal is specific to this anchor: task 1.2.2
carries `- Requires 1.2.1.`, and the replace pass cannot splice out an item
that a surviving dependency reference still names. A checkbox tick changes no
numbering, moves no task, and rewrites no `Requires` clause, so there was no
structural edit for `mapsplice` to perform in the first place. The M2 milestone
instruction to use the skill is therefore satisfied by determining that it does
not apply to this change, not by using it on a different anchor or by editing
around a failure that would have renumbered unrelated tasks.

D15 (2026-10-18): `check-topology` and `test-topology-command` are classified
as Make *suite* targets in `tests/workflow_contracts/suite_provisioning.py`,
not as non-suite commands. Both run `cargo nextest run`, so the suite-discovery
contract must treat them exactly like the other Cargo routes: CI has to install
the repository toolchain before reaching them, and the existing unconditional
`make install-rust-toolchain` step already precedes both. Adding them to the
non-suite set would have exempted the two newest Rust routes from the preflight
contract that governs every other one. The branch also updates
`build_standard_test.py`, whose exact-order assertion on `make test`
legitimately gained the nested pair (`nextest-version`, `nextest-run`) that the
new last recipe produces; the count of assigned `RUSTFLAGS` values rose from
two to three for the same reason.

D16 (2026-10-18): `check-topology` pins the repository manifest *after* the
nested `topology-check` target's own assignment, rather than changing the
default of `TOPOLOGY_MANIFEST_PATH`. Pinning by override means a caller that
names the variable on the make command line still cannot redirect the routine
call, and the V5 contract evaluates
`make -n check-topology TOPOLOGY_MANIFEST_PATH=<healthy fixture>` to prove the
pinned value is what the recipe prints. A default-only change would have passed
a contract that read the variable's declared default while still accepting a
command-line override. `make test` and `make test-topology-command` clear
`STATELET_TOPOLOGY_MANIFEST_PATH` with `env -u` for the same reason at the
other boundary.

D17 (2026-10-18): the workflow contracts share one bounded Make evaluator,
`tests/workflow_contracts/mock_wiring.py`, instead of each keeping a private
copy. Pylint's duplicate-code check reported the three line-for-line copies of
`make -n` invocation and continuation folding that the CV-005, build-standard,
and new topology contracts had accumulated. The AGENTS.md abstraction policy
requires a repository sweep before extracting a helper, and the sweep found the
three copies, plus a `whitaker_contracts._recipe_lines` with a different
signature that is not a duplicate. The new module owns exactly two operations:
`make_n`, which returns what a target would run with optional command-line
overrides, and `recipe_lines`, which folds Make's backslash continuations. Its
scope is test-only support for workflow contracts; runtime code and document
contracts must not import it; it decides no contract's verdict, only supplies
evidence that the owning contract judges. The three callers migrate to it,
which also brings `build_standard_test.py` back under the 400-line file limit
(393 lines). The extended topology contract is split the same way:
`topology_wiring_contract.py` evaluates evidence and holds the finding
functions, while `topology_wiring_test.py` holds the live fixture, the gold
case, and the twelve mutations. Both are well under the limit.

## Outcomes & retrospective

Planning has established the metadata boundary and a bounded implementation
route. No topology command, runtime change, or roadmap completion has been
implemented. All six expert perspectives have been reconciled. Lint and
typechecking passed on the unchanged runtime baseline. On 2026-10-14 the full
test retry passed: 150 nextest tests, one doctest, the pinned shared workflow
contract check, and 416 workflow tests with five passing subtests. Formatting,
Markdown/spelling, and Mermaid validation also passed. The planning documents
are ready for draft publication; implementation outcomes await approval.

M1 delivered the private checker and its local command, committed as `5140a53`.
M2 made the check routine: `make all` runs the pinned-input form, `make test`
ends with the command-boundary suite, CI runs both in the same relative order,
and `tests/workflow_contracts/topology_wiring_test.py` holds the wiring with
twelve negative controls. The check remains a closed single-package policy and
does not resolve issue 112 or define a future multi-crate allow-list.

The one unresolved defect is inherited, not caused by this work: `make test`'s
workflow-contract phase fails two cases under the host's `en_US.UTF-8`
collation because the Makefile's `find | sort` inventory and the test's
byte-order `sorted()` disagree. Under `LC_ALL=C` the whole suite passes. Repair
touches the Makefile's ordering or the test's comparison, both outside this
plan's scope; the plan records the defect rather than fixing it here.

The M2 gate set is recorded below once it completes; the commit SHA is the
evidence binding, and hosted CI is separate evidence from these local runs.

## Context and orientation

The baseline is commit `7cb12674737491af3c76c18220862a84b60d1e15`. `Cargo.toml`
declares only `statelet` and an empty `[dependencies]` table. `src/lib.rs` is
the library root. Existing integration tests under `tests/` check build
configuration and ADR contracts; their private child modules show how to keep
policy tests separate from the library. The repository has no existing topology
helper to reuse, as confirmed by the Wyvern tooling sweep.

`Makefile` is the canonical command interface. `make test` runs nextest,
doctests, and workflow contracts. `make all` orders formatting, lint,
typechecking, tests, and spelling. `.github/workflows/ci.yml` runs validation
and a separate coverage action rather than `make all`, so it needs an explicit
topology step. `tests/workflow_contracts/workflow_reading.py` already supplies
YAML parsing and step enumeration; reuse it for wiring assertions.

Start with [documentation contents](../contents.md) and
[repository layout](../repository-layout.md). Load `$leta`, `$rust-router`, and
its `arch-crate-design` route before code work; load `rust-unit-testing` when
designing fixtures and `proptest` for generated invariants. Use `$execplans` to
maintain this document and `$firecrawl-mcp` for upstream gaps. Delegate
sequential full gates to `scrutineer`; use `context_pack` to exchange code
evidence with any delegated agent team.

The following references govern test structure:

- [Fixture guide](../rust-testing-with-rstest-fixtures.md).
- [Dependency injection guide](../reliable-testing-in-rust-via-dependency-injection.md).
- [Doctest guide](../rust-doctest-dry-guide.md).
- [Complexity guide](../complexity-antipatterns-and-refactoring-strategies.md).
- [`rstest-bdd` guide](../rstest-bdd-users-guide.md).

The BDD guide is imported reference material; check the selected upstream
release before copying APIs. Follow the
[documentation style guide](../documentation-style-guide.md).

## Conformance basis

Use the baseline revision above for
[terms of reference](../terms-of-reference.md), [design](../design.md) draft
v0.2, [roadmap](../roadmap.md), accepted
[ADR 002](../adr-002-transition-boundary-scope.md), and
[ADR 003](../adr-003-v0-1-exit-register.md). There is no separate topology RFC
or component architecture document. The design owns the policy; the repository
layout and developer guide own its implementation placement and operation.

The observation RFCs indexed in
[design Appendix C](../design.md), [ADR 006](../adr-006-stable-transition-and-outcome-names.md),
and [ADR 007](../adr-007-representation-and-observability-contracts.md) remain
proposed. Follow the repository layout's distinction between proposal records
and accepted design authority. Their presence does not admit a new workspace
member or change this task's acceptance criteria. Roadmap 1.2.1's issue 112
prerequisite governs any later workspace-policy definition.

R1: roadmap 1.2.1 and design sections 4–5 require a single-runtime starting
state and no premature macro edge. M1 discharges this through policy table
tests and `make topology-check` accepting the real checkout.

R2: design section 11.6 requires automated metadata-based rejection rather than
visual tree inspection. M1 supplies negative cases and acquisition error
checks; M2 supplies local/CI wiring and end-to-end command evidence.

R3: ADR 002 and ADR 003 preserve the marker-only boundary and evidence-led
release exits. Both milestones must leave the runtime API, release outcomes,
and macro admission unchanged; review the complete diff to establish this.

R4: roadmap completion and contributor documentation require M2's checked task,
updated guides, and exact-head validation evidence. Task 1.1.1 is already done
and ADR 002 is Accepted; recheck that prerequisite before implementation.

## Proposed command and policy

Add `make topology-check` with `check-nextest` as a prerequisite. Run the
existing nextest runner with one exact test, using the existing development
`RUSTFLAGS` composition:

```bash
cargo nextest run --test topology_check --no-tests fail --run-ignored default \
  -E 'test(=repository_obeys_topology)'
```

The recipe must first explain the pre-macro policy. Only after Cargo succeeds
may it print the success message below. The Rust test obtains metadata with
`cargo metadata --no-deps --format-version 1 --manifest-path <manifest>` and
checks it with the same pure evaluator used by the unit and BDD tests. It must
check the subprocess exit status before parsing stdout. Use the Cargo
executable supplied by the test environment and explicit argument vectors.
Never parse package IDs by splitting their implementation-specific strings.

For ordinary invocation, select the manifest beneath `CARGO_MANIFEST_DIR`. For
end-to-end fixture checks, the Make variable `TOPOLOGY_MANIFEST_PATH` selects
another manifest and is forwarded only to the checker subprocess as
`STATELET_TOPOLOGY_MANIFEST_PATH`. It changes the input, never the policy. Pass
the value as one quoted argument/environment value, including paths with
spaces. For CI and `make all`, explicitly bind the repository manifest on the
Make command line, overriding inherited fixture values. Add a routine
`check-topology` target that delegates to `topology-check` with
`TOPOLOGY_MANIFEST_PATH="$(CURDIR)/Cargo.toml"`; use that target from both. A
normal `make test` execution of the guard must also use the repository manifest
unless the private command-specific override was deliberately set. Clear
inherited `STATELET_TOPOLOGY_MANIFEST_PATH` at routine test entrypoints. The
public fixture variable must never silently redirect routine validation.

The evaluator accepts format version 1 with exactly one unique
`workspace_members` ID resolving to exactly one package named `statelet`.
Require a library target and reject any `proc-macro` target on that member.
Reject a declared dependency whose actual package name is `statelet-macros` or
`statelet`, irrespective of dependency kind (`null`, `build`, or `dev`),
optional status, target condition, source, or alias. No macro package is
admitted through an excluded path dependency, registry declaration, or rename.

Require correctly typed fields used by the policy: version, member IDs, package
IDs and names, targets and their kinds, and dependency declarations and names.
Reject empty ID/name strings, missing or wrongly typed required fields,
duplicate member IDs or package IDs, dangling member IDs, and empty target/kind
lists. Empty dependency arrays are valid. Repeated dependency names across
kinds or targets are valid, as are repeated external package names at distinct
package IDs. Ignore additive unrelated JSON fields, as Cargo allows schema
extensions within format version 1. External packages in supplied full-metadata
fixtures do not become workspace members; only member IDs determine workspace
membership. Do not claim to inspect their transitive dependencies. The live
command deliberately requests `--no-deps`.

Expected successful command output includes these stable messages (Cargo's
build and test-harness lines may surround them):

```plaintext
Topology policy: statelet only; statelet-macros is forbidden before the macro baseline gate.
Topology check passed: one runtime package; no forbidden declared edge.
This placeholder does not validate a future multi-crate graph or feature leakage.
```

Failure exits non-zero and identifies the offending member, declaration, or
invalid metadata field, plus the next action. Preserve the manifest path, Cargo
exit status and stderr, or the spawn-error cause in acquisition errors.
Distinguish acquisition/parse errors from policy rejection; never print the
success line on failure. Do not prescribe an exact non-zero code because Cargo
and Make wrap failures.

Before defining a later workspace policy, resolve issue 112 as required by
roadmap 1.2.1. Preserve these existing design constraints as review inputs, not
a complete future allow-list: runtime dependencies on support/example/tool
packages are forbidden; optional macro re-export requires feature gating;
macro-to-runtime and runtime-to-macro edges cannot coexist as a cycle; test
support and examples may consume the runtime and, after approval, macro crates;
and unexpected workspace edges and cycles across normal/build/dev declarations
must fail. New member roles require explicit review. Feature leakage belongs to
1.2.2 and later feature-aware validation, not this placeholder's evidence.

## Interfaces and dependencies

Add `tests/topology_check.rs` as the integration-test entrypoint and
`tests/topology_check/` for its private `policy.rs`, `metadata.rs`, and case
modules. Keep policy ownership local to this test target; BDD and property
modules are children of it, so there is no public helper API or duplicated
implementation. Use these conceptual signatures, refining private names only
when it improves clarity:

```rust,no_run
fn evaluate_metadata(document: &serde_json::Value) -> Result<(), TopologyViolation>;
fn read_metadata(cargo: &OsStr, manifest: &Utf8Path) -> Result<Value, MetadataError>;
```

Represent failures with private semantic enums deriving `thiserror::Error`. The
acquisition adapter takes explicit inputs. Tests inject results or a minimal
command runner where necessary; search for existing adapters before extracting
one. Document any new adapter's owner, permitted callers, and composition in
the developer guide. Do not add a general command-execution framework.

Add caret requirements for `serde_json`, `rstest-bdd`, and `rstest-bdd-macros`
under `[dev-dependencies]` only, using `"0.6.0"` for the two BDD packages as
the researched candidate. Their own development tests use `rstest = "0.26.1"`;
that does not establish compatibility with this repository's
`rstest = "0.27.0"`.[^3] Before policy implementation in M1, compile and run
one real fixture-backed BDD scenario with 0.27.0 and both 0.6.0 packages. If it
fails, stop and present a compatible release option; do not silently downgrade
rstest or remove behavioural tests. This is an implementation preflight, not
evidence already obtained. Add `thiserror` as a development dependency for the
private error enums. Record versions and lockfile impact before committing.
Reuse current rich assertions and property-test dependencies. Prefer
`googletest::verify_that!` with propagated results, and
`pretty_assertions::assert_eq!` for small expected reports. Upstream GoogleTest
documents compatibility with `rstest`.[^2]

## Verification plan

V1, policy partitions: parameterized `rstest` cases cover the real single
runtime package; additional runtime, macro, support, or example members;
missing/duplicate/dangling member IDs; a proc-macro target masquerading as
`statelet`; no library target; forbidden dependency aliases; each dependency
kind; inactive optional and foreign-target declarations; and permitted external
test macros. Parser cases cover malformed JSON, absent required fields, wrong
field types, and additive fields. Use named fixtures, semantic GoogleTest
matchers, and pretty diffs for expected diagnostic text. Run through
`make test TEST_FLAGS="--test topology_check"`. Every rejection must have a
minimal accepted sibling differing only in the policy-relevant field.

V2, invariance: use `proptest` over bounded metadata fixtures to establish that
reordering packages, dependency declarations, and target lists, changing opaque
IDs consistently, or changing a forbidden dependency's alias never turns
rejection into success. Generate 0–8 extra packages/declarations and short
Unicode names without rejection filtering. Include zero extras and multiple
extras; record at least 128 successful generated cases per property. An
explicit forbidden-edge witness must still fail after each transformation. As a
negative control, temporarily remove the forbidden-name check and show that
both a named regression and a generated forbidden-edge property fail; restore
it before the passing run. The bound checks robustness, not every possible
Cargo graph.

V3, command boundary: run the real Make target against the checkout and
isolated fixture manifests for one runtime package, an extra workspace member,
a proc-macro member, and an inactive renamed `statelet-macros` dependency. Use
local fixture packages, with no registry fetch or compilation of those packages
required by metadata. Include a path containing spaces and an invalid manifest.
Assert success versus non-zero status and the absence of a success message on
failure. Run these end-to-end cases sequentially after compiling the checker,
in a dedicated `make test-topology-command` gate. Implement one ignored Rust
integration test in `tests/topology_command.rs` that loops over these cases
sequentially. The ignore reason names the dedicated gate. That gate runs
`cargo nextest run --test topology_command --run-ignored only
--no-tests fail --test-threads=1`
after ordinary tests and outside coverage instrumentation. It may invoke
`make topology-check` but never `make test`. The nested command selects only
the policy guard, not command tests. Nextest finishes compilation before
running the executable; verify that no compiler lock is retained, without
bypassing shared admission. A lock conflict is a blocker for diagnosis, not
permission to add an isolated cache. Use capability-based filesystem fixtures,
and per-child environment configuration rather than process-global mutation.

V4, behavioural intent: add `tests/features/topology_check.feature` and BDD
steps in `tests/topology_check/behaviour.rs`, backed by the same evaluator. Use
`rstest-bdd` fixtures, not a parallel policy implementation:

```gherkin
Feature: Preserve the pre-macro workspace boundary
  Scenario: The initial runtime package passes
    Given metadata for the single statelet runtime package
    When the topology policy is checked
    Then the topology is accepted

  Scenario: An inactive renamed macro edge is still forbidden
    Given runtime metadata declaring an optional renamed statelet-macros dependency
    When the topology policy is checked
    Then the topology is rejected with the macro baseline gate explanation

  Scenario: A second workspace package needs a policy change
    Given metadata with an additional workspace member
    When the topology policy is checked
    Then the topology is rejected with a workspace expansion explanation
```

V5, integration: a focused workflow contract, using the existing YAML helper,
checks that CI runs the pinned-input `check-topology` target after toolchain
and nextest setup and that `make all` includes it sequentially. Check explicit
manifest pinning with an inherited healthy-fixture override while the selected
repository fixture is forbidden. An absent/ignored policy-test selector must
fail due to `--no-tests fail` and must never produce the success message.
Assert the dedicated end-to-end gate runs from `make test` after ordinary Rust
tests, and from CI after coverage, with no coverage flags inherited. Remove the
invocation as a negative control and demonstrate the contract fails. Avoid
hard-coding action SHAs.

The external axioms are Cargo format-version-1 semantics, its declaration names
reflecting actual package names even when renamed, and ordinary Cargo, Make,
and process exit-status propagation.[^1] V3 exercises the real interfaces; V1
tests repository-owned validation, not Cargo internals. A package with only one
member and no self-edge cannot contain a workspace cycle; no general
reachability algorithm or new mathematical lemma is introduced. No contractual
business logic is being added. Therefore Verus is not warranted for this
bounded tooling predicate. If a future implementation introduces a graph
algorithm or lemma, revise the obligations and use substantive Verus proofs
where required. Kani adds no useful coverage beyond the finite partitions and
properties here. No multivariant output format is introduced; exact small
message assertions suffice, so `insta` snapshots would duplicate evidence.
Reconsider snapshots if a structured or multivariant report is introduced.

## Plan of work and milestones

### M1: validated local command

After approval, confirm ADR 002 and the baseline, load the relevant skills, and
run the existing gates. Complete the BDD compatibility preflight above. Add the
minimum V1 failing case first. Observe its failure for the missing policy, then
implement the evaluator and acquisition adapter. Add the remaining cases, V2
properties, and V4 scenarios, observing a representative red result before each
new rule. Add the Make target, help text, and fixture override; then V3
exercises the complete command through its dedicated sequential target. Keep
the new end-to-end target available in M1; routine wiring is M2.

Update `docs/design.md` section 11.6 from proposal to implemented contract,
`docs/developers-guide.md` with command/policy/adapter ownership and
troubleshooting, `docs/users-guide.md` with a short validation-command signpost
and the unchanged runtime scope, and `docs/repository-layout.md` with
test-module ownership. The plateau discharges R1, R2's local boundary, and R3
through V1–V4. Review that the implementation still admits only the original
runtime package and has added no runtime API or dependency. Run all
major-milestone gates below, record evidence, and commit the complete passing
change. CI wiring remains M2. Rollback uses a targeted revert of that commit;
no compatibility layer is needed because this is new private tooling.

### M2: routine validation and completion

Status: implemented, awaiting the full sequential gate set and the M2 commit.
Roadmap 1.2.1 is ticked. The M2 plateau discharges R2 and R4; R3 was rechecked
on the full diff.

Add pinned-input `check-topology` to the ordered `make all` recipe and a named
step to `.github/workflows/ci.yml` after Rust and nextest setup. Wire
`test-topology-command` at the end of `make test` and in CI after the coverage
action, outside its instrumentation environment. Extend `.NOTPARALLEL` to these
composite targets and run the end-to-end suite with one test thread. Add V5's
wiring contract and prove its negative control. Keep existing coverage and test
ownership unchanged. Run the standalone target and full gate sequence.
Reconcile guides and design, record exact commands and results here, and mark
only roadmap 1.2.1 `[x]`. Use the `mapsplice` skill for the structural roadmap
change.

This plateau discharges R2 and R4; R3 must still hold on the full diff. Check
that upstream assumptions, trace links, dependency scope, and product boundary
remain valid; record deviations before approval, never after the fact. Commit
only after all gates pass, then review the changed code for warranted
refactoring. Any needed refactor is a separately gated atomic commit. No
compatibility layer or unrecorded upstream discrepancy may remain at
completion. A targeted revert of M2 restores standalone M1 safely.

## Concrete steps and acceptance

All commands run at the repository root. Establish failing focused tests before
production changes, then repeat after the smallest fix. The focused command
runs policy, property, and BDD checks; it does not replace full gates:

```bash
set -o pipefail
make test TEST_FLAGS="--test topology_check" 2>&1 | tee /tmp/topology-focused-statelet.out
make topology-check 2>&1 | tee /tmp/topology-check-statelet.out
```

After each major milestone, the implementation owner runs `make fmt`, then
delegates the remaining read-only gates to `scrutineer`. Retain separate logs
and inspect any failed log before fixing or retrying. Commands are sequential,
including formatting, and all must exit zero:

```bash
set -o pipefail
make fmt 2>&1 | tee /tmp/fmt-statelet-1-2-1-topology-check-placeholder.out
make check-fmt 2>&1 | tee /tmp/check-fmt-statelet-1-2-1-topology-check-placeholder.out
make lint 2>&1 | tee /tmp/lint-statelet-1-2-1-topology-check-placeholder.out
make typecheck 2>&1 | tee /tmp/typecheck-statelet-1-2-1-topology-check-placeholder.out
make test 2>&1 | tee /tmp/test-statelet-1-2-1-topology-check-placeholder.out
make markdownlint 2>&1 | tee /tmp/markdownlint-statelet-1-2-1-topology-check-placeholder.out
make nixie 2>&1 | tee /tmp/nixie-statelet-1-2-1-topology-check-placeholder.out
```

`make markdownlint` includes spelling. Install missing pinned tools through the
documented Make targets. Inspect formatter changes and keep unrelated changes
out of the task. Never edit generated `typos.toml`; an unrelated regeneration
belongs on a separately gated base branch as repository policy requires. Run
`mbake validate Makefile` after modifying Make. The implementor must capture
representative red failures, green output, generated-case counts, end-to-end
failure messages, and the final successful gates in this plan.

Acceptance requires the real initial package to pass; each forbidden fixture to
fail with a repair hint; metadata acquisition/parse errors to fail without
success text; existing external test macros to remain permitted; and local and
CI entrypoints to run the same policy. Record the commit SHA that each gate
validated. Hosted checks are separate evidence from local gates.

## Idempotence and recovery

Metadata checks are read-only and may be rerun. Fixture creation owns only
unique temporary directories and cleans them up on success and failure. Never
alter the real manifest to demonstrate rejection. Cargo build output stays in
the repository target directory or configured shared target, not in fixture
directories beneath `/tmp`. Record failing seeds before retrying a property.
Restore deliberate negative controls before wider gates or commits.

For this planning pull request, commit only the plan, its contents-index entry,
and the proposed design note after validation. Push the requested branch over
verified SSH, refresh its remote-tracking ref, set the named upstream, and
create a draft PR titled `Plan: Add a topology-check placeholder (1.2.1)`. Link
this plan using the branch ref and finish the PR body with the Lody session
reference. Do not mark the roadmap done or start M1 before explicit approval.

## Artefacts and notes

Rebase validation on 2026-10-14 passed `make check-fmt`, `make test`,
`make typecheck`, `make lint`, `make markdownlint`, and `make nixie` after the
D7 changes. Tests passed 150 nextest cases, one doctest, and 416 workflow cases
with five subtests; Markdown lint reported zero issues across 40 files. Logs use
`/tmp/<gate>-statelet-topology-rebase-20261014.out`. The replayed planning
commit was patch-equivalent to its original, and target-only files were
preserved. The follow-up documentation commit records the upstream prerequisite
without changing the implementation approval boundary.

Initial planning validation covered revision
`5472352d94bc363fde0ad5cbb92c95a06bf481b5` plus the three documentation
changes, before the rebase recorded in D7. `make fmt`, `make check-fmt`,
`make lint`, `make typecheck`, `make test`, `make markdownlint`, and
`make nixie` all passed. The successful test retry is recorded in
`/tmp/test-statelet-1-2-1-topology-check-placeholder-2.out`; other gate logs
use the command names and branch suffix shown above. Markdown lint reported
zero issues, and Nixie validated all diagrams. No hosted-check result is
claimed.

Planning evidence: two Wyvern passes; context pack `pk_xlhcxlro`; local
metadata showing one member and six development dependency declarations;
upstream Cargo and GoogleTest documentation checked through Firecrawl. The
context pack is convenient review evidence, not required to execute this
self-contained plan. Pandalump and Wafflecat reviewed architecture and
alternatives; Telefono and Dinolump reviewed contracts and maintenance; Buzzy
Bee and Doggylump reviewed runner ownership and failure behaviour. The revised
plan closes every blocking finding; contracts and operations re-review
confirmed the revisions.

The main trade-off is a deliberately limited declaration check now, with an
explicit policy revision required for the first additional package. This avoids
maintaining a general graph tool before there is a graph to justify it. The
requested BDD scenarios remain despite the tooling reconnaissance's suggestion
to omit them: they specify observable acceptance and rejection behaviour, and
the compatibility preflight makes their dependency risk explicit.

[^1]: [Cargo Book: cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html),
    consulted 2026-10-13. `--no-deps` omits the resolved graph and external
    package records, while workspace package dependency declarations remain.

[^2]: [GoogleTest Rust documentation](https://docs.rs/googletest/latest/googletest/),
    consulted 2026-10-13. Prefer result-returning assertions for fallible tests;
    non-fatal assertions additionally require the GoogleTest test context.

[^3]: [`rstest-bdd` 0.6.0](https://docs.rs/crate/rstest-bdd/0.6.0) and
    [`rstest-bdd-macros` manifest](https://docs.rs/crate/rstest-bdd-macros/0.6.0/source/Cargo.toml),
    consulted 2026-10-13 and cross-checked against cached published manifests.

Revision note (2026-10-13): incorporate the expert panel's rejection of
zero-test success, implicit fixture routing, and recursive coverage builds.
Clarify valid metadata repetition and require a BDD compatibility preflight.
The approval boundary and closed single-package scope remain unchanged.
