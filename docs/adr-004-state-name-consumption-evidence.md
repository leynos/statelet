# Architectural decision record (ADR) 004: Define the StateName consumption evidence

## Status

Accepted.

## Date

2026-09-19.

## Context and problem statement

Statelet intends to publish a trait whose entire public surface is one method,
`fn state_name(&self) -> &'static str`. The technical design does not know
whether that return type is sufficient, and says so: the `mdtablefix` baseline
"must scrutinize whether `&'static str` is enough for the first slice", and the
default "remains `&'static str` until a real example consumes something
stronger" (design section 6.1).

Roadmap task 3.2.1 is scheduled to "finalize the `StateName` return shape" from
evidence "backed by observed example consumption, not anticipation". No
instrument exists for that observation. Nothing in the repository states what a
Phase 2 engineer records, where it is recorded, or which rule turns a record
into an outcome. Roadmap task 1.1.3 asks for that instrument, and this record
is its decision half.

This record settles two things and deliberately leaves a third open:

- the shape of a validation note, and which notes count as usable evidence;
- the rule that reads one note, or several notes, and yields an outcome;
- not the verdict. Whether `&'static str` survives is task 3.2.1's decision,
  taken against evidence this record cannot foresee.

Writing the rule before the evidence is the point. A rule written afterwards
can be reverse-engineered from a result somebody has grown fond of, and the
task's own success criterion — "not anticipation" — forbids exactly that.

The question is narrower than it first appears. The roadmap asks whether "a
stable numeric identifier is needed". The operative word is *stable*, not
*numeric*: a `&'static str` that defaults to the Rust variant name is not
stable across a rename, and design section 9 declares the generated field names
"public operational API", so renaming a variant silently renames a label that
dashboards and log queries may depend on. An instrument that asks only whether
some consumer needed an operation a string cannot perform records "no" and
loses the requirement Phase 2 is most likely to surface.

## Decision drivers

- The rule must be total. Every reachable state of the evidence must map to
  exactly one outcome, including the state in which no evidence exists.
- Admissibility and verdict must be separate. Conflating them produces a
  procedure that cannot terminate, because a note recording a naming defect is
  neither usable evidence nor a verdict.
- The verdict must be able to overturn the current default. An instrument that
  cannot record the inconvenient answer is not a decision procedure.
- Evidence must be observable, diffable and reviewable in a pull request,
  because the Phase 2 engineer and the task 3.2.1 reviewer are different people
  working months apart.
- The instrument must not freeze roadmap task numbers. The project renumbers
  tasks with its own tooling, and completing a bound task must not break the
  build.
- The instrument must not create speculative public API. Design section 6.2
  refuses public surface without a named consumer.

## Options considered

| Option   | What it records                                   | Outcome  |
| -------- | ------------------------------------------------- | -------- |
| Option 0 | Nothing; task 3.2.1 judges from memory            | Rejected |
| Option A | Whether a consumer needed a numeric identifier    | Rejected |
| Option B | Two axes: a verdict axis and a naming-defect axis | Rejected |
| Option C | One status register plus an aggregation register  | Chosen   |

*Table 1: Options considered for the evidence instrument.*

Option 0 leaves the task's own success criterion — evidence rather than
anticipation — unenforceable, and leaves no artefact for a reviewer to inspect.

Option A is the obvious reading of the roadmap sentence and is the reading this
record exists to refuse. It records only whether a consumer needed an operation
that a string cannot perform, so it records "no" for the requirement that
matters most: label stability across a rename. It is also unfalsifiable in
practice, because "no consumer needed it" without a stated search set is a bare
negative existential.

Option B separates concerns but spends half its cells on a refusal to answer. A
naming defect — a name synthesized from data, or a leaked `String` — is not a
verdict about the return type, so a register that treats it as one has no
terminating procedure for those notes. Cardinality and naming defects are
better placed where they belong: in admissibility.

Option C keys one register on `(field, status)` and gives each row an
admissibility flag and a contribution. Both the blocking set and the verdict
are then derivable from the document rather than hardcoded in the contract
test, so adding a status is a documentation edit: the register is the semantic
source of truth, and the contract reads it rather than restating it. The
contract's fixtures do pin the register row for row, so a status added to the
register is added to the fixture in the same change — the edit is a
documentation edit, not an *unaccompanied* one. A second, much smaller register
maps the states of the note multiset to the outcomes at task 3.2.1, because a
rule that resolves one note and says nothing about three is total over the
wrong domain.

