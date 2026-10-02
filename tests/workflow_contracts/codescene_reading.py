"""Read workflows for the CV-005 CodeScene coverage contract.

Every reader here errs towards seeing more. A workflow the contract cannot
see is a workflow it passes, so each place GitHub accepts more than one
spelling is read in all of them: both file extensions in either case, the
``on`` key as a string or as the boolean YAML 1.1 makes of it, a trigger
written as a scalar, a sequence or a mapping, and a reusable-workflow call
however its local path is prefixed.
"""

import typing as typ
from pathlib import Path

import yaml

WORKFLOW_DIR = Path(__file__).resolve().parents[2] / ".github" / "workflows"
WORKFLOW_PREFIX = ".github/workflows/"
WORKFLOW_EXTENSIONS = (".yml", ".yaml")

# Events that run a workflow on behalf of a pull request. The review events
# and merge_group run pull-request code as surely as pull_request does, and
# workflow_run chains a workflow onto another run, which may be a pull
# request's. Every one seeds the closure.
PULL_REQUEST_EVENTS = frozenset(
    {
        "pull_request",
        "pull_request_target",
        "pull_request_review",
        "pull_request_review_comment",
        "merge_group",
        "workflow_run",
    }
)

type Workflow = dict[typ.Any, typ.Any]
type Step = dict[typ.Any, typ.Any]


class ContractError(ValueError):
    """A workflow the contract refuses to read."""


def parse(name: str, text: str) -> Workflow:
    """Parse one workflow, refusing duplicate keys and a doubled trigger block.

    A root declaring both ``'on'`` and the boolean ``on`` is refused because
    GitHub merges the two blocks, and a reader that picks one is blind to the
    other.
    """
    loader = yaml.SafeLoader(text)
    try:
        parsed = _parse_document(name, loader)
    except yaml.YAMLError as error:
        raise ContractError(f"{name} is not valid YAML: {error}") from error
    finally:
        loader.dispose()
    if not isinstance(parsed, dict):
        raise ContractError(f"{name} is not a mapping")
    if "on" in parsed and True in parsed:
        raise ContractError(f"{name} declares its triggers under both 'on' and true")
    return parsed


def _parse_document(name: str, loader: yaml.SafeLoader) -> object:
    """Check YAML syntax before SafeLoader collapses duplicate mapping keys."""
    syntax = loader.get_single_node()
    if syntax is None:
        return None
    _reject_duplicate_keys(name, syntax, loader)
    _reject_ambiguous_booleans(name, syntax, is_root=True)
    return loader.construct_document(syntax)


def _reject_duplicate_keys(
    name: str, node: yaml.Node, loader: yaml.SafeLoader
) -> None:
    """Reject repeated mapping keys before SafeLoader keeps the last value."""
    match node:
        case yaml.MappingNode(value=entries):
            seen: set[typ.Any] = set()
            for key_node, value_node in entries:
                key = loader.construct_object(key_node, deep=True)
                if key in seen:
                    raise ContractError(
                        f"duplicate key {key!r} {key_node.start_mark} in {name}"
                    )
                seen.add(key)
                _reject_duplicate_keys(name, value_node, loader)
        case yaml.SequenceNode(value=children):
            for child in children:
                _reject_duplicate_keys(name, child, loader)


def _reject_ambiguous_booleans(
    name: str, node: yaml.Node, *, is_root: bool = False
) -> None:
    """Reject YAML 1.1 yes/no/on/off aliases except the root trigger key."""
    match node:
        case yaml.MappingNode(value=entries):
            for key, value in entries:
                is_root_on = (
                    is_root and isinstance(key, yaml.ScalarNode) and key.value == "on"
                )
                _reject_ambiguous_booleans(name, key, is_root=is_root_on)
                _reject_ambiguous_booleans(name, value)
        case yaml.SequenceNode(value=children):
            for child in children:
                _reject_ambiguous_booleans(name, child)
        case yaml.ScalarNode(tag="tag:yaml.org,2002:bool", value=value) if (
            value.lower() not in {"true", "false"} and not is_root
        ):
            raise ContractError(f"{name} uses ambiguous boolean {value!r}")


def is_workflow(name: str) -> bool:
    """Return whether ``name`` is a workflow file, in either extension or case."""
    return name.lower().endswith(WORKFLOW_EXTENSIONS)


