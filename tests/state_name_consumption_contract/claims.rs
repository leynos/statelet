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

/// The shortest revision `git` accepts as an abbreviation of an object name.
const MIN_ABBREVIATED_SHA: usize = 4;

/// The length of a full Git object name.
const FULL_SHA: usize = 40;

/// Whether a cell cites the revision it was observed against.
///
/// The message names `<repo>@<sha>:<path>`, so the predicate admits exactly that
/// shape and no weaker one. A check reading only "some word contains an `@`"
/// accepts `repo@revision` with no path and even a bare `@`, which is the defect
/// the message would then be promising something it never checked. Each of the
/// three components must also be non-empty: `@sha:path` and `repo@:path` are
/// citations of nothing.
///
/// The revision must be a commit name, which is what makes the citation do its
/// job. A cell is traceable to an observation only because the revision pins the
/// tree the observation was made in; `mdtablefix@main:src/process.rs` names a
/// *ref*, and a ref advances, so the moment `main` moves the cell stops
/// describing any revision in particular — the status has become an assertion
/// wearing a citation's shape. The rule is the one `git` uses for an abbreviated
/// object name: four to forty hexadecimal digits. Upper case is accepted because
/// it names the same object, and the bound is `git`'s rather than this
/// contract's invention: a revision failing it would not resolve.
///
/// `narrative_text` deliberately does *not* apply the revision rule. It strips
/// every citation-shaped word, and stripping a `main`-citing cell is the point:
/// leaving one in place would feed that cell's *path* — chosen by whoever wrote
/// the note — to the keyword scans below, which is the defect stripping exists
/// to prevent. Shape decides what is removed; only this predicate decides what
/// is acceptable.
pub(crate) fn is_citation_shaped(evidence: &str) -> bool {
    evidence
        .split_whitespace()
        .filter_map(citation_parts)
        .any(|(_, revision, _)| is_commit_revision(revision))
}

