"""Offline validation for Retroactive Review Sweep repository inputs."""

import json
import re


REPOSITORY = re.compile(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$")


def parse_repositories(value: str, current_repository: str) -> list[str]:
    """Resolve an empty, scalar-compatible, or JSON-list repository input."""
    raw = value.strip()
    if not raw:
        raw_values = [current_repository]
    elif raw[0] in '[{"' or raw in {"null", "true", "false"} or raw[0].isdigit():
        try:
            raw_values = json.loads(raw)
        except json.JSONDecodeError as error:
            raise ValueError("repos must be valid JSON list") from error
        if not isinstance(raw_values, list):
            raise ValueError("repos JSON value must be a list")
    else:
        raw_values = [raw]

    resolved: list[str] = []
    for repository in raw_values:
        if not isinstance(repository, str) or not REPOSITORY.fullmatch(repository):
            raise ValueError("each repository must be owner/name")
        if repository not in resolved:
            resolved.append(repository)
    if not resolved:
        raise ValueError("repos must not be an empty list")
    return resolved
