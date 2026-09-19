# Documentation contents

[Documentation contents](contents.md) is the index for Statelet's documentation
set.

## Project guides

- [User guide](users-guide.md) explains the current user-facing Statelet
  status, expected user model, and design signposts.
- [Developer guide](developers-guide.md) explains the local workflow and
  implementation tooling for contributors.
- [Repository layout](repository-layout.md) explains the generated project's
  top-level files, directories, and ownership boundaries.
- [Documentation style guide](documentation-style-guide.md) defines the
  spelling, structure, Markdown, Architecture Decision Record (ADR), Request
  for Comments (RFC), and roadmap conventions used by this documentation set.
- [Phase 2 validation note template](phase-2-validation-note-template.md) is
  the blank form a Phase 2 engineer copies to record a `StateName` consumption
  observation; it is the schema ADR 004's rule reads, not design material.
- [Validation notes](validation-notes/README.md) is the directory those filled
  forms are committed to, and explains how a note is named and marked.

## Execution plans

- [`docs/execplans/`](execplans/) holds the living design-and-delivery
  documents written before larger tasks:
  - [Record the transition-boundary scope decision as an ADR](execplans/1-1-1-record-the-transition-boundary-scope-decision-as-an-adr.md)
    carries ADR 002 from its question to its acceptance criteria.
  - [Record the three possible v0.1 exits](execplans/1-1-2-record-the-three-possible-v0-1-exits.md)
    carries ADR 003 and the exit register that selects among them.
  - [Define the `StateName` consumption question](execplans/1-1-3-define-state-name-consumption-question.md)
    carries ADR 004 and the contract test that guards it.

## Product and design

- [Terms of reference](terms-of-reference.md) defines the problem space,
  intended users, market gap, scope, and validation test for Statelet.
- [Context](context.md) defines the working glossary and document map for the
  initial Statelet design work.
- [Technical design](design.md) defines the initial crate split, macro
  contract, feature policy, and validation strategy.
- [Roadmap](roadmap.md) translates the terms of reference and technical design
  into sequenced delivery phases, dependencies, and validation tasks.

## Decision records

- [ADR 001: Select proving ground candidates](adr-001-proving-ground-candidates.md)
  proposes `wireframe` as the primary proving ground after `mdtablefix`, with
  `weaver`, `netsuke`, and `ddlint` ranked as narrower or weaker candidates.
- [ADR 002: Scope Statelet to transition-boundary marking](adr-002-transition-boundary-scope.md)
  defines the marker-only ownership boundary that keeps Statelet out of
  framework responsibilities.
- [ADR 003: Record the v0.1 exit register](adr-003-v0-1-exit-register.md)
  defines every v0.1 release scope and the evidence gate that selects it.
- [ADR 004: Define the StateName consumption evidence](adr-004-state-name-consumption-evidence.md)
  defines what a `StateName` consumption observation records and the rule that
  reads one note, or several, into an outcome at task 3.2.1.

## Rust reference material

- [Reliable testing in Rust via dependency injection](reliable-testing-in-rust-via-dependency-injection.md)
  explains how to keep tests deterministic by injecting environment, clock,
  filesystem, and other external dependencies.
- [Rust doctest Don't Repeat Yourself guide](rust-doctest-dry-guide.md)
  explains how to write maintainable, executable Rust documentation examples.
- [Rust testing with `rstest` fixtures](rust-testing-with-rstest-fixtures.md)
  explains fixture-based, parameterized, and asynchronous testing with `rstest`.

## Engineering practice

- [Complexity antipatterns and refactoring strategies](complexity-antipatterns-and-refactoring-strategies.md)
  explains cognitive complexity, the bumpy-road antipattern, and refactoring
  approaches for maintainable code.
- [Scripting standards](scripting-standards.md) explains the preferred Python
  scripting stack, command execution patterns, and test expectations for helper
  scripts.

## Imported tooling references

- [`rstest-bdd` user guide](rstest-bdd-users-guide.md) is an imported snapshot
  of prior proc-macro crate documentation used as Statelet design reference
  material.
- [`lading` user guide](lading-users-guide.md) is an imported snapshot of
  release-workflow tooling documentation used to inform dependency and release
  validation decisions.
- [`whitaker` user guide](whitaker-users-guide.md) is an imported snapshot of
  lint tooling documentation used to inform local validation and hygiene
  decisions.
