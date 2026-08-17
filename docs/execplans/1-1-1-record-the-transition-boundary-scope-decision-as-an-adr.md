# Record the transition-boundary scope decision as an ADR

This ExecPlan (execution plan) is a living document. The sections `Constraints`,
`Tolerances`, `Risks`, `Progress`, `Surprises & Discoveries`, `Decision Log`,
and `Outcomes & Retrospective` must be kept up to date as work proceeds.

Status: COMPLETE

## Purpose / big picture

Statelet's roadmap opens with a foundational phase whose first deliverable is
not runtime code but a documented, ratified decision. Roadmap item 1.1.1
requires a single accepted Architecture Decision Record (ADR) stating that
Statelet *marks* transition boundaries and does *not* own dispatch, events,
storage, transition tables, or graph safety.

This decision is the load-bearing constraint for every later slice. Phases 2
through 4 each branch on whether Statelet has overstepped that boundary: the
conventions-only baseline (Phase 2), the second proving ground (Phase 3), and
the conditional macro (Phase 4) are all defined by what Statelet refuses to
own. Until the boundary is recorded as an accepted ADR rather than as prose
scattered across the terms of reference and design documents, "ship nothing",
"ship conventions only", and "ship the macro" cannot be told apart cleanly, and
reviewers have no single citable authority for rejecting framework-shaped
feature requests.

After this change, a reader can open
`docs/adr-002-transition-boundary-scope.md` and find one accepted decision that
says, verbatim, that Statelet marks boundaries and does not own dispatch,
events, storage, transition tables, or graph safety; the decision is reachable
from the documentation index, the design document, and the terms of reference;
and the roadmap item is marked done. Success is observable by running the
repository's Markdown gates and a small content check that greps for the
required decision sentence and confirms every new cross-reference link resolves.

This is a documentation-only deliverable. It introduces no Rust code, no public
API, and no runtime behaviour, so the project's runtime test stack (`rstest`,
`rstest-bdd`, `proptest`, `kani`, `verus`, `insta`) does not apply here. The
`Validation and acceptance` section explains the documentation-appropriate
substitute for Red-Green-Refactor and records why the runtime stack is out of
scope for this item.

## Constraints

Hard invariants that must hold throughout implementation. Violation requires
escalation, not a workaround.

- The ADR must follow the repository ADR convention in
  `docs/documentation-style-guide.md` (the `## Status`, `## Date`,
  `## Context and problem statement` ordering and the
  `adr-NNN-short-description.md` naming), not any external or global ADR
  template. This repository's house style takes precedence over the global
  Y-Statement skill.
- The ADR file must be named `docs/adr-002-transition-boundary-scope.md`. ADR
  001 already exists; numbers are allocated sequentially and never reused.
- The ADR must contain the success sentence required by roadmap item 1.1.1,
  stating that Statelet marks boundaries and does not own dispatch, events,
  storage, transition tables, or graph safety. The five owned concerns
  (dispatch, events, storage, transition tables, graph safety) must all appear
  as explicitly out of scope.
- The decision must not contradict the existing scope prose in
  `docs/terms-of-reference.md` §§1-6 and §6.2, `docs/design.md` §§1-3 and §2.2,
  `README.md`, or `docs/users-guide.md`. The ADR ratifies and consolidates
  those statements; it does not introduce new scope or reverse any prior
  position.
- No runtime source under `src/`, no `Cargo.toml`, and no test under `tests/`
  may be modified by this plan. The change set is confined to Markdown under
  `docs/` plus the roadmap checkbox.
- All Markdown must satisfy the repository gates: `make markdownlint` (the
  `mdlint '**/*.md'` glob covers `docs/execplans/`, so this plan file is itself
  linted), `make check-fmt` for any formatting concerns, and `make nixie` if
  any Mermaid diagram is added. Prose wraps at 80 columns; fenced code wraps at
  120; tables and headings are exempt from MD013.

## Tolerances (exception triggers)

Thresholds that trigger escalation rather than autonomous continuation.

- Scope: if delivering the ADR appears to require editing any file outside
  `docs/` (for example, touching `src/`, `Cargo.toml`, or CI workflow files),
  stop and escalate.
- Content conflict: if drafting the ADR surfaces a genuine contradiction
  between the design document, the terms of reference, and the README about
  what Statelet owns, stop and escalate rather than silently picking a side.
  The ADR records an already-agreed decision; a newly discovered disagreement
  is a planning input, not an authoring choice.
- Status semantics: if there is any doubt about whether the ADR should be
  merged as `Accepted` or as `Proposed`, stop and confirm. Roadmap 1.1.1 asks
  for an *accepted* ADR; see the Decision Log entry on acceptance timing.
- Gate failure: if `make markdownlint`, `make check-fmt`, or `make nixie` fails
  after three focused correction attempts, stop and escalate with the captured
  log.
- Ambiguity: if more than one materially different decision wording is
  defensible (for example, whether "control flow" should be named as a sixth
  out-of-scope concern alongside the required five), present the options with
  trade-offs rather than choosing unilaterally.

## Risks

- Risk: the ADR drifts into re-explaining the whole design rather than
  recording one decision, blurring the ADR/design-document boundary the style
  guide draws. Severity: medium. Likelihood: medium. Mitigation: keep the ADR
  narrow; cite design and terms-of-reference sections by number instead of
  restating them; mirror the shape and length of the existing
  `docs/adr-001-proving-ground-candidates.md`.
- Risk: the decision wording omits one of the five required out-of-scope
  concerns, so the success criterion is not literally met. Severity: high.
  Likelihood: low. Mitigation: the acceptance check greps for the exact
  decision sentence and for each of the five concerns; the red check fails
  until all five are present.
- Risk: a cross-reference link added to `contents.md`, `design.md`, or
  `terms-of-reference.md` points at the wrong filename and silently rots.
  Severity: medium. Likelihood: low. Mitigation: the acceptance check resolves
  every new relative link against the filesystem and fails on any miss.
- Risk: `mdformat-all` (run by `make fmt`) rewraps prose and produces a diff
  after authoring, causing `make check-fmt`-style churn. Severity: low.
  Likelihood: medium. Mitigation: author at 80 columns from the start and run
  `make fmt` once before the final gate so the formatter is a no-op at commit
  time.
