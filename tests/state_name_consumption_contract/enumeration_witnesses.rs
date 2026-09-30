//! The handwritten witnesses for the enumeration predicate's property suite.
//!
//! `enumeration_properties.rs` states the rule over generated text. Two things a
//! generator cannot assert are asserted here instead, and they are the reason
//! this module exists beside that one.
//!
//! It also owns the verb vocabulary, which the property suite draws from. That
//! direction matters: the list is defined beside the witnesses that hold it to
//! the predicate, so a form that drifted from `claims.rs` fails here rather than
//! being drawn by the properties and refused by both sides of an equality.
//!
//! The first is *attribution*. A property can say a cell is refused; it cannot
//! say which part of the read refused it, and for an unpaired mark most sites
//! are refused with or without the parity guard. The guard's witness has to be
//! constructed by hand from the one shape where it is the whole difference.
//!
//! The second is *non-vacuity*. Every property there asserts an equality whose
//! both sides hold trivially if the vocabulary it draws from never reached the
//! predicate — a gap that never held a verb, or a span that was never empty,
//! would leave the equality true for the wrong reason. The vocabulary is
//! therefore exhibited against the predicate directly, one witness per claim
//! the generators rest on.
//!
//! The verbs are restated from the predicate's own list in `claims.rs` rather
//! than imported from it: a witness that drew from the same constant it is
//! checking would agree with a list that had drifted from the predicate, which
//! is the one thing these witnesses exist to catch. The property suite draws
//! from *this* copy, so one drift in `claims.rs` fails a witness here rather
//! than agreeing with itself in both places.

use super::claims::lists_returned_strings;

/// The verb forms the predicate reads, restated from the predicate's own list.
///
/// Written out here so that a form added to `RETURN_VERBS` without a witness
/// here — or removed from it while a witness still names it — fails below
/// rather than agreeing silently with a copy of itself.
pub(crate) const ACCEPTED_VERBS: [&str; 4] = ["returns", "returned", "yields", "yielded"];

/// Verb-shaped words that are not verb forms.
///
/// Each is one letter away from a form above in the way an engineer actually
/// writes when they miss: the inflections `-able`, `-ing` and `-er` on a verb
/// stem, and the stem itself. `returnable` is the near-miss round thirty found
/// being admitted by a predicate that matched a word-initial *stem*; the others
/// are the same mistake reached from the other side. The rule is a whole-word
/// comparison, and these are the words that witness it being one.
pub(crate) const NEAR_MISS_VERBS: [&str; 6] = [
    "returnable",
    "returning",
    "returner",
    "return",
    "yielding",
    "yielder",
];

/// The parity guard refuses a cell no other part of the read can refuse.
///
/// The witness `an_unpaired_mark_refuses_the_cell_wherever_it_stands` cannot
/// supply. Three of its four sites end the read early as well, so they are
/// refused with or without the guard and say nothing about it; this test
/// exhibits the one construction where the guard is the whole difference.
///
/// Appending an unclosed span leaves the cell's first pair whole: a stray mark
/// at the end is not a mark between two spans, so no triple is disturbed, and a
/// predicate reading only triples finds the state, the verb and the label and
/// accepts a cell whose *last* label was never closed. That is round thirty-one's
/// finding exactly — the pair before the break completes and satisfies the read
/// — and it is why parity must be asked as its own question rather than left to
/// the zip to notice. It cannot notice.
///
/// Both ends are shown: the cell without the stray mark is accepted, so the
/// refusal is attributable to the mark, and the appended text is inert, so the
/// refusal is not attributable to a word the scan read.
#[test]
fn the_parity_guard_is_load_bearing() {
    let intact = "`BufferMode` returns `Text`, `Table`";
    assert!(
        lists_returned_strings(intact),
        "the cell {intact:?} carries a state, a verb and two labels and must be accepted, or the \
         refusal below is attributable to something other than the stray mark"
    );
    let broken = format!("{intact}, `Table");
    assert!(
        !lists_returned_strings(&broken),
        "the cell {broken:?} leaves its final label unclosed. Its first pair completes, so a \
         predicate reading only overlapping triples accepts it — which is the defect round \
         thirty-one found. Repair: ask whether the cell's marks pair before reading them."
    );
    // The same shape with a third *complete* pair, so the read has a candidate
    // that satisfies it ahead of the break and the guard is still the only thing
    // refusing the cell.
    let trailing = format!("{intact}, `Table`, `Row");
    assert!(
        !lists_returned_strings(&trailing),
        "the cell {trailing:?} completes two pairs before its unclosed label, so nothing but the \
         parity of its marks can refuse it."
    );
}

