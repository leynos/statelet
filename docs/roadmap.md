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

The observation proposals in Phase 6 are separately gated extensions. Phase
numbers remain stable identifiers, not a requirement to complete unrelated
deferred work first; each task states its prerequisites. Proposed documents
are not implementation evidence, and their existence completes no task.

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
    name, optional identifier need, metrics cardinality, and tracing use:
    [ADR 004](adr-004-state-name-consumption-evidence.md).
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
  - See design.md §§4-5 and §11.6. Resolve the contradictory permitted
    runtime/proc-macro edges tracked in
    [issue 112](https://github.com/leynos/statelet/issues/112) before defining
    the later workspace policy. This roadmap update does not fix that issue.
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
  - See [the Phase 2 validation note
    template](phase-2-validation-note-template.md).
- [ ] 2.2.2. Annotate `mdtablefix` continuation handling with the baseline.
  - Requires 2.2.1.
  - Cover continuation mode handling and at least one fallible or infallible
    transition boundary if the code exposes one naturally.
  - Success: the validation note records boilerplate, diagnostics, tracing
    fields, and any `StateName` identifier pressure.
  - See design.md §§6.1 and 12.
  - See [the Phase 2 validation note
    template](phase-2-validation-note-template.md).
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
  - See [the Phase 2 validation note
    template](phase-2-validation-note-template.md).
- [ ] 3.1.3. Decide whether the runtime/conventions crate earns publication.
  - Requires 3.1.2.
  - Adopting any Phase 6 observation extension also requires 6.3.4.
    Rejecting or deferring those extensions leaves the original exit
    criteria intact; Trasic and async work are not new publication gates.
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
  - A macro that adopts Phase 6 observation extensions also requires 6.5.2.
    Handwritten evidence does not certify generated-code preservation.
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
  - Include tracing span behaviour across `.await`. If async support is
    accepted for implementation, step 6.6 supplies its completion and
    cancellation evidence before any support claim.
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

## 6. Observation integration contracts without framework ownership

Goal: downstream users can explain transition failures without changing domain
behaviour or mistaking incomplete observations for semantic evidence.

Idea: explicit capture, invocation context, and structured consumption improve
real diagnostics while preserving ordinary Rust ownership and control flow.
This idea is false if plain tracing remains clearer, the observations interfere
with domain execution, or the extra vocabulary has no demonstrated consumer.

The five RFCs and ADR 005 remain proposed until their review tasks record a
verdict. Start with handwritten, example-local experiments. Preserve ADR 003's
ship-nothing, conventions-only, and macro exits; preserve ADR 004's accepted
naming evidence. Trasic is supplementary to `mdtablefix` and `wireframe`, not a
replacement for the non-parser proving ground. No new crate or public observer
trait follows automatically from this phase.

### 6.1. Review the observation and naming contracts

This step decides what each proposal actually requires and what it excludes.
Its output is a reviewable contract disposition, not runtime code. Review may
accept a narrowed experiment, defer a feature, or reject a proposal entirely.

- [ ] 6.1.1. Review entry, return, and post-state capture semantics.
  - Requires 1.1.2.
  - Review [RFC 0001](rfcs/0001-transition-capture-semantics.md), requirements
    C1 to C5, including aliases, borrowing projections, and missing post-state.
  - Success: record accepted, revised, and rejected field semantics; no result
    variant or equal state label implies rollback or absence of effects.
- [ ] 6.1.2. Review explicit targets and caller context.
  - Requires 6.1.1.
  - Review [RFC 0002](rfcs/0002-explicit-observation-context.md), requirements
    H1 to H5, against caller-owned spans and context-parent alternatives.
  - Success: specify field declaration, disabled-target behaviour, and borrow
    boundaries without committing to a new public handle.
- [ ] 6.1.3. Review the non-interference guarantee and its limits.
  - Requires 6.1.2.
  - Review [RFC 0003](rfcs/0003-observational-non-interference.md), requirements
    N1 to N5, and the projection/subscriber preconditions.
  - Success: define capture counts, supported ownership shapes, and semantic
    comparisons for enabled, filtered, and feature-disabled configurations.
- [ ] 6.1.4. Review structured consumption and evidence completeness.
  - Requires 6.1.2.
  - Review [RFC 0004](rfcs/0004-structured-test-consumption.md), requirements
    T1 to T5; define a separate integration-note marker and schema.
  - Success: specify late-field handling, truncation, filtering scope, and
    absence assertions without modifying ADR 004's register or note parser.
- [ ] 6.1.5. Review synchronous completion and deferred cancellation semantics.
  - Requires 6.1.1 and 6.1.3.
  - Review [RFC 0005](rfcs/0005-incomplete-completion-and-cancellation.md),
    requirements L1 to L5, including explicit-finish-only as an outcome.
  - Success: distinguish return, observed unwinding, incomplete capture, and
    missing delivery; preserve the separate async gate at 5.2.1.
- [ ] 6.1.6. Review transition and outcome naming evidence.
  - Requires 1.1.3.
  - Review [ADR 005](adr-005-stable-transition-and-outcome-names.md) without
    reopening ADR 004's accepted aggregation rule implicitly.
  - Success: record the disposition and a consumer-led stability question;
    neither numeric identifiers nor a global registry become default policy.

### 6.2. Prove the synchronous contracts with handwritten code

This step tests whether the proposed capture and context patterns remain
ordinary Rust. Use small isolated fixtures and local helpers, not an early
proc-macro implementation. Reject a shape that needs domain restructuring.

- [ ] 6.2.1. Exercise entry and completion projections across ownership shapes.
  - Requires 2.1.3, 6.1.1, and 6.1.3 accepting a baseline experiment.
  - Cover result aliases, non-`Debug` errors, moved events, consumed state,
    borrowed returns, unavailable post-state, and error after mutation.
  - Success: pass fixtures preserve signatures and results; unsupported
    combinations have explicit omissions or focused diagnostics.
- [ ] 6.2.2. Exercise exact-target context under nesting and filtering.
  - Requires 6.2.1 and 6.1.2 accepting a baseline experiment.
  - Compare caller-created spans with standard children of context spans;
    cover late fields, missing declarations, siblings, and filtered children.
  - Success: child completion never updates an enabled parent by fallback,
    and no observation handle retains an application-state borrow.
- [ ] 6.2.3. Exercise explicit finish and incomplete observation.
  - Requires 6.2.1 and 6.1.5 accepting a synchronous experiment.
  - Cover normal return, `Err`, early return, `?`, omitted finish, nested
    cleanup, and unwind reporting only where the panic strategy permits it.
  - Success: no duplicate terminal observation, post-state read in `Drop`,
    inferred cancellation cause, or changed application cleanup occurs.
- [ ] 6.2.4. Falsify semantic non-interference with negative controls.
  - Requires 6.2.1, 6.2.2, and 6.2.3.
  - Compare uninstrumented, recorded, filtered, and feature-disabled results
    using injected token, random, resource, and geometry-service counters.
  - Success: bounded generated sequences preserve domain effects; deliberate
    duplicate execution, eager disabled capture, and repeated queries fail.

### 6.3. Consume evidence and decide the integration boundary

This step tests whether observations serve actual readers. It feeds extension
adoption, not a replacement B1/B2 decision procedure. Incomplete captures cannot
justify a feature. Publication can reject every proposed extension and retain
the existing conventions baseline.

- [ ] 6.3.1. Build an example-local structured recorder.
  - Requires 6.1.4 accepting an experiment and 6.2.2.
  - Record creation, late fields, events, parent context, and explicit
    completion with scoped subscriber installation and bounded collection.
  - Success: negative controls for missing updates, wrong-parent attribution,
    and hidden overflow fail; capture incompleteness blocks absence claims.
- [ ] 6.3.2. Validate stable transition and outcome names through source
  renames.
  - Requires 6.1.6 accepting an experiment and 6.3.1.
  - Rename a method and outcome variant while preserving declared labels;
    include a source-derived-label negative control and semantic-name change.
  - Success: record the consumer operation, ownership, cardinality policy,
    and stability requirement without assuming numeric representation.
- [ ] 6.3.3. Collect integration notes from both existing proving grounds.
  - Requires 2.2.2, 3.1.2, 6.2.4, and 6.3.2.
  - Compare plain tracing, local conventions, and proposed observations in
    `mdtablefix` and `wireframe`; keep source revisions and capture settings.
  - Success: notes cite diagnostic value, limitations, and keep/revise/reject
    evidence; any `StateName` finding also uses ADR 004's existing instrument.
- [ ] 6.3.4. Decide which observation extensions earn adoption.
  - Requires 6.3.3, or an explicit rejection/deferment of the experiment.
  - Decide field meanings, completion policy, context pattern, and naming
    conventions individually; keep helpers local unless reuse is demonstrated.
  - Success: update proposal status and governing design for adopted items;
    no public observer, wrapper, or test-support crate appears without a named
    consumer. Rejection leaves the original publication gates intact.

### 6.4. Test Trasic as a supplementary diagnostic integration

This optional step asks whether the conventions help explain semantic-scene
mismatches without requiring an FSM framework or identical reference traces.
It does not gate runtime publication or replace `wireframe` evidence.

- [ ] 6.4.1. Select real conditional and restoration boundaries in Trasic.
  - Requires 6.3.4 retaining useful conventions and available Trasic code.
  - Pin the inspected revision; identify one finite controller and one
    temporary-context boundary, with plain tracing as the control.
  - Success: each boundary exists independently of observation; no synthetic
    mode enum or universal elaborator state is introduced to fit Statelet.
- [ ] 6.4.2. Correlate a deliberately introduced scene mismatch with context.
  - Requires 6.4.1 and a usable Trasic conformance fixture.
  - Follow an object/provenance reference to its elaboration observation;
    compare the scene separately from the diagnostic transition record.
  - Success: the trace localizes the injected defect without demanding that
    HgPovRay and Trasic execute the same internal sequence.
- [ ] 6.4.3. Record the supplementary integration verdict.
  - Requires 6.4.2.
  - Compare reviewability, useful context, and non-interference with plain
    tracing; record missing capabilities and unnecessary abstractions.
  - Success: retain, revise, or remove the integration based on evidence;
    another parser does not substitute for the non-parser proving ground.

### 6.5. Revalidate accepted contracts in any later macro

This conditional step asks whether generated capture preserves the handwritten
contract. It applies only when the existing Phase 3 gate chooses a macro and
the adoption review retains the corresponding observation extension.

- [ ] 6.5.1. Prototype explicit macro projections and target access.
  - Requires 4.1.3 and 6.3.4 adopting the relevant contracts.
  - Keep automatic return inspection conservative; add explicit borrowing
    projections for aliases and preserve caller-controlled context attachment.
  - Success: expression spans remain useful, runtime paths support renamed
    dependencies, and no new result enum or broad trait bound is required.
- [ ] 6.5.2. Repeat preservation and consumption tests against generated code.
  - Requires 6.5.1 and 6.2.4.
  - Reuse ownership, filtering, capture-count, early-exit, and recorder
    negative controls; check only existing feature combinations.
  - Success: generated and handwritten domain results agree, diagnostics are
    focused, and the original macro cost and topology gates still pass.

### 6.6. Validate asynchronous completion only after its separate gate

This deferred step tests observation across suspension and abandonment. It
cannot justify moving async implementation into the synchronous baseline.

- [ ] 6.6.1. Exercise poll-scoped observation and dropped futures.
  - Requires 5.2.1 choosing to implement async support and 6.1.5.
  - Cover drop before first poll, suspension, resumption, drop after
    suspension, completion, nested futures, filtering, and context changes.
  - Success: the chosen capture-start policy is explicit; no entered-span
    guard crosses `.await`, and no unfinished future is labelled cancelled
    without an application-provided reason.
- [ ] 6.6.2. Record the async support boundary and limitations.
  - Requires 6.6.1.
  - Compare domain cleanup and attribution with the uninstrumented baseline;
    document panic-strategy and terminal-record delivery limitations.
  - Success: supported shapes have evidence, unsupported shapes stay explicit,
    and Statelet owns neither application cancellation nor rollback.