- Risk: acceptance timing confusion — merging an ADR labelled `Accepted`
  before the reviewing pull request has actually approved it. Severity: low.
  Likelihood: medium. Mitigation: see the Decision Log; the ADR is authored as
  `Accepted` because it ratifies decisions already settled in the design and
  terms of reference, and the reviewing pull request is the acceptance act. If
  the reviewer prefers `Proposed`-then-flip, that is a one-line change.

## Progress

- [x] (2026-07-22 20:39Z) Stage A: confirmed orientation, the five required
  out-of-scope concerns, and the exact decision sentence. The local ADR
  convention requires `Status`, `Date`, and `Context and problem statement` in
  that order. No deliverable files changed.
- [x] (2026-07-22 20:40Z) Stage B: ran the shared inline acceptance predicate
  before authoring. It returned
  `RED: docs/adr-002-transition-boundary-scope.md absent` with exit status 1,
  as required; no durable script was needed.
- [x] (2026-07-22 20:43Z) Stage C: authored
  `docs/adr-002-transition-boundary-scope.md` from the embedded artefact with
  the current acceptance date. The shared predicate returned
  `GREEN: decision sentence present with all five concerns in order`.
- [x] (2026-07-22 20:48Z) Stage D: synchronized companion documents, closed
  roadmap item 1.1.1, and completed validation. The four companion edits were
  tracked separately because they were the highest-risk-to-omit work:
  - [x] (2026-07-22 20:46Z) D1: added the ADR 002 entry to
    `docs/contents.md` "Decision records".
  - [x] (2026-07-22 20:46Z) D2: added ADR 002 to the `docs/design.md`
    companion list and a canonical-scope-authority citation in §1.
  - [x] (2026-07-22 20:46Z) D3: resolved the
    `docs/terms-of-reference.md` §10.2 candidate as an ADR 002 link and added
    it to the companion list.
  - [x] (2026-07-22 20:46Z) D4: ticked roadmap 1.1.1 and linked ADR 002 from
    its success line.
  - [x] (2026-07-22 20:48Z) D5: content and all-touched-files link checks
    passed; `make markdownlint`, `make check-fmt`, and `make nixie` passed;
    `coderabbit review --agent` completed without reporting a concern.
- [x] (2026-08-17) Review follow-up: tightened the ADR evidence and the
  acceptance/link predicates, and synchronized the embedded ADR. This is a
  documentation-integrity repair only; it adds no product scope.
  `make markdownlint` and `make nixie` passed after the spelling correction.

This section must always reflect the actual current state of the work. The
implementing agent updates checkboxes with timestamps as stages complete.

## Surprises & discoveries

- Observation: `make markdownlint` lints `**/*.md`, so this ExecPlan file is
  itself subject to MD013. Evidence: the `markdownlint` target runs
  `$(MDLINT) '**/*.md'` and `.markdownlint-cli2.jsonc` ignores only `target`,
  `node_modules`, and `.venv`. Impact: the plan is authored at an 80-column
  prose wrap so the gate stays green while the plan lives in-tree.
- Observation: `make fmt` rewrote an unrelated paragraph in the imported
  `docs/rstest-bdd-users-guide.md` despite no source change there. Evidence:
  the first formatter diff contained that file only in addition to the ADR and
  ExecPlan changes. Impact: restored the imported-guide churn before
  committing; the formatter is not a no-op on the pre-existing tree, so final
  validation uses `make fmt`, restores the unrelated change, then runs the
  check gates.
- Observation: review found that the ADR's market summary had no source
  citations, its competing-crate claim reached into ADR 001, and the embedded
  ADR had drifted from the delivered ADR's relative link. Impact: added ordered
  footnotes from Appendix A, limited the claim to crates named in this ADR,
  and synchronized the embedded artefact without changing the decision.
- Observation: the original link loop only printed `BROKEN` and could still
  exit successfully because the pipeline ran in a subshell. Impact: the
  validator now tracks failures explicitly and exits non-zero; the ExecPlan
  itself is included in the checked file set.
- Observation: follow-up reconnaissance found a useful contrast across the
  validation candidates: `mdtablefix` has inconsistent but infallible boundary
  instrumentation, `wireframe` has a fallible handwritten actor seam with no
  tracing, and `ddlint` has counter-based parser guards rather than a real
  mode. Impact: ADR 002 records them as validation evidence without changing
  its accepted scope or deciding that the macro should ship.

## Decision log

- Decision: name the file `docs/adr-002-transition-boundary-scope.md`.
  Rationale: ADR 001 is taken; the style guide mandates sequential
  `adr-NNN-short-description.md` naming; "transition-boundary-scope" matches
  the roadmap task and the terms-of-reference §10.2 candidate wording.
  Date/Author: 2026-06-24, planning agent.
- Decision: author the ADR with `Status: Accepted` and the merge date, rather
  than `Proposed`. Rationale: roadmap 1.1.1 success is "one accepted ADR".
  Unlike ADR 001 (which proposes a still-open proving-ground ranking), this ADR
  ratifies a scope boundary already stated as settled across the terms of
  reference (§§1-6, §6.2), the design (§§1-3, §2.2), the README, and the user's
  guide. The reviewing pull request is the acceptance act. This is recorded as
  a tolerance so the reviewer can request `Proposed` instead with a one-line
  change. Date/Author: 2026-06-24, planning agent.
- Decision: name only the five roadmap-mandated concerns (dispatch, events,
  storage, transition tables, graph safety) in the headline decision sentence,
  and treat "control flow / branch logic" and "typestate" as supporting
  non-goals carried from the design rather than as part of the mandated
  sentence. Rationale: the success criterion fixes those five; adding more to
  the headline risks diluting the literal match. Supporting non-goals still
  appear in the ADR's Goals and non-goals section, consistent with design §2.2.
  Date/Author: 2026-06-24, planning agent.
- Decision: do not apply the runtime test stack (`rstest`, `rstest-bdd`,
  `proptest`, `kani`, `verus`, `insta`) to this item. Rationale: the
  deliverable is prose with no executable behaviour. The honest validation is
  the Markdown gates plus a content/link check. The runtime stack becomes
  relevant from roadmap step 2.1 onward when `StateName` and tracing fields
  introduce real code. Date/Author: 2026-06-24, planning agent.
