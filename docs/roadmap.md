# Statelet roadmap

This roadmap translates the current terms of reference and technical design
into an outcome-oriented delivery sequence. It does not promise dates. Each
phase carries one testable idea at the Goals, Ideas, Steps, and Tasks (GIST)
level: phases are ideas, steps are workstreams that validate or falsify those
ideas, and tasks are review-sized execution units.

The primary source documents are
[terms of reference](terms-of-reference.md), [technical design](design.md), and
[context glossary](context.md).
The roadmap keeps the v0.1 boundary narrow: prove that transition-boundary
conventions are useful before publishing a macro, and ship nothing if the
conventions baseline does not beat local `#[tracing::instrument]` usage on real
code.

## 1. Foundational contracts and kill gates

Idea: if Statelet settles its exit criteria, naming contract, and validation
spine before feature work starts, later slices can falsify weak bets without
leaving behind speculative public API.

This phase is foundational because the design has explicit low-confidence bets.
The first deliverable is not runtime code; it is a set of documented decisions
and gates that make "ship nothing", "ship conventions only", and "ship the
macro" distinct outcomes.

### 1.1. Ratify the v0.1 product exits

This step answers what evidence kills the crate, what evidence justifies a
runtime/conventions crate, and what evidence justifies a proc macro. Its
outcome informs every later implementation slice. See terms-of-reference.md
§§7-9 and design.md §§11.1, 13.6-13.7.

- [x] 1.1.1. Record the transition-boundary scope decision as an ADR.
  - See terms-of-reference.md §§1-6 and design.md §§1-3.
  - Success: one accepted ADR states that Statelet marks boundaries and does
    not own dispatch, events, storage, transition tables, or graph safety:
    [ADR 002](adr-002-transition-boundary-scope.md).
- [x] 1.1.2. Record the three possible v0.1 exits.
  - Requires 1.1.1.
  - Capture the "ship nothing", "ship conventions only", and "ship macro"
    outcomes.
  - Success: the exit note maps B1 and B2 to concrete evidence from
    design.md §11.1 and failure modes in design.md §§13.6-13.7:
    [ADR 003](adr-003-v0-1-exit-register.md).
- [ ] 1.1.3. Define the `StateName` consumption question for Phase 2.
  - Requires 1.1.2.
  - Decide what `mdtablefix` must consume to prove whether `&'static str` is
    enough or whether a stable numeric identifier is needed.
  - Success: the Phase 2 validation note template has fields for state display
    name, optional identifier need, metrics cardinality, and tracing use.
  - See design.md §6.1 and context.md "State name".

### 1.2. Establish the validation spine

This step answers whether local and continuous validation can enforce the
design boundaries before a second crate or feature matrix exists. Its outcome
unlocks the runtime baseline work and prevents topology checks from becoming a
late clean-up task. See design.md §§5, 10, 11.5-11.7.

- [ ] 1.2.1. Add a topology-check command placeholder for the single-crate
  state.
  - Requires 1.1.1.
  - Implement the command so it passes on the initial runtime crate and has a
    documented forbidden-edge policy for later workspace members.
  - Success: the command runs locally and explains why no proc-macro edge is
    allowed before the macro gate passes.
  - See design.md §§4-5 and §11.6.
- [ ] 1.2.2. Add a feature-matrix command for the current crate.
  - Requires 1.2.1.
  - Use `cargo-hack` or an equivalent command from the first optional-feature
    commit.
  - Success: no-default, default, tracing, serde, and all-feature checks have
    one documented entrypoint even before every feature exists.
  - See design.md §§10 and 11.5.
- [ ] 1.2.3. Add a benchmark note template for macro cost budgets.
  - Requires 1.1.2.
  - Include fields for clean debug build time, release artefact size,
    per-annotation marginal cost, and synthetic 20-transition stress fixture
    results.
  - Success: Phase 4 cannot claim the macro gate without filling the template.
  - See design.md §11.7.

## 2. Vertical slice 1: Runtime conventions in `mdtablefix`

