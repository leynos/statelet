"""Discover build and lint routes across the workflow graph.

This module is private support for Statelet's workflow-contract tests. It
owns route inventory only; parsing, runner classification, and preflight
validation remain in ``suite_provisioning``.
"""

import re
from dataclasses import dataclass, field
from typing import Any

import codescene_reading as reading
from suite_provisioning import (
    COVERAGE_ACTION,
    EXPECTED_COVERAGE_WORKFLOWS,
    INSTALL_COMMAND,
    MUTATION_CALL_RE,
    MUTATION_CALLER,
    ContractError,
    _has_prior_install,
    _mutation_setup_findings,
    _run_routes,
    runner_platforms,
)


@dataclass(slots=True)
class SuiteAnalysis:
    """Evidence collected while walking workflow routes to Rust suites."""

    workflows: dict[str, reading.Workflow]
    findings: list[str]
    coverage_workflows: set[str] = field(default_factory=set)
    direct_commands: set[tuple[str, str, str]] = field(default_factory=set)
    mutation_calls: list[tuple[str, str, dict[str, Any]]] = field(default_factory=list)
    suite_routes: list[tuple[str, str, dict[str, Any], int]] = field(
        default_factory=list
    )


@dataclass(frozen=True, slots=True)
class JobRoute:
    """Context shared by the route records for one workflow job."""

    workflow_name: str
    job_id: str
    job: dict[str, Any]



def suite_findings(every: dict[str, reading.Workflow]) -> list[str]:
    """Return missing or unprovisioned Rust-suite routes in parsed workflows."""
    if not every:
        return ["no workflows were available to inspect"]

    analysis = SuiteAnalysis(every, reading.missing_callees(every, set(every)))
    for workflow_name, workflow in sorted(every.items()):
        for job_id, job in reading.jobs(workflow):
            route = JobRoute(workflow_name, job_id, job)
            _discover_job_routes(analysis, route)
    _required_route_findings(analysis)
    _linux_preflight_findings(analysis)
    return sorted(set(analysis.findings))


def _discover_job_routes(analysis: SuiteAnalysis, route: JobRoute) -> None:
    """Record either one reusable workflow call or all job step routes."""
    job_uses = route.job.get("uses")
    if isinstance(job_uses, str):
        _record_workflow_call(analysis, route, job_uses)
        return
    for step_index, step in enumerate(reading.job_steps(route.job)):
        _record_suite_step(analysis, route, step_index, step)


def _record_workflow_call(
    analysis: SuiteAnalysis, route: JobRoute, reference: str
) -> None:
    """Classify and record one job-level reusable workflow reference."""
    kind, local_name = reading.classify_call(reference)
    if kind == reading.REFUSED:
        analysis.findings.append(
            f"{route.workflow_name}:{route.job_id} has an unprovable local call"
        )
    elif kind == reading.LOCAL and local_name not in analysis.workflows:
        analysis.findings.append(
            f"{route.workflow_name}:{route.job_id} calls missing workflow {local_name!r}"
        )
    elif MUTATION_CALL_RE.fullmatch(reference):
        analysis.mutation_calls.append((route.workflow_name, route.job_id, route.job))
    elif kind == reading.REMOTE:
        _record_remote_call(analysis, route.workflow_name, route.job_id, reference)


def _record_remote_call(
    analysis: SuiteAnalysis, workflow_name: str, job_id: str, reference: str
) -> None:
    """Reject remote calls unless the known mutation caller is the target."""
    workflow_reference = _workflow_reference(reference)
    if workflow_reference == MUTATION_CALLER:
        return
    if workflow_reference:
        analysis.findings.append(
            f"{workflow_name}:{job_id} has an unresolved reusable call {reference!r}"
        )
    else:
        analysis.findings.append(
            f"{workflow_name}:{job_id} has an unclassified reusable call {reference!r}"
        )


