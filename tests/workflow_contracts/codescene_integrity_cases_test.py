"""Mutation probes for CV-005's two coverage lanes and publisher preflight."""

import codescene_baseline as baseline
import codescene_calls as calls
import codescene_integrity as integrity
import codescene_reading as reading
import codescene_rules as rules
import pytest


@pytest.fixture(name="every")
def _workflow_fixture() -> dict[str, reading.Workflow]:
    """Read isolated workflow values so each mutation has its own subject."""
    return reading.workflows(reading.WORKFLOW_DIR)


def _publisher_steps(every: dict[str, reading.Workflow]) -> list[reading.Step]:
    """Return the publisher's step list, which each probe mutates locally."""
    return every["coverage-main.yml"]["jobs"]["coverage-upload"]["steps"]


def _coverage(every: dict[str, reading.Workflow], name: str) -> reading.Step:
    """Return the single coverage step in a named workflow."""
    return next(step for step in reading.steps(every[name]) if rules.is_coverage(step))


@pytest.mark.parametrize(
    ("change", "reason"),
    [
        ("pr_action_sha", "unapproved coverage action SHA"),
        ("publisher_action_sha", "unapproved coverage action SHA"),
        ("setup_sha", "setup-rust action has an unapproved SHA"),
        ("upload_sha", "uploader has an unapproved action SHA"),
        ("pr_features", "mismatched coverage inputs"),
        ("pr_language", "mismatched coverage inputs"),
        ("publisher_targets", "mismatched coverage inputs"),
        ("publisher_language", "mismatched coverage inputs"),
        ("publisher_report", "mismatched coverage inputs"),
        ("pr_linker", "mismatched coverage toolchain/linker"),
        ("publisher_profile", "mismatched build profile"),
        ("pr_soft_failure", "coverage step may skip or fail softly"),
        ("no_install", "build-tool install is absent"),
        ("wrong_install_command", "build-tool install is absent"),
        ("install_soft_failure", "build-tool install is absent"),
        ("late_install", "preflight must precede coverage"),
        ("no_linker", "clang/lld installation is absent"),
        ("late_linker", "preflight must precede coverage"),
        ("linker_soft_failure", "clang/lld installation is absent"),
        ("ci_no_install", "ci.yml build-tool preflight"),
        ("ci_late_install", "ci.yml build-tool preflight"),
        ("ci_install_soft_failure", "ci.yml build-tool preflight"),
        ("extra_linux_suite", "exactly one coverage-upload job"),
        ("extra_trigger", "main push and dispatch only"),
        ("checkout_credentials", "disable persisted credentials"),
        ("duplicate_check", "token check command is absent or altered"),
    ],
)
def test_integrity_mutations_are_rejected(
    every: dict[str, reading.Workflow], change: str, reason: str
) -> None:
    """A single wrong pin, scope, preflight or job shape breaks the contract."""
    _apply_integrity_mutation(every, change)
    findings = integrity.coverage_contract_findings(every)
    assert any(reason in finding for finding in findings), (change, findings)


def _apply_integrity_mutation(
    every: dict[str, reading.Workflow], change: str
) -> None:
    if change in COVERAGE_MUTATIONS:
        _mutate_coverage(every, change)
    elif change in PREFLIGHT_MUTATIONS:
        _mutate_preflight(every, change)
    else:
        _mutate_workflow_shape(every, change)


COVERAGE_MUTATIONS = {
    "pr_action_sha",
    "publisher_action_sha",
    "setup_sha",
    "upload_sha",
    "pr_features",
    "pr_language",
    "publisher_targets",
    "publisher_language",
    "publisher_report",
    "pr_linker",
    "publisher_profile",
    "pr_soft_failure",
}
PREFLIGHT_MUTATIONS = {
    "no_install",
    "wrong_install_command",
    "install_soft_failure",
    "late_install",
    "no_linker",
    "late_linker",
    "linker_soft_failure",
    "ci_no_install",
    "ci_late_install",
    "ci_install_soft_failure",
}


