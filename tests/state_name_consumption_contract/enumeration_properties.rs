//! Property suite over the enumeration predicate: the invariant that decides
//! whether an evidence cell lists the strings its state can return.
//!
//! `claims_scenarios.rs` holds the same predicate to twelve named cells, and
//! that table is where the *defects* are commemorated — each rejected case
//! there was accepted by the predicate as it stood when the case was written.
//! What a table cannot state is the rule itself. The rule is a relation over
//! arbitrary text an engineer writes, so its cases cannot be enumerated either:
//! the table can name ten phrasings and the eleventh arrives with the next note.
//! The properties below are the ones that must survive that eleventh phrasing.
//!
//! Each is stated against text the generator builds, with its oracle computed by
//! *construction* rather than by calling the predicate's own helpers. The
//! central property compares the verdict against the rule written out over the
//! generator's own inputs — a state that is non-empty, a verb between it and the
//! label, a label that is non-empty — so it is the specification asked of the
//! text, not a second copy of the implementation asked of the same.
//!
//! Non-vacuity is asserted beside each property rather than argued for it. A
//! generator whose accepted branch never reached the predicate would leave every
//! equality true for the wrong reason, so the vocabulary the generated cells are
//! drawn from is shown to satisfy the predicate directly.

use proptest::prelude::*;

use super::{
    claim_properties::{INERT_WORD, citation, neutral_prose},
    claims::{is_citation_shaped, lists_returned_strings},
    enumeration_witnesses::{ACCEPTED_VERBS, NEAR_MISS_VERBS},
};

/// Arbitrary content for a quoted span, empty and whitespace included.
///
/// Deliberately letters and spaces rather than the inert vocabulary: the
/// predicate reads a span for whether it is empty and for nothing else, so a
/// generator restricted to inert words would leave the *content*-insensitivity
/// half of the rule unstated. The span holds no backtick, which is the one thing
/// the generated shape depends on — a span containing the mark that delimits it
/// would not be one span.
fn span_text() -> impl Strategy<Value = String> { "[a-zA-Z ]{0,12}" }

/// A return verb in arbitrary case.
///
/// The predicate compares the verb case-insensitively, and this is what holds
/// it to that. Every form is emitted with each letter independently upper- or
/// lower-cased, so the generator reaches `Returns`, `RETURNS`, `rEtUrNs` and
/// the canonical spelling alike. Drawing only the canonical form would leave
/// that half of the rule unstated, and a predicate switched to a case-sensitive
/// comparison would agree with every property here while refusing the sentence
/// case an engineer actually writes at the head of a note.
fn varied_case_verb() -> impl Strategy<Value = String> {
    prop::sample::select(ACCEPTED_VERBS.to_vec()).prop_flat_map(|verb| {
        prop::collection::vec(any::<bool>(), verb.len()).prop_map(move |upper| {
            verb.chars()
                .zip(upper)
                .map(|(letter, is_upper)| {
                    if is_upper {
                        letter.to_ascii_uppercase()
                    } else {
                        letter
                    }
                })
                .collect()
        })
    })
}

/// A gap that carries a return verb as a word of its own, and the fact that it
/// does.
///
/// The flag is the generator's own account of which branch it took, and the
/// property under test is stated against it. It is computed by *construction* —
/// this branch always writes the verb between two spaces — so it cannot inherit
/// a mistake from the predicate's word-splitting.
fn gap_with_verb() -> impl Strategy<Value = (String, bool)> {
    (neutral_prose(), varied_case_verb(), neutral_prose())
        .prop_map(|(before, verb, after)| (format!("{before} {verb} {after}"), true))
}

/// A gap that carries no return verb, and the fact that it carries none.
///
/// Three ways to spell it, and all three are needed. Inert prose holds no word
/// at all; a near-miss holds a word that is verb-*shaped*; and a stem-prefixed
/// miss is where a whole-word rule differs from a prefix rule. A generator
/// producing only the first would leave the property true over a predicate that
/// matched any word starting `return`.
fn gap_without_verb() -> impl Strategy<Value = (String, bool)> {
    prop_oneof![
        neutral_prose().prop_map(|prose| (prose, false)),
        (
            neutral_prose(),
            prop::sample::select(NEAR_MISS_VERBS.to_vec()),
            neutral_prose(),
        )
            .prop_map(|(before, verb, after)| (format!("{before} {verb} {after}"), false)),
    ]
}

/// A gap that either carries a return verb or does not, without saying which.
///
/// The property that consumes this draws the flag and asserts against it, so the
/// branch is chosen before the cell is built and the assertion is the
/// specification rather than the generator's summary of its own output.
fn enumeration_gap() -> impl Strategy<Value = (String, bool)> {
    prop_oneof![gap_with_verb(), gap_without_verb()]
}

