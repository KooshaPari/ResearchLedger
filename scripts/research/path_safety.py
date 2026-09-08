from pathlib import Path


def resolve_under(value: Path, root: Path) -> Path:
    base = root.expanduser().resolve()
    expanded = value.expanduser()
    candidate = (expanded if expanded.is_absolute() else base / expanded).resolve()
    try:
        candidate.relative_to(base)
    except ValueError as exc:
        raise ValueError(f"PATH_OUTSIDE_ALLOWED_ROOT: {candidate}") from exc
    return candidate
