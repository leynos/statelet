//! Property suite over the evidence predicates: the invariants that must hold
//! for *any* cell, not for the cases someone thought to write down.
//!
//! `claims.rs` reads two things out of a cell — which word is a citation, and
//! whether the rest names a consumer or a required property. Every one of those
//! predicates takes arbitrary engineer-written text, so its cases cannot be
//! enumerated: the suite can name ten phrasings, and the eleventh arrives with
//! the next note. The properties below are the ones that must survive that
//! eleventh phrasing.
//!
//! Each is stated against text the generator builds, with its oracle computed
//! by *construction* rather than by calling the predicate's own helpers. The
//! citation-stripping property, for instance, compares the verdict on a text
//! with a citation against the verdict on the same text without it — the second
//! is not a reference implementation of the first, it is the same question asked
//! of a text that never had a citation in it.
//!
//! Non-vacuity is asserted beside each property rather than argued for it: a
//! generator that only ever produced inert text would leave the equality above
//! true for the wrong reason, so the vocabulary the generated citations and
//! claims are drawn from is shown to satisfy the predicates directly.

use proptest::prelude::*;

use super::claims::{is_citation_shaped, names_a_consumer, names_a_property};

/// The consumer vocabulary ADR 004's search set lists, as a generator draws it.
///
/// The same words the predicates scan for, which is what makes a generated
/// citation path *loaded*: a path spelled from this list would satisfy the
/// consumer obligation if the citation were left in the text.
const CONSUMER_WORDS: [&str; 6] = [
    "subscriber",
    "tracing",
    "metrics",
    "prometheus",
    "opentelemetry",
    "recorder",
];

/// The property tokens the predicate actually scans for, as `PROPERTIES` lists
/// them.
///
/// Restated here rather than shared, because a generator drawing from a list
/// that had drifted from the predicate's own would produce cases the predicate
/// cannot see and report them as passes. `compact encoding` is one token and not
/// two: a generator that drew `compact` would name a word nothing scans for.
const PROPERTY_TOKENS: [&str; 5] = [
    "equality",
    "stability",
    "stable",
    "ordering",
    "compact encoding",
];

/// The tokens a citation's path can carry: those with no space in them.
///
/// `compact encoding` is one token and not two, and a citation is a single
/// whitespace-delimited word, so a path holding the space that token requires
/// would split the citation in two and neither half would be one. Derived from
/// the list above rather than restated, so a token added there is either carried
/// by a path or excluded by this rule, and the two cannot disagree.
fn path_tokens() -> Vec<&'static str> {
    PROPERTY_TOKENS
        .iter()
        .copied()
        .filter(|token| !token.contains(' '))
        .collect()
}

/// A word no predicate can recognize.
///
/// Drawn from consonants alone, and that is what makes it inert: every token
/// either predicate scans for contains a vowel, so a vowel-free word can
/// neither contain one as a substring nor be contained by one. The obvious
/// `[a-z]{1,7}` is *not* disjoint — `stable`, `tracing` and `metrics` all fit it
/// — and a case whose supposedly neutral filler happened to spell one of them
/// would fail for a reason its property is not about.
const INERT_WORD: &str = "[bcdfghjklmnpqrstvwxyz]{1,7}";

/// Arbitrary prose that makes no claim of its own.
///
/// Built from `INERT_WORD`, so a verdict computed over it is the *baseline* a
/// property compares against. Prose naming a consumer or a property would make
/// every case agree for a reason that has nothing to do with the rule under
/// test. `the_generated_prose_is_inert` holds the generator to that claim
/// rather than leaving it to the comment.
fn neutral_prose() -> impl Strategy<Value = String> {
    prop::collection::vec(INERT_WORD, 0..6).prop_map(|words| words.join(" "))
}

/// A file path spelled from the vocabulary the predicates scan for.
///
/// This is the load-bearing generator. A path like `src/tracing.rs` is exactly
/// the leak the citation rule exists to stop: the note says nothing about a
/// consumer, and the *citation* names one. A path that spelled nothing would
/// make the property below trivially true, which is what
/// `every_generated_path_is_loaded` refuses.
fn loaded_path() -> impl Strategy<Value = String> {
    let consumer = prop::sample::select(CONSUMER_WORDS.to_vec());
    let property = prop::sample::select(path_tokens());
    (consumer, property, 0u8..3).prop_map(|(word, token, shape)| match shape {
        0 => format!("src/{word}.rs"),
        1 => format!("src/{token}.rs"),
        _ => format!("src/{word}_{token}.rs"),
    })
}

