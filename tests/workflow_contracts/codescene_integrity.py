"""Bind CV-005 coverage lanes to the reviewed action and local preflight.

This module owns the exact coverage configuration and publisher shape. The
broader CodeScene flow and token rules remain in ``codescene_rules``.
"""

import codescene_reading as reading
from codescene_rules import (
    CHECK_COMMAND,
    COVERAGE_ACTION,
    TOKEN_INPUT,
    UPLOAD_ACTION,
    is_coverage,
    is_upload_action,
    publishes_from_main,
    step_input,
)

# The current generate-coverage pin was merged in Statelet PR #91 and preserves
# the CV-005 ratchet and LLVM coverage route. Statelet PR #98 advanced the
# main publisher's upload action to the current shared-actions revision.
APPROVED_COVERAGE_ACTION_SHA = "013346ccfd37bd1e02eb430525233f1a4cd942b8"
APPROVED_UPLOAD_ACTION_SHA = "ff1dd759dfffc0db3459e30e833f52437ee62b57"
SETUP_ACTION = "leynos/shared-actions/.github/actions/setup-rust"
SETUP_ACTION_SHA = "c4ed5ffaf0640b1907d5359a87fd1677034eec27"
CHECKOUT_ACTION = "actions/checkout@f548e57e544e1ff5a4c46bf1e1b8685f8e4a348a"
INSTALL_COMMAND = "make install-build-tools"
LINKER_INSTALL_COMMAND = (
    "set -euo pipefail\n"
    "export DEBIAN_FRONTEND=noninteractive\n"
    "sudo apt-get update\n"
    "sudo apt-get install --yes --no-install-recommends clang lld\n"
)
TEST_RUNNER_COMMAND = (
    "cargo binstall --no-confirm --strategies crate-meta-data,quick-install cargo-nextest"
)
EXPECTED_COVERAGE_ENV = {
    "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER": "clang",
    "RUSTFLAGS": "-C link-arg=-fuse-ld=lld",
    "CFLAGS": "-fuse-ld=lld",
    "LDFLAGS": "-fuse-ld=lld",
}
EXPECTED_COVERAGE_INPUTS = {
    "language": "rust",
    "output-path": "lcov.info",
    "format": "lcov",
    "with-ratchet": "true",
    "publish-artefact": "false",
}


def coverage_contract_findings(every: dict[str, reading.Workflow]) -> list[str]:
    """Prove the two coverage lanes and the publisher's executable preflight.

    The approved action pin is the reviewed descendant of the CV-005 floor;
    its coverage action explicitly overrides the Cranelift development backend
    with LLVM before invoking cargo-llvm-cov. Equality to this immutable pin
    binds that reviewed action implementation to both lanes.
    """
    publisher = every.get("coverage-main.yml", {})
    findings = _coverage_lane_findings(every, publisher)
    findings.extend(_publisher_findings(publisher))
    return findings


def _coverage_lane_findings(
    every: dict[str, reading.Workflow], publisher: reading.Workflow
) -> list[str]:
    findings = _required_coverage_findings(publisher)
    for name, workflow in (
        ("ci.yml", every.get("ci.yml", {})),
        ("coverage-main.yml", publisher),
    ):
        findings.extend(_named_lane_findings(name, workflow))
    for name in reading.pull_request_closure(every) - {"ci.yml"}:
        findings.extend(_pull_request_lane_findings(name, every[name]))
    return findings


def _required_coverage_findings(publisher: reading.Workflow) -> list[str]:
    if set(reading.trigger_names(publisher)) == {"push", "workflow_dispatch"} and (
        publishes_from_main(publisher)
    ):
        return []
    return ["publisher triggers must be main push and dispatch only"]


