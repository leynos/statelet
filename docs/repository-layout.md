# Repository layout

This document describes the generated Statelet repository layout. It is the
canonical reference for where source code, tests, configuration, automation,
and long-lived documentation belong.

## Top-level tree

The tree below shows the generated repository structure. It is intentionally
compact and omits build output such as `target/`.

```plaintext
.
├── .cargo/
│   └── config.toml
├── .github/
│   ├── dependabot.yml
│   └── workflows/
│       ├── act-validation.yml
│       ├── ci.yml

├── docs/
│   ├── contents.md
│   ├── developers-guide.md
│   ├── repository-layout.md
│   ├── rfcs/
│   ├── rust-baseline-execution.md
│   ├── users-guide.md
│   └── ...
├── src/

│   └── lib.rs

├── tests/
│   ├── state_name_consumption_contract.rs
│   ├── state_name_consumption_contract/
│   ├── topology_check.rs
│   ├── topology_check/
│   ├── topology_command.rs
│   ├── workflow_contracts/
│   └── v0_1_exit_register_contract.rs
├── scripts/
│   ├── build-tools-common.sh
│   ├── check-build-tools.sh
│   └── install-build-tools.sh
├── tools/
│   └── mold/
│       ├── SHA256SUMS
│       └── VERSION
├── AGENTS.md
├── .python-version
├── Cargo.toml
├── LICENSE
├── Makefile
├── README.md
├── clippy.toml
├── codecov.yml
├── dylint.toml
└── rust-toolchain.toml
```

## Path responsibilities

- `.cargo/config.toml`: Configures Cargo defaults for local development,
  including Linux linker and code-generation settings.
- `.python-version`: Selects managed CPython 3.14 for direct `uv` runs.
- `.github/dependabot.yml`: Configures automated dependency update checks.
- `.github/workflows/act-validation.yml`: Runs the generated workflow
  validation through `act` separately from main CI.
- `.github/workflows/ci.yml`: Runs the generated project's continuous
  integration checks.

- `docs/`: Holds long-lived reference documentation, guides, style rules, and
  design material.
- `docs/contents.md`: Indexes the documentation set and should be updated when
  documentation files are added, renamed, or removed.
- `docs/rust-baseline-execution.md`: Records selected lint and tooling
  baselines, measured coverage, and delivery evidence for the hardening change.
- `docs/users-guide.md`: Explains how to use the generated project and its
  public build and test commands.
- `docs/developers-guide.md`: Explains the contributor workflow and local
  tooling used to work on the generated project.
- `docs/repository-layout.md`: Documents the repository tree and path
  responsibilities.
- `docs/rfcs/`: Holds the numbered Requests for Comments that propose, and
  record the disposition of, integration contracts not yet accepted into the
  design. An RFC states requirements, alternatives, validation and
  falsification controls, and its own status; it is a proposal record, not a
  binding contract, and only an accepted ADR or the technical design carries
  authority. Each RFC is named `<number>-<subject>.md` and is indexed from
  `docs/contents.md`.

- `src/lib.rs`: Contains the library crate root and exported public API
  surface.

- `tests/`: Holds integration and behavioural tests that exercise public
  behaviour.
- `tests/v0_1_exit_register_contract.rs`: Guards ADR 003 against drift in its
  exit register, cited evidence, and roadmap gates.
- `tests/state_name_consumption_contract.rs`: Guards ADR 004 against drift in
  its status and aggregation registers, its gate table, the validation note
  template, and the source clauses it quotes. It also reads
  `docs/validation-notes/` to check any committed note carrying the
  `<!-- state-name-note -->` marker.
- `tests/macro_cost_note_contract.rs`: Checks the macro cost note template,
  its design and roadmap bindings, complete marked notes, and candidate-bound
  evidence selected by roadmap tasks 4.2.2 and 4.3.1. Its capability-filesystem
  adapter treats a missing notes directory as empty and reports read failures
  with the affected path.
- `tests/macro_cost_note_contract/`: Holds the private contract checker and
  in-memory forms and named cases. The schema stays within this integration
  test; it is not a general Markdown parser or shared runtime utility.
- `tests/topology_check.rs`: Guards the pre-macro workspace boundary named in
  design section 11.6. It applies a closed policy to Cargo's own metadata for
  this workspace, and it names the guard `make topology-check` selects.
- `tests/topology_check/`: Holds that check's private child modules —
  `policy.rs` for the pure evaluator, `metadata.rs` for the acquisition adapter
  and its two scoped environment reads, `fixtures.rs` and `policy_cases.rs` for
  the accepted document and the named partitions, `claim_properties.rs` for the
  metamorphic properties, and `behaviour.rs` for the `rstest-bdd` steps backing
  `tests/features/topology_check.feature`. The policy module is the only one
  that decides a verdict; the adapter is the only one that spawns a process.