## Decision outcome / proposed direction

Statelet records its `StateName` consumption evidence as validation notes
instantiated from `docs/phase-2-validation-note-template.md` and committed to
`docs/validation-notes/`. Each note carries one status for every field of the
status register, plus an evidence cell citing where that status was observed.

The status register separates two questions that the earlier drafts of this
instrument ran together: whether a note is usable evidence at all, and what the
note contributes to the verdict. The aggregation register states how task 3.2.1
reads several notes together.

This record pronounces no verdict. The `&'static str` default stands until the
evidence recorded under this rule overturns it.

## Status register

This register is load-bearing: `tests/state_name_consumption_contract.rs`
parses it, and both the blocking set and the verdict are read from it. A note
is *admissible* when no cell it selects has `Admissible: no`. A note's
*contribution* is the single non-`nothing` value among the contributions its
cells select; a note selecting two different contributions is contradictory and
is rejected rather than resolved.

<!-- status-register:begin -->
| Field               | Status            | Admissible | Contributes  |
| ------------------- | ----------------- | ---------- | ------------ |
| state-display-name  | Enumerated        | yes        | nothing      |
| state-display-name  | Not a named type  | no         | nothing      |
| identifier-need     | None              | yes        | Sufficient   |
| identifier-need     | Property required | yes        | Insufficient |
| metrics-cardinality | Bounded           | yes        | nothing      |
| metrics-cardinality | Unbounded         | no         | nothing      |
| tracing-use         | Full              | yes        | nothing      |
| tracing-use         | Partial           | yes        | nothing      |
| tracing-use         | None              | yes        | nothing      |
<!-- status-register:end -->

*Table 2: Every admissible status for every field, whether it blocks the note,
and what it contributes to the verdict.*

The fields are the four nouns of roadmap task 1.1.3's success criterion, and
`state-display-name` is `Enumerated` only when the note lists the actual
strings each annotated state can return. That is what the roadmap noun "state
display name" asks for. A status recording only whether a named type exists
would leave `metrics-cardinality: Bounded` as an unaudited assertion, and a
reviewer at task 3.2.1 would decide the fate of a `&'static str` without ever
seeing the strings. With the names enumerated, the cardinality bound is
derivable from the note rather than asserted by its author, and the stability
case can be argued against concrete labels.

Three obligations on the evidence cells are checked, not merely asked for.

First, every evidence cell carries a citation of the shape
`<repo>@<sha>:<path>`, so that a status is always traceable to an observation
rather than asserted. This applies to every field, including a field whose
status is `None`: the absence of a need is itself an observation, and it has a
place.

Second, the evidence cell for `identifier-need` names at least one consumer
drawn from the search set — the tracing subscriber, the metrics recorder or its
documented absence, any model checker, and generated documentation — or states
that none of them exist. The search set is what makes "no identifier is needed"
an observation rather than a bare negative existential.

Third, the evidence cell for `identifier-need` agrees with its status about
whether a property is required. A cell that names one of equality, stability
across releases, ordering, or compact encoding must accompany the status that
records a required property, and a cell that names none must not. A property
named only to deny it is not a claim: "no stability requirement was observed"
belongs with the status that records none, because it is a more informative way
of saying so than silence, and the rule must not demand a euphemism in place of
agreement. A status and an evidence cell that disagree are rejected, because
the disagreement is the one defect a reader cannot see.

## Admissibility

A status whose register row says `Admissible: no` blocks the note. The blocked
note resolves to `Not resolved` and names the blocking field and status; it
does not silently become a verdict, and it does not become a verdict by being
ignored.

The repair for a blocked note belongs to the phase that owns the annotated
code, not to Statelet. A name synthesized from data, a `String` label, or a
state that is not a named type at all is an upstream finding: the note records
a link to the upstream issue and blocks the *Statelet* gate. This record never
instructs a Phase 2 engineer to land a refactor in a repository this roadmap
does not own.

Blocking is therefore not a dead end. A blocked note is still committed: it
names its blocker, and it contributes nothing to the verdict. If every note is
blocked, the aggregation register's "no admissible evidence" row applies and
the publication path is stopped on the record rather than on silence.

## Aggregation register

Three notes reach task 3.2.1, from roadmap tasks 2.2.1, 2.2.2 and 3.1.2. The
register below maps every reachable state of that multiset to exactly one
outcome.