def _named_lane_findings(name: str, workflow: reading.Workflow) -> list[str]:
    measured = [step for step in reading.steps(workflow) if is_coverage(step)]
    if len(measured) != 1:
        return [f"{name} must have exactly one coverage step"]
    findings = _coverage_step_findings(name, measured[0])
    if name == "ci.yml" and not _ci_has_prior_build_tools(workflow, measured[0]):
        findings.append("ci.yml build-tool preflight must precede coverage")
    return findings


def _coverage_step_findings(name: str, step: reading.Step) -> list[str]:
    findings = []
    if reading.uses(step) != f"{COVERAGE_ACTION}@{APPROVED_COVERAGE_ACTION_SHA}":
        findings.append(f"{name} uses an unapproved coverage action SHA")
    if step.get("with") != EXPECTED_COVERAGE_INPUTS:
        findings.append(f"{name} has mismatched coverage inputs")
    if step.get("env") != EXPECTED_COVERAGE_ENV:
        findings.append(f"{name} has mismatched coverage toolchain/linker settings")
    if "if" in step or "continue-on-error" in step:
        findings.append(f"{name} coverage step may skip or fail softly")
    return findings


def _ci_has_prior_build_tools(
    workflow: reading.Workflow, measured: reading.Step
) -> bool:
    carrying = [
        reading.job_steps(job)
        for _, job in reading.jobs(workflow)
        if any(candidate is measured for candidate in reading.job_steps(job))
    ]
    return len(carrying) == 1 and _has_prior_build_tools(carrying[0], measured)


def _pull_request_lane_findings(name: str, workflow: reading.Workflow) -> list[str]:
    findings = []
    for step in reading.steps(workflow):
        if is_coverage(step):
            findings.extend(_coverage_step_findings(name, step))
    return findings


def _publisher_findings(publisher: reading.Workflow) -> list[str]:
    findings, steps = _publisher_job(publisher)
    if steps is None:
        return findings
    setup_positions = _publisher_setup_findings(steps)
    findings.extend(setup_positions[0])
    measure = setup_positions[1]
    findings.extend(_publisher_upload_findings(steps))
    findings.extend(_publisher_token_findings(steps, measure))
    return findings


def _publisher_job(
    publisher: reading.Workflow,
) -> tuple[list[str], list[reading.Step] | None]:
    findings = []
    jobs = reading.jobs(publisher)
    if len(jobs) != 1 or jobs[0][0] != "coverage-upload":
        return ["publisher must have exactly one coverage-upload job"], None
    job = jobs[0][1]
    if "if" in job or "continue-on-error" in job:
        findings.append("publisher job may skip or fail softly")
    _publisher_environment_findings(publisher, job, findings)
    steps = reading.job_steps(job)
    if not _valid_publisher_checkout(steps):
        findings.append("publisher checkout must disable persisted credentials")
    return findings, steps


def _publisher_environment_findings(
    publisher: reading.Workflow, job: reading.Step, findings: list[str]
) -> None:
    read_permissions = {"contents": "read"}
    if (
        publisher.get("permissions") != read_permissions
        or job.get("permissions") != read_permissions
    ):
        findings.append("publisher permissions exceed contents: read")
    if job.get("environment") != "codescene":
        findings.append("publisher has no codescene environment")
    if job.get("env") != {"CARGO_TERM_COLOR": "always", "BUILD_PROFILE": "debug"}:
        findings.append("publisher has mismatched build profile")


def _valid_publisher_checkout(steps: list[reading.Step]) -> bool:
    return bool(steps) and reading.uses(steps[0]) == CHECKOUT_ACTION and (
        step_input(steps[0], "persist-credentials") is False
    )