- Decision: encode the success criterion as one shared `check_adr` predicate
  that fixed-string-matches the full decision sentence against a
  whitespace-folded copy of the file, run identically for the red and green
  stages. Rationale: a community-of-experts review found that a line-oriented
  grep for the sentence produces a false RED once the ADR is wrapped at 80
  columns, and that matching the five concern words individually produces a
  false GREEN because those words also occur in prose. Folding whitespace and
  matching the whole sentence fixes both, and sharing one predicate guarantees
  the red and green stages test the same contract. Date/Author: 2026-06-24,
  planning agent (post-review revision).
- Decision: omit the optional ADR sections `Requirements`, `Migration plan`,
  and `Outstanding decisions` from ADR 002, and state the decision verbatim
  only once (in `Decision outcome`), with `Status` referring to it. Rationale:
  those sections are conditional in the style guide and absent from ADR 001
  too; this ADR ratifies a settled decision with no open questions and no
  phased build (phased work lives in the roadmap). Stating the sentence once
  removes a drift hazard between two copies. Date/Author: 2026-06-24, planning
  agent (post-review revision).
- Decision: retain the accepted marker-only decision and repair only its
  evidence and validation mechanics. Rationale: ordered Appendix A footnotes,
  a claim scoped to the competing crates named in this ADR, exact-once
  sentence validation, and a failing link check improve traceability without
  creating new product scope. Date/Author: 2026-08-17, review follow-up.
- Decision: record the three reconnaissance examples in ADR 002 as validation
  evidence. Rationale: concrete downstream seams make the scope boundary
  falsifiable while preserving user-owned dispatch, events, storage, and graph
  safety. Date/Author: 2026-08-17, follow-up documentation.

## Outcomes & retrospective

Delivered one accepted ADR that records the marker-only scope. It contains the
required decision sentence exactly once, is reachable from the documentation
index, design, and terms of reference, and closes roadmap item 1.1.1. No scope
conflict surfaced among the design, terms of reference, README, and user's
guide; `Accepted` was retained because the ADR ratifies that already-settled
position.

The documentation-focused red-green check was effective: it failed before the
ADR existed and then proved the exact, whitespace-wrapped sentence once
authored. `make fmt` was not idempotent for one unrelated imported guide, so
that pre-existing formatter churn was restored before each commit; the planned
files remained formatted and all check gates passed.

## Context and orientation

Statelet is a Rust crate, currently a single-crate skeleton (`src/lib.rs` plus
`tests/stub.rs`), whose documentation set under `docs/` is well ahead of its
code. The crate's thesis is deliberately narrow: it is a *transition-boundary
toolkit* for state machines a team has already written in ordinary Rust (enums,
structs, methods, and `match` expressions). It marks, names, and instruments
the points where such a machine decides to stay, move, emit, ignore, or fail.
It is explicitly not a state-machine framework, a transition-table DSL, or a
typestate library.

Key terms, defined for a reader new to the repository:

- Transition boundary: a method or function where stateful logic decides to
  stay in the current state, move to another, emit output, ignore input, or
  fail. Statelet's job is to make these boundaries recognizable and observable.
- Owning the transition table: generating the state-machine structure itself —
  the set of states and the legal moves between them — from a macro or DSL.
  `stateless` does this with a zero-cost macro; Statelet must not.
- Owning dispatch: providing the loop or runtime that accepts events and routes
  them to handlers. `statig` and `rust-fsm` do this; Statelet must not.
