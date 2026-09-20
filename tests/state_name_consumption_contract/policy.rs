//! The rules a note is read by: admissibility, verdict, and the obligations
//! each of its cells carries.
//!
//! Nothing here panics on a document defect. `unwrap_used` and
//! `indexing_slicing` are denied, so parsing uses `split_once`, slice patterns,
//! `get` and `let ... else`. The vocabulary is never hardcoded: the blocking set
//! and the verdict both come from ADR 004's status register, so adding a status
//! is a documentation edit.
//!
//! The register's own consistency — that its vocabulary is closed and that the
//! default can still fall — is `registers.rs`'s question, not this module's.

use super::types::{NoteRow, Resolution, StatusRow, field_order};

/// The field whose status can overturn the `&'static str` default.
pub(crate) const IDENTIFIER_NEED: &str = "identifier-need";

/// The status recording that a required property was observed.
pub(crate) const PROPERTY_REQUIRED: &str = "Property required";

/// The status recording that none was.
pub(crate) const NO_PROPERTY: &str = "None";

/// The contribution that ratifies the default.
pub(crate) const SUFFICIENT: &str = "Sufficient";

/// The contribution that overturns it.
pub(crate) const INSUFFICIENT: &str = "Insufficient";

/// The contribution that leaves the verdict untouched.
pub(crate) const NOTHING: &str = "nothing";

/// Resolves one note against the status register.
///
/// Admissibility and verdict are both read from the register, never hardcoded,
/// so the blocking set lives in the document.
pub(crate) fn resolve_note(rows: &[StatusRow], note: &[NoteRow]) -> Result<Resolution, String> {
    check_note_fields(rows, note)?;
    if let Some(blocked) = note.iter().find(|row| is_blocked(rows, row)) {
        return Ok(Resolution::NotResolved {
            field: blocked.field.clone(),
            status: blocked.status.clone(),
        });
    }
    contribution(rows, note)
}

/// Checks that a note carries one row per register field, in register order.
fn check_note_fields(rows: &[StatusRow], note: &[NoteRow]) -> Result<(), String> {
    let expected = field_order(rows);
    if note.len() != expected.len() {
        return Err(format!(
            "a StateName note records {} fields; the status register defines {}. Repair: fill one \
             row per register field, naming each exactly as ADR 004 does.",
            note.len(),
            expected.len()
        ));
    }
    for (row, field) in note.iter().zip(expected.iter()) {
        if &row.field != field {
            return Err(format!(
                "a StateName note records field {:?} where the status register defines {field:?}. \
                 Repair: use the register's field names, in the register's order.",
                row.field
            ));
        }
    }
    Ok(())
}

/// Returns whether the register marks this note cell's status inadmissible.
fn is_blocked(rows: &[StatusRow], note: &NoteRow) -> bool {
    rows.iter()
        .any(|row| row.field == note.field && row.status == note.status && !row.admissible)
}

/// Reduces an admissible note's cells to a verdict, rejecting a contradictory
/// note rather than resolving it.
///
/// ADR 004 defines a note's contribution as "the single non-`nothing` value
/// among the contributions its cells select", and says that a note selecting
/// two different contributions "is contradictory and is rejected rather than
/// resolved". Collapsing the pair into the stronger contribution would resolve
/// exactly the note the rule says to refuse, and would do it silently.
///
/// The pair is unreachable through the live register, whose only decisive
/// status sits on `identifier-need`; the guard is here because the register is
/// a document, and a future edit that adds a second decisive status must meet a
/// rejection rather than a verdict.
fn contribution(rows: &[StatusRow], note: &[NoteRow]) -> Result<Resolution, String> {
    let mut selected: Option<(&str, &str)> = None;
    for row in note {
        let Some(status) = rows
            .iter()
            .find(|status| status.field == row.field && status.status == row.status)
        else {
            continue;
        };
        if status.contributes == NOTHING {
            continue;
        }
        match selected {
            None => selected = Some((status.contributes.as_str(), row.field.as_str())),
            Some((contributes, _)) if contributes == status.contributes => {}
            Some((contributes, chosen)) => {
                return Err(format!(
                    "a StateName note selects {contributes} in field {chosen:?} and {} in field \
                     {:?}; a note is read as the single non-{NOTHING} contribution its cells \
                     select, and two different contributions are contradictory. Repair: leave one \
                     cell decisive and make the other contribute {NOTHING}.",
                    status.contributes, row.field
                ));
            }
        }
    }
    Ok(match selected {
        Some((contributes, _)) if contributes == INSUFFICIENT => Resolution::Insufficient,
        _ => Resolution::Sufficient,
    })
}

/// Checks a note's cells independently of its verdict: residual placeholders,
/// unknown statuses, citation shape, and the `identifier-need` obligations.
pub(crate) fn check_note_cells(rows: &[StatusRow], note: &[NoteRow]) -> Result<(), String> {
    for row in note {
        if row.status.contains("TBD") || row.evidence.contains("TBD") {
            return Err(format!(
                "a StateName note still holds TBD in field {:?}. Repair: the note records an \
                 observation, so every cell must be filled.",
                row.field
            ));
        }
        if !rows
            .iter()
            .any(|status| status.field == row.field && status.status == row.status)
        {
            return Err(format!(
                "a StateName note records status {:?} for field {:?}, which the status register \
                 does not define. Repair: use a status ADR 004 lists for that field.",
                row.status, row.field
            ));
        }
        if !is_citation_shaped(&row.evidence) {
            return Err(format!(
                "a StateName note records evidence for field {:?} that is not a citation of the \
                 shape <repo>@<sha>:<path>. Repair: cite the revision the observation was made \
                 against.",
                row.field
            ));
        }
    }
    check_identifier_need_evidence(note)
}

