//! Keep `StateName` validation notes outside the macro cost schema.

use super::{NoteFile, check, open_roadmap, template};

#[test]
fn unrelated_state_name_notes_are_ignored_for_open_tasks() {
    let note = NoteFile {
        path: "docs/validation-notes/state-name.md".into(),
        text: "<!-- state-name-note -->\nnot a macro cost register".into(),
    };
    check(&template(), &open_roadmap(), &[note]).expect("unrelated StateName notes are ignored");
}