def _publisher_setup_findings(
    steps: list[reading.Step],
) -> tuple[list[str], list[int]]:
    positions = _publisher_positions(steps)
    setup, linker, install, runner, measure = positions
    findings = []
    if len(setup) != 1 or reading.uses(steps[setup[0]]) != (
        f"{SETUP_ACTION}@{SETUP_ACTION_SHA}"
    ):
        findings.append("publisher setup-rust action has an unapproved SHA")
    if len(linker) != 1 or steps[linker[0]] != {
        "name": "Install coverage linker tools",
        "run": LINKER_INSTALL_COMMAND,
    }:
        findings.append("publisher clang/lld installation is absent or may fail softly")
    if len(install) != 1 or steps[install[0]] != {
        "name": "Install build tools",
        "run": INSTALL_COMMAND,
    }:
        findings.append("publisher build-tool install is absent or may fail softly")
    if len(runner) != 1 or steps[runner[0]].get("run") != TEST_RUNNER_COMMAND:
        findings.append("publisher test-runner installation is absent")
    if not _publisher_tools_precede_coverage(positions):
        findings.append("publisher build-tool preflight must precede coverage")
    return findings, measure


def _publisher_positions(
    steps: list[reading.Step],
) -> tuple[list[int], list[int], list[int], list[int], list[int]]:
    setup = [
        i for i, step in enumerate(steps) if reading.uses(step).startswith(SETUP_ACTION)
    ]
    linker = [
        i
        for i, step in enumerate(steps)
        if step.get("name") == "Install coverage linker tools"
    ]
    install = [
        i for i, step in enumerate(steps) if step.get("name") == "Install build tools"
    ]
    runner = [
        i for i, step in enumerate(steps) if step.get("name") == "Install test runner"
    ]
    measure = [i for i, step in enumerate(steps) if is_coverage(step)]
    return setup, linker, install, runner, measure


def _publisher_tools_precede_coverage(
    positions: tuple[list[int], list[int], list[int], list[int], list[int]],
) -> bool:
    setup, linker, install, runner, measure = positions
    all_present_once = (
        len(setup) == len(linker) == len(install) == len(runner) == len(measure) == 1
    )
    return all_present_once and setup[0] < linker[0] < install[0] < runner[0] < measure[0]


def _publisher_upload_findings(steps: list[reading.Step]) -> list[str]:
    uploads = [step for step in steps if is_upload_action(step)]
    findings = []
    if len(uploads) != 1 or reading.uses(uploads[0]) != (
        f"{UPLOAD_ACTION}@{APPROVED_UPLOAD_ACTION_SHA}"
    ):
        findings.append("publisher uploader has an unapproved action SHA")
    expected_inputs = {
        "path": "lcov.info",
        "format": "lcov",
        "mode": "upload",
        "access-token": TOKEN_INPUT,
    }
    if len(uploads) == 1 and uploads[0].get("with") != expected_inputs:
        findings.append("publisher has mismatched upload inputs")
    return findings


def _publisher_token_findings(steps: list[reading.Step], measure: list[int]) -> list[str]:
    checks = [i for i, step in enumerate(steps) if step.get("id") == "codescene-token"]
    upload_positions = [i for i, step in enumerate(steps) if is_upload_action(step)]
    findings = []
    if (
        len(checks) != 1
        or sum(step.get("run") == CHECK_COMMAND for step in steps) != 1
        or steps[checks[0]]
        != {"name": "Check CodeScene token", "id": "codescene-token", "run": CHECK_COMMAND}
    ):
        findings.append("publisher token check command is absent or altered")
    if not (
        len(checks) == len(upload_positions) == len(measure) == 1
        and measure[0] < checks[0] < upload_positions[0]
    ):
        findings.append("publisher token check must follow coverage before upload")
    return findings


def _has_prior_build_tools(steps: list[reading.Step], measured: reading.Step) -> bool:
    """Require one unconditional, failing preflight before coverage runs."""
    installs = [
        position
        for position, step in enumerate(steps)
        if step.get("name") == "Install build tools"
    ]
    return (
        len(installs) == 1
        and steps[installs[0]]
        == {"name": "Install build tools", "run": INSTALL_COMMAND}
        and installs[0] < next(i for i, step in enumerate(steps) if step is measured)
    )
