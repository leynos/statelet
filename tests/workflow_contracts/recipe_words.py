"""Read the leading ``VAR=value`` words of a Make recipe line."""

import re

_ASSIGNMENT = re.compile(r"[A-Za-z_]\w*=.*", re.DOTALL)


def leading_assignments(words: list[str]) -> tuple[dict[str, str], list[str]]:
    """Split split-out command words into leading assignments and the rest."""
    rest = list(words)
    environment: dict[str, str] = {}
    while rest and _ASSIGNMENT.fullmatch(rest[0]):
        name, _, value = rest.pop(0).partition("=")
        environment[name] = value
    return environment, rest