proptest! {
    /// The verdict is exactly the rule, over arbitrary text on both sides.
    ///
    /// The predicate's whole truth table, stated as an equality. An enumeration
    /// asks three things of a cell and nothing else: the state is non-empty, a
    /// return verb stands between it and what follows, and what follows is a
    /// non-empty span. The expected value below is that sentence written over
    /// the generator's own outputs — the state's and the label's emptiness are
    /// read off the strings this test built, not off anything the predicate
    /// computed — so a predicate that added a fourth condition, or dropped one,
    /// would disagree with it.
    ///
    /// The span content is arbitrary, which is the load-bearing half. A
    /// predicate reading the state for a *shape* — an upper-case initial, a
    /// suffix, a length — agrees with the rule on every case here and still
    /// refuses a state named `in_table`, which is one of the names the roadmap
    /// expects a note to record. Nothing but emptiness may be read from a span,
    /// and this is what says so.
    #[test]
    fn the_verdict_is_the_rule_over_arbitrary_span_text(
        state in span_text(),
        (gap, carries_verb) in enumeration_gap(),
        label in span_text(),
    ) {
        let cell = format!("`{state}` {gap} `{label}`");
        let expected =
            carries_verb && !state.trim().is_empty() && !label.trim().is_empty();
        prop_assert_eq!(
            lists_returned_strings(&cell),
            expected,
            "the cell {:?} must be {} by the rule that reads a non-empty state, a return verb \
             and a non-empty label. Repair: the predicate must ask for exactly those three, so \
             a cell failing one of them is refused and a cell meeting all three is accepted.",
            cell,
            if expected { "accepted" } else { "refused" }
        );
    }

    /// The verb is read where it joins the state to its labels, not anywhere.
    ///
    /// The metamorphic form of round thirty's finding. Moving the verb out of
    /// the gap and to either side of the cell leaves the same words in it and
    /// the same two spans, so a predicate that searched the whole cell would
    /// return the same verdict — and one that reads the gap must return the
    /// opposite. Both displacements are checked, because the two sides fail for
    /// different reasons: a verb ahead of the state is read by a whole-cell
    /// search from the prose *before* the first span, and one after the label is
    /// read from the prose beyond the last, which is the tail the zip never
    /// reaches.
    #[test]
    fn a_verb_away_from_the_gap_is_not_read(
        state in INERT_WORD,
        verb in prop::sample::select(ACCEPTED_VERBS.to_vec()),
        label in INERT_WORD,
    ) {
        let in_place = format!("`{state}` {verb} `{label}`");
        let ahead = format!("{verb} `{state}` `{label}`");
        let beyond = format!("`{state}` `{label}` {verb}");
        prop_assert!(
            lists_returned_strings(&in_place),
            "the cell {:?} carries the verb between the state and the label and must be \
             accepted, or the displacements below prove nothing",
            in_place
        );
        prop_assert!(
            !lists_returned_strings(&ahead),
            "the cell {:?} names the state and the label without saying the state returns one, \
             because its verb stands before both. A predicate that accepts this is reading the \
             verb from the cell rather than from the gap.",
            ahead
        );
        prop_assert!(
            !lists_returned_strings(&beyond),
            "the cell {:?} names the state and the label without saying the state returns one, \
             because its verb stands after both. A predicate that accepts this is reading the \
             verb from the cell rather than from the gap.",
            beyond
        );
    }

    /// The verb is read from whichever gap holds it, not only the first.
    ///
    /// A cell may quote more than two spans — the state, then several labels —
    /// and the candidate pairs overlap: the second span is the label of the
    /// first pair and the state of the second. A predicate that read only the
    /// first gap would refuse a cell whose verb stands before its third span,
    /// which is the shape a note takes when it annotates the state with its
    /// reader and then lists the strings. The chosen gap is drawn, so every
    /// position is exercised and none is privileged.
    #[test]
    fn the_verb_is_read_from_whichever_gap_holds_it(
        spans in prop::collection::vec(INERT_WORD, 4..5),
        gaps in prop::collection::vec(neutral_prose(), 3..4),
        verb in prop::sample::select(ACCEPTED_VERBS.to_vec()),
        site in 0usize..3,
    ) {
        let mut written = gaps;
        if let Some(gap) = written.get_mut(site) {
            *gap = format!("{gap} {verb}");
        }
        let cell = spans
            .iter()
            .enumerate()
            .map(|(index, span)| {
                let gap = written.get(index).map_or("", String::as_str);
                format!("`{span}` {gap} ")
            })
            .collect::<String>();
        prop_assert!(
            lists_returned_strings(&cell),
            "the cell {:?} carries its verb in gap {} and must be accepted, because every pair of \
             consecutive spans is a candidate and the verb is read from the gap between them.",
            cell,
            site
        );
    }

    /// An unpaired mark refuses the cell, wherever the mark stands.
    ///
    /// Round thirty-one's finding, stated over positions rather than over one
    /// example: a cell with an odd number of marks has no pairing at all, and
    /// every such cell must be refused however its stray mark is placed.
    ///
    /// This asserts the *refusal*, which is the invariant a note depends on. It
    /// deliberately does not assert that each site is refused *because of the
    /// parity guard*: three of the four sites end the read early as well, so a
    /// predicate without the guard refuses them too and this property cannot
    /// witness the guard. Only a stray mark that leaves every completed pair
    /// intact isolates it, and `the_parity_guard_is_load_bearing` is the
    /// witness for that one. Splitting the two claims is what keeps this
    /// property from reading as evidence it is not.
    #[test]
    fn an_unpaired_mark_refuses_the_cell_wherever_it_stands(
        state in INERT_WORD,
        verb in prop::sample::select(ACCEPTED_VERBS.to_vec()),
        label in INERT_WORD,
        site in 0usize..4,
    ) {
        let balanced = format!("`{state}` {verb} `{label}`");
        prop_assert!(
            lists_returned_strings(&balanced),
            "the cell {:?} is balanced and carries its verb and must be accepted, or the \
             refusal below is not attributable to the stray mark",
            balanced
        );
        let stray = match site {
            0 => format!("`{balanced}"),
            1 => format!("`{state}`` {verb} `{label}`"),
            2 => format!("`{state}` ` {verb} `{label}`"),
            _ => format!("{balanced}`"),
        };
        prop_assert!(
            !lists_returned_strings(&stray),
            "the cell {:?} has an unpaired mark at site {} and must be refused: a mark with no \
             partner has not backticked the label that follows it, so the span before the break \
             is prose rather than the quoted string the message asks for.",
            stray,
            site
        );
    }

    /// A citation is neither a state nor a label.
    ///
    /// The coupling round thirty's other half closed. A citation is a single
    /// word carrying its own pair of marks, and a predicate that split the cell
    /// before removing it would read the citation's *path* as a span — or its
    /// revision, which is hexadecimal and could stand where a label belongs. The
    /// citation is removed first, so the mark it carries never reaches the
    /// split, and a cell whose only state or only label is a citation has
    /// nothing quoted where the rule asks for something. The `is_citation_shaped`
    /// assertion keeps the refusals from being vacuous: a word that was not a
    /// citation at all would be refused for the wrong reason.
    #[test]
    fn a_citation_is_neither_a_state_nor_a_label(
        cited in citation(),
        state in INERT_WORD,
        verb in prop::sample::select(ACCEPTED_VERBS.to_vec()),
        label in INERT_WORD,
    ) {
        prop_assert!(
            is_citation_shaped(&cited),
            "the generated word {:?} is not citation-shaped, so the refusals below would hold \
             for a word no citation rule applies to",
            cited
        );
        let as_state = format!("{cited} {verb} `{label}`");
        let as_label = format!("`{state}` {verb} {cited}");
        prop_assert!(
            !lists_returned_strings(&as_state),
            "the cell {:?} quotes no state: its leading span is a citation, which the scan \
             removes before reading, so nothing is left for the verb to say anything about.",
            as_state
        );
        prop_assert!(
            !lists_returned_strings(&as_label),
            "the cell {:?} quotes no label: its trailing span is a citation, which the scan \
             removes before reading, so the state is said to return a path the note cites \
             rather than a string it returns.",
            as_label
        );
    }

    /// A citation cannot repair an unpaired mark.
    ///
    /// The two rules are independent, and this is the case where an
    /// implementation that confused them would show it. A citation carries a
    /// balanced pair of marks of its own, so a predicate that removed citations
    /// *after* splitting, or that counted marks over the whole cell, would let
    /// the citation's pair complete an otherwise unpaired mark and accept a
    /// cell whose label was never closed. Removing the citation first is what
    /// makes the parity a claim about the note's own marks.
    #[test]
    fn a_citation_cannot_repair_an_unpaired_mark(
        cited in citation(),
        state in INERT_WORD,
        verb in prop::sample::select(ACCEPTED_VERBS.to_vec()),
        label in INERT_WORD,
    ) {
        let broken = format!("`{state}` {verb} `{label} {cited}");
        prop_assert!(
            !lists_returned_strings(&broken),
            "the cell {:?} leaves its label unclosed, and the citation after it must not be \
             read as the mark that closes it: the citation is removed before the marks are \
             counted, so the note's own marks are still unbalanced.",
            broken
        );
    }
}
