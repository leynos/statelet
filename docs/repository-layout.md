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
│   ├── users-guide.md
│   └── ...
├── src/

│   └── lib.rs

├── tests/
│   ├── stub.rs
│   ├── state_name_consumption_contract.rs
│   ├── state_name_consumption_contract/
│   └── v0_1_exit_register_contract.rs
├── AGENTS.md
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
- `.github/dependabot.yml`: Configures automated dependency update checks.
- `.github/workflows/act-validation.yml`: Runs the generated workflow
  validation through `act` separately from main CI.
- `.github/workflows/ci.yml`: Runs the generated project's continuous
  integration checks.

- `docs/`: Holds long-lived reference documentation, guides, style rules, and
  design material.
- `docs/contents.md`: Indexes the documentation set and should be updated when
  documentation files are added, renamed, or removed.
- `docs/users-guide.md`: Explains how to use the generated project and its
  public build and test commands.
- `docs/developers-guide.md`: Explains the contributor workflow and local
  tooling used to work on the generated project.
- `docs/repository-layout.md`: Documents the repository tree and path
  responsibilities.

- `src/lib.rs`: Contains the library crate root and exported public API
  surface.

- `tests/`: Holds integration and behavioural tests that exercise public
  behaviour.
- `tests/stub.rs`: Keeps the generated test directory valid until real tests
  replace it.
- `tests/v0_1_exit_register_contract.rs`: Guards ADR 003 against drift in its
  exit register, cited evidence, and roadmap gates.
- `tests/state_name_consumption_contract.rs`: Guards ADR 004 against drift in
  its status and aggregation registers, its gate table, the validation note
  template, and the source clauses it quotes. It also reads
  `docs/validation-notes/` to check any committed note carrying the
  `<!-- state-name-note -->` marker.
- `tests/state_name_consumption_contract/`: Holds that contract's private
  child modules: the table parser, the register vocabulary, the note policy and
  its verdict checks, the cross-register consistency checks, quoted-clause
  resolution, the notes-directory scan, the row fixtures, and the four scenario
  modules — `anchor_scenarios.rs` for the template and roadmap bindings,
  `note_scenarios.rs` for note cells and their verdict, `register_scenarios.rs`
  for the registers, and `scan_scenarios.rs` for the directory scan.
- `docs/validation-notes/`: Holds the filled validation notes, one per task,
  named `<task>-<subject>.md`. Shared by several Phase 2 and Phase 3 decisions;
  a note declares which contract reads it with a marker comment.
- `docs/phase-2-validation-note-template.md`: The blank form a validation note
  is instantiated from. It is the schema ADR 004's rule reads, not design
  material, and it is never edited to record an observation.
- `dylint.toml`: Configures Whitaker lint behaviour not expressible in Rust
  source. It exempts one module — `state_name_consumption_contract::notes` —
  from `no_std_fs_operations`, which denies `std::fs` in integration-test
  crates and cannot be suppressed by any attribute. The exemption is
  path-scoped so the rest of that contract stays under the lint.
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
