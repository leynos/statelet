# Statelet

*Transition-boundary conventions for ordinary Rust state machines.*

Statelet is being designed for teams that already write explicit Rust state
machines with enums, structs, methods, and `match` expressions. It does not run
the machine, own the dispatch loop, or ask the code to become a graph DSL. Its
job is narrower: make transition boundaries easier to name, observe, and review.

This repository is currently in the design and bootstrap phase. The first
validation slice is deliberately non-macro: prove that runtime conventions,
stable state names, and documented tracing fields help real `mdtablefix` code
before publishing a procedural macro.

______________________________________________________________________

## Why Statelet?

- **Ordinary Rust stays ordinary**: branch logic, state mutation, errors, and
  side effects remain in project code.
- **The framework boundary is explicit**: use `statig`, `smlang`,
  `typed-fsm`, `rust-fsm`, or `stateless` when you want a machine definition or
  transition table. Use Statelet only if the machine already exists.
- **Observability is the product pressure**: the first useful surface is a
  shared state-name and tracing-field convention, not a runtime engine.
- **The macro must earn its place**: `#[statelet::transition]` is deferred
  until it beats `#[tracing::instrument]` plus local helpers on real code.

______________________________________________________________________

## Quick start

### Installation

Clone the repository and run the generated project gates:

```bash
git clone https://github.com/leynos/statelet.git
cd statelet
make all
```

### Basic usage

The current crate skeleton is intentionally small. The useful "hello world" is
to validate the repository and read the design gates before adding API:

```bash
make all
sed -n '1,120p' docs/roadmap.md
```

For the proposed user-facing workflow, start with the design and roadmap rather
than a speculative code sample:

- [Technical design](docs/design.md)
- [Roadmap](docs/roadmap.md)
- [Terms of reference](docs/terms-of-reference.md)

______________________________________________________________________

## Features

Current repository state:

- Rust 2024 crate skeleton with strict lint, formatting, and documentation
  gates.
- Terms of reference defining the market gap, non-goals, and validation test.
- Technical design sequencing a runtime conventions crate before any macro
  crate.
- Roadmap with explicit dependencies, kill gates, and validation signposts.

Planned core, pending validation:

- `StateName` conventions for cheap, stable state names.
- Semver-relevant tracing field names for transition observability.
- A conventions-only `mdtablefix` spike that can stop the project if the value
  is too weak.
- A procedural macro only if real examples prove that the boilerplate is worth
  hiding behind an attribute.

______________________________________________________________________

## Learn more

- [Users' Guide](docs/users-guide.md) - generated project commands and local
  usage notes.
- [Developers' Guide](docs/developers-guide.md) - contributor workflow and
  tooling.
- [Documentation contents](docs/contents.md) - the full documentation index.
- [Terms of reference](docs/terms-of-reference.md) - problem space, users, and
  validation criteria.
- [Technical design](docs/design.md) - crate boundaries, public API decisions,
  and validation gates.
- [Roadmap](docs/roadmap.md) - delivery phases, dependencies, and exit
  criteria.

______________________________________________________________________

## Licence

ISC - see [LICENSE](LICENSE) for details.

______________________________________________________________________

## Contributing

Contributions are welcome. Please read [AGENTS.md](AGENTS.md) and the
[Developers' Guide](docs/developers-guide.md) before changing the repository,
then keep changes small, gated, and committed with the relevant validation
output.