<!-- aggregation-register:begin -->
| Contributing notes | Any insufficient | Outcome if publication proceeds |
| ------------------ | ---------------- | ------------------------------- |
| None               | n/a              | Blocked: no admissible evidence |
| One or more        | No               | Ratify the current return type  |
| One or more        | Yes              | Amend design 6.1 before publish |
<!-- aggregation-register:end -->

*Table 3: How task 3.2.1 reads the committed notes together. Every outcome is
conditional on publication proceeding: if ADR 003's gate G2 has already
selected exit E1, Statelet ships nothing and no return type is ratified.*

The first column counts *contributing* notes rather than every committed one. A
note rejected as contradictory contributes nothing, exactly as a blocked note
does, so neither is counted: a contradiction is a defect to repair in the note
that carries it, not a verdict, and reading it as evidence either way would
credit the multiset with a decision no note made. If every note is blocked or
rejected, no note contributes and the first row applies, which is what that row
is for.

## Worked example (illustration, not evidence)

The block below shows a filled note as it would be committed by roadmap task
2.2.1. It is an *illustration*: the citations are placeholders, no such
observation was made, and this block is not evidence for any part of this
record. Its purpose is to show what an adequate evidence cell looks like, so
that a Phase 2 engineer has a shape to imitate and a reviewer has a shape to
compare against. The cells are abbreviated to keep the fenced block narrow; the
obligations stated under "Status register" govern the real ones, and the
illustration is not machine-checked.

```markdown
<!-- state-name-note -->
# Validation note: StateName consumption in `mdtablefix`

Roadmap task: 2.2.1.

<!-- note-register:begin -->
| Field | Status | Evidence |
| --- | --- | --- |
| state-display-name | Enumerated | `mdtablefix@abc1234:src/process.rs`, `LineMode` |
| identifier-need | None | `mdtablefix@abc1234:src/process.rs`, subscriber, metrics and docs; no unmet property |
| metrics-cardinality | Bounded | `mdtablefix@abc1234:src/process.rs`, three names |
| tracing-use | Full | `mdtablefix@abc1234:src/process.rs`, emits `transition.state.before` |
<!-- note-register:end -->

## Observations

The named state this note enumerates is `LineMode`: that is the state task
2.2.1 annotated, and it is what the `Enumerated` status above covers.

The `bool in_table` local in `ProcessBuffer` is explicitly out of scope here.
It is not a named type, so it cannot carry a `StateName` implementation, and a
note whose *subject* were that local would instead record `state-display-name`
as `Not a named type`, resolve to `Not resolved`, and contribute nothing to the
verdict. Under *Admissibility* that is an upstream finding, and its repair
belongs to the phase that owns the annotated code. Recording it here as an
out-of-scope observation, rather than folding it into the status above, is what
keeps `Enumerated` an honest claim about the state it names. A note is read
through *one* subject — the template names the file `<task>-<subject>.md` for
that reason — and this note's subject is `LineMode`, not the local.
```

## Gates

Each gate names the roadmap task whose validation note this record expects, by
task *title* rather than task number, so that completing a gate does not break
the build. Gate S4 is the decision gate: it consumes the notes and records the
outcome.

<!-- gate-table:begin -->
| Gate | Roadmap task title fragment           |
| ---- | ------------------------------------- |
| S1   | Annotate `mdtablefix` `ProcessBuffer` |
| S2   | Annotate `mdtablefix` continuation    |
| S3   | Apply the conventions-only baseline   |
| S4   | Finalize the `StateName` return shape |
<!-- gate-table:end -->

*Table 4: Gates bound to roadmap tasks by title.*

## Evidence the record preserves

The rule above rests on three statements in other documents. They are quoted
here, and `tests/state_name_consumption_contract.rs` checks that each still
resolves, whitespace-folded, within its named section of its source document.

<!-- evidence:begin -->

*"The default remains `&'static str` until a real example consumes something
stronger" — design `6.1 State naming`.*

*"backed by observed example consumption, not anticipation" — roadmap
`3.2.1. Finalize the StateName return shape`.*

*"production logs, tests, and the model the same" — adr-002
`wireframe: primary proving ground`.*

<!-- evidence:end -->

## Goals and non-goals

