# Add the single-crate topology-check placeholder

Status: DRAFT — awaiting explicit approval before implementation.

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
- [ ] Receive explicit plan approval.
- [ ] M1: implement and verify the private policy and local command.
- [ ] M2: integrate routine gates, reconcile documentation, and mark 1.2.1 done.

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

## Outcomes & retrospective

Planning has established the metadata boundary and a bounded implementation
route. No topology command, runtime change, or roadmap completion has been
implemented. All six expert perspectives have been reconciled. Lint and
typechecking passed on the unchanged runtime baseline. On 2026-10-14 the full
test retry passed: 150 nextest tests, one doctest, the pinned shared workflow
contract check, and 416 workflow tests with five passing subtests. Formatting,
Markdown/spelling, and Mermaid validation also passed. The planning documents
are ready for draft publication; implementation outcomes await approval.

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
