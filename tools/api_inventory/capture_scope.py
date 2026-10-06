"""Select explicit toolkit capture scopes while retaining the full pinned source manifest."""

from __future__ import annotations

from .inputs import JSONObject, object_rows, text_field


def owner_toolkits(header: JSONObject) -> set[str]:
    """Read every toolkit owning a header; e.g. `owner_toolkits(header)`."""
    return {
        text_field(owner, "toolkit")
        for owner in object_rows(header["owners"], "header owners")
    }


def scoped_manifest(manifest: JSONObject, toolkits: list[str]) -> JSONObject:
    """Restrict probes explicitly without mutating their full source authority."""
    if not toolkits:
        return manifest
    headers = object_rows(manifest["headers"], "headers")
    requested = set(toolkits)
    known = set().union(*(owner_toolkits(row) for row in headers))
    unknown = requested - known
    if unknown:
        raise ValueError(
            f"requested toolkits {sorted(unknown)!r}: expected manifest owners from {sorted(known)!r}"
        )
    selected = [row for row in headers if owner_toolkits(row) & requested]
    return {**manifest, "headers": selected}


def scoped_commands(
    commands: dict[str, tuple[str, list[str]]], toolkits: list[str]
) -> dict[str, tuple[str, list[str]]]:
    """Select requested owners before choosing commands for multiply-owned headers."""
    return {
        name: command
        for name, command in commands.items()
        if not toolkits or name in toolkits
    }
