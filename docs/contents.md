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

## Product and design

- [Terms of reference](terms-of-reference.md) defines the problem space,
  intended users, market gap, scope, and validation test for Statelet.
- [Context](context.md) defines the working glossary and document map for the
  initial Statelet design work.
- [Technical design](design.md) defines the initial crate split, macro
  contract, feature policy, and validation strategy.
- [Roadmap](roadmap.md) translates the terms of reference and technical design
  into sequenced delivery phases, dependencies, and validation tasks.

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