Idea: if Statelet's runtime conventions improve `mdtablefix` without a macro,
the project has evidence that the B1 wedge exists before paying proc-macro
maintenance cost.

This phase delivers the smallest useful product shape: state naming, documented
transition fields, and local helper guidance applied to the real `mdtablefix`
parser path. It should be able to fail cleanly. If plain
`#[tracing::instrument]` plus local convention is enough, the roadmap stops
before publishing a crate.

### 2.1. Deliver the runtime convention surface

This step answers whether Statelet can expose a useful public contract without
owning transitions. Its outcome informs the `mdtablefix` baseline and the shape
of any later macro. See design.md §§3.1-3.2, 6.1, 9, and 10.

- [ ] 2.1.1. Implement the `StateName` trait and handwritten examples.
  - Requires steps 1.1-1.2.
  - Keep derive support out of this task unless the macro gate has passed.
  - Success: enum states can expose stable names without requiring `Debug` as
    the observability contract.
  - See design.md §6.1.
- [ ] 2.1.2. Publish the transition tracing field contract.
  - Requires 2.1.1.
  - Document `transition.name`, `transition.state.before`,
    `transition.event`, `transition.outcome`, and `transition.error` as
    semver-relevant operational fields.
  - Success: downstream examples can use the same fields with ordinary
    `#[tracing::instrument(fields(...))]`.
  - See design.md §9.
- [ ] 2.1.3. Document the conventions-only usage pattern.
  - Requires 2.1.1 and 2.1.2.
  - Provide a user-facing example that marks a transition boundary without
    `statelet-macros`.
  - Success: the example is useful even when the `macros` feature does not
    exist.
  - See terms-of-reference.md §§5-7 and design.md §11.2.

### 2.2. Apply the non-macro baseline to `mdtablefix`

This step answers whether Statelet's conventions improve the motivating
parser-shaped code. Its outcome decides whether the project ships nothing,
ships conventions only, or proceeds to a macro spike. See design.md §§11.1,
11.2, and 12.

- [ ] 2.2.1. Annotate `mdtablefix` `ProcessBuffer` with the conventions-only
  baseline.
  - Requires 2.1.3.
  - Use `StateName`, documented fields, local helpers, and ordinary
    `#[tracing::instrument(fields(...))]`.
  - Success: branch logic remains in ordinary Rust and the validation note can
    compare before/after reviewability.
  - See design.md §12.
- [ ] 2.2.2. Annotate `mdtablefix` continuation handling with the baseline.
  - Requires 2.2.1.
  - Cover continuation mode handling and at least one fallible or infallible
    transition boundary if the code exposes one naturally.
  - Success: the validation note records boilerplate, diagnostics, tracing
    fields, and any `StateName` identifier pressure.
  - See design.md §§6.1 and 12.
- [ ] 2.2.3. Decide the Phase 2 exit.
  - Requires 2.2.1 and 2.2.2.
  - Compare plain `#[tracing::instrument]`, the conventions baseline, and the
    original code.
  - Success: the note chooses one of the three exits from 1.1.2 and cites the
    evidence. If the conventions baseline is weak, the roadmap stops with
    "ship nothing".
  - See terms-of-reference.md §7 and design.md §§11.1, 13.6-13.7.

## 3. Vertical slice 2: A second non-toy validation domain

Idea: if the same transition-boundary conventions help `wireframe` connection
actors, Statelet is less likely to be an `mdtablefix`-specific style extraction.

This phase validates the wedge outside Markdown table repair. ADR 001 selects
`wireframe` as the candidate because connection lifecycle and active-output
transitions create explicit stateful boundaries without requiring Statelet to
own routing, protocol modelling, or graph validation.

### 3.1. Apply the conventions baseline to `wireframe`

This step answers whether Statelet's runtime conventions transfer to a
connection-actor lifecycle outside Markdown table repair. Its outcome informs
B1 and B6 before any macro work. See terms-of-reference.md §§7-9, design.md
§§11.1, 12, and
[adr-001-proving-ground-candidates.md](adr-001-proving-ground-candidates.md).

