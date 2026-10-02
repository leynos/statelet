"""Find every push-reachable writer of the CodeScene ratchet baseline."""

import codescene_reading as reading
from codescene_rules import is_ratcheted_coverage


def baseline_writers(every: dict[str, reading.Workflow]) -> list[str]:
    """Return the workflow of every ratcheted coverage step a push can reach.

    ``generate-coverage`` saves the baseline on a push to ``main``, so each
    entry is a baseline writer, one per step. Reached through the push
    closure, a called workflow that ratchets is a writer as surely as its
    caller.
    """
    return [
        name
        for name in sorted(reading.push_closure(every))
        for step in reading.steps(every[name])
        if is_ratcheted_coverage(step)
    ]