- Graph safety: compile-time or model-checked guarantees that only legal
  transitions can occur. Out of scope for Statelet; downstream projects keep
  their own (for example `wireframe`'s Stateright model, per ADR 001).
- ADR: an Architecture Decision Record. In this repository, ADRs capture one
  accepted decision in a fixed section order and are indexed from
  `docs/contents.md`.

Files this plan reads or writes, by full repository-relative path:

- `docs/adr-002-transition-boundary-scope.md` — new file, the deliverable.
- `docs/adr-001-proving-ground-candidates.md` — existing ADR; the structural
  and tonal template to mirror.
- `docs/documentation-style-guide.md` — normative ADR template, naming, and
  Markdown rules.
- `docs/terms-of-reference.md` — §§1-6 (problem and scope), §6.2 (non-goals),
  §10.2 (the ADR candidate this fulfils).
- `docs/design.md` — §§1-3 (context, goals/non-goals, design decisions), §2.2
  (the ten non-goal bullets), §11.1 (bet register), §§13.6-13.7 (failure
  modes), and the companion-documents list at the top.
- `docs/contents.md` — the documentation index, "Decision records" section.
- `docs/roadmap.md` — item 1.1.1, whose checkbox is ticked on completion.
- `README.md`, `docs/users-guide.md` — scope prose that must stay consistent.

Relevant skills to load when implementing this plan: `execplans` (this format),
the documentation style guide above, and — only if a Mermaid diagram is added —
the `make nixie` validation path. The `arch-decision-records` skill encodes a
Y-Statement template that this repository does *not* use; prefer the local
style guide. `leta` remains the default code-navigation tool, though this item
touches no code.

## Plan of work

The work proceeds in four stages with explicit go/no-go validation at each
boundary. Because the deliverable is documentation, the "test" is a content and
link check rather than a runtime suite; it plays the Red-Green-Refactor role.

### Stage A: understand and propose (no file changes)

Re-read `docs/documentation-style-guide.md` ADR section, the existing
`docs/adr-001-proving-ground-candidates.md`, and the cited design and
terms-of-reference sections. Confirm three things: the required ADR section
order, the five out-of-scope concerns the headline sentence must name, and the
exact "instead, use …" redirections (`statig`, `rust-fsm`, `smlang`,
`stateless`, `typed-fsm`, `sfsm`, `finny`, `macro-machines`). Produce no edits.
Go/no-go: the implementer can state the decision sentence and the five concerns
from memory before writing.

### Stage B: red acceptance check

Add a small, self-contained acceptance check that encodes the success criterion
and the link integrity requirement. The simplest form is an inline shell
snippet (recorded under `Concrete steps`) that:

1. matches the full canonical decision sentence (which names all five concerns
   in order) against a whitespace-folded copy of
   `docs/adr-002-transition-boundary-scope.md`, and
2. resolves every relative Markdown link in each touched file against the
   filesystem.

Matching the whole folded sentence — rather than the bare words in isolation —
avoids two failure modes the review surfaced: a false RED when the 80-column
wrap splits the sentence across lines, and a false GREEN when a word like
"storage" appears only in unrelated prose. Run the shared predicate before the
ADR exists and confirm it exits non-zero for the expected reason (the file is
absent). This is the red step. If the team prefers a durable check, the snippet
may be promoted into `scripts/` later, but that is out of scope here and must
not pull in new tooling without escalation.

### Stage C: author the ADR

Create `docs/adr-002-transition-boundary-scope.md` from the artefact embedded in
`Artefacts and notes`, adjusting only the acceptance date to the merge date.
Keep it within the same order of length as ADR 001. Re-run the Stage B check
and confirm it passes (green). Run `make markdownlint` and `make fmt`; the
formatter must be a no-op at commit time.

### Stage D: synchronize companion documents and close the item

Make the minimal cross-reference edits so the decision is reachable and the
candidate list reflects that it is now recorded:

1. `docs/contents.md`: add a "Decision records" entry for ADR 002 directly
   after the ADR 001 entry, with a one-line audience-focused description.
2. `docs/design.md`: add `docs/adr-002-transition-boundary-scope.md` to the
   companion-documents list at the top, and add a single citation from §1
   (Design context) or §2 (Goals and non-goals) naming ADR 002 as the canonical
   scope authority. This both satisfies the AGENTS.md rule that substantive
   decisions are recorded in an ADR referenced from the design document, and
   gives the duplicated non-goal prose (README, user's guide, terms of
   reference §6.2, design §2.2) one authority to defer to, so future scope
   edits point at the ADR rather than drifting independently. Do not delete the
   existing non-goal prose; only point it at the ADR.
3. `docs/terms-of-reference.md`: convert the §10.2 candidate bullet "ADR:
   `statelet` is a transition-boundary toolkit, not a state-machine framework."
   into a resolved inline link to ADR 002 (mirroring how the ADR 001 bullet is
   already a link), and add ADR 002 to the companion-documents list at the top.
4. `docs/roadmap.md`: tick the item 1.1.1 checkbox (`[ ]` → `[x]`) and append a
   link to the accepted ADR in its success line.

Do not edit `README.md` or `docs/users-guide.md` unless Stage A surfaced a
genuine inconsistency; the ADR is designed to match their existing wording. Run
the full Markdown gate suite, then request a CodeRabbit review and clear all
concerns before the item is considered done.

Each stage ends with validation; do not proceed past a failing stage.

## Concrete steps

Run all commands from the repository root (the directory containing
`Cargo.toml` and `Makefile`). Capture long gate output with `tee` to a log under
`/tmp` for review, per the repository command conventions.

Stage A/B/C — one shared acceptance predicate. Define it once and run the
identical function before authoring (red) and after authoring (green). It folds
the file to a single whitespace-normalized stream first, so the 80-column wrap
the ADR is authored at cannot cause a false RED, and it matches the *full*
decision sentence — all five concerns in order — so a stray occurrence of a
word like "storage" in unrelated prose cannot cause a false GREEN:

```bash
ADR=docs/adr-002-transition-boundary-scope.md
# The canonical decision sentence the success criterion requires (roadmap 1.1.1).
# Matched as a fixed string against a whitespace-folded copy of the file.
SENTENCE='marks boundaries and does not own dispatch, events, storage, transition tables, or graph safety'

check_adr() {
  local all_text outcome_text total_count outcome_count
  if [ ! -f "$ADR" ]; then
    echo "RED: $ADR absent"
    return 1
  fi
  all_text=$(tr '\n' ' ' < "$ADR" | tr -s ' ')
  outcome_text=$(awk '
    /^## Decision outcome \/ proposed direction$/ { in_outcome=1; next }
    in_outcome && /^## / { exit }
    in_outcome { print }
  ' "$ADR" | tr '\n' ' ' | tr -s ' ')
  total_count=$(printf '%s\n' "$all_text" | grep -oF "$SENTENCE" | wc -l || true)
  outcome_count=$(printf '%s\n' "$outcome_text" | grep -oF "$SENTENCE" | wc -l || true)
  if [ "$total_count" -eq 1 ] && [ "$outcome_count" -eq 1 ]; then
    echo "GREEN: decision sentence occurs once in Decision outcome and nowhere else"
    return 0
  fi
  echo "RED: decision sentence must occur once in Decision outcome and nowhere else"
  return 1
}
```

Stage B — run the red check before the ADR exists. A non-zero exit is the
expected RED signal:

```bash
check_adr; echo "exit=$?"   # expect: RED: ... absent / exit=1
```

Stage C — author the file directly with a file-writing tool or a heredoc (do
not invoke an interactive `$EDITOR`, which can hang under automation), copying
the embedded artefact and setting the acceptance date to the merge date. Then
re-run the same predicate; a zero exit is the expected GREEN:

```bash
# write docs/adr-002-transition-boundary-scope.md from the embedded artefact
check_adr; echo "exit=$?"   # expect: GREEN ... / exit=0
```

Stage C/D — Markdown gates (capture to /tmp). Confirm `make nixie` is already a
clean no-op on the pre-change tree so it is not blamed for this doc-only item:

```bash
set -euo pipefail

make markdownlint 2>&1 | tee /tmp/markdownlint-statelet-adr002.out
make check-fmt    2>&1 | tee /tmp/check-fmt-statelet-adr002.out
make nixie        2>&1 | tee /tmp/nixie-statelet-adr002.out
```

Stage D — resolve every relative Markdown link in each touched file (covering
the roadmap link and the ADR's own outbound link to ADR 001, not only links
*into* the new ADR). Any `BROKEN` line is a failure:

```bash
set -euo pipefail

broken=0
for f in docs/contents.md docs/design.md docs/terms-of-reference.md \
         docs/roadmap.md docs/adr-002-transition-boundary-scope.md \
         docs/execplans/1-1-1-record-the-transition-boundary-scope-decision-as-an-adr.md; do
  dir=$(dirname "$f")
  while read -r tgt; do
    case "$tgt" in http*|/*) continue ;; esac
    if ! test -f "$dir/$tgt"; then
      echo "BROKEN: $f -> $tgt"
      broken=1
    fi
  done < <(
    awk '
      /^```/ { in_fence = !in_fence; next }
      !in_fence { print }
    ' "$f" \
      | grep -oE '\[[^]]+\]\([^)]+\.md(#[^)]*)?\)' \
      | sed -E 's/.*\(([^)#]+\.md)(#[^)]*)?\)/\1/' || true
  )
