//! Execution-boundary tests for the documented-example runner.

use std::cell::RefCell;

use super::{CommandOutcome, CommandRunner, run_examples, run_examples_with};
use crate::loader::Example;

/// Builds an example for one command.
fn example(command: &str) -> Example { named_example("fixture.md", "sample", command) }

/// Builds an example with a chosen document and identifier.
fn named_example(document_path: &'static str, id: &str, command: &str) -> Example {
    Example {
        id: id.to_owned(),
        document_path,
        language: "bash".to_owned(),
        command: command.to_owned(),
    }
}

/// Simulates a shell so orchestration is tested without spawning a process.
struct SimulatedShell;

impl CommandRunner for SimulatedShell {
    fn run(&self, command: &str) -> Result<CommandOutcome, String> {
        if command == "unstartable" {
            return Err("no such shell".to_owned());
        }
        let succeeded = command != "fail";
        Ok(CommandOutcome {
            succeeded,
            status: if succeeded {
                "exit status: 0".to_owned()
            } else {
                "exit status: 7".to_owned()
            },
            stdout: format!("stdout for {command}"),
            stderr: if succeeded {
                String::new()
            } else {
                "stderr for fail".to_owned()
            },
        })
    }
}

/// Records each command in the order the runner receives it.
#[derive(Default)]
struct RecordingShell {
    commands: RefCell<Vec<String>>,
}

impl CommandRunner for RecordingShell {
    fn run(&self, command: &str) -> Result<CommandOutcome, String> {
        self.commands.borrow_mut().push(command.to_owned());
        Ok(CommandOutcome {
            succeeded: true,
            status: "exit status: 0".to_owned(),
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

/// Runs every example that the simulated shell accepts.
#[test]
fn runs_successful_examples() {
    let examples = [example("ok")];
    run_examples_with(&examples, &SimulatedShell).expect("a successful example should pass");
}

/// Delivers every passing example to the runner in document order.
#[test]
fn runs_every_example_through_the_runner_in_order() {
    let examples = [
        named_example("first.md", "first", "echo one"),
        named_example("second.md", "second", "echo two"),
    ];
    let shell = RecordingShell::default();
    run_examples_with(&examples, &shell).expect("both examples should pass");
    assert_eq!(
        shell.commands.into_inner(),
        vec!["echo one".to_owned(), "echo two".to_owned()],
        "the runner should receive each command once, in document order"
    );
}

/// Reports the later failing example after an earlier one has passed.
#[test]
fn reports_the_later_failing_example() {
    let examples = [
        named_example("first.md", "first", "ok"),
        named_example("second.md", "second", "fail"),
    ];
    let error = run_examples_with(&examples, &SimulatedShell)
        .expect_err("the later failing example should be rejected");
    assert!(
        error.contains("second.md example `second` exited with exit status: 7"),
        "the error should name the failing example: {error}"
    );
    assert!(
        error.contains("stdout for fail") && error.contains("stderr for fail"),
        "the error should carry the failing example's output: {error}"
    );
    assert!(
        !error.contains("first.md"),
        "the passing example should not be reported: {error}"
    );
}

/// Reports a failing example with its exit status and captured output.
#[test]
fn reports_a_failing_example_with_its_output() {
    let examples = [example("fail")];
    let error = run_examples_with(&examples, &SimulatedShell)
        .expect_err("a failing example should be rejected");
    assert!(
        error.contains("fixture.md example `sample` exited with exit status: 7"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("stdout for fail"),
        "unexpected error: {error}"
    );
    assert!(
        error.contains("stderr for fail"),
        "unexpected error: {error}"
    );
}

/// Reports a shell that cannot be started rather than ignoring the example.
#[test]
fn reports_a_shell_that_cannot_start() {
    let examples = [example("unstartable")];
    let error = run_examples_with(&examples, &SimulatedShell)
        .expect_err("a start failure should be reported");
    assert!(
        error.contains("could not run fixture.md example `sample`: no such shell"),
        "unexpected error: {error}"
    );
}

/// Rejects an example whose fence declared a language other than Bash.
#[test]
fn rejects_an_example_with_an_unsupported_language() {
    let mut sample = example("ok");
    sample.language = "sh".to_owned();
    let error = run_examples_with(&[sample], &SimulatedShell)
        .expect_err("a non-Bash example should be rejected");
    assert!(
        error.contains("unsupported language `sh`"),
        "unexpected error: {error}"
    );
}

/// Runs a real command through the Bash adapter.
#[test]
fn bash_runner_executes_a_successful_command() {
    run_examples(&[example("printf 'ok\\n'")]).expect("Bash should run a successful command");
}

/// Propagates the real shell's non-zero exit through the Bash adapter.
#[test]
fn bash_runner_reports_a_failing_command() {
    let error =
        run_examples(&[example("exit 7")]).expect_err("Bash should report a non-zero exit status");
    assert!(error.contains("exited with"), "unexpected error: {error}");
}
