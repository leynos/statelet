# Validation notes

This directory holds the validation notes Statelet requires for its
evidence-gated decisions. A validation note is the record a validation task
produces: the observations that task made, written down in a shape a later
decision can read.

Notes live here, one file per task, named `<task>-<subject>.md` — for example
`2.2.1-mdtablefix-processbuffer.md`. The directory is shared by more than one
Phase 2 and Phase 3 decision; a note declares which contract it belongs to with
a marker comment, so a note written for one decision is not read as evidence
for another.

## Writing a note

1. Create a file here named `<task>-<subject>.md` after the task that produced
   the note and the subject it observed — `2.2.1-mdtablefix-processbuffer.md`,
   for example. Copy the fenced block from
   `docs/phase-2-validation-note-template.md` into it, marker comment and all,
   rather than the whole template file: the prose around that block is
   instructions for the person filling the form, not part of the note.
2. Fill every cell. A cell that still reads `TBD` is an incomplete note and
   will fail the contract test that owns it.
3. Cite where each observation was made, as `<repo>@<sha>:<path>`. A status
   with no citation is an assertion, not an observation.
4. Commit the note with the work that produced it, in the same pull request.

Never edit the template to record an observation. The template is the blank
form; editing it changes the schema for every future note and breaks the
contract that checks the form against the register it instantiates.

## Which contracts read this directory

- `tests/state_name_consumption_contract.rs` reads notes carrying the
  `<!-- state-name-note -->` marker. Those are the notes for roadmap task
  1.1.3's instrument and its `StateName` return-shape decision at task 3.2.1.
  The rule those notes are read under is
  `docs/adr-004-state-name-consumption-evidence.md`.

Further contracts will read notes carrying their own markers. A note is ignored
by every contract whose marker it does not declare.

## The marker declares the contract

A note's marker comment is its interface. It is what lets several unrelated
decisions share one directory without one decision's notes failing another
decision's checks. Adding a marker to a file is therefore a decision to have
that file checked by the owning contract, and the file must satisfy it.