done
if [ "$broken" -ne 0 ]; then
  echo "link check failed"
  exit 1
fi
echo "link check complete (no BROKEN lines above means pass)"
```

Each companion-document edit in Stage D must be idempotent. Before inserting a
line into `contents.md`, `design.md`, or `terms-of-reference.md`, grep-guard it
("add only if the `adr-002-transition-boundary-scope.md` reference is absent")
so re-running Stage D cannot append a duplicate index entry or companion-list
line.

Commit after the ADR is authored and green, then again after companion-document
sync, so the history shows the decision and its wiring as separate, reviewable
steps. Run `make markdownlint`, `make check-fmt`, and `make nixie` before each
commit. Request `coderabbit review --agent` only after the deterministic gates
pass, and clear every concern before marking the roadmap item done.

## Validation and acceptance

Acceptance is behavioural and observable without any runtime code:

- Decision presence: the shared `check_adr` predicate from `Concrete steps`
  matches the full decision sentence ("… marks boundaries and does not own
  dispatch, events, storage, transition tables, or graph safety") against a
  whitespace-folded copy of `docs/adr-002-transition-boundary-scope.md`. The
  one predicate is the contract: because it matches the whole sentence, all
  five concerns are verified together and in order, and the wrap the ADR is
  authored at cannot break the match. Before Stage C it exits non-zero (red);
  after Stage C it exits zero (green). Run this content check and the link
  check *before* the Markdown gates so a structural failure is not masked by a
  formatting pass. This is the documentation-appropriate Red-Green-Refactor
  substitute, recorded here in place of a runtime test because the deliverable
  has no executable behaviour.
- Single source of decision wording: the full decision sentence appears exactly
  once, in `## Decision outcome / proposed direction`. The `## Status` summary
  refers to it rather than restating the five-concern list, so the two cannot
  drift.
- Structure: the ADR contains, in order, `## Status` (Accepted, dated, with a
  one-line summary), `## Date`, and `## Context and problem statement`,
  matching the style-guide template and the shape of ADR 001. The optional
  `Requirements`, `Migration plan`, and `Outstanding decisions` sections are
  deliberately omitted — the decision is settled and carries no open questions,
  and phased work lives in the roadmap, matching the ADR 001 precedent.
