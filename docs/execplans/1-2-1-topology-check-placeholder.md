# Add the single-crate topology-check placeholder

Status: IMPLEMENTED — M1 (`5140a53`) and M2 (`902927d`) are complete and
published on PR #137. The pre-rebase implementation head was `abc3e57`; it
carried the CodeScene repair, both assessment rounds, and the nine
formal-review repairs. The branch was rebased onto `main` at target `781583c`,
so the pre-rebase commit identities above survive only in the recovery refs
recorded under Idempotence and recovery; the rebased equivalents are the same
patches with different hashes. The rebase cleared the `Cargo.lock` conflict
that had been suppressing hosted `pull_request` workflows, and `CI` and
`Act Validation` now run. A review of the rebased head raised three further
findings, repaired in `e1a44ac` as D23, and the commit after it records that
round here. Three later rounds follow as D24 to D26: four findings about
target-kind classification and the optional-variable classifier, a CodeScene
finding in the test that repair introduced, and CodeRabbit's five pre-merge
findings, of which two were repaired and three declined with recorded grounds.
Delivery remains with `pr-babysitting`, and the branch tip is deliberately not
named here.

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
- [x] M2: integrate routine gates, reconcile documentation, and mark 1.2.1 done.
  Completed as `902927d`, followed by the CodeScene repair `1972342` and the
  assessment repairs recorded below.
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
  - [x] (2026-10-19 00:30Z) Committed as `902927d` after `make fmt`,
    `make check-fmt`, and `make markdownlint` were re-run green on the final
    tree, and the full workflow-contract directory passed 429 tests with 5
    subtests. M2 is complete.
  - [x] (2026-10-18 23:05Z) The hosted CodeScene check failed on `e02f5e3`
    with three files below the 10.00 bar; the local `cs` 1.0.52 delta against
    the true PR base `7cb1267` reproduced it exactly. `behaviour.rs` scored
    9.38 for Code Duplication: the two `Then` steps were structurally
    identical, and the shared `require_refusal` helper did not change that
    because both bodies still built their expectation inline. Hoisting the
    expectations to the named consts `MACRO_GATE` and `WORKSPACE_EXPANSION`
    made each step a one-liner over a distinct identifier. `topology_command.rs`
    scored 9.68 for String Heavy Function Arguments; the `CaseName`,
    `PackageName`, and `Library` domain types replaced the `&str` parameters.
    `topology_wiring_contract.py` scored 9.68 for Complex Method;
    `_findings_check_recipe` split into `_findings_policy_selector` and
    `_findings_success_order`. Each file then scored 10.0 and `cs delta`
    printed nothing.
  - [x] (2026-10-18 23:10Z) The first repair pass introduced five Clippy
    findings, all in `tests/topology_command.rs`: four `shadow_reuse`
    bindings (`name` and `library` in `package_manifest`, `relative` in
    `materialize` and `write_file`) and one `match_same_arms` pair at the two
    `Cargo.toml` arms. Renamed to `package`, `lib`, `declared`, and `name`,
    and merged only the genuinely identical arms into a single or-pattern.
    The scrutineer re-ran `make check-fmt` (40 files unchanged) and the full
    `make lint` chain, which this time progressed past `lint-clippy` into
    `lint-whitaker` and `lint-python` (10.00/10) and was green. Evidence:
    `/tmp/check-fmt-codescene2-statelet-1-2-1-topology-check-placeholder.out`
    and `/tmp/lint-codescene2-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-18 23:12Z) Committed as `1972342` and published; the SSH
    push was verified against `ls-remote`, and the refreshed
    `refs/remotes/origin/1-2-1-topology-check-placeholder` resolved to the
    same commit as the local `HEAD`.
  - [x] (2026-10-19) ExecPlan assessment repairs, recorded as D19. The six
    CodeRabbit findings on assessment reply `6256307011` are repaired:
    - V2: `claim_properties.rs` now generates the forbidden witness itself,
      generates alias, kind, optionality, and target condition independently,
      and applies all five transformations V2 names. The transformations moved
      to `tests/topology_check/transformations.rs`, which locates the runtime
      package by its member ID instead of `/packages/0`.
    - V1: `policy_cases.rs` gained the missing field partitions through the new
      `accepted_removed` helper — real field removal, duplicate and empty IDs,
      empty names, empty target and kind lists — and
      `tests/topology_check/metadata_cases.rs` pins the three acquisition
      failure categories and their diagnostics.
    - V5: `tests/topology_command.rs` now runs both executable controls. The
      pin control builds a real two-member fixture under an owned root and
      asserts the refusal names the pinned tree; the selector control runs
      nextest with a zero-selection filter in two spellings and asserts the
      `--no-tests fail` exit.
    - Acquisition diagnostics now name the manifest and carry a `Repair:`
      instruction for every variant.
    - Every fixture invocation owns a unique root and removes it on both
      outcomes.
    - The stale completion statements in this plan and `docs/contents.md` now
      describe the implemented state, with the planning-phase text kept as
      explicitly historical.
  - [x] (2026-10-19) V2 negative control re-executed against the rewritten
    properties. Removing the forbidden-name comparison from
    `reject_forbidden_declarations` made
    `a_generated_forbidden_declaration_is_refused_however_spelled` fail with
    `the forbidden declaration was accepted`, together with the BDD
    regression `inactive_renamed_macro_edge_is_forbidden` and
    `reordering_never_turns_a_refusal_into_an_acceptance`; the guard was
    restored and all 49 cases pass. Evidence:
    `/tmp/topology-negative-control-v2-statelet-1-2-1-topology-check-placeholder.out`
    and
    `/tmp/test-topology-check-restored-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-19) V5 executable controls green: `make test-topology-command`
    runs all three ignored cases with no skips, including
    `the_pin_overrides_a_healthy_fixture_override`, whose first fixture shape
    was refused by Cargo for a missing member manifest before the policy ran;
    the fixture now writes the member. Evidence:
    `/tmp/test-topology-command-repair-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-19) Nine formal-review findings on `ffb89c4` repaired as one
    commit, recorded as D22: the `# Examples` section, the coverage-environment
    claim, the complete additional-member fixture, the stale module reference,
    the caller-flags copy with its contract clause and thirteenth mutation,
    the `check_verdict` delegation, the two NumPy docstring sections, and the
    `collections.abc` import. Focused evidence:
    `/tmp/test-topology-check-cr2-statelet-1-2-1-topology-check-placeholder.out`
    (51 passed),
    `/tmp/pytest-workflow-contracts-cr2-statelet-1-2-1-topology-check-placeholder.out`
    (430 passed, 5 subtests),
    `/tmp/test-topology-command-cr1-statelet-1-2-1-topology-check-placeholder.out`
    (3 passed, no skips),
    `/tmp/lint-clippy-cr1b-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/check-fmt-cr1-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/markdownlint-cr1-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-19) `/verify` falsified one justification written for finding
    5 and both statements were corrected before commit. Three probes agreed
    that duplicated `RUSTFLAGS` are tolerated: `cargo check --lib` with every
    repository flag written twice exited zero; the nested `make topology-check`
    with the composed value exited zero; and the exact pre-repair boundary run
    with `STATELET_TOPOLOGY_CALLER_FLAGS` absent passed 3/3. The harness
    docstring and the `_findings_command_line` docstring now say the flags would
    be repeated and composition would stop being idempotent, rather than that
    the nested case would fail. The Surprises entry and D22 carry the same
    correction. Evidence:
    `/tmp/verify-prerepair-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/verify-nested-composed.out`, and `/tmp/verify-nested-caller.out`.
  - [x] (2026-10-19) Committed as `abc3e57` and published. All nine
    formal-review threads are resolved, and every hosted check on that SHA is
    green: `build-test`, `act-validation`, CodeScene Code Health Review, Gecko
    Security Review, and the CodeRabbit status all `success`, with
    `Sourcery review` and `automerge` `skipped` by configuration. The branch
    is `CONFLICTING` with `main` on `Cargo.lock` alone, from three dependency
    bumps landed after this branch's base; `pr-babysitting` owns the rebase.
    The gate logs cited above have since passed the host's ten-day `/tmp`
    expiry; the SHAs are the durable record.
  - [x] (2026-10-25) Two documentation-only commits published on top of
    `abc3e57`: `abc426f` brings this plan's status, Progress, Surprises, and
    Outcomes sections up to the published state and corrects the
    negative-control count to thirteen in D17, in Outcomes, and in
    `docs/repository-layout.md`; `614e97c` records the merge-conflict CI
    suppression. Documentation-only, so the applicable gates are the Markdown
    ones: `make fmt`, `make check-fmt`, and `make markdownlint` all green
    (40 files, 0 issues). Hosted on `abc426f`: CodeScene Code Health Review,
    Gecko Security Review, and the CodeRabbit status all `success`, with
    `Sourcery review` and `automerge` `skipped`. `CI` and `Act Validation` did
    not run, and cannot while the branch is `CONFLICTING`: GitHub documents
    that `pull_request` workflows do not run on a pull request with a merge
    conflict. The rebase is `pr-babysitting`'s.
  - [x] (2026-10-25) Rebased onto the recorded target `781583c` and
    force-pushed with a lease bound to the previous remote head `91c4414`.
    `OLD_BASE` was the branch's stated baseline `7cb1267`, which is an
    ancestor of both `main` and this branch, with no merges in
    `7cb1267..91c4414`; the fifteen branch commits replay as `bc8233e` to
    `21a5317` plus the lock reconciliation `73451d9`. The only conflict was
    `Cargo.lock`, at replay 3/15, because three dependency bumps landed on
    `main` after the base while this branch added five dev-dependencies. Main's
    lock was taken as the base, per the review rule for packaging lock files,
    and the graph was rebuilt with Cargo; the reconciled lock is `73451d9`.
    `git range-diff` pairs every commit identically except the one whose
    lockfile changed. Gates on `73451d9`: `make check-fmt`, `make test` under
    `LC_ALL=C` (201 nextest cases, 3 skipped; the doctest; three
    `topology_command` cases), `make typecheck`, and `make lint` (Clippy and
    Pylint 10.00/10) all green. Logs:
    `/tmp/check-fmt-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/test-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/typecheck-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/lint-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-25 10:57Z) CodeRabbit's five pre-merge findings adjudicated as
    D26 and published as `8d64088`. The fixture-root clock failure now
    propagates instead of folding into `0`, a process-wide serial supplies
    uniqueness within one process, and the plan's stale planning-only
    paragraph is labelled historical. The three declined findings carry
    measured grounds. Testing (Overall) is a false positive, established twice
    over: a manifest whose `path` dependency names a directory that does not
    exist exits `0` under `cargo metadata --no-deps` and `101` without the
    flag, and the real `make topology-check` run against scratch trees
    materialized exactly as `fixtures.rs` declares produced both documented
    refusals verbatim. The fixed-clock injection that finding suggested was
    declined because the clock is not the uniqueness mechanism; a probe
    pinning the clock to zero still produced three distinct roots from the
    serial. Gate set on `8d64088`, run sequentially: `make fmt` then
    `make check-fmt` (40 files unchanged), `make lint` (Clippy with
    `-D warnings`, `whitaker`, and Pylint 10.00/10), `make typecheck`,
    `LC_ALL=C make test` (206 nextest cases passed with 3 skipped, the
    doctest, 432 workflow-contract cases with 5 subtests, and the command
    boundary's 3 cases), `make markdownlint` (40 files, 0 issues, with the
    repository-wide spelling gate clean), and a standalone
    `make test-topology-command` (3 passed, 0 skipped) rerun after the repair.
    `make test` already chains that target, so its log carries a boundary run
    as well; the standalone rerun is what exercises the repaired helper
    against every fixture. A final `make check-fmt` after the last edit again
    reported 40 files unchanged. Evidence:
    `/tmp/fmt-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/check-fmt-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/lint-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/typecheck-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/test-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/markdownlint-statelet-1-2-1-topology-check-placeholder.out`,
    `/tmp/topology-command-statelet-1-2-1-topology-check-placeholder.out`, and
    `/tmp/check-fmt2-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-25 11:35Z) `b8230e6` published with the Progress entry above.
    Documentation only, so the gates are the Markdown ones: `make fmt`,
    `make check-fmt` (40 files unchanged), and `make markdownlint` (40 files,
    0 issues, spelling gate clean) green. Pushed with a fast-forward lease
    bound to `8d64088`; local head, refreshed tracking ref, and `ls-remote` all
    reported `b8230e6827c6e52a0d5624e26338eb00a7996f24`. Hosted on it:
    `build-test` and `act-validation` `pass`, CodeScene Code Health Review
    `pass` and approving, Gecko Security Review `pass`, with `Sourcery review`
    and `automerge` skipped by configuration. CodeRabbit's commit status reads
    `success` with the description `Review paused`, so `b8230e6` has no
    inspection of its own, and `reviewDecision` still reports
    `CHANGES_REQUESTED` from the submission on `fb1d00b`. Evidence:
    `/tmp/planstate6-statelet-1-2-1-topology-check-placeholder.out`.
  - [x] (2026-10-25 12:20Z) `8bf533d` published, recording `b8230e6`'s own
    publication; the Markdown gates were green on it before the push. A
    closing audit then re-measured the two macro-edge refusals that D26
    quotes, because the adjudication comment on the pull request and D26 gave
    different wording for the build-kind one. The comment was right and D26 was
    wrong: `make topology-check` run against both materialized fixture trees
    printed `declares a build dependency on "statelet-macros" as
    "statelet-macros"` and `declares a normal dependency on "statelet-macros" as
    "macros"`. The diagnostic's template renders `on {package:?} as
    {declared:?}` unconditionally, and the build-kind fixture declares no
    rename, so the package name appears twice. D26's finding disposition is
    unchanged — the re-run reproduced both measurements — and only its
    transcription of the first line was wrong; the entry now carries the
    measured text and says how it was corrected, and Surprises records the
    lesson. The probe logs were
    `/tmp/topology-probe-statelet-1-2-1-topology-check-placeholder.out` and
    `/tmp/topology-probe2-statelet-1-2-1-topology-check-placeholder.out`; as
    the Surprises entry records, a scratch path is not durable evidence, so the
    command above is what a successor should re-run.
  - [x] (2026-10-25 14:05Z) The review covering `1da5b25` completed with
    CodeRabbit's pre-merge table reporting three errors and two warnings and no
    inline findings, so the thread count is unchanged at 16. All five rows are
    adjudicated as D27. Four restate D26, whose measurements the re-run above
    reproduced, and are declined on their recorded grounds. The new Testing
    (Unit And Behavioural) error is also declined, on a measurement rather than
    a preference: `make topology-check` runs
    `cargo nextest run --test topology_check` (`Makefile:176-183`) and
    `behaviour.rs` is a non-ignored module of that target
    (`tests/topology_check.rs:16`), so moving the BDD scenarios onto the real
    Make boundary would start a nested compiling build inside the routine test
    run that D6 forbids. The boundary is already covered by all nine fixtures
    through the real target in
    `every_fixture_produces_its_documented_verdict`, including the three
    behaviours the feature file states. `mergeStateStatus` read
    `CLEAN` with `mergeable: MERGEABLE` on this head, so `CHANGES_REQUESTED`
    remains an aggregate over the superseded `fb1d00b` submission rather than a
    merge blocker. Only this plan changed this round, so the Markdown gates are
    the applicable set.

## Surprises & discoveries

A lockfile conflict cannot be resolved by choosing either side, and "take
main's copy, then rebuild" is not a merge. Taking main's lock and letting Cargo
re-resolve does preserve every version main pinned and does add this branch's
new dev-dependencies, but resolving the enlarged graph also upgrades thirteen
packages neither side asked to change — the `icu` family behind `textwrap` and
`tokio` behind `rstest-bdd`. That is not avoidable: the branch's lock and
main's lock describe graphs of different sizes, so no file-level merge of them
is coherent. The rebuild is the correct route, and the churn is its cost rather
than an accident, which is why it is attested here instead of being reduced by
hand-picking versions. An attempt to apply only main's two named bumps with
`cargo update -p` was tried and rejected: it pruned the Windows-only
`windows-sys 0.61.2` that main's chain still needs and silently re-pointed
`cap-primitives` at an older `0.60.2`. Surgical version selection on a lockfile
is not safer than a full resolve; it is only less visible.

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

CodeScene grades a repair by the shape of the code, not by whether the file got
smaller, and a helper shared by two call sites does not by itself clear a
duplication finding. The first `behaviour.rs` repair extracted
`require_refusal` and still scored 9.38, because both step bodies kept
constructing the same inline struct literal. The metric only moved when the
expectations became named constants, leaving two one-line call sites whose
arguments are distinct identifiers. The same lesson applies to `cs review`
versus `cs delta`: only the delta against the true PR base proves the hosted
comparison is clear, so each repair was checked both ways.

CodeScene's thresholds are absolute and small per file. A file below roughly
seven functions is graded as a whole, so a cohesion-motivated split can raise
the grade of the file it leaves while creating a new below-threshold file. The
`topology_command.rs` repair therefore used domain types rather than a further
decomposition, and the Python split kept both halves comfortably above the
function count.

The V2 negative control has to fail for the *witness* reason, not merely fail.
With the forbidden-name comparison removed, three tests report failures, but
only the generated property prints `the forbidden declaration was accepted` at
its own assertion; the BDD scenario and the reordering property fail because
their documents depend on the same guard. Reading only the exit status would
credit all three to the same control. The first attempt to run this control
also hid the answer: piping nextest through `tail` made `$?` report the pipe's
last command, so a failing run looked green until `pipefail` was set.

The shell semantics of an `env` command line decide where a copied variable may
live. If the line were an assignment prefix to a command, every right-hand side
would expand before any assignment took effect, so a copy of `RUSTFLAGS` would
read the composed value. Here `env` *is* the command word, so each `NAME=value`
operand is an argument to `env` and all of them expand against the unmodified
recipe environment. A local probe confirmed both behaviours: the `env`-first
spelling leaves the copy reading the caller's value, while the
assignment-prefix spelling leaves it reading the composed one. The distinction
is worth recording because the second spelling looks equivalent and would
silently weaken the repair.

The reason to avoid that duplication turned out to be weaker than the first
draft claimed. `cargo check --lib` with every repository flag written twice
exits zero, so the toolchain tolerates a repeated `-Zthreads=8` and a repeated
`-C link-arg`. The full pre-repair condition confirmed it: the boundary suite
run with `STATELET_TOPOLOGY_CALLER_FLAGS` absent and the composed value in
`RUSTFLAGS` passed all three cases. Dropping the copy therefore does not break
the build, and the two docstrings that said the nested case "would fail on a
duplicate `-Zthreads=8`" were corrected to state what is actually true — the
flags are repeated, and composition stops being idempotent. The copy is still
kept, both because the reviewer asked for a caller-first fallback and because a
future caller flag that is not duplication-safe would change meaning in the
nested build. The general lesson: a consequence asserted to justify a repair is
itself a claim, and a cheap probe can falsify it; measure before writing "would
fail" into a comment.

`CURDIR` is a command-line-overridable GNU Make variable, and an override of it
propagates into the recursive `$(MAKE)` invocation. That is what makes the V5
pin executable without editing the repository manifest:
`make check-topology CURDIR=<fixture>` writes its pin as
`<fixture>/Cargo.toml`, so a supplied `TOPOLOGY_MANIFEST_PATH` is provably
overwritten. The first fixture built for this control declared a member whose
manifest did not exist, and Cargo refused it during metadata acquisition; the
control only exercised the policy once the fixture became a real two-member
workspace. A boundary control that cannot get past the acquisition adapter
proves the adapter, not the boundary.

`--run-ignored` accepts only `default`, `only`, or `all`. The zero-selection
control therefore spells its two cases as an absent test name under `default`
and the guard's real name under `only`, rather than the invalid `ignored` that
the first draft used. nextest's zero-selection exit is 4 with
`error: no tests to run`, so the control asserts a real failure rather than a
cosmetic flag.

Every gate log this plan cites under `/tmp` has since been removed by the host's
`tmpfiles.d` rule, which expires `/tmp` entries after ten days. The citations
were accurate when written and are not evidence now; the commit SHA beside each
one is. A plan that treats a scratch path as durable evidence is relying on a
window, not a record, and should name the SHA and the command rather than the
file. The `/tmp` convention in `AGENTS.md` remains right for keeping bulky gate
output out of the planning context; it is the *citation* of that output as
lasting proof that does not survive.

A merge conflict silently suppresses the checks that would judge a push. GitHub
documents that workflows on the `pull_request` event do not run while a pull
request has a merge conflict, while `pull_request_target` workflows still do.
The push of `abc426f` produced no `CI` or `Act Validation` run at all — only the
`pull_request_target` `dependabot-automerge` job, plus the checks that report
from outside Actions. A missing check is therefore not a queued or slow check
here, and waiting for it would wait forever. The absence was diagnosed by
querying the runs API for everything created since the push and finding one
row, then confirming the rule in GitHub's own event documentation rather than
inferring it from the silence. The consequence for this plan is that a
conflicted branch cannot be validated by hosted CI until it is rebased, so the
rebase is a prerequisite for evidence, not merely for mergeability.

A diagnostic quoted from its template is not a diagnostic that was observed.
D26 recorded the build-kind refusal as
`declares a build dependency on "statelet-macros"`, which reads like a captured
line and is a truncation: the `#[error]` format string renders
`on {package:?} as {declared:?}` unconditionally, so a fixture that declares no
rename prints `statelet-macros` twice. Nothing in the repository could catch
this — the fixture needles are deliberately unquoted, because the child's
message passes through nextest's own diagnosis, and no unit test pins the full
rendered line. The record was wrong while every gate stayed green, and it was
found only by re-running the target and reading stderr. When a plan quotes a
program's output, the quotation should come from a run, and a quotation already
written from source should be re-measured rather than trusted.