- [ ] 3.1.1. Identify `wireframe` transition-boundary candidates.
  - Requires phase 2 unless 2.2.3 chose "ship nothing".
  - Select connection actor and active-output transition boundaries that
    currently rely on local convention.
  - Success: the candidate list explains why `stateless` or another
    graph-first crate is not the more honest model.
  - See design.md Appendix A and
    [adr-001-proving-ground-candidates.md](adr-001-proving-ground-candidates.md).
- [ ] 3.1.2. Apply the conventions-only baseline to selected `wireframe`
  boundaries.
  - Requires 3.1.1.
  - Use the same `StateName` and tracing field contract proven or revised in
    Phase 2.
  - Success: the validation note records reviewability, diagnostic value,
    boilerplate, and whether the conventions carry across domains.
  - See design.md §§9, 11.1, and 12.
- [ ] 3.1.3. Decide whether the runtime/conventions crate earns publication.
  - Requires 3.1.2.
  - Success: the decision cites both `mdtablefix` and `wireframe`; it either
    stops the project, publishes conventions only, or unlocks Phase 4.
  - See terms-of-reference.md §7 and design.md §§11.1, 13.7.

### 3.2. Tighten the runtime API from validation evidence

This step answers whether the validated examples require changes before
publication. Its outcome stabilizes v0.1 runtime API and prevents speculative
types from leaking into semver. See design.md §§6.1, 6.2, and 14.

- [ ] 3.2.1. Finalize the `StateName` return shape.
  - Requires 2.2.3 and 3.1.3.
  - Decide whether `&'static str` is enough or whether a stable identifier is
    needed for low-cardinality metrics.
  - Success: the public trait shape is backed by observed example
    consumption, not anticipation.
  - See design.md §6.1.
- [ ] 3.2.2. Decide whether `TransitionOutcome` remains deferred.
  - Requires 3.2.1.
  - Publish no outcome type unless the examples consume one or a macro needs
    one later.
  - Success: the release candidate either has no `TransitionOutcome` or has a
    documented consumer and compatibility rationale.
  - See design.md §6.2.
- [ ] 3.2.3. Decide the `tracing` default.
  - Requires 3.1.3.
  - Compare dependency cost with observed user value from both validation
    domains.
  - Success: the release note records why `tracing` is default or opt-in.
  - See design.md §§3.4 and 10.

## 4. Vertical slice 3: Macro only if the baseline proves boilerplate

Idea: if real validation shows repeated boundary boilerplate that a macro can
remove without hiding control flow, `statelet-macros` can be added as a narrow
wrapper rather than a framework.

This phase is conditional. It starts only when Phase 3 chooses "ship macro". If
the runtime/conventions crate is the honest product, this phase remains
deferred.

### 4.1. Introduce the proc-macro crate without changing the model

This step answers whether a second crate can wrap existing transition methods
without changing user-owned state, event, error, storage, or dispatch models.
See design.md §§3.1, 4-7, and Appendix A.

- [ ] 4.1.1. Split the workspace to add `statelet-macros`.
  - Requires 3.1.3 choosing "ship macro".
  - Keep `statelet` usable without macro dependencies.
  - Success: topology checks show no forbidden runtime-to-macro requirement.
  - See design.md §§4-5.
- [ ] 4.1.2. Implement real-expression attribute parsing.
  - Requires 4.1.1.
  - Parse `state(self.mode)` and `event(line)` as `syn::Expr`, not string
    literals.
  - Success: diagnostics point at user-written expression tokens.
  - See design.md §§6.3, 7, and 13.8.
- [ ] 4.1.3. Implement descriptive fallibility handling.
  - Requires 4.1.2.
  - Inspect return syntax to decide whether `transition.error` can be emitted;
    make `check_return` the only hard compile gate.
  - Success: alias-blind cases omit unsupported fields unless the user opted
    into `check_return`.
  - See design.md §§8 and 11.3.