def _record_suite_step(
    analysis: SuiteAnalysis,
    route: JobRoute,
    step_index: int,
    step: dict[str, Any],
) -> None:
    """Record suite actions and executable routes from one workflow step."""
    action = reading.uses(step)
    if action.split("@", maxsplit=1)[0].lower() == COVERAGE_ACTION:
        _record_coverage_action(analysis, route, step_index, action)
    run = step.get("run")
    if isinstance(run, str):
        _record_run_routes(analysis, route, step_index, run)


def _record_coverage_action(
    analysis: SuiteAnalysis, route: JobRoute, step_index: int, action: str
) -> None:
    """Add a coverage action to the route inventory and validate its pin."""
    analysis.coverage_workflows.add(route.workflow_name)
    if not re.fullmatch(rf"{re.escape(COVERAGE_ACTION)}@[0-9a-f]{{40}}", action):
        analysis.findings.append(
            f"{route.workflow_name}:{route.job_id} has an unresolved coverage action pin"
        )
    analysis.suite_routes.append(
        (route.workflow_name, route.job_id, route.job, step_index)
    )


def _record_run_routes(
    analysis: SuiteAnalysis, route: JobRoute, step_index: int, run: str
) -> None:
    """Collect recognized Cargo and Make routes from a shell block."""
    commands, errors = _run_routes(run)
    analysis.findings.extend(
        f"{route.workflow_name}:{route.job_id} {error}" for error in errors
    )
    analysis.direct_commands.update(
        (route.workflow_name, route.job_id, command) for command, _ in commands
    )
    analysis.suite_routes.extend(
        (route.workflow_name, route.job_id, route.job, step_index)
        for _command, _ in commands
    )


def _required_route_findings(analysis: SuiteAnalysis) -> None:
    """Require coverage lanes, direct Make routes, and the mutation caller."""
    missing_coverage = EXPECTED_COVERAGE_WORKFLOWS - analysis.coverage_workflows
    analysis.findings.extend(
        f"{name} has no recognized pinned coverage suite action"
        for name in sorted(missing_coverage)
    )
    required_commands = {
        ("ci.yml", "make lint", "ci.yml has no direct Make lint route"),
        ("act-validation.yml", "make test", "act-validation.yml has no direct Make test route"),
    }
    for workflow, command, finding in required_commands:
        if not any(
            name == workflow and route == command
            for name, _job, route in analysis.direct_commands
        ):
            analysis.findings.append(finding)
    if len(analysis.mutation_calls) != 1:
        analysis.findings.append(
            "expected exactly one pinned mutation suite caller; "
            f"found {len(analysis.mutation_calls)}"
        )
    for workflow_name, job_id, job in analysis.mutation_calls:
        analysis.findings.extend(_mutation_setup_findings(workflow_name, job_id, job))
        analysis.suite_routes.append((workflow_name, job_id, job, -1))


def _linux_preflight_findings(analysis: SuiteAnalysis) -> None:
    """Check every discovered Linux Rust-suite route has an earlier preflight."""
    if not analysis.suite_routes:
        analysis.findings.append("no Rust suite routes were recognized")
    for workflow_name, job_id, job, step_index in analysis.suite_routes:
        if step_index < 0:  # The remote mutation caller supplies setup-commands.
            continue
        analysis.findings.extend(
            _route_preflight_findings(workflow_name, job_id, job, step_index)
        )


def _route_preflight_findings(
    workflow_name: str, job_id: str, job: dict[str, Any], step_index: int
) -> list[str]:
    """Return any missing local build-tool preflight for one suite route."""
    try:
        platforms = runner_platforms(job)
    except ContractError as error:
        return [f"{workflow_name}:{job_id} {error}"]
    if "linux" not in platforms or _has_prior_install(job, step_index):
        return []
    return [
        f"{workflow_name}:{job_id} reaches a Linux suite before an "
        f"unconditional {INSTALL_COMMAND}"
    ]



def _workflow_reference(reference: str) -> str:
    """Return the remote workflow path named by a reusable-workflow reference."""
    match = re.fullmatch(r"[^/]+/[^/]+/(\.github/workflows/[^@]+)@[^@]+", reference)
    return match.group(1) if match else ""
