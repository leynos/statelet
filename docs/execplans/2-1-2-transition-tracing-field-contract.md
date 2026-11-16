# Publish the transition tracing field contract (2.1.2)

This ExecPlan (execution plan) is a living document. The sections
`Constraints`, `Tolerances`, `Risks`, `Progress`, `Surprises & discoveries`,
`Decision log`, `Outcomes & retrospective`, `Conformance basis`, and
`Verification plan` must be kept up to date as work proceeds.

Status: DRAFT (awaiting expert review and user approval)

Branch: `2-1-2-transition-tracing-field-contract`

Roadmap item: `2.1.2` in `docs/roadmap.md`.

## Purpose / big picture

Statelet is a small Rust library for marking transition boundaries in ordinary
handwritten state machines. A transition boundary is the function or method
where a state machine handles one input and decides what happens next, for
example `ProcessBuffer::handle_line` in the `mdtablefix` Markdown formatter.
Statelet does not own the state machine; it supplies naming and observability
conventions that the application applies to its own code.

The technical design, `docs/design.md` §9, already lists five tracing field
names (`transition.name`, `transition.state.before`, `transition.event`,
`transition.outcome`, and `transition.error`). It calls them public
operational API: dashboards, alerts, and log queries may depend on them, so
renaming one is a breaking change that the Rust compiler cannot detect. Today
those names exist only as a bullet list in a design document written for a
future macro. Nothing tells a downstream author what each field means, when it
is recorded, what value it carries, or how to declare it with the ordinary
`tracing` crate. The design documents that do discuss them disagree (see
`Context and orientation`).

After this change, a downstream crate author can do three things.

1. Read one authoritative contract, recorded in a new ADR and summarized in
   `docs/design.md` §9 and `docs/users-guide.md`, that states each field's
   meaning, value rules, presence rules, and semantic-versioning policy.
2. Name the fields either as dotted literals or through zero-dependency
   constants that Statelet exports, for example:

   ```rust
   use statelet::{StateName, fields};

   #[tracing::instrument(
       skip_all,
       fields(
           {fields::TRANSITION_NAME} = "handle_line",
           {fields::TRANSITION_STATE_BEFORE} = self.mode.state_name(),
           {fields::TRANSITION_EVENT} = line_kind.label(),
           {fields::TRANSITION_OUTCOME} = tracing::field::Empty,
           {fields::TRANSITION_ERROR} = tracing::field::Empty,
       )
   )]
   fn handle_line(&mut self, line_kind: LineKind) -> Result<Decision, ParseError> {
       // ordinary Rust; record the outcome or the error before returning
   }
   ```

3. Run `make test` and see the downstream-shaped integration tests prove that
   spans built with ordinary `#[tracing::instrument(fields(...))]` carry exactly
   the contracted field names, with late fields absent until recorded.

The roadmap's success criterion for 2.1.2 is that "downstream examples can use
the same fields with ordinary `#[tracing::instrument(fields(...))]`". That is
observed by the integration test `tests/transition_fields.rs`, which compiles
as an external crate against the public `statelet` API, and by the rustdoc
example on `statelet::fields`, which the doctest suite compiles and runs.

This task deliberately does not add a `tracing` feature, a runtime dependency,
a helper for recording outcomes, a macro, an outcome enum, or a usage-pattern
guide. Roadmap 2.1.3 owns the user-facing conventions-only usage pattern;
roadmap 3.2.3 owns the `tracing` default-feature decision; ADR 0003 decides
whether any of this ships.

## Constraints

- Await explicit user approval of this plan before changing any file other
  than this ExecPlan.
- Add no runtime dependency, no `[features]` table, no optional dependency, no
  workspace member, and no proc macro. `[dependencies]` in `Cargo.toml` stays
  empty (design §3.4, §10, §13.5; ADR 0002).
- The only public API addition is the module `statelet::fields` containing
  exactly five `pub const` items of type `&'static str`, as decided in D2. Any
  other public item (a type, function, trait, macro, or re-export) requires
  escalation.
- The contract fixes field names and their meaning, presence, and value rules.
  It must not fix the label vocabulary of any field value: transition names,
  state labels, event classes, and outcome labels remain application-owned
  (ADR 0004 leaves `StateName`'s return shape to task 3.2.1; proposed ADR 0006
  owns label-value stability).
- Do not publish `TransitionOutcome` or any outcome vocabulary (design §6.2,
  ADR 0002).
- Do not add `transition.state.after`, `transition.completion`,
  `transition.return_kind`, or any other field proposed by RFCs 0001–0005 or
  ADR 0007. Those documents are `Proposed` and state that existing field
  meanings stay unchanged while proposed.
- Do not edit the sentence "The default remains `&'static str` until a real
  example consumes something stronger" or other clauses quoted by
  `tests/state_name_consumption_contract` and
  `tests/v0_1_exit_register_contract`. Edits to `docs/design.md` are confined
  to §9, the companion-document list, and §14 if needed.
- `docs/users-guide.md` may contain only Bash fences carrying a
  `<!-- tested-example: ... -->` marker (`tests/documented_examples/loader.rs`).
  Rust examples belong in rustdoc or tests and are linked from the guide.
- Do not modify `mdtablefix`, the topology policy, the feature-matrix recipe,
  CI workflows, or completed ExecPlans (including
  `docs/execplans/2-1-1-state-name-trait-examples.md`).
- Respect `AGENTS.md`: files under 400 lines, `//!` module docs on every
  module, an assertion message on every `assert!`-family call, no `.expect()`
  outside tests, `shadow_reuse` denied, `missing_docs_in_private_items` denied,
  caret version requirements, and en-GB-oxendict spelling.
- Preserve the shared Cargo cache and compiler admission. Run gates
  sequentially; never use `/tmp` as a build directory.
- Keep roadmap 2.1.2 unchecked until every milestone's acceptance passes; then
  mark it `[x]`.

## Tolerances (exception triggers)

ExecPlan files are excluded from every count below, including this file. The
comparison base is the merge base with `origin/main` (`e69af49` at drafting).
Classification is whole-file by purpose: `*.rs` files and
`tests/features/*.feature` are code (including their rustdoc and comments);
`Cargo.toml` and `Cargo.lock` are configuration; `docs/**/*.md` other than
`docs/execplans/**`, plus `README.md`, are documentation. Each limit is net
lines (additions minus deletions), measured with:

