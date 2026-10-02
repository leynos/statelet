"""Prove CV-005 closure across local and reviewed remote workflow calls."""

from __future__ import annotations

import codescene_reading as reading


# The pinned automerge caller is reviewed separately. At 6dea567, its workflow
# blob is 931290b9d32e42c6a28ae1b05a4960f3c427bad7. It checks out that
# shared workflow repository at the OIDC-proven job_workflow_sha and executes
# only workflow_scripts/dependabot_automerge.py, blob
# 9a5fba9738611d0e428c490c9a48293461a80db9. Neither blob contains a
# CodeScene coverage/token route. Its imported GraphQL client is blob
# a7fdbeec2f62cfa086ba138dab1ca50ad24f70b1 and fixes the endpoint to
# https://api.github.com/graphql. The caller forwards event metadata only.
REVIEWED_AUTOMERGE_CALL = (
    "leynos/shared-actions/.github/workflows/dependabot-automerge.yml"
    "@6dea5677a84fec60ca51b07202570e3af12ffdb4"
)
REVIEWED_AUTOMERGE_INPUTS = {
    "pull-request-number": "${{ inputs.pull-request-number || github.event.pull_request.number }}"
}


def unprovable_callees(
    every: dict[str, reading.Workflow], names: set[str]
) -> list[str]:
    """Refuse calls whose executed workflow is outside the proven closure."""
    findings = reading.missing_callees(every, names)
    for name in sorted(names):
        for job_id, job in reading.jobs(every[name]):
            if "uses" not in job:
                continue
            reference = job["uses"]
            if not isinstance(reference, str):
                findings.append(f"{name}:{job_id} has an indeterminate call")
                continue
            kind, _ = reading.classify_call(reference)
            if kind == reading.REFUSED:
                findings.append(f"{name}:{job_id} has an unprovable call {reference!r}")
            is_reviewed = (
                name == "dependabot-automerge.yml"
                and job_id == "automerge"
                and reference == REVIEWED_AUTOMERGE_CALL
                and job.get("with") == REVIEWED_AUTOMERGE_INPUTS
                and "secrets" not in job
            )
            if kind == reading.REMOTE and not is_reviewed:
                findings.append(f"{name}:{job_id} has an unprovable call {reference!r}")
    return findings