### 4.2. Prove the macro gate under realistic and synthetic load

This step answers whether the macro actually beats the conventions baseline
without unacceptable compile-time, binary-size, or feature-topology cost. See
design.md §§11.2-11.7 and 12.

- [ ] 4.2.1. Reapply the macro to the `mdtablefix` validation boundaries.
  - Requires 4.1.3.
  - Compare against the Phase 2 conventions baseline.
  - Success: the validation note states the concrete value added by the macro
    in one paragraph.
  - See design.md §§11.2 and 12.
- [ ] 4.2.2. Measure macro cost with a synthetic stress fixture.
  - Requires 4.1.3.
  - Use a 20-transition synthetic downstream crate to calculate marginal
    clean-debug build cost per annotation and release artefact growth.
  - Success: the results stay inside design.md §11.7 or the design is revised
    before the macro ships.
- [ ] 4.2.3. Add feature-matrix and topology coverage for the macro crate.
  - Requires 4.1.1.
  - Exercise no-default, default, tracing, serde, macros, derive, and
    all-feature combinations where those features exist.
  - Success: CI catches both dependency cycles and feature leakage.
  - See design.md §§5, 10, 11.5-11.6, and 13.9.

### 4.3. Decide the macro publication boundary

This step answers whether `statelet-macros` is publishable or should remain a
local experiment. Its outcome informs v0.1 release notes and future ADRs. See
design.md §§13-14.

- [ ] 4.3.1. Record the macro publication decision.
  - Requires steps 4.1-4.2.
  - Include compile-time budget, binary-size budget, topology results, and
    baseline comparison.
  - Success: one decision note either publishes the macro, defers it, or
    removes it from v0.1 scope.
- [ ] 4.3.2. Update user and developer documentation for the chosen boundary.
  - Requires 4.3.1.
  - Keep graph-first users pointed at `stateless` and other graph-owning
    crates.
  - Success: documentation shows the conventions-only path and, only if
    accepted, the macro path.
  - See design.md Appendix A and terms-of-reference.md §§3-6.

## 5. Deferred extensions after the core promise

Idea: if the core transition-boundary promise is already useful and boring to
operate, broader extensions can be evaluated on product value instead of
destabilizing v0.1.

This phase collects work the design names but deliberately defers. None of
these tasks should block the conventions baseline, the two validation domains,
or the macro gate.

### 5.1. Evaluate graph-adjacent metadata without owning the graph

This step answers whether documentation metadata can help without competing with
`stateless` or graph-first frameworks. See design.md §§2.2, 3.5, 13.3, and
Appendix A.

- [ ] 5.1.1. Decide whether diagram or test metadata remains deferred.
  - Requires phase 4 if the macro ships; otherwise requires phase 3.
  - Success: the decision either rejects graph-adjacent metadata or defines it
    as optional documentation metadata that does not shape user code.
- [ ] 5.1.2. Reassess `TransitionOutcome` after macro and validation results.
  - Requires 5.1.1 and 3.2.2.
  - Success: the type remains deferred unless a real consumer exists.
  - See design.md §6.2.

### 5.2. Evaluate advanced integration surfaces

This step answers whether async, serde, and embedded-style support belong after
the core product is proven. See design.md §§2.2, 10, and 14.

- [ ] 5.2.1. Decide whether `async fn` support belongs in the next release.
  - Requires phase 4 if the macro ships.
  - Include tracing span behaviour across `.await`.
  - Success: async support is either tested explicitly or documented as out of
    scope.
  - See design.md §§11.3 and 14.
- [ ] 5.2.2. Decide whether `serde` support has a runtime consumer.
  - Requires phase 3.
  - Success: `serde` remains absent unless a published runtime type needs it.
  - See design.md §10.
- [ ] 5.2.3. Reassess `no_std` and embedded suitability only after v0.1.
  - Requires phase 3.
  - Success: embedded claims remain out of scope unless a new ToR revision
    changes the target user.
  - See terms-of-reference.md §6.2 and design.md §2.2.