def _mutate_coverage(every: dict[str, reading.Workflow], change: str) -> None:
    publisher = every["coverage-main.yml"]
    pr_coverage = _coverage(every, "ci.yml")
    main_coverage = _coverage(every, "coverage-main.yml")
    match change:
        case "pr_action_sha":
            pr_coverage["uses"] = pr_coverage["uses"].replace(
                f"@{integrity.APPROVED_COVERAGE_ACTION_SHA}", "@4fb8eb7"
            )
        case "publisher_action_sha":
            main_coverage["uses"] = main_coverage["uses"].replace(
                f"@{integrity.APPROVED_COVERAGE_ACTION_SHA}", "@4fb8eb7"
            )
        case "setup_sha":
            setup = next(
                step
                for step in _publisher_steps(every)
                if "setup-rust" in reading.uses(step)
            )
            setup["uses"] += "x"
        case "upload_sha":
            uploader = next(
                step
                for step in _publisher_steps(every)
                if rules.is_upload_action(step)
            )
            uploader["uses"] += "x"
        case "pr_features":
            pr_coverage["with"]["all-features"] = "true"
        case "pr_language":
            pr_coverage["with"]["language"] = "mixed"
        case "publisher_targets":
            main_coverage["with"]["all-targets"] = "true"
        case "publisher_language":
            main_coverage["with"].pop("language")
        case "publisher_report":
            main_coverage["with"]["output-path"] = "other.info"
        case "pr_linker":
            pr_coverage["env"]["RUSTFLAGS"] = ""
        case "publisher_profile":
            publisher["jobs"]["coverage-upload"]["env"]["BUILD_PROFILE"] = "release"
        case "pr_soft_failure":
            pr_coverage["continue-on-error"] = True


def _named_step(steps: list[reading.Step], name: str) -> reading.Step:
    return next(step for step in steps if step.get("name") == name)


def _move_named_step(steps: list[reading.Step], name: str, *, move_to_end: bool) -> None:
    step = _named_step(steps, name)
    steps.remove(step)
    if move_to_end:
        steps.append(step)


def _mutate_preflight(every: dict[str, reading.Workflow], change: str) -> None:
    match change:
        case "ci_no_install" | "ci_late_install" | "ci_install_soft_failure":
            _mutate_ci_preflight(every, change)
        case (
            "no_install"
            | "late_install"
            | "wrong_install_command"
            | "install_soft_failure"
            | "no_linker"
            | "late_linker"
            | "linker_soft_failure"
        ):
            _mutate_publisher_preflight(every, change)
        case _:
            raise AssertionError(f"unrecognized preflight mutation {change!r}")


def _mutate_publisher_preflight(
    every: dict[str, reading.Workflow], change: str
) -> None:
    steps = _publisher_steps(every)
    match change:
        case "no_install" | "late_install":
            _move_named_step(
                steps, "Install build tools", move_to_end=change == "late_install"
            )
        case "wrong_install_command":
            _named_step(steps, "Install build tools")["run"] = "true"
        case "install_soft_failure":
            _named_step(steps, "Install build tools")["continue-on-error"] = True
        case "no_linker" | "late_linker":
            _move_named_step(
                steps,
                "Install coverage linker tools",
                move_to_end=change == "late_linker",
            )
        case "linker_soft_failure":
            _named_step(steps, "Install coverage linker tools")[
                "continue-on-error"
            ] = True
        case _:
            raise AssertionError(
                f"unrecognized publisher preflight mutation {change!r}"
            )


def _mutate_ci_preflight(every: dict[str, reading.Workflow], change: str) -> None:
    ci_steps = every["ci.yml"]["jobs"]["build-test"]["steps"]
    match change:
        case "ci_install_soft_failure":
            _named_step(ci_steps, "Install build tools")["continue-on-error"] = True
        case "ci_no_install" | "ci_late_install":
            _move_named_step(
                ci_steps, "Install build tools", move_to_end=change == "ci_late_install"
            )
        case _:
            raise AssertionError(f"unrecognized CI preflight mutation {change!r}")


