# State representation and observability roadmap

Status: Proposed companion roadmap

Scope: Evidence-producing work for the refinements proposed by
[ADR 004](adr-004-representation-and-observability-contracts.md).

Audience: Maintainers sequencing the `mdtablefix` and `wireframe` proving
grounds and deciding whether Statelet ships nothing, conventions only, or a
macro.

This roadmap supplements the primary [Statelet roadmap](roadmap.md). It does not
replace the existing product-exit, topology, feature, build-time, or artefact
size gates. Accepted work should be folded into the primary roadmap before it
claims completion.

## 1. Ratify the refined contracts

Idea: separate downstream representation correctness from Statelet's
instrumentation value before either proving ground runs.

### 1.1. Define the representation prerequisite

- [ ] 1.1.1. Review ADR 004's honest-representation rule.
  - Confirm that project-owned algebraic data types (ADTs), newtypes, and
    typestate remain outside Statelet ownership.
  - Confirm that a synthetic enum introduced only for tracing is a negative
    control.
  - Success: ADR 004 is accepted, revised, or rejected unambiguously.
- [ ] 1.1.2. Add separate representation and instrumentation verdicts to the
  proving-ground note.
  - Requires 1.1.1.
  - Success: a downstream type refactor cannot count as Statelet product value.
- [ ] 1.1.3. Record unsuitable seams explicitly.
  - Include counter-balance, token tables, build graphs, and decorative state
    projections.
  - Success: "do not use Statelet" is an evidence-backed outcome.

### 1.2. Settle the review vocabulary

- [ ] 1.2.1. Define `StateName` acceptance for unit, tuple, struct, and generic
  enum variants without new payload bounds.
- [ ] 1.2.2. Ratify semantic event labels.
  - Prefer `line_kind`, `frame_kind`, or `peer_closed` over raw input.
- [ ] 1.2.3. Reserve `transition.state.after` as optional.
  - Keep it absent where the boundary cannot observe it honestly.
- [ ] 1.2.4. Keep project-owned outcome enums first-class.
  - Leave the crate-level `TransitionOutcome` deferred.

## 2. Validate the contracts in `mdtablefix`

Idea: establish an honest parser state before measuring the value of Statelet's
conventions.

### 2.1. Establish the representation baseline

- [ ] 2.1.1. Evaluate a payload-carrying table state such as
  `TableRun::Buffering(Vec<String>)`.
  - Success: contradictory mode and buffer combinations are unrepresentable or
    explicitly justified.
- [ ] 2.1.2. Encapsulate mandatory finalization.
  - Prefer a consuming operation such as `finish(self)` where it makes required
    ceremony atomic.
- [ ] 2.1.3. Record what became safer through downstream Rust types before
  applying Statelet.

### 2.2. Apply the conventions-only baseline

- [ ] 2.2.1. Name parser states without inspecting payloads.
- [ ] 2.2.2. Define bounded line-kind and domain-outcome labels locally.
  - Success: traces expose no arbitrary Markdown content by default.
- [ ] 2.2.3. Compare before-state with honestly observable after-state.
  - Record which success and error paths omit the latter.
- [ ] 2.2.4. Compare original code, plain tracing, and Statelet conventions.
  - Success: the result feeds the existing product-exit gate without crediting
    Statelet for the ADT refactor.

## 3. Transfer the contracts to `wireframe`

Idea: test whether the same vocabulary fits generic protocol states, fallible
boundaries, verification names, and asynchronous control flow.

### 3.1. Validate existing connection-state seams

- [ ] 3.1.1. Apply `StateName` to `RunState` and `ActiveOutput<F, E>`.
  - Success: no new `F` or `E` bounds appear.
- [ ] 3.1.2. Instrument one fallible connection boundary with semantic event and
  project-owned error labels.
- [ ] 3.1.3. Instrument one infallible active-output boundary with the
  same field vocabulary.
- [ ] 3.1.4. Record shared convention value separately from formal proof and
  project-test obligations.

### 3.2. Keep secondary refactors independent

- [ ] 3.2.1. Evaluate `MessageSeries` only through Wireframe's own design work.
  - Statelet must not introduce a computed state projection solely for tracing.
- [ ] 3.2.2. Apply Statelet only after Wireframe independently adopts an ADT it
  would keep without Statelet.

### 3.3. Validate asynchronous conventions

- [ ] 3.3.1. Apply the non-macro vocabulary to one async boundary.
  - Success: borrowing, cancellation, and error behaviour remain unchanged.
- [ ] 3.3.2. Record that cancellation cannot emit an ordinary completed
  after-state.
- [ ] 3.3.3. Decide whether async macro support is required for v0.1.
  - Success: the macro gate gains the Wireframe-shaped matrix or documentation
    states that `async fn` is unsupported.

## 4. Prove or reject the procedural macro

Idea: publish a macro only if it preserves the refined contracts and still
beats the conventions baseline.

### 4.1. Settle after-state and expression semantics

- [ ] 4.1.1. Choose explicit `state_after(...)`, safe state re-evaluation, an
  outcome adapter, or deferral.
- [ ] 4.1.2. Document how often every expression argument is evaluated.
- [ ] 4.1.3. Define after-state behaviour for success, errors before and after
  mutation, and early returns.

### 4.2. Prove synchronous compatibility

- [ ] 4.2.1. Add derive pass tests for unit, tuple, struct, and generic
  variants.
- [ ] 4.2.2. Add examples using low-cardinality event and domain-outcome labels.
- [ ] 4.2.3. Add after-state path tests when 4.1.1 accepts macro support.
- [ ] 4.2.4. Re-run the primary diagnostic, feature, topology, build-time, and
  artefact-size gates.
  - Success: the macro still beats the conventions baseline.

### 4.3. Prove asynchronous compatibility or defer it

- [ ] 4.3.1. When async is in scope, build a fixture containing `&mut self`,
  `Result`, `?`, an early return, `.await`, and state used on both sides.
- [ ] 4.3.2. Prove that no entered-span guard crosses `.await` and cancellation
  emits no fabricated completion.
- [ ] 4.3.3. Record the async publication boundary.

## 5. Decide the publication surface

- [ ] 5.1. Compare both proving grounds without counting downstream refactors as
  Statelet value.
- [ ] 5.2. Give every proposed public contract a named consumer or defer it.
- [ ] 5.3. Select the existing ship-nothing, conventions-only, or macro exit.
- [ ] 5.4. If ADR 004 is accepted, integrate surviving requirements into
  `design.md`, `roadmap.md`, and user guidance.