/// Whether a revision names a commit rather than a mutable ref.
///
/// A shape test and not a resolution test: no suite can check that `abc1234`
/// exists in a repository this contract does not read, so what is enforced is
/// that the revision *can* name one — which is exactly the difference between a
/// citation and a ref written where one belongs.
fn is_commit_revision(revision: &str) -> bool {
    (MIN_ABBREVIATED_SHA..=FULL_SHA).contains(&revision.len())
        && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// The three components of a citation-shaped word, or `None` if it is not one.
///
/// A citation ends the clause that cites it, so it arrives wearing the
/// punctuation that closed that clause. ADR 004's own worked example writes
/// "`mdtablefix@abc1234:src/process.rs`, `LineMode`", and an engineer copying
/// that shape writes a comma; a cell ending its citation with a full stop
/// writes a full stop. Trailing sentence punctuation is therefore stripped
/// before the shape is read, or the one shape the document teaches would be the
/// shape the check refused.
///
/// Only *trailing* punctuation is forgiven, and only the three marks that end a
/// clause. Nothing may precede the opening backtick, so a citation embedded in
/// a larger word is still not a citation, and a closing backtick inside a path
/// is still the end of the citation rather than a stray quote.
///
/// The revision is returned unvalidated. Whether it names a commit is a
/// different question from whether the word is citation-shaped — `narrative_text`
/// wants the shape alone — so `is_citation_shaped` asks the further question of
/// what this returns.
fn citation_parts(word: &str) -> Option<(&str, &str, &str)> {
    let trimmed = word.trim_end_matches([',', '.', ';']);
    let inner = trimmed.strip_prefix('`')?.strip_suffix('`')?;
    let (repo, rest) = inner.split_once('@')?;
    let (revision, path) = rest.split_once(':')?;
    (!repo.is_empty() && !revision.is_empty() && !path.is_empty()).then_some((repo, revision, path))
}

/// Whether one word is citation-shaped, whatever revision it names.
///
/// The shape alone, because that is what `narrative_text` needs: a word that
/// looks like a citation must come out of the scan even when its revision does
/// not name a commit, or a cell the shape check rejects would still leak its
/// path into the keyword predicates on the way to being rejected.
fn is_citation(word: &str) -> bool { citation_parts(word).is_some() }

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

/// The negations that scope a *requirement* rather than the property itself.
///
/// These are the determiners and the preposition that take the requirement away:
/// "no stability requirement", "none of them need ordering", "without stable
/// ordering". English puts them in the quantifier's own position, so wherever one
/// appears in the window it is denying that anybody asked for the property.
const REQUIREMENT_NEGATIONS: [&str; 4] = ["no", "none", "never", "without"];

/// The verbs that ask for a property in the first place.
///
/// Read only to disambiguate the adverb: "not" denies a requirement when it
/// governs one of these — "they do not need ordering" — and in no other case.
const REQUIREMENT_VERBS: [&str; 4] = ["need", "needs", "require", "requires"];

/// Whether a cell names one of the four properties ADR 004 admits.
///
/// A keyword counts only when the cell *asserts* it, and the two ways a cell can
/// mention a property without asserting it have to be told apart, because they
/// fall on opposite sides of the register.
///
/// The first is a denial: "no stability requirement was observed" is an honest
/// `None` cell — the property is named precisely to record that nobody asked for
/// it — and a bare substring test reads the word "stability" and rejects the note
/// for disagreeing with its own status. That leaves an engineer no way to record
/// a negative except by not mentioning the property at all, which punishes the
/// more informative note; the rule would be demanding a euphemism rather than
/// agreement.
///
/// The second is the record of a *lacking* property, which ADR 004 expects as its
/// central evidence: "the subscriber requires labels that are not stable across
/// releases" is exactly the observation that overturns the default, and reading
/// its "not stable" as a denial would reject the one note the instrument exists
/// to collect. The two are distinguished by what the negation scopes, which is
/// where English puts the difference: a determiner takes the requirement away
/// ("no stability requirement"), while the adverb negates whatever follows it and
/// therefore denies the requirement only when it governs a requirement verb —
/// "do not need ordering" denies one, "are not stable" says the property is
/// required and unmet.
///
/// The negation is looked for in the words just before the keyword, which is
/// where English puts it, and never across a clause break, so a negation in one
/// clause cannot silence an assertion in the next. It is a bounded heuristic
/// rather than a parse, deliberately a loose one: wording it does not cover is
/// read as an assertion, which is the side that leaves the judgement where ADR
/// 004 puts it — with the reviewer at task 3.2.1 — rather than rejecting a note
/// for its phrasing. A cell negating the property with neither a determiner nor
/// a requirement verb, as "the labels are never stable" does, is read as naming
/// it; so is one whose negation the window is too short to reach.
pub(crate) fn names_a_property(evidence: &str) -> bool {
    let lowered = narrative_text(evidence);
    PROPERTIES.iter().any(|property| {
        lowered
            .match_indices(property)
            .any(|(index, _)| !is_negated(lowered.get(..index).unwrap_or_default()))
    })
}

/// One word with the punctuation a writer put beside it trimmed away.
///
/// "no," and "not." are the ordinary forms, so the marks have to go before the
/// word is compared. An apostrophe is kept, so that "doesn't" survives to the
/// contraction test rather than being cut at its contraction.
fn bare_word(word: &str) -> &str {
    word.trim_matches(|character: char| !character.is_alphanumeric() && character != '\'')
}

/// Whether the words just before a keyword deny that the property was required.
fn is_negated(before: &str) -> bool {
    let words = before
        .rsplit(['.', ';', ','])
        .next()
        .unwrap_or(before)
        .split_whitespace()
        .map(bare_word)
        .collect::<Vec<&str>>();
    let window = words.len().saturating_sub(NEGATION_WINDOW);
    words.iter().enumerate().skip(window).any(|(offset, word)| {
        if REQUIREMENT_NEGATIONS.contains(word) {
            return true;
        }
        // "not" and the `n't` contractions. What they deny is what follows
        // them: a requirement verb names the requirement and so takes it
        // away, and anything else — the property word itself above all —
        // leaves the requirement standing and negates the value instead. A
        // contraction is the same adverb, so it is read the same way:
        // "doesn't need ordering" denies, "isn't stable" does not.
        if *word != "not" && !word.ends_with("n't") {
            return false;
        }
        words
            .get(offset + 1)
            .is_some_and(|next| REQUIREMENT_VERBS.contains(next))
    })
}