```bash
base=$(git merge-base HEAD origin/main)
git diff --numstat "$base" -- '*.rs' 'tests/features/*.feature' \
  | awk '{a+=$1; d+=$2} END {print "code", a-d}'
git diff --numstat "$base" -- Cargo.toml Cargo.lock \
  | awk '{a+=$1; d+=$2} END {print "config", a-d}'
git diff --numstat "$base" -- docs README.md ':!docs/execplans' \
  | awk '{a+=$1; d+=$2} END {print "docs", a-d}'
git diff --name-only "$base" -- ':!docs/execplans' | wc -l
```

- Files: more than 16 changed non-ExecPlan files.
- Code: more than 1,100 net lines.
- Configuration: more than 20 net lines.
- Documentation: more than 550 net lines.
- Any single code file reaching 400 lines.
- Interface: any public item beyond the five constants in `statelet::fields`.
- Dependencies: any new runtime dependency, any feature, or any
  dev-dependency other than `tracing` and `tracing-core`, or any increase in
  the number of resolved packages in `Cargo.lock`.
- Architecture: any change to design §§3–8 or §10, ADRs 0002–0004, or the
  topology or feature-matrix policy.
- Semantics: if the probe or tests show that the constant form and the dotted
  literal form do not produce identical field names under the locked `tracing`
  versions, stop (D2 depends on it).
- Iterations: three unsuccessful fix attempts for the same gate failure.
- Ambiguity: any reviewer finding that the contract's field meanings conflict
  with an accepted document.

There is no elapsed-time limit; a healthy compiler queue is not a failure.

Budget rationale. The surveyed surface is `src/lib.rs` (75 lines, one trait),
`tests/state_name.rs` (134 lines), the 21-line `tests/documented_examples.rs`
root with its `#[path]` modules, two BDD feature files, and the documents
listed in `Context and orientation`. The estimate is `src/fields.rs` about 150
lines (five constants with full rustdoc and one runnable doctest),
`src/lib.rs` +6, the integration-test root about 30, a capture subscriber
about 140, a downstream fixture about 110, contract-table tests about 100, a
document-agreement test about 130, BDD steps about 150, and a feature file
about 45: roughly 860 lines. Historical evidence matters here: the 1.2.1
topology plan budgeted 900 added Rust lines and measured 1,736 (D13 of that
plan), almost entirely because of rustdoc and comment density that this
repository's lints require. The 2.1.1 plan budgeted 600 lines for code and
documentation together and recorded no revision. The code budget therefore
carries roughly 25% contingency over the estimate. Documentation is estimated
at ADR 0008 about 230 lines, design §9 +45, users' guide +55, developers'
guide +70, `docs/contents.md` +3, `docs/repository-layout.md` +20,
`docs/context.md` +10, and roadmap ±1: about 435 lines plus contingency.
Configuration is two dev-dependency entries with their explanatory comments
(about 8 lines) and the lockfile's dependency-list edits (about 3 lines).

## Risks

- Risk: the contract is mistaken for a release commitment. ADR 0003 places the
  published field contract in exits E2 and E3 only; exit E1 keeps conventions
  project-local. Severity: medium. Likelihood: medium. Mitigation: ADR 0008
  and the users' guide state that the contract is semver-relevant from the
  first published release, that no release exists, and that ADR 0003 still
  decides whether it ships.
- Risk: `transition.outcome` is given a meaning that a later accepted RFC
  contradicts. Severity: medium. Likelihood: medium. Mitigation: D4 follows the
  accepted documents (ADR 0002 and the `docs/context.md` glossary) and records
  that RFC 0001, if accepted, amends ADR 0008 through the supersession process.
- Risk: authors forget to declare late fields at span creation, and
  `Span::record` silently drops the value. Severity: high for downstream
  users. Likelihood: high. Mitigation: the presence rules require declaration;
  rustdoc, the users' guide, and a negative-control test demonstrate the
  silent drop (`AXIOM-undeclared-dropped`).
- Risk: an author enables `#[instrument]` without `skip_all`, so every argument
  (including raw input lines) is recorded with `Debug`. Severity: medium.
  Likelihood: medium. Mitigation: every example uses `skip_all`; the value
  rules say `transition.event` is a semantic class, not raw input.
- Risk: the capture subscriber is unfaithful to the real `tracing`
  dispatch path, making tests pass vacuously. Severity: high. Likelihood: low.
  Mitigation: it implements `tracing::Subscriber` directly against the locked
  `tracing-core`, including `current_span` (the planning probe showed that
  omitting it silently drops every `Span::current().record`), and non-vacuity
  controls prove it observes recorded and unrecorded fields.
- Risk: `{path::CONST}` field-name syntax behaves differently under strict
  lints or future `tracing-attributes` versions. Severity: medium. Likelihood:
  low. Mitigation: the probe confirmed it on the locked 0.1.44/0.1.31 pair;
  the downstream tests compile both forms under `make lint`; the users' guide
  keeps the dotted-literal form as an equal alternative.
- Risk: document-agreement tests become brittle prose checks. Severity: low.
  Likelihood: medium. Mitigation: they read only the contract table rows and
  backticked `transition.*` tokens inside one named section, not sentences.
- Risk: line budgets overrun from rustdoc density, as in 1.2.1. Severity: low.
  Likelihood: medium. Mitigation: 25% contingency; measure at each milestone.

## Progress

- [x] (2026-11-16T18:30Z) Renamed branch to
      `2-1-2-transition-tracing-field-contract` and set the Lody session title.
- [x] (2026-11-16T18:35Z) Three Wyvern reconnaissance reports received
      (implementation shape, governing documents, gates and conventions).
- [x] (2026-11-16T18:42Z) External research and feasibility probe `EP-M0`
      completed (see `Artefacts and notes`).
- [x] (2026-11-16T19:10Z) Initial draft written.
- [ ] Community-of-experts design review completed and findings reconciled.
- [ ] Draft PR opened for the plan.
- [ ] Plan approved by the user.
- [ ] `EP-M1` contract recorded and constants exported.
- [ ] `EP-M2` downstream instrumentation proven by capture tests and BDD.
- [ ] `EP-M3` documentation agreement enforced and delivery closed.

## Surprises & discoveries

- Observation: a handwritten `tracing::Subscriber` whose `enter` and `exit`
  are no-ops makes `Span::current()` return a disabled span, so every late
  `record` call silently vanishes. Evidence: the first probe run printed no
  `transition.outcome` or `transition.error` values; adding a span stack and
  `current_span` made both appear. Impact: the capture subscriber needs
  `tracing-core` (for `span::Current`, which `tracing` does not re-export) and
  must implement `current_span`; this is recorded as D9.