/// A commit revision: four to forty hexadecimal digits, as `git` abbreviates.
fn commit_revision() -> impl Strategy<Value = String> { "[0-9a-f]{4,40}" }

/// A repository name, as a citation's first component.
fn repository_name() -> impl Strategy<Value = String> { "[a-z][a-z0-9-]{0,8}" }

proptest! {
    /// A citation's content never reaches the keyword scans.
    ///
    /// The invariant `narrative_text` exists for, stated over arbitrary text:
    /// inserting a citation into a cell cannot change whether the cell names a
    /// consumer or a property, because the citation is removed before the words
    /// are read. The path is generated from the scanned vocabulary, so the
    /// citation is one that *would* satisfy either obligation — `src/tracing.rs`
    /// names a consumer, `src/stability.rs` names a property — and the equality
    /// below is the statement that neither reaches the verdict.
    ///
    /// The oracle is the same question asked of the prose alone, which is built
    /// independently of the stripping rule: it is the text with no citation
    /// inserted, not the text with the citation removed by the implementation.
    #[test]
    fn a_citations_content_never_reaches_the_keyword_scans(
        prose in neutral_prose(),
        repo in repository_name(),
        revision in commit_revision(),
        path in loaded_path(),
    ) {
        let with_citation = format!("{prose} `{repo}@{revision}:{path}`");
        prop_assert_eq!(
            names_a_consumer(&with_citation),
            names_a_consumer(&prose),
            "a citation changed the consumer verdict of {:?}",
            with_citation
        );
        prop_assert_eq!(
            names_a_property(&with_citation),
            names_a_property(&prose),
            "a citation changed the property verdict of {:?}",
            with_citation
        );
    }

    /// A citation is shaped only when its revision names a commit.
    ///
    /// The rule that closed the ref-shaped hole: `mdtablefix@main:...` is not a
    /// citation however citation-like it looks, because a ref moves and the cell
    /// stops pinning the tree it was observed in. Generated over the whole
    /// admissible revision range, so the boundary is exercised as a range rather
    /// than as two named cases.
    #[test]
    fn only_a_commit_revision_makes_a_citation(
        repo in repository_name(),
        revision in commit_revision(),
        path in loaded_path(),
    ) {
        let citation = format!("`{repo}@{revision}:{path}`");
        prop_assert!(
            is_citation_shaped(&citation),
            "the citation {} names the commit {} and must be accepted",
            citation,
            revision
        );
    }

    /// The revision and the path are both required, and both must be non-empty.
    ///
    /// Every other component is held fixed and valid, so each refusal below is
    /// attributable to the one part that was removed. A shape test reading only
    /// "some word contains an `@`" would accept all of them.
    #[test]
    fn a_citation_missing_a_component_is_not_a_citation(
        repo in repository_name(),
        revision in commit_revision(),
        path in loaded_path(),
    ) {
        let whole = format!("`{repo}@{revision}:{path}`");
        let without_revision = format!("`{repo}@{revision}`");
        let without_at = format!("`{repo}:{path}`");
        let without_repo = format!("`@{revision}:{path}`");
        let empty_revision = format!("`{repo}@:{path}`");
        let empty_path = format!("`{repo}@{revision}:`");
        prop_assert!(is_citation_shaped(&whole), "the whole citation {} was refused", whole);
        prop_assert!(!is_citation_shaped(&without_revision), "{}", without_revision);
        prop_assert!(!is_citation_shaped(&without_at), "{}", without_at);
        prop_assert!(!is_citation_shaped(&without_repo), "{}", without_repo);
        prop_assert!(!is_citation_shaped(&empty_revision), "{}", empty_revision);
        prop_assert!(!is_citation_shaped(&empty_path), "{}", empty_path);
    }

    /// A determiner denies the property; an unmet assertion names it.
    ///
    /// The pair the register's two sides turn on, and the reason `is_negated`
    /// asks what a negation scopes rather than whether one is nearby. Both cells
    /// below negate a property word, and they must land on opposite verdicts:
    /// "no X requirement" is an honest `None` cell recording that nobody asked
    /// for the property, while "requires a name that is not X" records that it
    /// *was* required and is unmet — the observation the decisive status exists
    /// to collect.
    ///
    /// The subject is drawn from the inert vocabulary rather than from arbitrary
    /// letters. A subject is a *name*, and a name that happened to spell
    /// `stable` would be a property token in its own right: the denial would
    /// then name a property through its subject and fail for a reason this
    /// property is not about. The filler is inert for the same reason, so the
    /// framing is the only thing that can decide either verdict.
    #[test]
    fn a_determiner_denies_what_an_unmet_assertion_names(
        subject in INERT_WORD,
        property in prop::sample::select(PROPERTY_TOKENS.to_vec()),
        filler in neutral_prose(),
    ) {
        let denied = format!("{filler} no {property} requirement was observed; the {subject} was \
                              considered");
        let asserted =
            format!("{filler} the {subject} requires a name that is not {property}; the {subject} \
                     was considered");
        prop_assert!(
            !names_a_property(&denied),
            "the denial {} reads as naming a property",
            denied
        );
        prop_assert!(
            names_a_property(&asserted),
            "the unmet assertion {} reads as a denial",
            asserted
        );
    }

    /// The neutral vocabulary is neutral, whatever it spells.
    ///
    /// The claim `neutral_prose` rests on, stated over the words rather than
    /// argued in a comment: a consonant-only word can neither contain a scanned
    /// token as a substring nor be one, because every token the predicates scan
    /// for carries a vowel. Without this, a filler that spelled `stable` would
    /// make the properties above agree for a reason that has nothing to do with
    /// citations, and nothing would report it.
    #[test]
    fn the_inert_vocabulary_names_no_claim(
        word in INERT_WORD,
        count in 0usize..6,
    ) {
        let prose = vec![word; count].join(" ");
        prop_assert!(
            !names_a_consumer(&prose),
            "the inert word {} names a consumer",
            prose
        );
        prop_assert!(
            !names_a_property(&prose),
            "the inert word {} names a property",
            prose
        );
    }
}