def workflows(directory: Path) -> dict[str, Workflow]:
    """Read every workflow in ``directory``, failing loudly on an empty one.

    The directory is the caller's choice; the contract tests pass
    ``WORKFLOW_DIR``. Every failure is a ``ContractError``, with any
    ``OSError`` chained, so a directory the contract cannot read never passes
    as one with nothing to judge.

    Raises:
        ContractError: when the directory cannot be listed, a workflow cannot
            be read or parsed, or the directory holds no workflow.
    """
    paths = _workflow_paths(directory)
    found = {
        path.name: parse(path.name, _read(path))
        for path in paths
        if is_workflow(path.name)
    }
    if not found:
        raise ContractError(f"no workflows found under {directory}")
    for name, workflow in found.items():
        _validate_workflow(name, workflow)
    return found


def _workflow_paths(directory: Path) -> list[Path]:
    """List paths once, preserving unreadable-directory failures."""
    try:
        return sorted(directory.iterdir())
    except OSError as error:
        raise ContractError(f"cannot list {directory}: {error}") from error


def _validate_workflow(name: str, workflow: Workflow) -> None:
    """Reject ambiguous triggers and incomplete job maps before auditing them."""
    _validate_triggers(name, workflow)
    jobs_block = _jobs_mapping(name, workflow)
    for job_id, job in jobs_block.items():
        _validate_job(name, job_id, job)


def _validate_triggers(name: str, workflow: Workflow) -> None:
    """Reject trigger names that YAML or GitHub would read ambiguously."""
    block = _trigger_block(workflow)
    if isinstance(block, (list, dict)) and any(
        not isinstance(event, str) for event in block
    ):
        raise ContractError(f"{name} has an ambiguous event spelling")
    if not trigger_names(workflow):
        raise ContractError(f"{name} has no determinate trigger")


def _jobs_mapping(name: str, workflow: Workflow) -> dict[typ.Any, typ.Any]:
    """Return the job map only when its complete shape is inspectable."""
    jobs_block = workflow.get("jobs")
    if not isinstance(jobs_block, dict) or not jobs_block:
        raise ContractError(f"{name} has no determinate jobs")
    return jobs_block


def _validate_job(name: str, job_id: object, job: object) -> None:
    """Require an inspectable step list or a reusable-workflow call."""
    if not isinstance(job, dict):
        raise ContractError(f"{name}:{job_id} has an indeterminate job")
    match job.get("steps"):
        case list() as step_list:
            if not all(isinstance(step, dict) for step in step_list):
                raise ContractError(f"{name}:{job_id} has an indeterminate step")
        case _ if isinstance(job.get("uses"), str):
            return
        case _:
            raise ContractError(f"{name}:{job_id} has an indeterminate job")


def _read(path: Path) -> str:
    """Return one workflow's text, raising ``ContractError`` when unreadable."""
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as error:
        raise ContractError(f"cannot read {path}: {error}") from error


def _trigger_block(workflow: Workflow) -> object:
    """Return the trigger block, under the string or the boolean key."""
    return workflow.get("on", workflow.get(True))


def trigger_names(workflow: Workflow) -> list[str]:
    """Return the event names a workflow declares, in any of the three forms.

    A mapping-only reader stringifies ``on: [push, pull_request]`` into one
    key named after the whole list, and the workflow escapes every clause.
    """
    block = _trigger_block(workflow)
    match block:
        case str():
            return [block]
        case list():
            return [name for name in block if isinstance(name, str)]
        case dict():
            return [name for name in block if isinstance(name, str)]
        case _:
            return []


def trigger(workflow: Workflow, event: str) -> object:
    """Return one trigger's configuration, when it is written as a mapping."""
    block = _trigger_block(workflow)
    return block.get(event) if isinstance(block, dict) else None


def starts_on_pull_request(workflow: Workflow) -> bool:
    """Return whether a workflow starts on behalf of a pull request."""
    return any(name in PULL_REQUEST_EVENTS for name in trigger_names(workflow))


def starts_on_push(workflow: Workflow) -> bool:
    """Return whether a workflow starts on a push, to any branch or tag."""
    return "push" in trigger_names(workflow)


def jobs(workflow: Workflow) -> list[tuple[str, dict[typ.Any, typ.Any]]]:
    """Return every job as ``(job id, job mapping)``."""
    block = workflow.get("jobs")
    if not isinstance(block, dict):
        return []
    return [(str(key), job) for key, job in block.items() if isinstance(job, dict)]


def job_steps(job: dict[typ.Any, typ.Any]) -> list[Step]:
    """Return the steps of one job, in order."""
    listed = job.get("steps")
    if not isinstance(listed, list):
        return []
    return [step for step in listed if isinstance(step, dict)]


def steps(workflow: Workflow) -> list[Step]:
    """Return every step of every job."""
    return [step for _, job in jobs(workflow) for step in job_steps(job)]


def uses(step: Step) -> str:
    """Return a step's ``uses`` reference, or the empty string."""
    reference = step.get("uses")
    return reference if isinstance(reference, str) else ""


LOCAL = "local"
REFUSED = "refused"
REMOTE = "remote"