- Observation: `tracing` 0.1.44 and `tracing-core` 0.1.36 are already resolved
  in `Cargo.lock` through `rstest-bdd`, so declaring them as dev-dependencies
  adds no resolved package. Evidence: `cargo tree -i tracing -e normal,dev`.
  Impact: D8 has no supply-chain cost beyond two manifest lines.
- Observation: the design documents disagree on the five fields. Evidence: see
  `Context and orientation`. Impact: ADR 0008 must settle meanings explicitly
  rather than restate design §9.

## Decision log

- D1. Decision: the contract fixes the five field names, their meaning, their
  presence rules, and their value-shape rules; it does not fix any field's
  label vocabulary. Rationale: accepted ADR 0004 leaves `StateName`'s return
  shape to task 3.2.1, and proposed ADR 0006 says label-value stability
  becomes public only after task 6.3.4. Fixing names and meanings is the part
  dashboards and queries depend on. Date/Author: 2026-11-16, planning agent.
- D2. Decision: export the names as five `pub const ...: &str` items in a new
  zero-dependency module `statelet::fields`: `TRANSITION_NAME`,
  `TRANSITION_STATE_BEFORE`, `TRANSITION_EVENT`, `TRANSITION_OUTCOME`, and
  `TRANSITION_ERROR`. Rationale: design §3.2 makes the runtime crate the owner
  of the tracing field contract, and §11.2 requires the baseline to be the
  strongest non-macro alternative. Constants let a downstream crate name a
  field through the compiler (`{fields::TRANSITION_NAME} = ...` in
  `#[instrument(fields(...))]`, and `Span::record(fields::TRANSITION_OUTCOME,
  ...)`), so a misspelt constant fails to compile while a misspelt literal
  silently creates a different field. They need no `tracing` dependency, so
  the no-default build stays tracing-free (§13.5), and a later macro can emit
  `::statelet::fields::...` paths (§7). Rejected alternatives: documentation
  only (leaves drift prevention as an unearned macro advantage and gives the
  macro nothing to reference); a `FieldName` newtype (tracing's field-name
  position and `Span::record` take `&str`, so every use would need
  `.as_str()`); an enum or a `tracing`-typed `FieldSet` (requires a runtime
  `tracing` dependency, which §3.4 defers). The dotted literal form remains a
  documented, equal alternative; the constants are a convenience, not a
  requirement. Date/Author: 2026-11-16, planning agent.
- D3. Decision: record the contract in a new ADR,
  `docs/adrs/adr-0008-publish-the-transition-tracing-field-contract.md`, and
  make design §9 reference it. Rationale: the decision resolves conflicts
  among four documents, creates semver-relevant public API, and reserves a
  namespace; `AGENTS.md` requires an ADR for substantive decisions. No open
  pull request claims ADR 0008 (checked with `gh pr list` on 2026-11-16).
  Date/Author: 2026-11-16, planning agent.
