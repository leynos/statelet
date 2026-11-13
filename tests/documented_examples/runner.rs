//! Runs published Bash examples through a narrow process boundary.
//!
//! The loader parses examples; this module executes them. Execution sits behind
//! [`CommandRunner`] so the failure paths can be exercised with a simulated
//! shell while the published documents still run their real commands.
//! [`BashRunner`] is the only production implementation.
//!
//! Scope and re-use: this port belongs to the documented-example test harness
//! alone. It is not a repository-wide command-execution abstraction, and a
//! second caller should reuse the harness rather than widen this trait.

use std::process::Command;

use super::loader::Example;

/// Executes each published Bash example from the checkout root.
///
/// # Errors
///
/// Returns an error if the shell cannot be started or if an example exits
/// unsuccessfully. The diagnostic includes captured standard output and error.
pub(super) fn run_examples(examples: &[Example]) -> Result<(), String> {
    run_examples_with(examples, &BashRunner)
}

/// Executes examples through an injected process boundary.
///
/// # Errors
///
/// Propagates a runner start failure, or reports the first unsuccessful example
/// with its captured output.
fn run_examples_with<R: CommandRunner>(examples: &[Example], runner: &R) -> Result<(), String> {
    for example in examples {
        if example.language != "bash" {
            return Err(format!(
                "{} example `{}` uses unsupported language `{}`",
                example.document_path, example.id, example.language
            ));
        }
        let outcome = runner.run(&example.command).map_err(|error| {
            format!(
                "could not run {} example `{}`: {error}",
                example.document_path, example.id
            )
        })?;
        if !outcome.succeeded {
            return Err(format!(
                "{} example `{}` exited with {}\nstdout:\n{}\nstderr:\n{}",
                example.document_path, example.id, outcome.status, outcome.stdout, outcome.stderr
            ));
        }
    }
    Ok(())
}

/// Captured result of running one documented example.
struct CommandOutcome {
    /// Whether the command exited successfully.
    succeeded: bool,
    /// Human-readable exit description used in diagnostics.
    status: String,
    /// Captured standard output.
    stdout: String,
    /// Captured standard error.
    stderr: String,
}

/// The narrow process boundary used to execute one example command.
trait CommandRunner {
    /// Runs one example command and captures its outcome.
    ///
    /// # Errors
    ///
    /// Returns an error when the command cannot be started.
    fn run(&self, command: &str) -> Result<CommandOutcome, String>;
}

/// Runs examples with the system Bash, from the checkout root.
struct BashRunner;

impl CommandRunner for BashRunner {
    fn run(&self, command: &str) -> Result<CommandOutcome, String> {
        let output = Command::new("bash")
            .args(["-e", "-u", "-o", "pipefail", "-c"])
            .arg(command)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .map_err(|error| error.to_string())?;
        Ok(CommandOutcome {
            succeeded: output.status.success(),
            status: output.status.to_string(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
#[path = "runner_cases.rs"]
mod tests;