def _mutate_workflow_shape(every: dict[str, reading.Workflow], change: str) -> None:
    publisher = every["coverage-main.yml"]
    steps = _publisher_steps(every)
    match change:
        case "extra_linux_suite":
            publisher["jobs"]["other-linux-suite"] = {
                "runs-on": "ubuntu-latest",
                "steps": [{"run": "make test"}],
            }
        case "extra_trigger":
            publisher["on"]["schedule"] = [{"cron": "0 0 * * *"}]
        case "checkout_credentials":
            steps[0]["with"]["persist-credentials"] = True
        case "duplicate_check":
            steps.insert(-1, {"id": "other-check", "run": rules.CHECK_COMMAND})


@pytest.mark.parametrize(
    "reference",
    [
        "leynos/statelet/.github/workflows/called.yml@6dea5677a84fec60ca51b07202570e3af12ffdb4",
        "leynos/other/.github/workflows/called.yml@6dea5677a84fec60ca51b07202570e3af12ffdb4",
        "./.github/workflows/called.yml@main",
    ],
    ids=["qualified_self", "remote", "local_with_ref"],
)
def test_unprovable_pull_request_call_is_rejected(reference: str) -> None:
    """A qualified or remote call cannot silently escape the local closure."""
    caller = reading.parse(
        "caller.yml",
        "on: pull_request\njobs:\n  call:\n    uses: " + reference + "\n",
    )
    every = {"caller.yml": caller}
    findings = calls.unprovable_callees(every, reading.pull_request_closure(every))
    assert any("unprovable call" in finding for finding in findings), findings


def test_reviewed_remote_call_is_tied_to_its_immutable_pin() -> None:
    """The audited Dependabot route has one exact workflow, job and pin."""
    caller = reading.parse(
        "dependabot-automerge.yml",
        "on: pull_request_target\njobs:\n  automerge:\n    uses: "
        + calls.REVIEWED_AUTOMERGE_CALL
        + "\n    with:\n"
        + (
            "      pull-request-number: ${{ inputs.pull-request-number || "
            "github.event.pull_request.number }}\n"
        ),
    )
    every = {"dependabot-automerge.yml": caller}
    assert calls.unprovable_callees(every, reading.pull_request_closure(every)) == [], (
        "test_reviewed_remote_call_is_tied_to_its_immutable_pin contract failed"
    )
    caller["jobs"]["automerge"]["secrets"] = "inherit"
    assert any("inherit" in finding for finding in rules.pull_request_findings(caller)), (
        "test_reviewed_remote_call_is_tied_to_its_immutable_pin contract failed"
    )


@pytest.mark.parametrize(
    "prefix", ["./", "$/"], ids=["dot_prefixed", "dollar_prefixed"]
)
def test_a_called_baseline_writer_is_counted(prefix: str) -> None:
    """A push reaching a second ratcheted step through a call is two writers."""
    caller = reading.parse(
        "caller",
        "on:\n  push:\n    branches: ['**']\njobs:\n  call:\n"
        f"    uses: {prefix}.github/workflows/called.yml\n",
    )
    called = reading.parse(
        "called",
        "on: workflow_call\njobs:\n  measure:\n    steps:\n"
        "      - uses: leynos/shared-actions/.github/actions/generate-coverage@abc\n"
        "        with:\n          with-ratchet: 'true'\n",
    )
    every = {
        "coverage-main.yml": reading.workflows(reading.WORKFLOW_DIR)[
            "coverage-main.yml"
        ],
        "caller.yml": caller,
        "called.yml": called,
    }
    assert baseline.baseline_writers(every) == ["called.yml", "coverage-main.yml"], (
        "test_a_called_baseline_writer_is_counted contract failed"
    )
