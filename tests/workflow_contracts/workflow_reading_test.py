"""Exercise generic YAML invariants through generated nested containers.

Recursive properties explore generated mapping/sequence positions and shrink
failures to minimal paths. A separate depth-four corpus preserves named edge
coverage; it does not claim to prove unbounded recursion.
"""

from itertools import product

import pytest
from hypothesis import given, settings, strategies as st

import workflow_reading as reading

NESTED_PATHS = [
    path
    for depth in range(5)
    for path in product(("mapping", "sequence"), repeat=depth)
]
NESTED_CONTAINERS = st.recursive(
    st.just(()),
    lambda children: st.tuples(
        st.sampled_from(("mapping", "sequence")), children
    ).map(lambda wrapper: (wrapper[0], *wrapper[1])),
)
BOOLEAN_ALIASES = ("yes", "Yes", "YES", "no", "No", "NO", "on", "On", "ON", "off", "Off", "OFF")


def _nested_yaml(leaf: str, containers: tuple[str, ...]) -> str:
    """Place one leaf under a deterministic sequence of YAML containers."""
    value = leaf
    for container in containers:
        indented = "\n".join(f"  {line}" for line in value.splitlines())
        value = f"nested:\n{indented}" if container == "mapping" else f"-\n{indented}"
    return "payload:\n" + "\n".join(f"  {line}" for line in value.splitlines())


@settings(derandomize=True)
@given(containers=NESTED_CONTAINERS)
def test_duplicate_keys_are_rejected_at_generated_nested_positions(
    containers: tuple[str, ...],
) -> None:
    """Duplicate keys remain invalid through recursively generated containers."""
    source = _nested_yaml("same: first\nsame: second", containers)
    with pytest.raises(reading.ContractError, match="duplicate key"):
        reading.parse("generated-nested.yml", source)


@pytest.mark.parametrize("alias", BOOLEAN_ALIASES)
@settings(derandomize=True)
@given(containers=NESTED_CONTAINERS)
def test_boolean_aliases_are_rejected_at_generated_nested_positions(
    alias: str, containers: tuple[str, ...],
) -> None:
    """Each ambiguous alias stays invalid at generated mapping/sequence paths."""
    source = _nested_yaml(f"enabled: {alias}", containers)
    with pytest.raises(reading.ContractError, match="ambiguous boolean"):
        reading.parse("generated-nested.yml", source)


@pytest.mark.parametrize("containers", NESTED_PATHS)
def test_duplicate_keys_are_rejected_at_every_nested_position(
    containers: tuple[str, ...],
) -> None:
    """No nested mapping can lose its first value through key collapsing."""
    source = _nested_yaml("same: first\nsame: second", containers)
    with pytest.raises(reading.ContractError, match="duplicate key"):
        reading.parse("nested.yml", source)


@pytest.mark.parametrize("containers", NESTED_PATHS)
@pytest.mark.parametrize("alias", ["yes", "no", "on", "off", "YES", "Off"])
def test_boolean_aliases_are_rejected_at_every_nested_position(
    containers: tuple[str, ...], alias: str,
) -> None:
    """YAML 1.1 aliases cannot silently change a nested input's type."""
    source = _nested_yaml(f"enabled: {alias}", containers)
    with pytest.raises(reading.ContractError, match="ambiguous boolean"):
        reading.parse("nested.yml", source)


@pytest.mark.parametrize("containers", NESTED_PATHS)
def test_unique_mappings_and_standard_booleans_survive_nesting(
    containers: tuple[str, ...],
) -> None:
    """Valid unique maps and explicit booleans are accepted on every path."""
    source = _nested_yaml("enabled: true\ndisabled: false\nname: 'on'", containers)
    assert "payload" in reading.parse("nested.yml", source), (
        f"valid nested YAML lost its root mapping for {containers!r}"
    )


@pytest.mark.parametrize("trigger", ["on", "'on'"])
def test_root_trigger_key_is_accepted(trigger: str) -> None:
    """The GitHub root trigger spelling is permitted despite YAML 1.1."""
    document = reading.parse("trigger.yml", f"{trigger}: [push, pull_request]\n")
    assert reading.trigger_names(document) == ["push", "pull_request"], (
        f"root trigger {trigger!r} must preserve both GitHub event names"
    )


@pytest.mark.parametrize("leaf", ["enabled: true", "enabled: yes", "same: first\nsame: second"])
def test_reused_yaml_anchor_preserves_the_same_validation(leaf: str) -> None:
    """Repeated aliases preserve both valid data and nested rejection rules."""
    indented = "\n".join(f"  {line}" for line in leaf.splitlines())
    source = f"original: &shared\n{indented}\ncopied: *shared\n"
    if leaf == "enabled: true":
        document = reading.parse("anchors.yml", source)
        assert document["original"] == document["copied"] == {"enabled": True}, (
            "a reused YAML anchor must preserve its validated boolean mapping"
        )
    else:
        reason = "ambiguous boolean" if "yes" in leaf else "duplicate key"
        with pytest.raises(reading.ContractError, match=reason):
            reading.parse("anchors.yml", source)


def test_multiple_yaml_documents_are_rejected() -> None:
    """A workflow reader cannot silently ignore a second YAML document."""
    with pytest.raises(reading.ContractError, match="not valid YAML"):
        reading.parse("multiple.yml", "name: first\n---\nname: second\n")