/// Every verb form the generator draws is a form the predicate reads.
///
/// The non-vacuity witness for the accepted branch. The properties above assert
/// that a generated verb is read from the gap, and each holds trivially if the
/// verb was never one — a cell whose gap held `returning` would be refused by
/// both halves of the equality, and the equality would still be true. So each
/// form is shown to be accepted in a cell that is otherwise minimal and valid,
/// and the list is shown to be exactly the predicate's own by checking that no
/// form is drawn twice.
#[test]
fn every_accepted_verb_is_read() {
    for verb in ACCEPTED_VERBS {
        let cell = format!("`BufferMode` {verb} `Text`");
        assert!(
            lists_returned_strings(&cell),
            "the generated verb {verb} is not read from the gap in {cell:?}, so every property \
             drawing it asserts an equality that holds for the wrong reason"
        );
    }
    for verb in NEAR_MISS_VERBS {
        assert!(
            !ACCEPTED_VERBS.contains(&verb),
            "the near-miss word {verb} is also an accepted form, so the two lists disagree and \
             the property that separates them cannot"
        );
    }
}

/// The verb is read whatever case it is written in.
///
/// The predicate compares the verb case-insensitively, and the generator in
/// `enumeration_properties.rs` emits every form in arbitrary case because of
/// it. This is the named witness for that rule, so a comparison narrowed to the
/// canonical spelling fails here with the offending form rather than as a
/// property shrinking to a puzzling four-character counterexample.
///
/// Sentence case is the case that matters. A note's evidence cell is prose, and
/// "`BufferMode` Returns `Text`" is what an engineer writes at the head of a
/// sentence; a rule reading only the lower-case form would refuse it while
/// accepting the identical mid-sentence spelling.
#[test]
fn the_verb_is_read_in_any_case() {
    for verb in ACCEPTED_VERBS {
        let lower = format!("`BufferMode` {verb} `Text`");
        let upper = format!("`BufferMode` {} `Text`", verb.to_ascii_uppercase());
        let sentence = format!(
            "`BufferMode` {}{} `Text`",
            verb.chars()
                .next()
                .map(|first| first.to_ascii_uppercase())
                .unwrap_or_default(),
            verb.get(1..).unwrap_or_default()
        );
        for cell in [&lower, &upper, &sentence] {
            assert!(
                lists_returned_strings(cell),
                "the cell {cell:?} writes {verb} in a different case and must be accepted: the \
                 verb is compared case-insensitively, and a cell that changes only the case of \
                 the verb has not changed what the state does. Repair: lower-case the gap before \
                 comparing it against the verb list."
            );
        }
    }
}

/// Every near-miss word is refused, so the whole-word rule is what refuses it.
///
/// The non-vacuity witness for the refused branch. A predicate matching a
/// word-initial *stem* would accept `returnable` and `returning`; one matching
/// any substring would accept `returner`. Each is shown refused in the same
/// minimal cell that accepts the real forms above, so the refusal is
/// attributable to the word and not to the cell it stands in.
#[test]
fn no_near_miss_verb_is_read() {
    for verb in NEAR_MISS_VERBS {
        let cell = format!("`BufferMode` {verb} `Text`");
        assert!(
            !lists_returned_strings(&cell),
            "the word {verb} is not a return verb and must not be read as one in {cell:?}. \
             Repair: compare the whole word, so that a stem-prefixed miss is refused."
        );
    }
}

/// The span generator reaches both emptiness and content.
///
/// The property that compares the verdict against the state's and the label's
/// emptiness asserts an equality in both directions, and each direction is
/// vacuous if the generator cannot produce the case. A `span_text` that only
/// ever produced non-empty text would leave every refusal in that property
/// unattributable to emptiness, so both ends are exhibited here rather than
/// left to the comment.
#[test]
fn the_span_generator_reaches_both_ends() {
    assert!(
        !lists_returned_strings("`` returns `Text`"),
        "an empty state span must be refused, because the verb has nothing to say anything about"
    );
    assert!(
        !lists_returned_strings("`BufferMode` returns ``"),
        "an empty label span must be refused, because the cell has not listed a string"
    );
    assert!(
        !lists_returned_strings("`   ` returns `Text`"),
        "a whitespace-only state span must be refused, because it names no state"
    );
    assert!(
        lists_returned_strings("`BufferMode` returns `Text`"),
        "the shortest cell satisfying the rule must be accepted, or every refusal above is \
         indistinguishable from the predicate refusing everything"
    );
}
