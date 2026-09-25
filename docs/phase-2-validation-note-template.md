# Phase 2 validation note template

This is the blank form for a `StateName` consumption observation. Copy the
block below into a new file in `docs/validation-notes/`, named
`<task>-<subject>.md` after the task that produced the note and the subject it
observed — `2.2.1-mdtablefix-processbuffer.md`, for example — fill every cell,
and commit it with the work it describes.

The form is not documentation of a decision. It is the schema of one, and it is
machine-checked: `tests/state_name_consumption_contract.rs` reads this file,
extracts the table below, and compares its fields, in order, against the status
register in
[ADR 004](adr-004-state-name-consumption-evidence.md#status-register). Editing
this file therefore changes the schema for every future note, and the contract
will fail if the change and the register disagree.

Read ADR 004 before filling the form. It defines what makes a note *admissible*
and the rule that reads one note, or several, into an outcome. What follows is
only the mechanical instruction for making the cells legible to that rule.

The form:

```markdown
<!-- state-name-note -->
# Validation note: StateName consumption in `<example>`

Roadmap task: `<n.n.n>`.

## Note register

<!-- note-register:begin -->
| Field | Status | Evidence |
| --- | --- | --- |
| state-display-name | TBD | TBD |
| identifier-need | TBD | TBD |
| metrics-cardinality | TBD | TBD |
| tracing-use | TBD | TBD |
<!-- note-register:end -->
```

Two parts of that block are load-bearing. The marker comment is the note's
interface: it is what tells the contract this file is a `StateName` note rather
than a benchmark note, an exit note, or a decision note sharing the same
directory. The `## Note register` heading is what the contract bounds the table
by. Copy the block verbatim, including both.

## Filling each field

Every `Status` cell takes one of the statuses the status register lists for
that field, and no other. Every `Evidence` cell takes a citation of the shape
`<repo>@<sha>:<path>`, so that a status is traceable to an observation rather
than asserted. A status whose evidence is prose is rejected, including a status
of `None`: the absence of a need is itself an observation, and it has a place.

- `state-display-name` records the actual strings the annotated state can
  return, so that the cardinality bound below is derivable from the note rather
  than asserted by its author. A field recording only whether a named type
  exists is not admissible, because a reviewer deciding the fate of a
  `&'static str` would never see the strings. Neither is a cell whose evidence
  is the citation alone: `mdtablefix@abc1234:src/process.rs` on its own says
  where an observation was made and never what was observed. The citation is
  required here as it is everywhere, and here it is not sufficient.
- `identifier-need` names at least one consumer drawn from the search set — the
  tracing subscriber, the metrics recorder or its documented absence, any model
  checker, and generated documentation — or states that none of them exist. The
  search set is what makes "no identifier is needed" an observation rather than
  a bare negative existential. Its evidence must also agree with its status
  about whether a property is required: a cell naming equality, stability
  across releases, ordering, or compact encoding belongs with the status
  recording a required property, and a cell naming none must not.
- `metrics-cardinality` records whether the number of distinct labels the
  annotated state can produce is bounded, so its status is `Bounded` or
  `Unbounded` — never a count. The bound is read off the strings listed under
  `state-display-name`, which is why that field must enumerate them: a count
  written here would be an assertion by the note's author that nothing
  contradicts, and the status register would have nothing to audit.
- `tracing-use` records how the annotated state names itself in a trace.

## When a note blocks

Some statuses are inadmissible. A note selecting one is still committed, still
useful, and still checked: it resolves to `Not resolved`, names the blocking
field and status, and contributes nothing to the verdict. ADR 004's
*Admissibility* section states where the repair for such a note belongs.

A note is never deleted to make a suite pass. A blocked note that names its
blocker is a finding, and the finding is the point.