- Reachability: `docs/contents.md` lists ADR 002 in "Decision records";
  `docs/design.md` and `docs/terms-of-reference.md` cite it; the link check
  resolves every relative link in all touched files (including `roadmap.md` and
  the ADR's outbound link to ADR 001) and prints no `BROKEN` line.
- Roadmap closure: `docs/roadmap.md` item 1.1.1 shows `[x]` and links the
  accepted ADR.
- Gates: `make markdownlint` reports no errors; `make check-fmt` is clean;
  `make nixie` passes (trivially, if no diagram is added);
  `coderabbit review --agent` raises no outstanding concerns.

Quality criteria (what "done" means):

- Documentation: one accepted ADR satisfying roadmap 1.1.1, reachable from the
  index, design, and terms of reference, with the candidate bullet resolved.
- Lint/format: `make markdownlint` and `make check-fmt` pass; `make nixie`
  passes.
- No code: `git diff --stat` touches only files under `docs/` (the ADR, the
  three synced documents, the roadmap, and this plan). No change under `src/`,
  `tests/`, or `Cargo.toml`.

Quality method (how we check): run the gate commands above with `tee`, run the
content and link checks, and review the `git diff --stat` before committing.

## Idempotence and recovery

Every step is safe to repeat. Authoring the ADR is a single file create;
re-running it overwrites the same content. The companion-document edits are
grep-guarded insertions and a checkbox flip ("add only if the
`adr-002-transition-boundary-scope.md` reference is absent"), so a blind re-run
of Stage D cannot append a duplicate index entry or companion-list line; if a
link is wrong, fix the one line and re-run the link check. There is nothing
destructive: no migrations, no deletions, no generated artefacts. If a gate
fails, correct the Markdown and re-run; the environment is left clean because
only tracked Markdown changes.

## Artefacts and notes

The following is the full proposed text for
`docs/adr-002-transition-boundary-scope.md`. The implementer creates the file
with this content, changing only the acceptance date to the merge date and
adjusting wording only if a Stage A review requires it. It mirrors the section
order, tone, and length of `docs/adr-001-proving-ground-candidates.md`.

````markdown
# Architectural decision record (ADR) 002: Scope Statelet to transition-boundary marking

## Status

Accepted, 2026-06-24. Statelet is scoped as a transition-boundary toolkit
(marker-only). The full decision is stated once, verbatim, under "Decision
outcome / proposed direction"; this summary does not restate it, so the two
cannot drift.

## Date

2026-06-24.

## Context and problem statement

Statelet enters a crowded market. On 2026-06-13, crates.io listed 154 crates
under the `state-machine` keyword,[^1] spanning hierarchical event-driven
engines,[^2] transition-table domain-specific languages (DSLs),[^3] static
embedded generators,[^4] typestate APIs[^5], and diagram or logging
helpers.[^6] A broad "another Rust state-machine framework" position is
therefore weak.

The terms of reference (§§1-6) and the technical design (§§1-3) instead stake a
narrow claim: Statelet helps teams that have *already* written an ordinary Rust
state machine — enums for modes, structs for accumulated state, methods for
transition boundaries, and `match` expressions for branch logic — by
standardizing how those boundaries are named, made fallible-explicit, and
observed. That claim only holds if Statelet refuses the responsibilities that
turn a helper into a framework.

That refusal is currently expressed as prose distributed across the terms of
reference, the design document, the README, and the user's guide. The roadmap
(item 1.1.1) requires it to be ratified once, as a single accepted decision,
because every later phase branches on it: the conventions-only baseline, the
second proving ground, and the conditional macro are each defined by what
Statelet does not own. Without one citable decision, "ship nothing", "ship
conventions only", and "ship the macro" are hard to separate, and reviewers
have no settled authority for declining framework-shaped requests.

The question this ADR answers is therefore narrow: what does Statelet own, and
what does it explicitly leave to user code and to existing crates?

## Decision drivers

- A defensible wedge requires a boundary that competing crates do not already
  occupy. Across the competing state-machine crates named in this ADR, each
  *generates* the machine; the marker-only position — instrumenting a machine
  the author has already written — is under-served (see Options considered).
- The primary user keeps explicit Rust control flow and wants it treated as a
  strength, not replaced by a generated dispatcher or transition table.
- The crate must be able to fail cleanly: if marking boundaries adds little
  over plain `#[tracing::instrument]`, the honest outcome is to ship nothing
  (the design's "Conventions baseline is also too weak" failure mode, §13.7). A
  clear scope makes that falsifiable.
- Reviewers need one authority to redirect framework-shaped feature requests
  to existing crates rather than re-litigating scope per issue.
- The decision must not pre-empt still-open questions (macro versus
  conventions, tracing defaults, second proving ground); it fixes only the
  ownership boundary.

## Validation evidence

The following reconnaissance examples test the boundary; they do not broaden it
or decide that the optional macro should ship. They use the proposed surface
from design §§6-9: an enum `StateName` derive returning stable labels,
documented `transition.*` fields, and, only if it beats the baseline, an
attribute whose `state(...)` and `event(...)` arguments are Rust expressions.
None supplies a dispatcher, event enum, or transition table.

### `mdtablefix`: conventions baseline

`mdtablefix` remains the first spike (design §12). `ContinuationMode` is an
existing enum in `src/wrap/paragraph/pending.rs`, while `ProcessBuffer` tracks
table mode as `bool in_table` in `src/process/buffer.rs`. The latter project
roadmap already proposes promoting that boolean to a small enum.
Instrumentation is currently uneven: `ProcessBuffer` emits `debug!`,
continuation handling uses `trace!`, and paths such as `handle_fence_line` are
silent.

The first phase must therefore keep the existing branches and apply a common
convention with ordinary `tracing` instrumentation:

```rust,ignore
#[derive(Debug, PartialEq, StateName)]
enum ContinuationMode {
    Normalize,
    TightCodeSpan,
    VerbatimFlush,
}

#[derive(StateName)]
enum BufferMode {
    Text,
    Table,
}

impl ProcessBuffer {
    #[tracing::instrument(
        level = "trace",
        skip(self, line),
        fields(
            transition.name = "handle_table_line",
            transition.state.before = self.mode.state_name(),
            transition.event = "source_line",
        )
    )]
    pub(super) fn handle_table_line(&mut self, line: String) -> Option<String> {
        // Existing branch logic remains here.
    }
}
```

Only demonstrated repetition or drift may justify the equivalent macro form:

```rust,ignore
#[statelet::transition(
    state(self.mode),
    event(line),
    infallible,
    tracing(level = "trace")
)]
pub(super) fn handle_table_line(&mut self, line: String) -> Option<String> {
    // Body remains byte-for-byte ordinary Rust.
}
```

The relevant `ProcessBuffer`, fence-tracker, and continuation paths do not
return `Result`. The design's requirement for a fallible transition therefore
does not apply to this spike. It remains a useful test of whether a convention
eliminates `debug!`-versus-`trace!` drift and names silent boundaries, but not
of `fallible` or `transition.error`.

### `wireframe`: primary proving ground

`wireframe` supplies the complementary case selected by
[ADR 001](adr-001-proving-ground-candidates.md). Its connection actor has the
explicit `RunState { Active, ShuttingDown, Finished }` in
`src/connection/state.rs`, generic `ActiveOutput<F, E>` in
`src/connection/output.rs`, and a fallible `ConnectionActor::dispatch_event` in
`src/connection/dispatch.rs`. The connection module has no tracing today, so
Statelet would add observability rather than duplicate an existing local
convention.

The boundary demonstrates both generic enum derivation and why the state
argument must accept an expression rather than a string: the actor state is a
function parameter, not a field on `self`.

```rust,ignore
#[derive(StateName)]
enum RunState {
    Active,
    ShuttingDown,
    Finished,
}

#[derive(StateName)]
enum ActiveOutput<F, E> {
    None,
    Response(FrameStream<F, E>),
    MultiPacket(MultiPacketContext<F>),
}

#[statelet::transition(
    state(state.run_state),
    event(event),
    fallible,
    tracing(level = "debug")
)]
fn dispatch_event(
    &mut self,
    event: Event<F, E>,
    state: &mut ActorState,
    out: &mut Vec<F>,
) -> Result<(), WireframeError<E>> {
    // The existing event match remains the dispatcher.
}
```

An error from `Event::Response` can then carry `transition.name`, the prior
state, and `transition.error` without a Statelet-shaped domain type.
`ActiveOutput::shutdown`, which replaces the active output with `None`, is the
matching infallible boundary. The distinct benefit is label alignment:
`crates/wireframe-verification` retains its Stateright model and proof
responsibilities, while `StateName` can give production logs, tests, and the
model the same `Active`, `ShuttingDown`, and `Finished` labels.

### `ddlint`: negative control

`ddlint` confirms why [ADR 001](adr-001-proving-ground-candidates.md) treats it
as the weakest fit. Its `StructLiteralState` in
`src/parser/expression/pratt.rs` is two `usize` counters, `active` and
`suspension`, manipulated by activation and suspension wrappers around closures.
`allows_struct_literals()` derives a boolean from those counters; there is no
state enum to name, and parser warnings use `log`, not `tracing`.

Forcing Statelet into this seam would first require a synthetic projection:

```rust,ignore
#[derive(StateName)]
enum GuardMode {
    Inactive,
    Active,
    Suspended,
}

impl StructLiteralState {
    fn mode(&self) -> GuardMode {
        // Derived solely from `active` and `suspension`.
    }
}
```

That would be decorative parser scaffolding. The projection would exist only to
satisfy a marker; its before-state describes a scoped region rather than a
boundary decision, and the important invariant remains counter balance and
underflow prevention. `transition.*` fields do not express that invariant.
Statelet should therefore not be introduced unless `ddlint` promotes this to a
real mode representation and moves the relevant diagnostics to `tracing`.

Together, the examples bracket the validation bet: `mdtablefix` tests whether
conventions beat ad hoc instrumentation but cannot exercise fallibility;
`wireframe` tests a fallible parameter-held state expression, generic derives,
and verification-name alignment; and `ddlint` is the negative control where the
correct outcome is "do not use Statelet". `TransitionOutcome` remains
unpublished (design §6.2), so any initial `transition.outcome` field must use
the project-local decision or return shape already present at the boundary.

## Options considered

<!-- markdownlint-disable MD013 -->

| Option                                  | What Statelet would own                                               | Closest existing crates              | Verdict           |
| --------------------------------------- | --------------------------------------------------------------------- | ------------------------------------ | ----------------- |
| 0. Ship nothing / keep convention local | Nothing; redirect users to `#[tracing::instrument]` plus helpers      | n/a (the status quo)                 | Deferred fallback |
| A. Transition-boundary marker (chosen)  | Naming and observability conventions at boundaries that already exist | No crate at the marker-only position | Selected          |
| B. Transition-table or graph owner      | The state set and legal moves, generated from a macro or DSL          | `stateless`, `smlang`                | Rejected          |
| C. Dispatch and event-engine owner      | An event-accepting runtime and dispatch loop                          | `statig`, `rust-fsm`, `finny`        | Rejected          |
| D. Diagram and graph-metadata owner     | A transition graph rendered to diagrams or validated for safety       | `macro-machines`; graph-first crates | Rejected          |

<!-- markdownlint-enable MD013 -->

*Table 1: Scope postures considered for Statelet and the crates that already
serve each posture.*

### Option 0: Ship nothing / keep the convention project-local

Statelet need not exist as a crate at all. If a stable state-naming contract
and documented tracing fields add little over plain
`#[tracing::instrument(fields(state = %self.mode))]`, the honest outcome is to
ship nothing and keep the pattern project-local. The design preserves this
outcome deliberately (its "Conventions baseline is also too weak" failure mode,
§13.7). This option is not selected *now* — it is the fallback the validation
phases may reach — but it is recorded here because the scope decision and the
ship/no-ship decision are distinct: choosing the marker-only scope does not
commit the project to publishing anything.

### Option A: Transition-boundary marker (chosen)

Statelet owns a small vocabulary and a set of conventions for *marking* the
points where an existing machine transitions: a stable state-naming contract,
documented `transition.*` tracing fields, and local helper guidance. The
machine's states, events, storage, error types, dispatch, and control flow stay
in user code. This marker-only position is under-served rather than crowded: of
the surveyed crates, every one generates the state machine, and the nearest
neighbours still do so even where they overlap Statelet's framing — `stateless`
shares the "separate structure from behaviour" wording but generates the table,
and `macro-machines` adds logging but generates the machine. Statelet instead
instruments one the author has already written.

This option fixes *scope*, not *delivery vehicle*. Whether the marker surface
ships as conventions only, as a trait crate, or eventually as an attribute
macro is explicitly **not** decided here; that fork is owned by the validation
phases and the design's deferred decisions (§14). The macro is added only if it
later beats the non-macro baseline (the "Macro is only `instrument` with extra
steps" gate, §13.6).

### Option B: Transition-table or graph owner

Statelet could generate the transition table — the states and the legal moves —
as `stateless` does with its zero-cost macro that separates structure from
behaviour. This is the closest dangerous edge: `stateless` already shares the
"separate structure from behaviour" framing, yet it still owns the generated
enums and lookup function. Taking this on would put Statelet in direct
competition with a mature crate and would move branch logic out of the user's
`match` expressions, contradicting the product thesis. Rejected.

### Option C: Dispatch and event-engine owner

Statelet could provide an event-accepting runtime and dispatch loop, as
`statig` and `rust-fsm` do. This is the well-served centre of the market and
would require users to hand over their control flow — the one thing the target
user most wants to keep. Rejected.

### Option D: Diagram and graph-metadata owner

Statelet could record a transition graph and render diagrams or assert graph
safety, as `macro-machines` does for Graphviz output. Diagram and safety
metadata pull the crate toward graph ownership and toward competing with
graph-first and typestate crates. v0.1 records no graph metadata and generates
no diagrams (the design's "Defer diagrams and transition-table metadata"
decision, §3.5). Rejected for now; any future metadata must not reshape user
code.

## Decision outcome / proposed direction

Statelet is scoped as a transition-boundary toolkit. **Statelet marks
boundaries and does not own dispatch, events, storage, transition tables, or
graph safety.** Its initial surface is expected to be state naming and
documented `tracing` fields around boundaries that already exist in ordinary
Rust code, but the delivery vehicle (conventions, trait crate, or macro) and
the tracing-default question remain open and are decided elsewhere; this ADR
fixes only the ownership boundary.

In the context of entering a crowded state-machine market, facing the risk that
Statelet drifts into a framework and competes with mature crates, the project
decides for a marker-only scope, and against owning transition tables (as
`stateless` does), dispatch and events (as `statig` and `rust-fsm` do),
storage, or graph safety, to keep the user's explicit Rust control flow as the
model and keep the wedge falsifiable, accepting that the resulting product is
smaller than a framework and may, on validation, prove too thin to ship at all.

## Goals and non-goals

- Goals:
  - Record one accepted boundary that later roadmap phases can cite.
  - Keep user-owned state, event, output, error, storage, and dispatch models
    in user code.
  - Position Statelet as complementary to, not competitive with, existing
    state-machine crates, with explicit redirections.
  - Preserve the ability to ship nothing if the boundary-marking surface proves
    too weak (the design's "Macro is only `instrument` with extra steps" and
    "Conventions baseline is also too weak" failure modes, §§13.6-13.7).
- Non-goals:
  - Own a generated dispatcher, a required event enum, a guard or action DSL, a
    transition table, or a state-machine object.
  - Provide compile-time graph safety or diagram generation in v0.1.
  - Provide a typestate API or claim `no_std`, interrupt-safety, or embedded
    suitability.
  - Hide branch logic; transitions should be easier to recognize, not harder to
    read.
  - Decide the still-open questions of macro-versus-conventions, the tracing
    default, or the second proving ground. Those are tracked separately.

## Known risks and limitations

- A marker-only scope may add too little over plain
  `#[tracing::instrument(fields(...))]`. This is an accepted, designed-for
  outcome: the validation phases may conclude "ship nothing" (design §13.7).
- The boundary against `stateless` is narrow. Reviewers must watch for feature
  requests that quietly reintroduce table or graph ownership.
- Observability polish can be mistaken for product validation. A clear scope
  reduces, but does not remove, that risk; the baseline comparison in design
  §11.2 remains the real gate.
- This ADR fixes scope only. It does not justify the macro, the second proving
  ground, or any dependency-default decision.

## Architectural rationale

This decision preserves Statelet's core boundary: ordinary Rust owns the state
machine, while Statelet standardizes names and observability at selected
transition boundaries. It aligns with the terms of reference (§§1-6, §6.2), the
design context and non-goals (§§1-3, §2.2), the README, and the user's guide,
consolidating their shared position into one citable record. It complements
[ADR 001](adr-001-proving-ground-candidates.md), which selects `wireframe` as
the proving ground that will test this boundary on production-shaped code
without Statelet taking ownership of `wireframe`'s Stateright model.

[^1]: crates.io keyword API for `state-machine`, reporting 154 crates on
    2026-06-13: `https://crates.io/api/v1/keywords/state-machine`
[^2]: docs.rs documentation for `statig`, accessed 2026-06-13:
    `https://docs.rs/statig/latest/statig/`
[^3]: docs.rs documentation for `smlang`, accessed 2026-06-13:
    `https://docs.rs/smlang/latest/smlang/`
[^4]: crates.io API for `sm`, accessed 2026-06-13:
    `https://crates.io/api/v1/crates/sm`
[^5]: docs.rs documentation for `sfsm` and `typed-fsm`, accessed 2026-06-13:
    `https://docs.rs/sfsm/latest/sfsm/`;
    `https://docs.rs/typed-fsm/latest/typed_fsm/`
[^6]: docs.rs source page for `macro-machines`, accessed 2026-06-13:
    `https://docs.rs/crate/macro-machines/latest/source/`; crates.io API for
    `macro-machines`, accessed 2026-06-13:
    `https://crates.io/api/v1/crates/macro-machines`
````

## Interfaces and dependencies

This item introduces no programmatic interface. Its "interfaces" are
documentation contracts:

- New file: `docs/adr-002-transition-boundary-scope.md`, conforming to the ADR
  template in `docs/documentation-style-guide.md`.
- Edited files: `docs/contents.md` (index entry), `docs/design.md` (companion
  list plus one citation), `docs/terms-of-reference.md` (companion list plus
  the resolved §10.2 candidate bullet), `docs/roadmap.md` (item 1.1.1 checkbox
  and success-line link).
- No new external dependency, no `Cargo.toml` change, no CI change. Introducing
  any of these is a tolerance breach (see `Tolerances`).

Signposted documentation and skills for the implementer:

- `docs/documentation-style-guide.md` — the normative ADR template, naming, and
  Markdown rules (MD013 wrap 80 / 120; tables and headings exempt).
- `docs/adr-001-proving-ground-candidates.md` — the structural and tonal model.
- `docs/terms-of-reference.md` and `docs/design.md` — the cited authorities the
  ADR consolidates.
- `execplans` skill — this plan's format and living-section discipline.
- The repository AGENTS.md rule that substantive decisions are recorded in an
  ADR and referenced from the design document.
- The `arch-decision-records` skill is intentionally *not* used: it encodes a
  Y-Statement template that this repository overrides with its own style guide.

## Revision note

- 2026-06-24: initial DRAFT. Establishes the four-stage plan, the embedded
  proposed ADR 002 artefact, the documentation-appropriate Red-Green-Refactor
  substitute, and the companion-document sync set. Remaining work is execution
  pending approval; no implementation has begun.
- 2026-06-24: post-review revision after a community-of-experts pass. Replaced
  the line-oriented acceptance grep with one shared, whitespace-folding
  `check_adr` predicate run for both red and green (fixing a false-RED on the
  80-column wrap and a false-GREEN from prose word matches); broadened the link
  check to all touched files including `roadmap.md` and the ADR's outbound
  link; made the companion-document edits grep-guarded and idempotent; replaced
  the interactive `$EDITOR` step with a direct file write; expanded Stage D
  Progress into sub-checkboxes. In the embedded ADR: added Option 0 (ship
  nothing / keep convention local) so the table spans the design's real
  decision axis; softened the "genuinely vacant" claim to "under-served" and
  scoped it to surveyed crates; trimmed unverified crate citations; stated the
  decision once with `Status` referring to it; noted that the
  conventions-vs-macro vehicle and tracing default are decided elsewhere; and
  switched brittle design-section numbers to name-plus-number citations. These
  changes affect only validation mechanics and ADR wording, not the plan's
  scope or stage structure.
- 2026-07-22: implementation began after plan approval. Stage A confirmed the
  local ADR convention and found no scope conflict among the design, terms of
  reference, README, and user's guide. Stage B remains the next step.
- 2026-07-22: completed the red-green documentation contract. The inline
  predicate failed because ADR 002 was absent, then passed after the ADR was
  created with the required decision sentence. Companion-document wiring and
  final validation remain.
- 2026-07-22: synchronized the index, design, terms of reference, and roadmap.
  The content predicate remains green and the all-touched-files link check
  produced no `BROKEN` output. Final gates and review remain.
- 2026-07-22: completed Stage D. All required Markdown gates passed and both
  requested CodeRabbit milestone reviews completed without reported concerns.
  The plan status is now COMPLETE; no implementation work remains.
- 2026-08-14: corrected five US-spelling occurrences to the repository's
  required Oxford British `artefact` after the post-rebase spelling gate
  reported them. This terminology-only repair does not change the completed
  ADR decision.
- 2026-08-17: review follow-up added ordered Appendix A footnotes for the
  market count and six capability categories, limited the competing-crate
  claim to this ADR, strengthened `check_adr` and the relative-link validator,
  and synchronized the embedded ADR's ADR 001 link. These are traceability and
  validation repairs only, not new product scope; `make markdownlint` and
  `make nixie` passed after the spelling correction.
- 2026-08-17: added validated `mdtablefix`, `wireframe`, and `ddlint`
  reconnaissance examples to ADR 002 and synchronized the embedded artefact.
  The examples preserve the marker-only decision: they test conventions,
  a fallible handwritten boundary, and an intentional non-adoption case rather
  than decide the macro or add framework responsibilities.