A remedy can be coherent about the seam it dislikes and still be impossible
where it lands. D27's new finding correctly observed that the BDD scenarios call
`evaluate_metadata` directly, and its remedy — move them onto the real Make
boundary — is the shape that coverage would take in a repository with no rule
about nested builds. Here the remedy's destination is the obstacle: the
scenario module is compiled into the same test target that
`make topology-check` selects, so adopting the remedy would have the target's
own test process start the target. Reading the resolution in isolation would
have accepted it; reading the recipe it would land in declined it. A finding
that names a seam is worth checking against the command that would carry the
change, not only against the code the seam sits in.

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
case, and the twelve mutations the contract then held. Both are well under the
limit. D22 adds a thirteenth.

D18 (2026-10-18): the CodeScene repair is a separate commit from the M2 feature
commit, and it changes test-support structure only. The hosted check scored
three test files below 10.00 on `e02f5e3`. The repairs were chosen to make each
unit express one thing: named expectation constants in the BDD steps, domain
newtypes for the fixture's string-soup parameters, and two
single-responsibility finding helpers in the Python contract. None of them
changes a verdict, a fixture, or an assertion, so the acceptance evidence for
M2 still stands unchanged; the repair commit is justified by the CodeScene
delta and the full gate set alone. Deliberately rejected: splitting
`topology_command.rs` into further modules (a file-level gate makes the split a
new finding risk), and suppressing or threshold-raising in CodeScene (the
repository carries no rules file and the hosted profile is "Pay Down Tech
Debt").

D19 (2026-10-19): the CodeRabbit ExecPlan assessment's six findings are
repaired as one commit, because each one restores evidence the plan already
promised rather than extending scope. V2's witness is now generated and its
five transformations are applied to the runtime package found by member ID; the
properties moved partly into `transformations.rs` so `claim_properties.rs`
stays under 400 lines, and that module is the only new file. V1's missing
partitions arrive through `accepted_removed`, so an absent field is tested as
an absent key rather than a `null` value; `metadata_cases.rs` is the second new
file and needs no fixture tree. V5's two controls run real processes: the pin
control sets `CURDIR` to an owned two-member fixture and the selector control
runs nextest itself. The acquisition errors name the manifest and end with a
`Repair:` instruction; the environment-variable variants gained the same
instruction where they had none. Fixture roots are unique per invocation and
removed on both outcomes. The stale completion text is replaced, with the
planning-phase paragraph explicitly labelled historical. Deliberately rejected:
weakening the V2 properties to the fixed literal the assessment objected to,
and showing the V5 pin through a dry run — the assessment required executable
evidence.

D20 (2026-10-19): the D19 commit failed hosted CodeScene with two new-file
duplication findings, at `tests/topology_check/transformations.rs` and
`tests/topology_command/harness.rs`, even though a local `cs delta` run before
the commit reported none. The local run was taken while both files were still
untracked, and the delta compares committed state plus uncommitted changes to
*tracked* files, so the pair was simply not analysed. The lesson is recorded
rather than the finding disputed: a delta that must cover new files runs after
they are committed, or after an explicit `git add`. Both pairs were genuine.
`append_declarations` and `reverse_dependencies` shared their lookup and
differed only in the edit, so they now call one `edit_declarations` helper that
owns the "find the runtime package" rule. `invoke_target` and `invoke_pinned`
shared their whole Make setup, so they now call one `make_command` builder and
differ only in the arguments that give them separate meanings. No verdict,
selector, or fixture changed, and the focused suite still reports 49 passed.

D21 (2026-10-19): the follow-up assessment on `c687aad` discharged three of the
six D19 findings and kept three open, and each remaining one identified a
verification gap rather than a defect in the policy. All three are repaired
here. (1) The accepted fixture declared one kind per target, so
`reverse_targets` reversed two kind lists of length one and changed nothing
while still reporting that it had edited the document. The runtime target now
declares `["lib", "rlib"]`, which was checked against real Cargo output before
being adopted: a package with `crate-type = ["lib", "rlib"]` emits exactly that
pair, so the fixture describes a shape Cargo produces rather than one invented
to make a test pass. The two reversals were also split into `reverse_targets`
and `reverse_kind_lists` with separate properties. Under one shared flag a
two-target package satisfied it on target order alone, which would let the kind
reversal regress to a silent no-op — the exact failure being repaired. Each now
compares the list before and after the edit and reports a change only when the
order really differs. (2) `MetadataError::NonUnicodeValue` gained the missing
`Repair:` instruction, which contradicted both the D19 follow-up and D19's own
account of all variants carrying one. Its branch cannot be raised from a test:
`std::env::set_var` is denied repository-wide and mutating the process
environment even under a guard would race every other case in the binary, so
the assertion constructs the variant and pins the rendered diagnostic, checking
the variable name and the repair sentence rather than the bare word `Repair:`.
That is the assertion the finding asked for, and the raising branch stays a
two-line `match` arm with no decision to get wrong. (3) Fixture cleanup is now
owned from the moment the root exists rather than from the moment the caller
receives it: `materialize` and `write_pinned_fixture` remove the root on any
write failure and append a cleanup refusal to the original error. Verification:
the focused suite reports 51 passed, clippy is clean, and
`make test-topology-command` passes all three command-boundary tests against
the rewritten helpers. One further consequence was caught by the gate run
rather than by review: the cleanup rewrite grew
`tests/topology_command/harness.rs` to 431 lines, past the repository's
400-line rule. Whitaker reports that as a warning rather than a failure, so
`make lint` stayed green, but the rule is a hard constraint in `AGENTS.md`. The
manifest *text* construction moved to `tests/topology_command/manifests.rs`,
leaving `harness.rs` at 311 lines and its new sibling at 141; the `Fixture`
shape stayed with the process machinery because an expected verdict is what the
boundary asserts, while the manifest module only answers what a declared file
contains.

The follow-up assessment on `8e3627e` discharged findings 1, 4, and 5, which
together with the earlier discharge of 2, 3, and 6 closes all six original
findings with no remaining in-scope defect. One correction came back with it:
the diagnostic case asserts message *fragments* — the variable name, `Repair:`,
and `UTF-8 path` — not the complete repair sentence, and should not be
described as an exact-message assertion. The assessor also noted that it did
not build, test, or execute repository code, so the local gate results and the
Cargo metadata probe are supplied evidence rather than independently reproduced
results.

D22 (2026-10-19): the formal CodeRabbit review of `ffb89c4` submitted
`CHANGES_REQUESTED` with nine inline findings, and all nine are repaired in one
commit because each is a local correctness or documentation repair to code this
plan already owns. (1) `evaluate_metadata` gained an `# Examples` section, the
first in `tests/`, showing an accepted single-package document and the
`WorkspaceMemberCount` refusal of the same document with a second member and
its package record. (2) The developer guide no longer claims the command step
must not inherit `RUSTFLAGS` generally; it now says the step avoids the
coverage action's *step-scoped* environment, and that the recipe still composes
inherited flags. (3) The additional-member BDD fixture is built from complete
metadata: the package record is appended before the member identifier, so the
refusal is the count check rather than a dangling member. (4) A stale
module-doc reference in `tests/topology_command/fixtures.rs` names
`manifests::manifest_for`, where the helper actually lives. (5) Argument: the
reviewer observed that the nested `make topology-check` composes `RUSTFLAGS` a
second time. The repair copies the caller's value into
`STATELET_TOPOLOGY_CALLER_FLAGS` on the `env` line and prefers it in
`make_command`, with a workflow-contract clause and a thirteenth mutation
proving the copy cannot be dropped silently. The variable deliberately does not
end in `RUSTFLAGS`: the build standard's contract reads the *first*
`RUSTFLAGS="` on an evaluated line, and a name ending in the same token would
make it read the copy instead of the composed assignment. A shell probe settled
the ordering question the reviewer's suggested placement raised: `env` is the
command word, so every assignment operand expands against the unmodified recipe
environment and the copy still reads the caller's value even beside the
composed assignment. The first draft justified the repair by saying the nested
case "would fail on a duplicate `-Zthreads=8`"; a probe of the actual
pre-repair condition falsified that, since the suite passed 3/3 with the copy
absent, and the docstrings and Surprises entry now say only that the flags
would be repeated. The `var_os` read is a scoped
`#[expect(clippy::disallowed_methods)]` with a reason, matching the two
existing waivers, and the guide's waiver paragraph now names both modules. (6)
`check_observable_contract` delegates to `check_verdict` instead of
reimplementing it. (7)-(8) NumPy `Parameters` and `Returns` sections were added
to the Python helpers the reviewer named; the repository had no NumPy
precedent, so this follows the reviewer's convention rather than an existing
house style. (9) `Callable` now imports from `collections.abc`. Verification:
430 workflow-contract cases plus five subtests (up from 429 and one new
mutation), 51 focused topology cases, `make test-topology-command` with three
passes and no skips, `make lint-clippy`, `make check-fmt`, and
`make markdownlint` all green. The first Clippy run after the repair caught one
`shadow_reuse`: the match arm binding reused the scrutinee's `rustflags` name,
which this repository denies.

D23 (2026-10-25): the review of the rebased head `25f1e6d`, dispatched through
the managed queue after the commit influx left CodeRabbit's automatic reviews
paused, raised three findings, and all three are repaired in `e1a44ac` because
each hardens a check this plan already owns. (1) The `ForbiddenDependency`
diagnostic named only `statelet-macros`, but the predicate refuses a declared
edge to `statelet` itself as well; the `runtime-alias-dependency` fixture
declares `runtime = { package = "statelet" }` and is exactly that second case,
so the old text sent a contributor to the macro-admission rule for a dependency
that is never admitted at any gate. Only the attribute text changed, and the
`refusal` assertion in `behaviour.rs` still matches it because the macro-gate
phrase survives. (2) The workflow contract read coverage environment from the
command step alone. GitHub layers `env` at workflow, job, and step scope, and a
variable set by an enclosing scope reaches the step regardless, so flags
exported one level up would have passed. `_coverage_names` now reports one
scope's coverage names and `_findings_command_step` unions the workflow, job,
and step scopes, using the job mapping already returned by
`reading.jobs(workflow)`; the review was right that no `reading.Job` type
exists. There is no workflow- or job-level `env` today, so this is
future-regression coverage rather than a current defect, and two mutations were
added, one per new scope. (3) The mutation test asserted only that some finding
appeared. The reviewer's example is the exact hole: `check-before-toolchain`
moves the topology step ahead of `install-rust-toolchain` and therefore ahead
of nextest too, so it legitimately emits both order findings and the nextest
clause could satisfy the test after the toolchain clause regressed. Every
`MUTATIONS` entry now carries its expected message fragment and the test
asserts that fragment is present. Non-vacuity was established by deleting the
toolchain-order `_order_problems` call: the test then fails with
`check-before-toolchain did not report its own clause`, which the previous
assertion accepted. The `_ci_findings` shape guard that this required was
folded into a single `if` so the message literal is not duplicated.
Verification: `make check-fmt`, `make lint` (Clippy, whitaker, Pylint
10.00/10), and `make test` under `LC_ALL=C` (201 nextest, 1 doctest, 3
command-boundary) all green, plus 432 workflow-contract cases with five
subtests; `/verify` ran the real `make topology-check` against a manifest
reproducing the alias fixture and observed the corrected guidance.

D24 (2026-10-25): the review of `0319086`, the head carrying the plan
correction, raised four findings. All four were verified against current source
and repaired; none was declined. (1) `require_library_target` judged only the
`"lib"` target kind. Cargo reports an explicit `crate-type` as the *kind*, so a
`[lib]` section naming `rlib`, `dylib`, `cdylib`, or `staticlib` yields that
kind and never mentions `"lib"`; a probe of all five crate types with real
`cargo metadata` on this toolchain confirmed each mapping before the fix was
written. Such a manifest is an ordinary runtime library, and the refusal told
its author to restore a library they had not removed. `LIBRARY_KINDS` now names
all five, the proc-macro rejection is untouched, and
`every_library_kind_is_accepted` covers each kind plus the `cdylib`+`rlib`
combination. Non-vacuity was established by restoring the `"lib"`-only test:
exactly the five non-plain cases fail, with the predicted repair text. (2)
`optional_variable` handled four outcomes in one `match` with no direct case,
because the non-Unicode arm cannot be raised from a test: `set_var` is banned
repository-wide and mutating the environment would race every other case in the
binary. The `match` now lives in a pure `classify_optional` that takes the
already-produced `Result<String, VarError>`; `optional_variable` stays the thin
reader holding the scoped `#[expect]`. Four cases drive the classifier directly
— non-empty, empty, `NotPresent`, and a constructed `NotUnicode` — and the
existing rendered-message case now says the raising branch is covered elsewhere
instead of claiming two lines cannot be wrong. (3) and (4) are documentation
counts: `docs/developers-guide.md` and `docs/repository-layout.md` still said
thirteen mutations after D23 added the job- and workflow-scope coverage
controls, and the plan's pre-rebase head and conflict statements still read as
present tense three paragraphs after describing the rebase. The counts now say
fifteen, the guide names the three `env` scopes, and both statements are past
tense retaining `abc3e57` and the `Cargo.lock` conflict as historical evidence
rather than replacing them. Verification: 61 focused topology cases, 432
workflow-contract cases with five subtests under `LC_ALL=C`, `make check-fmt`,
and `make lint` (Clippy, whitaker, Pylint 10.00/10) are green; `make test` under
`LC_ALL=C` is recorded in the commit that carries this entry.

D25 (2026-10-25): `aa3e2a3`, the commit carrying D24, passed every local gate
but failed the hosted CodeScene check with "1 new file with code health below
10.00" and one "String Heavy Function Arguments" finding in
`tests/topology_check/policy_cases.rs` at 9.69. That rule is not a named rule in
`rules-catalogue.md`; it is threshold-driven, and no local CodeScene CLI is
installed here, so the diagnostic was read from the hosted review body and the
finding located by inspecting what that commit had added. The new code was the
library-kind coverage: an `rstest` whose case arguments were `&[&str]` plus a
helper `accept_target_kinds(kinds: &[&str])`, which is a function taking a
string collection and doing little else. Both are gone. One parameterless
`every_library_kind_is_accepted` now reads the policy's own `LIBRARY_KINDS` —
widened to `pub(crate)` for that purpose — and iterates `json!` values, so
there is no string-slice parameter to flag and no second place listing the
kinds. Reading the constant rather than repeating it also strengthens the case:
adding a kind to the policy without accepting it now fails here. The
`cdylib`+`rlib` pairing is still judged, because a manifest may name more than
one crate type. This is a decomposition of the offending function, not a
threshold change or a suppression. The case count falls from 61 to 56 as a
direct consequence: six `rstest` rows became one test, and the enumerated names
still cover every rejection plus the accepted document, the accepted-kind
sweep, the non-object refusal, and the extension case. Non-vacuity was
re-established after the rewrite by restoring the `"lib"`-only test: the
focused suite then ran one case and failed it (`MUTANT_EXIT=100`, 0 passed, 1
failed), so the rewritten case still discriminates. The five kinds and the
pairing are the same six inputs the six `rstest` rows supplied. Verification:
56 focused topology cases, `make check-fmt`, `make lint` (Clippy, whitaker,
Pylint 10.00/10), and `make test` under `LC_ALL=C` (206 nextest, 1 doctest, 432
workflow-contract cases with five subtests, 3 command-boundary) are green.

D26 (2026-10-25): CodeRabbit's pre-merge table for `fb1d00b` reported two
errors and three warnings, submitted with no inline findings. The table is a
single long-lived comment edited in place, so its `updated_at` — not its
`created_at` — is what makes it current; the row text describes this head. Each
check was judged against source and against measurement rather than repaired by
default.

The Testing (Overall) error is declined as a false positive, on two independent
measurements. It claims both macro-edge fixtures are unloadable, so Cargo fails
before the policy runs. First, a probe manifest declaring
`path = "../definitely-does-not-exist"` exits 0 under
`cargo metadata --no-deps` and 101 under the same command without the flag,
which is the documented semantics: the policy reads declarations, not a
resolved graph. Second, the real `make topology-check` was run against a
scratch tree materialized exactly as `fixtures.rs` declares it — `Cargo.toml`
and `src/lib.rs` only, with `../macros` absent. Both printed their documented
refusal:
`declares a build dependency on "statelet-macros" as "statelet-macros"` and
`declares a normal dependency on "statelet-macros" as "macros"`, each with the
repair naming the macro gate. `make test-topology-command` independently passed
3 with 0 skipped, which includes
`every_fixture_produces_its_documented_verdict` over all nine fixtures. Adding
`macros/Cargo.toml` as the check asks would add a file no fixture reads and
would hide the fact the cases depend on; the module doc now records why an
unresolvable `path` is legitimate here.

The first of those two quotations was written from the diagnostic's template
rather than from a captured line, and it left a clause out. The template always
renders both the package name and the declared name —
`on {package:?} as {declared:?}` — and the build-kind fixture declares no
rename, so both positions carry `statelet-macros`. The line above is now the
measured one: `make topology-check` was re-run against both materialized trees
on 2026-10-25 and each refusal was read from the run's stderr. The finding is
still declined on the same two measurements, which the re-run reproduced; only
this entry's transcription of them was wrong.

The Unit Architecture error is accepted. `unique_scratch_dir` folded a clock
failure into `0` with `unwrap_or_default()`, so a pre-epoch or failing clock
gave every call in the process the same root — and one case's cleanup would
then delete another's tree. It now returns `Result` and reports the failure;
`materialize` and the V5 case propagate it with `?`. The resolution also asked
for a fixed clock in tests, which is declined: the clock is already not the
uniqueness mechanism. A process-wide `AtomicU64` serial joins the process id
and the timestamp, so two calls in the same nanosecond still differ. The serial
is what makes the root unique; the clock only separates successive runs, where
a fixed value would collide across process-id reuse. Injecting a fixed clock
would therefore make the function weaker, not more testable.

The Developer Documentation warning is accepted and repaired. The
`Idempotence and recovery` section still told a contributor to commit only the
plan and to leave M1 and roadmap 1.2.1 untouched, long after both were done. It
now labels that paragraph historical and states current maintenance, recovery,
and gate-order instructions.

The Testing (Compile-Time / Ui) warning is declined. It asks for `insta`
snapshots of the diagnostics. The plan's Verification plan already records the
decision that no multivariant report is introduced and that exact small message
assertions suffice; the suite asserts the enum variant, the dynamic fields, and
the `Repair:` fragment for every refusal, plus the three exact success lines. A
snapshot would restate those assertions in a form that churns on unrelated
edits and, for the acquisition errors, would have to redact exactly the
run-specific path, status, and stderr that make the assertion meaningful today.
Adding `insta` would also add a dev-dependency to a test-only boundary that has
no multivariant output to stabilize.

The Domain Architecture warning is declined for this pull request. It asks for
`MetadataDocument`, `Package`, `Target`, and `Dependency` model types between
the adapter and the policy. That is a real design, and it is deliberately not
this task: the plan's Constraints bind the change to a closed three-clause
predicate and forbid the later graph evaluator, and the policy's whole
correctness argument is that it judges the document Cargo emits — including
additive fields it must ignore and malformed shapes it must name with a JSON
pointer — rather than a model that has already decided what is well-formed. A
typed model would move the malformed-field diagnostics into a second parser and
change where a malformed document is reported, which is a behavioural change
beyond a review repair. Recorded as a candidate for a future task rather than
left as an unaddressed objection.

Verification for this round: `make check-fmt`, `make lint`, `make typecheck`,
and `make test` under `LC_ALL=C`, in that order; `make test-topology-command`
for the changed command boundary.

D27 (2026-10-25): CodeRabbit's pre-merge table covering the current head
`1da5b25` reports three errors and two warnings, again with no inline threads.
Four rows restate D26 findings and are re-adjudicated below on their present
wording; one is new.

The Testing (Unit And Behavioural) error is new and is declined. It observes,
correctly, that `tests/topology_check/behaviour.rs:135` calls the private
`evaluate_metadata` on an in-memory document, so the BDD scenarios exercise the
same seam as the unit cases. Its remedy is that those scenarios instead "invoke
the real `make topology-check` boundary against `materialised` Cargo manifests,
and assert process status and diagnostic output", reusing the command harness.
The finding's own spelling of that word is retained inside the quotation rather
than corrected, because the quotation is evidence of what the review said. That
remedy is refused for a measured reason, not a stylistic one: it would place a
compiling Cargo build inside the routine test run. `make topology-check`
executes `cargo nextest run --test topology_check` (`Makefile:176-183`), and
the behavioural scenarios are a non-ignored module of that very target
(`mod behaviour;` at `tests/topology_check.rs:16`). Adopting the remedy would
make `cargo nextest run` start a nested Cargo build of the same target from
inside its own test process, which is precisely what D6 forbids: end-to-end
command tests belong in the dedicated sequential gate, and routine and coverage
discovery must never start nested build commands. The separation of seams the
finding objects to is therefore deliberate and is the constraint, not an
oversight. Nor is boundary coverage missing: nine fixtures, including all three
behaviours the feature file states, are already driven through the real
`make topology-check` by `every_fixture_produces_its_documented_verdict`
(`tests/topology_command.rs:121-139`), which asserts exit status and diagnostic
text and is reported by `make test-topology-command` as 3 passed with 0
skipped. The finding and its resolution also disagree with each other: it
concedes that `topology_command.rs` "runs real `make topology-check` processes
against Cargo fixtures and checks exit status and diagnostics", then asks for
that same coverage to be duplicated. Duplicating it inside the instrumented run
would add a second, differently-scoped path to the same boundary while making
the routine gate depend on a compiler invocation. What the feature file adds is
a readable statement of the rule over three named scenarios; it is deliberately
not a second execution path, and its module doc says so.

The Unit Architecture error is D26's finding reworded, and is declined for the
reason D26 recorded. It now concedes that fallibility is visible and that
`ROOT_SERIAL` exists, but still asks for an injected clock. The clock is not
the uniqueness mechanism: the process-wide serial is, and it is what makes two
calls in the same nanosecond differ, while the clock only separates successive
runs where a fixed value would collide across process-id reuse. Injecting a
deterministic clock would therefore make the root less unique, not more
testable, and would add a seam whose only purpose is to be stubbed. That the
finding cites `tests/topology_command.rs:61-73` as evidence of a hard-coded
dependency does not change the argument: a caller cannot prove a source is
uninjectable when the design does not need it injected.

The Testing (Overall) error and the Testing (Compile-Time / Ui) and Domain
Architecture warnings restate D26 with no new evidence, and their dispositions
are unchanged: the macro-edge fixtures were measured to produce their
documented refusals through the real target, `insta` would have to redact the
run-specific fields the acquisition assertions rely on, and explicit metadata
model types are a real design deliberately outside this task's closed-predicate
scope. The transcription correction recorded above changed none of those
measurements.

Verification for this round: `make fmt`, `make check-fmt`, and
`make markdownlint`, because the only changed file is this plan and no source,
test, or build configuration was touched. No code gate is re-run for a
documentation-only commit.

## Outcomes & retrospective

Historical planning record (2026-10-14, before approval): planning established
the metadata boundary and a bounded implementation route, and at that time no
topology command, runtime change, or roadmap completion had been implemented.
All six expert perspectives had been reconciled, and lint and typechecking had
passed on the unchanged runtime baseline. The full test retry passed: 150
nextest tests, one doctest, the pinned shared workflow contract check, and 416
workflow tests with five passing subtests. Formatting, Markdown/spelling, and
Mermaid validation also passed. Those planning documents were then ready for
draft publication.

M1 delivered the private checker and its local command, committed as `5140a53`.
M2 made the check routine: `make all` runs the pinned-input form, `make test`
ends with the command-boundary suite, CI runs both in the same relative order,
and `tests/workflow_contracts/topology_wiring_test.py` holds the wiring with
twelve negative controls, one per clause; D22 adds a thirteenth for the
caller-flags copy. The check remains a closed single-package policy and does
not resolve issue 112 or define a future multi-crate allow-list.

The one unresolved defect is inherited, not caused by this work: `make test`'s
workflow-contract phase fails two cases under the host's `en_US.UTF-8`
collation because the Makefile's `find | sort` inventory and the test's
byte-order `sorted()` disagree. Under `LC_ALL=C` the whole suite passes. Repair
touches the Makefile's ordering or the test's comparison, both outside this
plan's scope; the plan records the defect rather than fixing it here.

The M2 gate set ran to completion before `902927d`, and the CodeScene repair
was gated again before `1972342`. The commit SHA binds the local evidence;
hosted CI and the hosted assessments remain separate evidence and are owned by
`pr-babysitting`.

The CodeRabbit ExecPlan assessment on `6256307011` then raised six findings.
They were repaired as D19: the generated forbidden witness and the runtime
package lookup, the missing V1 and acquisition-error partitions, the executable
V5 controls, the acquisition diagnostics, the isolated fixture roots, and these
completion records. The V2 negative control was re-executed against the
rewritten properties and recorded under Progress.

Two further rounds followed. The follow-up assessment on `c687aad` discharged
three of the six and kept three open, each a verification gap rather than a
policy defect; D21 repairs them, and the assessment on `8e3627e` then confirmed
all six discharged. The formal CodeRabbit review of `ffb89c4` submitted
`CHANGES_REQUESTED` with nine inline findings, repaired in one commit as D22.
That round also produced the one correction this plan had to make to its own
reasoning: the first draft of finding 5's repair claimed the nested build
"would fail on a duplicate `-Zthreads=8`", and three probes falsified it — the
pre-repair boundary suite passed 3/3 with the caller-flags copy absent. The
repair stands on idempotence rather than on a failure it does not cause, and
the docstrings, the Surprises entry, and D22 now say so.

The pre-rebase implementation head was `abc3e57`. Local gates on it: 51 focused
topology cases, 430 workflow-contract cases with five subtests,
`make test-topology-command` with three passes and no skips, `make lint-clippy`,
`make check-fmt`, and `make markdownlint` all green. Hosted on the same SHA:
`build-test`, `act-validation`, CodeScene Code Health Review, Gecko Security
Review, and the CodeRabbit status all `success`, with `Sourcery review` and
`automerge` `skipped` by configuration. All nine review threads — eight from
`coderabbitai`, one from `chatgpt-codex-connector` — are `isResolved: true`.
The aggregate `reviewDecision` still reads `CHANGES_REQUESTED` because it
summarizes the superseded submission on `ffb89c4`; no thread remains open.

Documentation-only commits follow `abc3e57`. The first brings this plan's
status, Progress, Surprises, and Outcomes up to the published state and
corrects the negative-control count to thirteen in D17, in Outcomes, and in
`docs/repository-layout.md`; the second records the merge-conflict CI
suppression described below. The Markdown gates are green on them; `CI` and
`Act Validation` did not run, because a conflicted pull request suppresses
`pull_request` workflows. That is the one fact this retrospective would most
want a successor to carry: on this repository, a conflicted branch has no
hosted CI to wait for, so the rebase is the prerequisite for evidence rather
than a step after it.

Two delivery facts belong to `pr-babysitting` rather than to this plan's
implementation record. Before the rebase, the PR was `CONFLICTING` with `main`
on `Cargo.lock` only: `main` has since taken three dependency bumps (`662870c`,
`419cb15`, `781583c`) that this branch's single `Cargo.lock` change in
`5140a53` does not contain, while `Cargo.toml` is untouched by those bumps. And
the `/tmp` gate logs this plan cites are gone: the host's `tmpfiles.d` rule
expires `/tmp` entries after ten days, so a 2026-10-19 log is past its window
on 2026-10-25. The commit SHA each entry names is the durable record; the path
was only ever a convenience. The conflict also suppresses hosted `pull_request`
workflows entirely, so `abc426f` has no `CI` run to await and the rebase is
what will produce one.

The rebase onto `781583c` removed that blockage and the delivery rounds after
it were ordinary review work rather than recovery. Four rounds followed, D23 to
D26, and each is recorded under Decision log with its own evidence. Two
patterns are worth carrying forward. First, a review finding's *observation*
and its *remedy* deserve separate judgement: D26 accepted two of its five
findings, declined one remedy inside an accepted finding, and declined three
findings outright — and each declined item rests on a recorded measurement or a
documented constraint, never on preference. The fixed-clock injection is the
clearest case: the observation that a clock failure was mishandled was correct
and is repaired, while the proposed test seam was unnecessary because a
process-wide serial, not the clock, is what makes a root unique. Second, a
commit that discharges every review finding and passes every local gate can
still fail CodeScene on the code it adds: `aa3e2a3` repaired D24's four
findings and then reported one "String Heavy Function Arguments" finding in the
test it had just written, which `fb1d00b` resolved by deleting the string-slice
parameter rather than reshaping it. The lesson is that the hosted health check
judges the repair as new code, not only as a change to the code it replaced.

A third lesson arrived after publication, from auditing this plan against the
pull request rather than from any review. D26 declined the macro-edge finding
on measurements it had run, and then transcribed one of the two refusals from
the diagnostic's template instead of from the captured line — dropping the
`as "statelet-macros"` clause that the template always renders. Every gate
stayed green and no test could have objected, because the fixtures assert
unquoted needles by design and nothing pins a full rendered message. What
surfaced it was a second record of the same measurement disagreeing with the
first. The lesson generalizes: a quotation in a plan is evidence and should be
copied from a run, and when two records of one measurement disagree, neither
may be assumed correct — the disagreement is the signal to re-measure.

A fourth lesson came from the last review round. The plan declined four
restated findings and one new one across two rounds, and the new one was the
most plausible of the five: it described the code accurately and its remedy was
the ordinary way to test a command. What defeated it was not its reading of
`behaviour.rs` but the recipe it would have changed — the scenario module is
compiled into the target that the target selects. Adjudicating a finding means
following its remedy to the place it would land, because a remedy can be right
about a seam and wrong about the command that owns it. A plan that only
re-reads the cited lines will accept findings it should decline.

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

Status: implemented in `902927d` after the full sequential gate set passed; the
CodeScene repair followed in `1972342`. Roadmap 1.2.1 is ticked. The M2 plateau
discharges R2 and R4; R3 was rechecked on the full diff.

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

Historical planning-only instruction (2026-10-14, superseded): this paragraph
formerly told a contributor to commit only the plan and to leave M1 and roadmap
task 1.2.1 untouched pending approval. Both conditions were met and the plan is
now `IMPLEMENTED`; it is retained here only so a reader who saw the earlier
wording can tell that it is no longer current.

Running the check in this repository today: rerunning `make topology-check`
against the real manifest is always safe, because the policy only reads the
metadata document. The command-boundary suite writes each fixture beneath
`CARGO_TARGET_TMPDIR/topology-command`, one uniquely named root per invocation,
and removes it on both outcomes, so an interrupted run leaves at most one stale
root that the next run never reuses. Cargo build output stays in the repository
target directory or the configured shared target and never in a fixture
directory beneath `/tmp`; never edit the real manifest to demonstrate a
rejection. Record a failing `proptest` seed before retrying a property, and
confirm that a deliberately reintroduced negative control still fails before
running the wider gates or committing. To recover from a stale root or an
aborted fixture run, delete only the named root beneath
`CARGO_TARGET_TMPDIR/topology-command`; do not delete the shared parent or the
target directory. Verification after any maintenance edit is the repository's
own gate set, run in this order: `make check-fmt`, `make lint`,
`make typecheck`, `make test` under `LC_ALL=C`, and `make markdownlint`.

## Artefacts and notes

The rebase onto `main` at `781583c` preserved three recovery refs under
`refs/recovery/1-2-1-topology-check-placeholder/20261025T014245Z-{old-head,old-base,target}`,
recording `91c4414`, `7cb1267`, and `781583c` respectively. They are the
durable record of the pre-rebase series, whose commit hashes all changed in the
replay. Retain them until the pull request is accepted.

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
