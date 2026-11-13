//! Checks that user-facing Markdown examples remain marked and executable.

#[path = "documented_examples/behaviour.rs"]
mod behaviour;
#[path = "documented_examples/loader.rs"]
mod loader;
#[path = "documented_examples/runner.rs"]
mod runner;

/// Loads the published user-facing documents, checks global identifier
/// uniqueness, and executes every Bash example from the repository root.
#[test]
fn published_user_facing_examples_are_executable() -> Result<(), String> {
    let mut examples = Vec::new();
    for document in loader::USER_FACING_DOCUMENTS {
        examples.extend(loader::load_non_empty_document(document)?);
    }

    loader::ensure_unique_identifiers(&examples)?;
    runner::run_examples(&examples)
}