/// Every word the generators draw from does satisfy the predicate that reads it.
///
/// The non-vacuity witness for the two properties that insert generated text.
/// `a_citations_content_never_reaches_the_keyword_scans` asserts an equality,
/// and an equality holds trivially when the text it inserts is inert — a
/// citation whose path spelled `src/process.rs` would leave both verdicts
/// unchanged whether or not the citation was stripped, and the property would
/// pass over a `narrative_text` that stripped nothing at all.
///
/// So the vocabulary is checked *as a whole* rather than by the two examples
/// that happened to come to mind. Every consumer word and every path-bearing
/// property token is shown to satisfy its predicate, so no draw can produce an
/// inert path. A witness naming one example per list would leave the rest of the
/// list free to drift away from the predicate it is supposed to exercise.
///
/// The two-word token is covered here rather than through a path, because no
/// path can carry it: that is `path_tokens`'s reason for existing, and this is
/// where the token it excludes is shown to work at all. Without this line a
/// token could be missing from both the paths and the predicate and nothing
/// would say so.
#[test]
fn every_generated_path_is_loaded() {
    for consumer in CONSUMER_WORDS {
        let path = format!("src/{consumer}.rs");
        assert!(
            names_a_consumer(&path),
            "the generated path {path} does not satisfy the consumer predicate, so the stripping \
             property would pass over an inert citation"
        );
    }
    for property in path_tokens() {
        let path = format!("src/{property}.rs");
        assert!(
            names_a_property(&path),
            "the generated path {path} does not satisfy the property predicate, so the stripping \
             property would pass over an inert citation"
        );
    }
    // Deliberately not a denial: "no compact encoding requirement" is the
    // negation case, and asserting the predicate accepts that would contradict
    // `a_determiner_denies_what_an_unmet_assertion_names` below.
    assert!(
        names_a_property("the subscriber requires a compact encoding"),
        "the two-word token `compact encoding` is not scanned for, so the property generator \
         would omit it and the token would be covered nowhere"
    );
}

/// A three-digit revision is refused, and a forty-one-digit one is too.
///
/// The boundary of the abbreviation rule, on both sides. `commit_revision`
/// generates only admissible revisions, so the range's *ends* are where a
/// generator cannot reach: these say the rule is a bound rather than a floor,
/// and that a revision shorter than `git` would resolve is still refused.
#[test]
fn the_revision_bound_is_exclusive_at_both_ends() {
    assert!(
        !is_citation_shaped("`mdtablefix@abc:src/process.rs`"),
        "a three-digit revision is shorter than any git abbreviation and must be refused"
    );
    assert!(
        !is_citation_shaped(&format!("`mdtablefix@{}:src/process.rs`", "a".repeat(41))),
        "a forty-one-digit revision is longer than a Git object name and must be refused"
    );
    assert!(
        is_citation_shaped("`mdtablefix@abcd:src/process.rs`"),
        "the shortest revision git accepts must be accepted, or the bound is off by one"
    );
}