def classify_call(reference: str) -> tuple[str, str]:
    """Classify a job-level ``uses`` reference as local, refused or remote.

    Matched by shape: a leading ``./`` or GitHub's documented ``$/`` is
    stripped, and what remains is local when it is a path under
    ``.github/workflows/``. A local-shaped reference carrying an ``@ref`` or
    naming a subdirectory is refused rather than read as remote: it resolves
    to no file here, so whatever it runs would escape the closure in silence.
    """
    path = reference
    for prefix in ("./", "$/"):
        if path.startswith(prefix):
            path = path[len(prefix) :]
            break
    if not path.startswith(WORKFLOW_PREFIX):
        return (REMOTE, "")
    file = path[len(WORKFLOW_PREFIX) :]
    return (LOCAL, file) if _names_one_file(file) else (REFUSED, "")


def _names_one_file(file: str) -> bool:
    """Return whether ``file`` names one workflow, with no directory or ``@ref``."""
    return bool(file) and not any(mark in file for mark in "/@")


def job_calls(workflow: Workflow) -> list[tuple[str, str]]:
    """Return each job's ``uses`` reference with the job's id."""
    return [
        (job_id, job["uses"])
        for job_id, job in jobs(workflow)
        if isinstance(job.get("uses"), str)
    ]


def closure(
    every: dict[str, Workflow], seeds: typ.Callable[[Workflow], bool]
) -> set[str]:
    """Return the workflows reachable from those ``seeds`` selects.

    A workflow declaring only ``workflow_call`` names no event of its own,
    yet it runs with whatever the caller hands it, including the caller's
    secrets under ``secrets: inherit``. So every clause about what an event
    can reach runs over this closure, never over the trigger list alone.
    """
    reached = {name for name, workflow in every.items() if seeds(workflow)}
    pending = list(reached)
    while pending:
        fresh = (_local_callees(every[pending.pop()]) & every.keys()) - reached
        reached |= fresh
        pending.extend(fresh)
    return reached


def _local_callees(workflow: Workflow) -> set[str]:
    """Return the local workflows ``workflow`` calls, by file name."""
    calls = (classify_call(reference) for _, reference in job_calls(workflow))
    return {callee for kind, callee in calls if kind == LOCAL}


def pull_request_closure(every: dict[str, Workflow]) -> set[str]:
    """Return the workflows a pull request can reach."""
    return closure(every, starts_on_pull_request)


def push_closure(every: dict[str, Workflow]) -> set[str]:
    """Return the workflows a push can reach."""
    return closure(every, starts_on_push)


def missing_callees(every: dict[str, Workflow], names: set[str]) -> list[str]:
    """Return each local call, from the named workflows, to a missing file."""
    return [
        f"{name} calls {reference!r}, which is not there"
        for name in sorted(names)
        for _, reference in job_calls(every[name])
        if classify_call(reference)[0] == LOCAL
        and classify_call(reference)[1] not in every
    ]


def rendered(value: object) -> str:
    """Return every key and scalar in a parsed value, one per line.

    Reading the parse rather than the file is deliberate: comments are gone,
    so prose explaining why the token is absent does not read as the token
    being present, while a block scalar's content, which Actions expands, is
    read in full. Keys carry their ``:`` so the computed-secret clause can
    tell a ``secrets:`` key from an expression.
    """
    match value:
        case None:
            return ""
        case dict() as mapping:
            return "".join(
                f"{rendered(key).rstrip()}:\n{rendered(item)}"
                for key, item in mapping.items()
            )
        case list() as items:
            return "".join(rendered(item) for item in items)
        case bool() as boolean:
            return f"{str(boolean).lower()}\n"
        case _:
            return f"{value}\n"


def computes_a_secret(text: str) -> bool:
    """Return whether ``text`` reaches the ``secrets`` context other than by name.

    ``secrets['CS_' + ...]`` and ``toJSON(secrets)`` hand a step the token
    without spelling it. Every ``secrets`` word is judged by what follows: a
    ``.`` is a named reference, a ``:`` is a YAML key, and a following word
    is prose. Anything else is a computed or whole-context access. Actions
    resolves context names without regard to case, so the scan is folded.
    """
    text = text.lower()
    start = text.find("secrets")
    while start != -1:
        before = text[start - 1] if start else ""
        following = text[start + len("secrets") :].lstrip()
        after = following[:1]
        is_word_start = not before or not (before.isalnum() or before in "_.")
        # A following word makes it prose ("keeps secrets out of logs").
        is_prose = after.isalnum() or after == "_"
        is_expression = after not in (".", ":", "") and not is_prose
        if is_word_start and is_expression:
            return True
        start = text.find("secrets", start + 1)
    return False