- `tests/topology_command.rs`: Exercises `make topology-check` itself against
  isolated Cargo fixtures, one make invocation per case. Its cases are ignored
  by default and run only through `make test-topology-command`, because each
  starts a nested Cargo process that routine tests and coverage must not host.
- `tests/features/topology_check.feature`: The Gherkin scenarios the topology
  check's behavioural suite runs.
- `tests/workflow_contracts/topology_wiring_contract.py` and
  `tests/workflow_contracts/topology_wiring_test.py`: Guard the topology gate's
  routine wiring. The contract module evaluates `make -n` output for `all`,
  `test`, and `check-topology`, parses `ci.yml` with the strict workflow
  reader, and holds the finding functions that require the pinned CI step, the
  post-coverage command step, the cleared manifest environment, the closed
  policy selector, and both `.NOTPARALLEL` entries. The test module supplies
  the live evidence and fifteen negative controls, one per clause and one per
  `env` scope the coverage boundary reads.
- `tests/workflow_contracts/mock_wiring.py`: The single test-only boundary for
  evaluating Make recipes without running them, shared by the workflow
  contracts that assert on `make -n` output.
- `tests/state_name_consumption_contract/`: Holds that contract's private
  child modules: the table parser, the register vocabulary, the evidence-cell
  predicates, the note policy and its verdict checks, the cross-register
  consistency checks, quoted-clause resolution, the roadmap's task-record
  grammar with the two checks that bind it, the notes-directory scan, the row
  fixtures, and the scenario modules — `anchor_scenarios.rs` for the template
  and roadmap bindings, `claims_scenarios.rs` for what an evidence cell says,
  `clause_scenarios.rs` for whether the quoted clauses resolve, and against
  which task's record a roadmap quotation is bound, `criterion_scenarios.rs`
  for which copy of a sentence is the success criterion, `note_scenarios.rs`
  for note cells and the verdict they yield, `register_scenarios.rs` for the
  registers, and `scan_scenarios.rs` for the directory scan.
  `claim_properties.rs` is not a scenario module: it holds the property suite
  over the evidence predicates — generated properties, not named scenarios.
- `docs/validation-notes/`: Holds the filled validation notes, one per task,
  named `<task>-<subject>.md`. Shared by several Phase 2 and Phase 3 decisions;
  a note declares which contract reads it with a marker comment.
- `docs/phase-2-validation-note-template.md`: The blank form a validation note
  is instantiated from. It is the schema ADR 004's rule reads, not design
  material, and it is never edited to record an observation.
- `docs/macro-cost-benchmark-note-template.md`: The blank real and synthetic
  macro cost evidence form, checked by `tests/macro_cost_note_contract.rs`.
- `dylint.toml`: Configures Whitaker lint behaviour not expressible in Rust
  source. It exempts one module — `state_name_consumption_contract::notes` —
  from `no_std_fs_operations`, which denies `std::fs` in integration-test
  crates and cannot be suppressed by any attribute. The exemption is
  path-scoped so the rest of that contract stays under the lint.
- `scripts/`: Installs and checks the pinned local build tools used by the
  standard Make targets.
- `tools/mold/`: Records the approved linker release and archive checksums used
  by the build-tool installer.
- `AGENTS.md`: Provides repository-specific working instructions for agents and
  contributors.
- `Cargo.toml`: Defines package metadata, dependencies, lint policy, and Cargo
  configuration.
- `LICENSE`: Records the project licence text.
- `Makefile`: Provides the public build, lint, test, coverage, and
  documentation validation commands.
- `README.md`: Introduces the project and gives the shortest useful
  getting-started path.
- `clippy.toml`: Configures Clippy lint behaviour that is not expressed
  directly in `Cargo.toml`.
- `codecov.yml`: Configures coverage reporting behaviour.
- `rust-toolchain.toml`: Pins the Rust toolchain channel and required
  components.

## Ownership boundaries

- Keep generated source code under `src/`. Add modules below `src/` when a
  feature grows beyond a small entrypoint or crate root.
- Keep black-box integration tests and externally observable workflow tests
  under `tests/`.
- Keep reusable documentation under `docs/`. Update `docs/contents.md` whenever
  a documentation file is added, renamed, or removed.
- Keep build and validation entrypoints in `Makefile`; prefer adding or
  extending a Make target over documenting an ad hoc command.
- Keep continuous integration workflow changes under `.github/workflows/` and
  dependency-update policy under `.github/dependabot.yml`.
- Do not commit generated build output such as `target/`, coverage artefacts,
  or local editor state.

## Updating this document

Update this document when the repository gains a new top-level directory, a new
long-lived documentation category, a new workflow file, or a changed ownership
boundary that would otherwise make the tree misleading.
