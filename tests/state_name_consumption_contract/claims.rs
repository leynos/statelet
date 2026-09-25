//! What an evidence cell *says*: which of its words are a citation, and whether
//! the rest names a consumer or a required property.
//!
//! These are predicates over text, not obligations. `policy.rs` decides what
//! each answer obliges; this module only reads the cell. The split follows
//! tolerance 5's 300-line trigger, which `policy.rs` passed (D30): knowing what
//! a cell claims is a different job from judging the claim, and every predicate
//! here exists to answer a question the note obligations ask.
//!
//! Both name predicates read the cell *after* its citation has been removed,
//! which is `narrative_text`'s whole purpose: the citation's path is chosen by
//! whoever wrote the note, so a keyword scan that includes it reads a filename
//! as a claim.

/// Whether a cell cites the revision it was observed against.
///
/// The message names `<repo>@<sha>:<path>`, so the predicate admits exactly that
/// shape and no weaker one. A check reading only "some word contains an `@`"
/// accepts `repo@revision` with no path and even a bare `@`, which is the defect
/// the message would then be promising something it never checked. Each of the
/// three components must also be non-empty: `@sha:path` and `repo@:path` are
/// citations of nothing.
pub(crate) fn is_citation_shaped(evidence: &str) -> bool {
    evidence.split_whitespace().any(is_citation)
}

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

/// The words of a cell that make a claim, with its citations removed.
///
/// Every evidence cell carries a `<repo>@<sha>:<path>` citation, and that path
/// is chosen by whoever wrote the note: `src/tracing.rs` cites a citation
/// containing "tracing", and `src/stability.rs` one containing "stability".
/// A keyword scan over the whole cell therefore reads the *citation* as the
/// claim, so an engineer who observes nothing about tracing can satisfy the
/// consumer obligation by naming a file, and a cell recording a required
/// property can satisfy it from the path alone. Both obligations are about
/// what the note *says*, so the citation — which is checked separately, by
/// shape — is taken out before the words are read.
fn narrative_text(evidence: &str) -> String {
    evidence
        .split_whitespace()
        .filter(|word| !is_citation(word))
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase()
}

/// Whether a cell names one of the consumers ADR 004's search set lists, or
/// states that none exist.
///
/// The negative phrases cover both halves of the obligation's two phrasings:
/// the ADR's own wording, "states that none of them exist", and the shorter
/// forms an engineer is likely to write. "none of them exist" therefore appears
/// beside "none exist" rather than instead of it — the obligation is on the
/// *claim*, and a check that accepted only one wording would reject an honest
/// note for choosing the other.
pub(crate) fn names_a_consumer(evidence: &str) -> bool {
    let lowered = narrative_text(evidence);
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
        "none of them exist",
        "none exist",
        "no consumer",
    ]
    .iter()
    .any(|consumer| lowered.contains(consumer))
}

/// The four properties ADR 004 admits, as its third obligation names them,
/// plus the adjective one of them is normally written with.
///
/// The array holds five tokens for four properties: "stable" is the
/// adjectival form of "stability across releases", and an engineer writing
/// "requires stable ordering" has named the property as surely as one writing
/// "requires stability". Both spellings are here because the predicate is a
/// substring scan, and "stability" is not a substring of "stable".
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
pub(crate) fn names_a_property(evidence: &str) -> bool {
    let lowered = narrative_text(evidence);
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
