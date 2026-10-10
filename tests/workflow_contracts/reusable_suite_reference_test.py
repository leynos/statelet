"""Accept only the reviewed immutable reusable workflow exemption."""

import pytest

from suite_discovery import suite_findings
from suite_provisioning import load_workflows

PIN = "5bc2b2611f5921ef2c56e4ee3fde2b879a319361"


@pytest.mark.parametrize(
    "reference",
    [
        f"foreign/shared-actions/.github/workflows/dependabot-automerge.yml@{PIN}",
        f"leynos/foreign/.github/workflows/dependabot-automerge.yml@{PIN}",
        "leynos/shared-actions/.github/workflows/dependabot-automerge.yml@main",
        "leynos/shared-actions/.github/workflows/dependabot-automerge.yml@" + "a" * 40,
    ],
    ids=["foreign-owner", "foreign-repository", "mutable-ref", "unreviewed-commit"],
)
def test_unreviewed_reusable_workflow_cannot_escape_suite_discovery(reference: str) -> None:
    """Matching the workflow filename alone does not establish provenance."""
    workflows = load_workflows()
    workflows["dependabot-automerge.yml"]["jobs"]["automerge"]["uses"] = reference
    findings = suite_findings(workflows)
    assert any(reference in finding for finding in findings), findings