/// Checks the two obligations ADR 004 places on the `identifier-need` cell.
fn check_identifier_need_evidence(note: &[NoteRow]) -> Result<(), String> {
    let Some(row) = note.iter().find(|row| row.field == IDENTIFIER_NEED) else {
        return Ok(());
    };
    if !names_a_consumer(&row.evidence) {
        return Err(
            "a StateName note's identifier-need cell names no consumer from the search set. \
             Repair: name the tracing subscriber, the metrics recorder or its documented absence, \
             a model checker, or generated documentation — or state that none exist."
                .to_owned(),
        );
    }
    let names_property = names_a_property(&row.evidence);
    match row.status.as_str() {
        PROPERTY_REQUIRED if !names_property => Err("a StateName note records identifier-need: \
                                                     Property required but its evidence names \
                                                     none of equality, stability across \
                                                     releases, ordering, or compact encoding. \
                                                     Repair: name the property the variant-name \
                                                     &'static str fails to supply."
            .to_owned()),
        NO_PROPERTY if names_property => Err("a StateName note records identifier-need: None but \
                                              its evidence names a required property. Repair: \
                                              record Property required, or drop the claim — a \
                                              status and its evidence must agree."
            .to_owned()),
        _ => Ok(()),
    }
}

/// Whether a cell cites the revision it was observed against.
///
/// The message names `<repo>@<sha>:<path>`, so the predicate admits exactly that
/// shape and no weaker one. A check reading only "some word contains an `@`"
/// accepts `repo@revision` with no path and even a bare `@`, which is the defect
/// the message would then be promising something it never checked. Each of the
/// three components must also be non-empty: `@sha:path` and `repo@:path` are
/// citations of nothing.
fn is_citation_shaped(evidence: &str) -> bool { evidence.split_whitespace().any(is_citation) }

/// Whether one whitespace-delimited word is a complete citation.
fn is_citation(word: &str) -> bool {
    let Some(inner) = word
        .strip_prefix('`')
        .and_then(|rest| rest.strip_suffix('`'))
    else {
        return false;
    };
    let Some((repo, rest)) = inner.split_once('@') else {
        return false;
    };
    let Some((revision, path)) = rest.split_once(':') else {
        return false;
    };
    !repo.is_empty() && !revision.is_empty() && !path.is_empty()
}

/// Whether a cell names one of the consumers ADR 004's search set lists, or
/// states that none exist.
fn names_a_consumer(evidence: &str) -> bool {
    let lowered = evidence.to_lowercase();
    [
        "subscriber",
        "tracing",
        "metrics",
        "prometheus",
        "opentelemetry",
        "recorder",
        "model checker",
        "stateright",
        "documentation",
        "none exist",
        "no consumer",
    ]
    .iter()
    .any(|consumer| lowered.contains(consumer))
}

/// The four properties ADR 004 admits, as its third obligation names them.
const PROPERTIES: [&str; 5] = [
    "equality",
    "stability",
    "stable",
    "ordering",
    "compact encoding",
];

/// How many words before a keyword are searched for a negation.
///
/// Four rather than the three English usually needs, so that "none of them
/// need ordering" is read as the negative it is. Widening it further starts to
/// suppress genuine claims — "no crash was observed; requires stable ordering"
/// survives at four and not at much more.
const NEGATION_WINDOW: usize = 4;

/// Whether a cell names one of the four properties ADR 004 admits.
///
/// A keyword counts only when the cell *asserts* it. "no stability requirement
/// was observed" is an honest `None` cell — the property is named precisely to
/// record that nobody asked for it — and a bare substring test reads the word
/// "stability" and rejects the note for disagreeing with its own status. That
/// leaves an engineer no way to record a negative except by not mentioning the
/// property at all, which punishes the more informative note; the rule would be
/// demanding a euphemism rather than agreement.
///
/// The negation is therefore looked for in the words just before the keyword,
/// which is where English puts it, and never across a clause break, so a
/// negation in one clause cannot silence an assertion in the next. It is a
/// bounded heuristic rather than a parse, and deliberately a loose one: its
/// failure mode on unusual phrasing is to accept a note a stricter reader would
/// reject, which leaves the judgement where ADR 004 puts it — with the reviewer
/// at task 3.2.1 — instead of failing an honest note on its wording.
fn names_a_property(evidence: &str) -> bool {
    let lowered = evidence.to_lowercase();
    PROPERTIES.iter().any(|property| {
        lowered
            .match_indices(property)
            .any(|(index, _)| !is_negated(lowered.get(..index).unwrap_or_default()))
    })
}

/// Whether the words just before a keyword negate it.
fn is_negated(before: &str) -> bool {
    const NEGATIONS: [&str; 5] = ["no", "not", "never", "without", "none"];
    before
        .rsplit(['.', ';', ','])
        .next()
        .unwrap_or(before)
        .split_whitespace()
        .rev()
        .take(NEGATION_WINDOW)
        .any(|word| {
            // Punctuation is stripped because the word being tested arrives with
            // whatever a writer put beside it: "no," and "not." are the ordinary
            // forms. An apostrophe is kept, so that "doesn't" survives to the
            // `n't` test below rather than being cut at the contraction.
            let bare = word
                .trim_matches(|character: char| !character.is_alphanumeric() && character != '\'');
            NEGATIONS.contains(&bare) || bare.ends_with("n't")
        })
}
