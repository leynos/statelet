//! Behaviour-driven checks for the published user-facing examples.

use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{ScenarioState, given, scenario, then, when};

use super::{
    loader::{self, Document},
    runner,
};

/// Carries the chosen published document and the result of running its
/// examples between scenario steps.
#[derive(Default, ScenarioState)]
struct ExamplesWorld {
    document: Slot<Document<'static>>,
    result: Slot<Result<(), String>>,
}

/// Creates the empty slots each behavioural scenario fills.
#[fixture]
fn world() -> ExamplesWorld {
    // Every scenario starts with empty slots and selects its document in Given.
    ExamplesWorld::default()
}

/// Stores the published README for the scenario.
#[given("the published README")]
fn published_readme(world: &ExamplesWorld) -> Result<(), String> {
    world.document.set(loader::find_document("README.md")?);
    Ok(())
}

/// Stores the published user's guide for the scenario.
#[given("the published user's guide")]
fn published_users_guide(world: &ExamplesWorld) -> Result<(), String> {
    world
        .document
        .set(loader::find_document("docs/users-guide.md")?);
    Ok(())
}

/// Loads, validates, and executes the stored document's published examples.
#[when("its documented examples run")]
fn run_documented_examples(world: &ExamplesWorld) -> Result<(), String> {
    let document = world
        .document
        .get()
        .ok_or_else(|| "the scenario did not select a user-facing document".to_owned())?;
    let result = loader::load_non_empty_document(document).and_then(|examples| {
        loader::ensure_unique_identifiers(&examples)?;
        runner::run_examples(&examples)
    });
    world.result.set(result);
    Ok(())
}

/// Requires the stored document's examples to pass with unique identifiers.
#[then("every example passes and has a unique identifier")]
fn examples_pass(world: &ExamplesWorld) -> Result<(), String> {
    match world
        .result
        .get()
        .ok_or_else(|| "the example-running step did not record a result".to_owned())?
    {
        Ok(()) => Ok(()),
        Err(error) => Err(error),
    }
}

/// The README's published examples run successfully.
#[scenario(
    path = "tests/features/documented_examples.feature",
    name = "README examples run from published text"
)]
fn readme_examples_run(world: ExamplesWorld) { let _ = world; }

/// The user's guide published examples run successfully.
#[scenario(
    path = "tests/features/documented_examples.feature",
    name = "User guide examples run from published text"
)]
fn users_guide_examples_run(world: ExamplesWorld) { let _ = world; }