- Goals:
  - Give every `StateName` consumption observation one shape, one home and one
    reader.
  - Separate admissibility from verdict, so a note that records a defect is
    neither silently accepted nor silently dropped.
  - Make the rule that reads several notes total, including the case of no
    evidence at all.
  - Keep the verdict open. The default stands until evidence overturns it.
- Non-goals:
  - Decide whether `&'static str` is sufficient. That is task 3.2.1.
  - Publish any trait, trait method or identifier type. `src/` is untouched.
  - Collect evidence. Task 1.1.3 supplies the instrument; Phase 2 supplies the
    observations.
  - Alter ADR 002's marker-only ownership boundary.

## Known risks and limitations

- A stable numeric identifier is not available for free.
  `std::mem::discriminant` does not qualify: the standard library documents
  that the discriminant of an enum variant may change if the enum definition
  changes,[^1] and transmuting `Discriminant<T>` to a primitive is undefined
  behaviour.[^1] An `as` cast on a fieldless enum is not sufficient either: a
  cast reports the variant's position, so inserting or reordering a variant
  silently changes it. Measured: one variant cast to `u8` gave `2`, then `3`
  after a fourth variant was inserted before it, then `0` after the enum was
  reordered. A value is durable only when every variant's discriminant is
  *explicitly assigned*, with a `#[repr(u8)]`-style representation when a width
  or representation contract is also required.[^2] Any such value must be
  hand-assigned, which means new public API and a new obligation on every user
  of the derive: a durable value they must not renumber. This is the cost that
  the verdict has to justify, and it is a cost the string default does not
  incur.
- A numeric identifier cannot reduce metric cardinality.[^3] `StateName` is a
  total function from a state to a label, and any identifier is a total
  function from the same state to a value. Substituting one for the other
  relabels the same domain, so the number of distinct values an observability
  backend sees is unchanged. Cardinality is a property of the value set, not of
  the type that represents it, which is why it gates admissibility and never
  decides the verdict.
- A note can be written from the type declaration rather than from observation:
  ninety seconds of reading an enum, dressed as evidence. The mitigation is
  procedural and partly machine-checked: every evidence cell must be a citation
  of the shape `<repo>@<sha>:<path>`, the contract test checks that shape, and
  the `identifier-need` cell must name its search set.
- The gates are validation points, not Rust language guarantees, and a later
  rewrite of a source document can make a quoted clause stale. The contract
  test turns that drift into a test failure rather than a silent divergence.
- The instrument is deliberately narrow. It records what the annotated examples
  consume; it cannot record a consumer that does not exist yet.

## Outstanding decisions

- Whether `&'static str` is sufficient, or whether a stable identifier is
  needed, remains open until task 3.2.1 evaluates the committed notes.
- Whether design section 6.1 grows explicit rename support — "allow explicit
  renames later only if examples prove the need" — is part of the same verdict,
  because the rename support is the cheap remedy that a stability finding
  selects.
- The technical design's deferred decisions remain governed by
  `docs/design.md` section 14.

## Architectural rationale

ADR 002 establishes that Statelet marks transition boundaries without owning
dispatch, events, storage, transition tables or graph safety, and leaves the
identifier question open. This record supplies the missing instrument for that
open question while keeping the boundary intact: it adds documents and a
contract test, and no runtime code.

The shape of the register follows from what the evidence has to be able to say.
The most likely real finding is not that some consumer needs an operation a
string cannot perform; it is that the variant-name string is not stable across
a rename, in a field design section 9 declares semver-relevant. A register
whose only admissible verdict axis is "a named consumer required a numeric
operation" could not record that finding at all. The instrument therefore
records a *required property* rather than an *operation*, and the axis is
labelled by the requirement rather than by the existence of a consumer — a
label that a careless filler would otherwise tick with a named consumer in hand.

The separation of admissibility from verdict is what makes the procedure
terminate. Cardinality is a property of the value set, names synthesized from
data are an upstream defect, and neither is a statement about the return type.
Placing both on the verdict axis spent half the domain on "cannot answer",
which is why this record gates them instead.

[^1]: `std::mem::discriminant` documentation, accessed 2026-09-20:
    `https://doc.rust-lang.org/std/mem/fn.discriminant.html`
[^2]: The Rust Reference, "Enumerations", accessed 2026-09-20:
    `https://doc.rust-lang.org/reference/items/enumerations.html`
[^3]: Prometheus documentation, "Metric and label naming", accessed
    2026-09-20: `https://prometheus.io/docs/practices/naming/`