- D4. Decision: `transition.outcome` is the stable label of the result the
  boundary returned on normal completion, chosen by the application (for
  example from a project-local `Decision` enum). It is recorded only when the
  boundary returns normally and has a labelled result; it is absent when the
  boundary returns an error. Rationale: this follows the accepted documents,
  the `docs/context.md` glossary ("the value returned by a transition boundary
  to describe the visible result") and ADR 0002 ("must use the project-local
  decision or return shape already present"). RFC 0001's separation of domain
  outcome from return classification is `Proposed`; if accepted, it amends
  ADR 0008 through the supersession process. Date/Author: 2026-11-16,
  planning agent.
- D5. Decision: presence rules. Every contracted field the boundary may record
  must be declared when the span is created; fields known only after the body
  runs are declared `tracing::field::Empty`. `transition.name` is always
  recorded at creation. `transition.state.before` is recorded at creation when
  the boundary observes a state. `transition.event` is recorded at creation
  when the boundary handles an input. `transition.outcome` follows D4.
  `transition.error` is recorded only when a fallible boundary returns an
  error; infallible boundaries may omit the declaration. An absent field means
  "not captured by this boundary", never a default or guessed value.
  Rationale: `tracing` silently drops values recorded for undeclared fields
  (confirmed by the probe), and the state-representation design and ADR 0007
  both define absence as "not captured". Design §9's "default generated
  fields" wording describes the future macro, while the conventions-only
  contract needs a minimum set; `transition.name` is the only field every
  boundary has. Date/Author: 2026-11-16, planning agent.
- D6. Decision: value rules. `transition.name`, `transition.state.before`,
  `transition.event`, and `transition.outcome` carry stable, low-cardinality
  labels, preferably `&'static str` literals; `transition.state.before`
  should be the `StateName::state_name()` label; `transition.event` names the
  semantic class of the input, never the raw input. `transition.error` carries
  a deliberately selected human-readable rendering of the error (normally its
  `Display` output via `%`), never requires `Debug`, and must not contain
  secrets or raw payloads. Letter case of values is not fixed. Rationale:
  design §9 prefers documented omissions over `Debug` bounds; the
  state-representation design and ADR 0007 say events are semantic classes
  and raw input is excluded; `AGENTS.md` keeps secrets and raw payloads out of
  traces. Casing belongs to proposed ADR 0006. Date/Author: 2026-11-16,
  planning agent.
- D7. Decision: reserve the `transition.` field-name prefix for Statelet.
  Downstream code must not define its own `transition.*` fields. Adding a new
  reserved field is additive; renaming or removing a field, or changing its
  meaning, presence rules, or value shape incompatibly, is a breaking change
  under semantic versioning once a release containing the contract exists.
  Rationale: a reserved prefix lets RFC 0001's `transition.state.after` or
  similar additions arrive without colliding with application fields; this
  mirrors the OpenTelemetry naming rule that "two attributes … MUST NOT share
  the same name" and its namespace practice. Date/Author: 2026-11-16,
  planning agent.
- D8. Decision: add `tracing` (`"0.1.44"`) and `tracing-core` (`"0.1.36"`) as
  dev-dependencies only. Rationale: downstream-shaped tests and the rustdoc
  example need `#[tracing::instrument]`; the capture subscriber needs
  `tracing_core::span::Current`. Both are already resolved in `Cargo.lock`, so
  the resolved package count is unchanged. A runtime dependency or `tracing`
  feature is deferred by design §3.4, §10, and roadmap 3.2.3. Date/Author:
  2026-11-16, planning agent.
- D9. Decision: capture spans with a small test-only `tracing::Subscriber`
  rather than `tracing-subscriber` or `tracing-test`. Rationale:
  `tracing-subscriber` would add several resolved packages for a 140-line
  need; `tracing-test` asserts on formatted log text, which cannot distinguish
  a declared-but-empty field from an absent one. The subscriber is installed
  with `tracing::subscriber::with_default`, which is scoped to the current
  thread, so tests run in parallel without global state or `serial_test`. It
  must implement `current_span` (see `Surprises & discoveries`). Date/Author:
  2026-11-16, planning agent.
- D10. Decision: no Verus, Kani, proptest, or `insta` for this task. Rationale:
  see `Formal verification decision`. Date/Author: 2026-11-16, planning agent.
- D11. Decision: no `examples/` directory and no usage-pattern guide in this
  task. Rationale: roadmap 2.1.3 owns the user-facing conventions-only
  example; 2.1.2 proves usability through rustdoc and downstream-shaped
  integration tests. Date/Author: 2026-11-16, planning agent.

## Outcomes & retrospective

Not started. Complete at each milestone and at completion.

## Context and orientation

Inspected base revision: `e69af49` ("Bump rust-toolchain from
nightly-2026-11-13 to nightly-2026-11-16 (#193)"). Line numbers below refer to
that revision; relocate symbols with `grep -n`.

The crate is a single library package, `statelet` 0.1.0, edition 2024, with an
empty `[dependencies]` table and no `[features]` table (`Cargo.toml`). Its only
source file is `src/lib.rs`, whose sole public item is
`pub trait StateName { fn state_name(&self) -> &'static str; }`, added by
roadmap 2.1.1 (PR 172). There is no `examples/` directory.

Tests live under `tests/`. Each top-level `tests/<name>.rs` is an integration
test crate, which compiles as an external crate against the public API; larger
suites use a root file with `#[path = "<name>/<module>.rs"] mod <module>;`
declarations (for example `tests/documented_examples.rs`). Behavioural tests
use `rstest-bdd` 0.6.0 with Gherkin files in `tests/features/` and step
modules that define a `#[derive(ScenarioState)]` world with `rstest_bdd::Slot`
fields (see `tests/documented_examples/behaviour.rs`). Dev-dependencies
available today: `rstest` 0.28, `rstest-bdd` and `rstest-bdd-macros` 0.6,
`googletest` 0.14, `pretty_assertions` 1.4, `proptest` 1.11, `serde_json`,
`thiserror`, `toml`, `cap-std`, and `camino`. There is no `insta`, `trybuild`,
Kani, or Verus configuration in the repository.

Gates, from the `Makefile`: `make check-fmt` (rustfmt check and `mdtablefix`
check), `make lint` (rustdoc with `-D warnings`, Clippy with `--all-targets
--all-features -D warnings`, Whitaker, Pylint, and `interrogate`), and
`make test` (`cargo nextest run --all-targets --all-features`, then
`cargo test --workspace --doc --all-features`, workflow contracts, and the
topology and feature-matrix command-boundary suites). `make markdownlint` runs
`make spelling` first. `make fmt` rewrites Rust and Markdown formatting.
`make nixie` validates Mermaid diagrams. The topology policy
(`tests/topology_check/policy.rs`) is a denylist of `statelet-macros` and
`statelet`, so new dev-dependencies do not trip it; `make feature-matrix`
checks only `--lib` configurations, which do not compile dev-dependencies.

The governing documents disagree about the five fields:

- `docs/design.md` §9 lists all five as "default generated fields" of the
  future macro, calls the names public operational API, and says the first
  implementation may record a subset.
- `docs/state-representation-and-observability-design.md` (status `Proposed`)
  marks `transition.outcome` and `transition.error` optional, adds a proposed
  `transition.state.after`, and defines `transition.event` as "the stable
  semantic class of the handled input".
- `docs/context.md` defines a transition outcome as "the value returned by a
  transition boundary", and accepted ADR 0002 says the initial outcome field
  "must use the project-local decision or return shape already present".
- RFC 0001 (`Proposed`) redefines outcome as domain meaning distinct from the
  return classification and adds `transition.completion` and
  `transition.return_kind`; it says existing field meanings stay unchanged
  while it is proposed.
- ADR 0002's example declares only three fields in `fields(...)`, which would
  silently drop a late-recorded outcome or error.
- No document chooses `Display` or `Debug` for `transition.error`.

Accepted ADR 0003 places "the documented `transition.*` tracing-field
contract" in exit scopes E2 and E3; E1 keeps conventions project-local. Task
2.1.2 precedes the B1 gate at 3.1.3, so the contract is written now but its
publication is still conditional.

User-facing Markdown under `docs/users-guide.md` and `README.md` may contain
only Bash fences carrying `<!-- tested-example: <id> -->` markers; they are
executed by `tests/documented_examples.rs`. Rust examples therefore live in
rustdoc and tests and are linked from the guide, as 2.1.1 did for `StateName`.

Relevant `tracing` facts, verified against docs.rs for `tracing` 0.1.44 and a
planning probe (see `Artefacts and notes`):

- `#[instrument(fields(...))]` accepts dotted field names
  (`fields(http.uri = ...)`) and constant field names in braces
  (`fields({FOOBAR} = ...)`, where `const FOOBAR: &str = "foo.bar"`); the
  probe confirmed that a cross-crate path such as
  `{fakelet::fields::TRANSITION_NAME}` also works.
- `fields(x = tracing::field::Empty)` and bare `fields(x)` both declare a field
  without a value; `Span::current().record("x", value)` fills it later.
- Recording a field that was not declared at span creation is silently
  ignored.
- `%value` records with `Display`, `?value` with `Debug`. `skip_all` stops
  `#[instrument]` from recording every argument with `Debug`.
- `#[instrument(err)]` records into a field named `error`, not
  `transition.error`, so the contract cannot rely on it.

Documentation and skills to consult while implementing: `docs/design.md`
§§3.1–3.4, 6.1, 9, 10, 13.5; `docs/adrs/adr-0002-*.md`, `adr-0003-*.md`,
`adr-0004-*.md`; `docs/documentation-style-guide.md` (ADR template at its
"Architectural decision records" section); `docs/developers-guide.md`
("StateName trait reuse", "Tested user-facing examples");
`docs/rust-testing-with-rstest-fixtures.md`; `docs/rstest-bdd-users-guide.md`;
`docs/reliable-testing-in-rust-via-dependency-injection.md` (the capture
subscriber is injected per test rather than installed globally);
`docs/rust-doctest-dry-guide.md` (one runnable doctest on the module, no
duplicated examples); `docs/complexity-antipatterns-and-refactoring-strategies.md`
(keep step functions and the subscriber small). Skills: `rust-router`,
`rust-types-and-apis` (constant API shape), `rust-unit-testing` (rstest,
googletest, pretty_assertions), `rust-verification` (method selection),
`execplans`, `en-gb-oxendict-style`, and `commit-message`.

Budget survey and history are recorded in `Tolerances`.

## Conformance basis

Upstream artefacts at `e69af49`:

- Terms of reference: `docs/terms-of-reference.md` (Draft v0.2), §§5–7 and the
  conventions baseline at line 47 ("documented conventions, `StateName`,
  stable tracing field names").
- Technical design: `docs/design.md` (Draft v0.2, last substantive revision
  2026-09-19), §§3.2, 3.4, 6.1, 6.2, 9, 10, 11.2, 13.5, 13.7.
- ADRs: 0002 (accepted), 0003 (accepted), 0004 (accepted). ADRs 0006 and 0007
  and RFCs 0001–0005 are `Proposed` and are context, not requirements.
- Roadmap: `docs/roadmap.md` task 2.1.2.
- Glossary: `docs/context.md` "Transition outcome" and "State name".

Requirements derived for this plan:

- `REQ-names`: the five names are exactly `transition.name`,
  `transition.state.before`, `transition.event`, `transition.outcome`, and
  `transition.error` (design §9, roadmap 2.1.2).
- `REQ-semver`: renaming a field or changing its meaning is documented as
  semver-relevant (design §9, roadmap 2.1.2).
- `REQ-plain-instrument`: downstream code can use the fields with ordinary
  `#[tracing::instrument(fields(...))]` (roadmap 2.1.2 success criterion).
- `REQ-no-tracing-dep`: the default and no-default builds compile without
  `tracing` (design §13.5, §3.4).
- `REQ-runtime-owns-vocabulary`: the runtime crate owns the field contract
  (design §3.2).
- `REQ-no-outcome-type`: no published outcome vocabulary (design §6.2,
  ADR 0002).
- `REQ-label-shape-open`: the `StateName` return shape is not ratified
  (ADR 0004).

Observed gaps:

- `GAP-meaning`: no document states each field's meaning, presence, and
  value rules consistently.
- `GAP-declaration`: no guidance requires late fields to be declared, so
  values can be silently lost.
- `GAP-evidence`: no executable evidence shows the fields working with plain
  `#[instrument]`.
- `GAP-drift`: nothing ties the documented names to code, so the design,
  guide, and any constants could drift apart.

Trace links:

```plaintext
design §9 -> REQ-names -> D2 -> EP-M1 -> transition_fields::contract::names_are_exact
design §9 -> REQ-semver -> D3, D7 -> EP-M1 -> ADR 0008 "Semantic versioning"
roadmap 2.1.2 -> REQ-plain-instrument -> D2, D5 -> EP-M2 -> transition_fields::downstream, BDD scenarios
design §13.5 -> REQ-no-tracing-dep -> D8 -> EP-M1 -> transition_fields::contract::runtime_has_no_tracing_dependency
GAP-meaning -> D4, D5, D6 -> EP-M1 -> ADR 0008, design §9, users' guide
GAP-declaration -> D5 -> EP-M2 -> BDD "An undeclared late field is lost"
GAP-drift -> EP-M3 -> transition_fields::docs_agree
REQ-no-outcome-type, REQ-label-shape-open -> Constraints -> every milestone's conformance check
```

## Verification plan

### Abstract model

Let `N` be the set of five contracted names. A span `s` created by
`#[instrument]` has a declared field set `decl(s)`, fixed at creation, and a
partial map `val(s)` from declared names to recorded strings. `record(s, k,
v)` sets `val(s)[k] = v` if `k ∈ decl(s)` and does nothing otherwise. A
transition boundary `b` has properties `has_state(b)`, `has_event(b)`,
`fallible(b)`, and a run result `r ∈ {Ok(label), Err(e)}`.

The contract requires, after a run of `b` that follows the presence rules:

- `transition.name ∈ val(s)`;
- `transition.state.before ∈ val(s)` iff `has_state(b)`;
- `transition.event ∈ val(s)` iff `has_event(b)`;
- `transition.outcome ∈ val(s)` iff `r = Ok(label)`, with value `label`;
- `transition.error ∈ val(s)` iff `fallible(b) ∧ r = Err(e)`, with the
  selected rendering of `e`;
- no key in `val(s)` starts with `transition.` unless it is in `N`.

### Invariants and lemmas

- `INV-exact-names`: `statelet::fields` exports exactly five constants whose
  values are the five strings of `REQ-names`.
- `INV-well-formed`: every name is lowercase ASCII, consists of dot-separated
  non-empty `[a-z_]+` segments, starts with `transition.`, and no name is a
  dot-prefix of another (so no name is both a field and a namespace).
- `INV-form-equivalence`: for any boundary, the dotted-literal declaration and
  the `{fields::...}` declaration produce identical `decl(s)` and `val(s)`.
- `INV-presence`: the presence equations in the abstract model hold for the
  downstream fixture's fallible-success, fallible-error, and infallible runs.
- `INV-absence-not-default`: a declared `Empty` field that is never recorded
  is absent from `val(s)`, not present with an empty or placeholder value.
- `INV-no-tracing-runtime`: `Cargo.toml` `[dependencies]` contains no
  `tracing` crate and no `[features]` table is introduced.
- `INV-docs-agree`: the ADR 0008 contract table, the design §9 field list, the
  users' guide table, and `statelet::fields` name the same five fields, and
  none of those sections names another `transition.*` field.
- `L-declaration-required` (connecting `AXIOM-undeclared-dropped` to D5):
  if a late field is not declared at creation, its value is lost; therefore
  the presence rules are necessary, not stylistic.

### Axioms (trusted, not verified here)

- `AXIOM-dotted-and-const-names`: `tracing-attributes` 0.1.31 accepts dotted
  identifiers and brace-enclosed constant expressions as field names, and
  uses the string value of the constant as the field name (docs.rs
  `tracing::instrument`, "Adding fields"). Depended on by D2 and
  `INV-form-equivalence`; exercised by the downstream tests.
- `AXIOM-undeclared-dropped`: `Span::record` ignores fields absent from the
  span's metadata field set (tracing documentation; confirmed by the probe).
  Exercised by the negative-control scenario.
- `AXIOM-current-span`: `Span::current()` resolves through the dispatcher's
  `Subscriber::current_span`. Depended on by the capture subscriber.
- `AXIOM-with-default-thread-scope`: `tracing::subscriber::with_default`
  scopes the dispatcher to the current thread for the closure's duration, so
  parallel tests do not observe each other's spans.

### Methods per obligation

- Obligation: `INV-exact-names`. Method: `rstest` parameterized table pairing
  each constant with an expected literal written in the test, plus a
  completeness check that the table has exactly five distinct rows. Rationale:
  the domain is five values; a table is exhaustive. Artefact:
  `tests/transition_fields/contract.rs`, `names_are_exact`. Evidence: red
  before `src/fields.rs` exists (compile failure on the unresolved path
  `statelet::fields`), green after. Non-vacuity: the expected literals come
  from test data, not from the constants; seeded fault: change
  `TRANSITION_STATE_BEFORE` to `"transition.state_before"` locally and confirm
  `names_are_exact::case_2` fails, then revert.
- Obligation: `INV-well-formed`. Method: `rstest` table over the five
  constants applying a test-local grammar predicate, plus a pairwise
  dot-prefix check. Artefact: `contract.rs`, `names_are_well_formed` and
  `no_name_is_a_namespace_of_another`. Non-vacuity: the predicate has its own
  negative cases (`"Transition.name"`, `"transition..event"`,
  `"transition.state"` alongside `"transition.state.before"`) that it must
  reject.
- Obligation: `INV-form-equivalence`. Method: behavioural test that runs two
  downstream fixtures, one declared with dotted literals and one with
  `{statelet::fields::...}` constants, under the capture subscriber and
  compares the captured `decl` and `val` with `pretty_assertions::assert_eq!`.
  Artefact: `tests/transition_fields/downstream.rs` and
  `tests/transition_fields.rs`, `literal_and_constant_forms_agree`
  (parameterized over success and error runs). Non-vacuity: before
  comparing, assert that each captured span declares all five fields and
  records at least four values, so two empty captures cannot agree vacuously.
- Obligation: `INV-presence` and `INV-absence-not-default`. Method:
  `rstest-bdd` scenarios backed by exact full-map assertions using
  `googletest` matchers (`has_entry`, `not(contains_key)`) on the captured
  span. Artefact: `tests/features/transition_fields.feature`,
  `tests/transition_fields/behaviour.rs`. Evidence: scenarios fail before the
  fixture records outcome or error (red), pass after. Non-vacuity: each
  scenario asserts both presence of the expected fields and absence of the
  others; the success scenario's `transition.error` absence is meaningful
  only because the error scenario shows the same fixture can record it.
- Obligation: `L-declaration-required` / `AXIOM-undeclared-dropped`. Method:
  negative-control BDD scenario with a fixture that records
  `transition.outcome` without declaring it. Expected evidence: the captured
  span lacks `transition.outcome`. Non-vacuity: the same recording call in the
  declared fixture does produce the value (paired scenario).
- Obligation: capture-subscriber faithfulness. Method: unit tests of the
  subscriber through the real `tracing` macros (`tracing::info_span!` with a
  declared `Empty` field, then `record`). Artefact:
  `tests/transition_fields/capture.rs`. Non-vacuity: one test records via
  `Span::current()` inside an entered span (the case the probe found broken),
  and one asserts that a span with no recorded late field is captured without
  that key.
- Obligation: `INV-no-tracing-runtime`. Method: parse `Cargo.toml` with the
  existing `toml` dev-dependency and assert that `dependencies` has no
  `tracing` or `tracing-core` key and that no `features` table exists; the
  existing `make feature-matrix` no-default and default `--lib` checks remain
  the compile-level evidence. Artefact: `contract.rs`,
  `runtime_has_no_tracing_dependency`. Non-vacuity: assert that
  `dev-dependencies` does contain `tracing`, proving the test reads the right
  manifest.
- Obligation: `INV-docs-agree`. Method: document-agreement test that reads
  `docs/adrs/adr-0008-*.md`, `docs/design.md`, and `docs/users-guide.md` with
  `include_str!`, slices each named contract section by its heading, extracts
  backticked tokens matching `transition.[a-z_.]+`, and compares the sets with
  the constants. Artefact: `tests/transition_fields/docs_agree.rs`.
  Non-vacuity: assert each extracted set is non-empty and that the section
  slicer returns an error (not an empty slice) when a heading is missing;
  seeded fault: add `transition.state.after` to the users' guide table
  locally and confirm failure, then revert.
- Obligation: rustdoc usability. Method: one runnable doctest on
  `statelet::fields` that instruments a local function with all five
  constants and records the outcome. Evidence: `cargo test --doc` (part of
  `make test`).

### Formal verification decision

No Verus proof, Kani harness, or proptest property is warranted (D10). The
change introduces five constant strings and documentation; there is no
business logic, arithmetic, state transition, ordering, or unbounded input
domain inside Statelet. Every Statelet-owned invariant (`INV-exact-names`,
`INV-well-formed`, `INV-no-tracing-runtime`, `INV-docs-agree`) ranges over a
finite set of five values or a fixed set of files, so an `rstest` table is
already exhaustive; a Verus lemma would restate the constants' definitions,
which the execplans guidance names as a vacuous proof. The behavioural
invariants (`INV-form-equivalence`, `INV-presence`) are properties of the
interaction with `tracing`, a third-party interface treated as axioms; they
are exercised against the real interface rather than modelled. `insta` adds
no value: the observable output is a small map asserted in full with
`pretty_assertions`, and a snapshot would duplicate those assertions while
adding a dependency. Stronger methods would add confidence only about
`tracing` internals, which this plan does not verify.

### Suggested BDD features

Proposed path: `tests/features/transition_fields.feature`, executed by
`tests/transition_fields/behaviour.rs` under `make test`. Links: `REQ-plain-
instrument`, `GAP-declaration`, `EP-M2`.

```gherkin
Feature: Transition tracing fields with ordinary instrument
  A downstream transition boundary instrumented with plain
  tracing::instrument records the contracted transition fields.

  Scenario: A fallible transition that succeeds records its outcome
    Given a fallible boundary in state "Normalize" handling a "table_row" input
    When the boundary returns the outcome "stay"
    Then the span records the transition name "handle_line"
    And the span records state "Normalize", event "table_row", and outcome "stay"
    And the span does not record a transition error

  Scenario: A fallible transition that fails records its error
    Given a fallible boundary in state "Normalize" handling a "table_row" input
    When the boundary returns the error "unterminated table row"
    Then the span records the transition error "unterminated table row"
    And the span does not record a transition outcome

  Scenario: An infallible transition never records an error
    Given an infallible boundary in state "TightCodeSpan" handling a "fence" input
    When the boundary returns the outcome "move"
    Then the span records state "TightCodeSpan", event "fence", and outcome "move"
    And the span does not declare a transition error

  Scenario: Constant and literal field names are interchangeable
    Given the same fallible boundary declared once with statelet constants and once with dotted literals
    When both boundaries return the outcome "stay"
    Then both spans declare and record identical fields

  Scenario: An undeclared late field is lost
    Given a boundary that declares only the transition name
    When the boundary records the outcome "stay"
    Then the span does not record a transition outcome
```

## Plan of work

### `EP-M0` feasibility probe (completed during planning)

Question: does `#[instrument(fields({crate::path::CONST} = ...))]` with
dotted constant values produce the same fields as dotted literals, and does a
handwritten subscriber observe late `record` calls? Done in a scratch crate
under `target/probe-2-1-2/` (deleted afterwards) against the locked
`tracing` 0.1.44 and `tracing-core` 0.1.36. Result: go. Transcript in
`Artefacts and notes`.

### `EP-M1` contract recorded and constants exported

Assigned: `REQ-names`, `REQ-semver`, `REQ-no-tracing-dep`,
`REQ-runtime-owns-vocabulary`, `GAP-meaning`, D1–D8.

1. Red. Add `tracing = "0.1.44"` and `tracing-core = "0.1.36"` to
   `[dev-dependencies]` in `Cargo.toml`, each with a comment naming its
   purpose, "D8 in the 2.1.2 plan", and "already resolved through
   `rstest-bdd`; adds no package". Create `tests/transition_fields.rs` with a
   `//!` module doc and `#[path]` declarations for `contract`, then
   `tests/transition_fields/contract.rs` holding `names_are_exact`,
   `names_are_well_formed`, `no_name_is_a_namespace_of_another`, and
   `runtime_has_no_tracing_dependency`. Run
   `cargo nextest run --test transition_fields 2>&1 | tee /tmp/red-m1-statelet-2-1-2.out`;
   expect a compile error naming the unresolved module `statelet::fields`.
2. Green. Create `src/fields.rs` with a `//!` doc stating the contract summary
   and linking ADR 0008, five `pub const` items with `///` docs giving each
   field's meaning, presence, and value rules, and one runnable doctest using
   `#[tracing::instrument(skip_all, fields({statelet::fields::...} = ...))]`
   on a local function that records `transition.outcome`. Add
   `pub mod fields;` to `src/lib.rs` with a one-line crate-doc mention. Rerun
   the focused tests; expect all green.
3. Write `docs/adrs/adr-0008-publish-the-transition-tracing-field-contract.md`
   using the style guide's ADR template: status `Accepted` with the approval
   date, context (the conflicts listed above), decision (D1, D2, D4–D7 as a
   contract table with columns field, meaning, recorded when, value rules),
   semantic-versioning section, consequences (including ADR 0003
   conditionality and the RFC 0001 amendment route), and alternatives
   (documentation only, newtype, `tracing`-typed constants, a `tracing`
   feature).
4. Update `docs/design.md` §9: split it into "9.1 Field contract" (a table of
   the five fields matching ADR 0008, the reserved `transition.` prefix, the
   semver rule, and the `statelet::fields` constants) and "9.2 Generated
   instrumentation" (the existing macro paragraphs, unchanged in meaning).
   Add ADR 0008 to the companion-document list. Do not touch §6.1.
5. Refactor. Run `make fmt`, then the focused tests.

Acceptance: `cargo nextest run --test transition_fields` passes;
`cargo test --doc` passes the new doctest; `make check-fmt`, `make lint`, and
`make test` pass. Commit: "Publish the transition field name constants".

### `EP-M2` downstream instrumentation proven

Assigned: `REQ-plain-instrument`, `GAP-evidence`, `GAP-declaration`, D5, D9.

1. Red. Add `tests/transition_fields/capture.rs` (the capture subscriber:
   `Capture` with a span map and a span stack, `CapturedSpan { name,
   declared: Vec<String>, values: BTreeMap<String, String> }`, a `Visit`
   implementation recording `record_str` verbatim and `record_debug` via
   `format!("{value:?}")`, `current_span`, and a `capture(|| ...)` helper
   built on `tracing::subscriber::with_default` that returns the captured
   spans). Add `tests/transition_fields/downstream.rs` with a local
   `Mode` enum implementing `StateName`, a local `Decision` enum with a
   `label()` method, a local error type implementing `Display` only, and
   four instrumented boundaries: fallible with constants, fallible with
   literals, infallible with constants, and an undeclared-late-field
   negative control. Initially the boundaries return without recording
   outcome or error. Add `tests/features/transition_fields.feature` and
   `tests/transition_fields/behaviour.rs`. Run the focused suite; expect the
   outcome and error scenarios to fail on the missing keys and the
   negative-control scenario to pass.
2. Green. Record `transition.outcome` or `transition.error` in the fixtures
   with `tracing::Span::current().record(...)`. Add
   `literal_and_constant_forms_agree` and the capture-subscriber unit tests.
   Rerun; expect green.
3. Seeded faults (not committed): remove the `Empty` declaration of
   `transition.outcome` from the constant fixture and confirm the success
   scenario fails; make the capture subscriber's `current_span` return
   `Current::none()` and confirm the capture unit test fails. Revert both and
   record the observed failures in `Artefacts and notes`.

Acceptance: focused suite green; seeded faults rejected; `make check-fmt`,
`make lint`, and `make test` pass. Commit: "Prove transition fields with
plain instrument".

### `EP-M3` documentation agreement and delivery closure

Assigned: `GAP-drift`, `REQ-semver`, documentation duties.

1. Red. Add `tests/transition_fields/docs_agree.rs` before the users' guide
   section exists; expect failure reporting the missing heading.
2. Green. In `docs/users-guide.md`, update "Current status" (the runtime API
   is now `StateName` and the `statelet::fields` constants) and add
   "Transition tracing fields" after "Naming a state without `Debug`": a
   captioned table of the five fields, the presence and value rules in prose,
   the `skip_all` and declare-before-record warnings, the semver and
   conditional-publication statement, and links to the rustdoc example and
   ADR 0008. No Rust fence.
3. `docs/developers-guide.md`: add "Transition field contract" covering the
   reuse policy for `statelet::fields` (applications and a future macro
   reference the constants; nothing else in the crate redefines the strings;
   new fields require an ADR 0008 amendment, a constant, and updated
   agreement tests), the capture subscriber's scope (test-only, local to
   `tests/transition_fields/`, not a shared utility, promoted only when a
   second suite needs it), and how the agreement test reads the documents.
4. `docs/context.md`: add a "Transition field contract" glossary entry.
   `docs/contents.md`: list ADR 0008. `docs/repository-layout.md`: add
   `src/fields.rs`, `tests/transition_fields.rs`,
   `tests/transition_fields/`, and `tests/features/transition_fields.feature`.
5. Run `make fmt`, `make markdownlint`, `make nixie`, then `make check-fmt`,
   `make lint`, `make test`. Measure budgets with the `Tolerances` commands.
6. Mark roadmap 2.1.2 `[x]` in `docs/roadmap.md`, update this plan's
   `Progress`, `Outcomes & retrospective`, and status to `COMPLETE`.

Acceptance: all gates pass; the seeded document fault is rejected; roadmap
2.1.2 is checked. Commit: "Document the transition field contract".

## Milestones and plateaus

- `EP-M1`: the contract exists in ADR 0008 and design §9, and the constants
  compile and are pinned. Requirements: `REQ-names`, `REQ-semver`,
  `REQ-no-tracing-dep`, `REQ-runtime-owns-vocabulary`, `GAP-meaning`.
  Conformance check: the only public addition is the five constants; no
  runtime dependency or feature; §6.1 untouched; ADR 0004 and 0002 clauses
  unchanged; trace links current. Recovery: revert the milestone commit.
  Remaining gaps: behavioural evidence and guide text. Compatibility: none
  (pre-1.0, unreleased).
- `EP-M2`: plain `#[instrument]` usage is proven by capture tests and BDD.
  Requirements: `REQ-plain-instrument`, `GAP-evidence`, `GAP-declaration`.
  Conformance check: tests compile as an external crate; no test-only API in
  `src/`; no new resolved package. Recovery: revert the milestone commit; M1
  remains coherent. Remaining gaps: guide text and agreement test.
  Compatibility: none.
- `EP-M3`: documents agree with code under test, and the roadmap is updated.
  Requirements: `GAP-drift`, documentation duties. Conformance check: every
  `Constraints` item rechecked against the full diff; budgets measured.
  Recovery: revert the milestone commit. Remaining gaps: none for 2.1.2;
  2.1.3 owns the usage pattern. Compatibility: none.

## Concrete steps

Run from the repository root, sequentially, with `tee` logs:

```bash
cargo nextest run --test transition_fields 2>&1 | tee /tmp/focused-statelet-2-1-2-transition-tracing-field-contract.out
make check-fmt 2>&1 | tee /tmp/check-fmt-statelet-2-1-2-transition-tracing-field-contract.out
make lint 2>&1 | tee /tmp/lint-statelet-2-1-2-transition-tracing-field-contract.out
make test 2>&1 | tee /tmp/test-statelet-2-1-2-transition-tracing-field-contract.out
make markdownlint 2>&1 | tee /tmp/markdownlint-statelet-2-1-2-transition-tracing-field-contract.out
make nixie 2>&1 | tee /tmp/nixie-statelet-2-1-2-transition-tracing-field-contract.out
```

Expected focused output after `EP-M2` resembles:

```plaintext
PASS [ ... ] statelet::transition_fields contract::names_are_exact::case_1
...
PASS [ ... ] statelet::transition_fields behaviour::an_undeclared_late_field_is_lost
Summary [ ... ] N tests run: N passed, 0 skipped
```

## Validation and acceptance

The work is accepted when:

- `make check-fmt`, `make lint`, `make test`, `make markdownlint`, and
  `make nixie` pass on the final commit;
- `tests/transition_fields.rs` passes, including every BDD scenario above;
- the `statelet::fields` doctest compiles and runs under `cargo test --doc`;
- the seeded faults listed in `EP-M1`, `EP-M2`, and `EP-M3` were each
  observed to fail and were reverted;
- `cargo tree -e normal` for `statelet` lists no dependencies, and the
  resolved package count in `Cargo.lock` is unchanged;
- roadmap 2.1.2 is marked `[x]`.

## Idempotence and recovery

Every step is re-runnable. `make fmt` is idempotent apart from the known
`mdtablefix --renumber` hazard: never let a wrapped prose line start with
`<digit>.` in edited documents. Seeded faults are applied to a clean tree and
reverted with `git restore <path>`; check `git status --short` afterwards.
Each milestone is one commit and can be reverted independently in reverse
order.

## Artefacts and notes

Probe transcript (`EP-M0`, 2026-11-16, scratch crate deleted afterwards).
Rows 1–2 use dotted literals, rows 3–4 use `{fakelet::fields::...}` constants
(rows 2 and 4 are error runs), row 5 records an undeclared field, and row 6
uses a bare `fields(transition.outcome)` declaration:

```plaintext
1 literal declared=[name, state.before, event, outcome, error]
  values={event: table_row, name: handle_line, outcome: stay, state.before: Normalize}
2 literal declared=[name, state.before, event, outcome, error]
  values={error: bad row, event: table_row, name: handle_line, state.before: Normalize}
3 constant declared=[name, state.before, event, outcome, error]
  values={event: table_row, name: handle_line, outcome: stay, state.before: Normalize}
4 constant declared=[name, state.before, event, outcome, error]
  values={error: bad row, event: table_row, name: handle_line, state.before: Normalize}
5 undeclared declared=["transition.name"] values={"transition.name": "handle_line"}
6 bare declared=["transition.name", "transition.outcome"] values={"transition.name": "handle_line"}
```

(The `transition.` prefix is elided in rows 1–4 for width.) Before
`current_span` was implemented, rows 1–4 showed no `outcome` or `error`
values.

## Interfaces and dependencies

At the end of `EP-M1`, `src/fields.rs` must define exactly:

```rust
//! Transition tracing field names (see ADR 0008).

/// Stable label naming the transition boundary.
pub const TRANSITION_NAME: &str = "transition.name";
/// `StateName` label of the state observed when the boundary is entered.
pub const TRANSITION_STATE_BEFORE: &str = "transition.state.before";
/// Semantic class of the input the boundary handles.
pub const TRANSITION_EVENT: &str = "transition.event";
/// Label of the result the boundary returned on normal completion.
pub const TRANSITION_OUTCOME: &str = "transition.outcome";
/// Selected rendering of the error a fallible boundary returned.
pub const TRANSITION_ERROR: &str = "transition.error";
```

with fuller `///` documentation than shown. `src/lib.rs` gains
`pub mod fields;`. Dev-dependencies gain `tracing = "0.1.44"` and
`tracing-core = "0.1.36"`. No other public item or dependency changes.

## Revision note

Initial draft, 2026-11-16. Pending community-of-experts review.
