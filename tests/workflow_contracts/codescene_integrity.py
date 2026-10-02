"""Bind CV-005 coverage lanes to the reviewed action and local preflight.

This module owns the exact coverage configuration and publisher shape. The
broader CodeScene flow and token rules remain in ``codescene_rules``.
"""

from __future__ import annotations

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
    lane = every.get("ci.yml", {})
    findings: list[str] = []
    if set(reading.trigger_names(publisher)) != {"push", "workflow_dispatch"} or not (
        publishes_from_main(publisher)
    ):
        findings.append("publisher triggers must be main push and dispatch only")
    for name, workflow in (("ci.yml", lane), ("coverage-main.yml", publisher)):
        measured = [step for step in reading.steps(workflow) if is_coverage(step)]
        if len(measured) != 1:
            findings.append(f"{name} must have exactly one coverage step")
            continue
        step = measured[0]
        if reading.uses(step) != f"{COVERAGE_ACTION}@{APPROVED_COVERAGE_ACTION_SHA}":
            findings.append(f"{name} uses an unapproved coverage action SHA")
        if step.get("with") != EXPECTED_COVERAGE_INPUTS:
            findings.append(f"{name} has mismatched coverage inputs")
        if step.get("env") != EXPECTED_COVERAGE_ENV:
            findings.append(f"{name} has mismatched coverage toolchain/linker settings")
        if "if" in step or "continue-on-error" in step:
            findings.append(f"{name} coverage step may skip or fail softly")
        if name == "ci.yml":
            carrying = [
                reading.job_steps(job)
                for _, job in reading.jobs(workflow)
                if any(candidate is step for candidate in reading.job_steps(job))
            ]
            if len(carrying) != 1 or not _has_prior_build_tools(carrying[0], step):
                findings.append("ci.yml build-tool preflight must precede coverage")
    for name in reading.pull_request_closure(every) - {"ci.yml"}:
        for step in reading.steps(every[name]):
            if not is_coverage(step):
                continue
            if reading.uses(step) != f"{COVERAGE_ACTION}@{APPROVED_COVERAGE_ACTION_SHA}":
                findings.append(f"{name} uses an unapproved coverage action SHA")
            if step.get("with") != EXPECTED_COVERAGE_INPUTS:
                findings.append(f"{name} has mismatched coverage inputs")
            if step.get("env") != EXPECTED_COVERAGE_ENV:
                findings.append(
                    f"{name} has mismatched coverage toolchain/linker settings"
                )

    jobs = reading.jobs(publisher)
    if len(jobs) != 1 or jobs[0][0] != "coverage-upload":
        findings.append("publisher must have exactly one coverage-upload job")
        return findings
    job = jobs[0][1]
    if "if" in job or "continue-on-error" in job:
        findings.append("publisher job may skip or fail softly")
    if publisher.get("permissions") != {"contents": "read"} or job.get(
        "permissions"
    ) != {"contents": "read"}:
        findings.append("publisher permissions exceed contents: read")
    if job.get("environment") != "codescene":
        findings.append("publisher has no codescene environment")
    if job.get("env") != {"CARGO_TERM_COLOR": "always", "BUILD_PROFILE": "debug"}:
        findings.append("publisher has mismatched build profile")
    steps = reading.job_steps(job)
    if (
        not steps
        or reading.uses(steps[0]) != CHECKOUT_ACTION
        or step_input(steps[0], "persist-credentials") is not False
    ):
        findings.append("publisher checkout must disable persisted credentials")
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
    if not (
        len(setup) == len(linker) == len(install) == len(runner) == len(measure) == 1
        and setup[0] < linker[0] < install[0] < runner[0] < measure[0]
    ):
        findings.append("publisher build-tool preflight must precede coverage")
    uploads = [step for step in steps if is_upload_action(step)]
    if len(uploads) != 1 or reading.uses(uploads[0]) != (
        f"{UPLOAD_ACTION}@{APPROVED_UPLOAD_ACTION_SHA}"
    ):
        findings.append("publisher uploader has an unapproved action SHA")
    if len(uploads) == 1 and uploads[0].get("with") != {
        "path": "lcov.info",
        "format": "lcov",
        "mode": "upload",
        "access-token": TOKEN_INPUT,
    }:
        findings.append("publisher has mismatched upload inputs")
    checks = [i for i, step in enumerate(steps) if step.get("id") == "codescene-token"]
    upload_positions = [i for i, step in enumerate(steps) if is_upload_action(step)]
    if (
        len(checks) != 1
        or sum(step.get("run") == CHECK_COMMAND for step in steps) != 1
        or steps[checks[0]]
        != {
            "name": "Check CodeScene token",
            "id": "codescene-token",
            "run": CHECK_COMMAND,
        }
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
