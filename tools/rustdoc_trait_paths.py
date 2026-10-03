"""Render trait implementation paths through reachable trait reexports."""

from __future__ import annotations

from api_inventory_json import JSONValue, array_value, object_value, string_value
from api_inventory_json import declaration_digest


def trait_export_catalog(crates: list[dict[str, JSONValue]]) -> dict[str, set[str]]:
    """Index canonical identities by public names; `trait_export_catalog(crates)`."""
    catalog: dict[str, set[str]] = {}
    for crate in crates:
        for value in array_value(crate.get("declarations"), "declarations"):
            entry = object_value(value, "declaration")
            canonical = entry.get("canonical_trait_path")
            if entry.get("kind") == "trait" and isinstance(canonical, str):
                path = string_value(entry.get("path"), "trait path")
                catalog.setdefault(canonical, set()).add(path)
    return catalog


def implementation_trait(entry: dict[str, JSONValue]) -> str | None:
    """Read the compiler trait identity; `implementation_trait(declaration)`."""
    context = entry.get("impl_context")
    trait = context.get("trait") if isinstance(context, dict) else None
    if not isinstance(trait, dict):
        return None
    canonical = trait.get("resolved_item_path", trait.get("path"))
    return canonical if isinstance(canonical, str) else None


def renamed_trait_member(
    entry: dict[str, JSONValue], trait: str
) -> dict[str, JSONValue]:
    """Keep implementation provenance when renaming; `renamed_trait_member(row, alias)`."""
    path = string_value(entry.get("path"), "implementation path")
    owner, separator, tail = path.rpartition("::<")
    _, member_separator, member = tail.rpartition(">::")
    if not separator or not member_separator:
        raise ValueError(
            f"implementation path {path!r}; expected Owner::<Trait>::member"
        )
    renamed = dict(entry)
    renamed["path"] = f"{owner}::<{trait}>::{member}"
    renamed["declaration_sha256"] = declaration_digest(renamed)
    return renamed


def resolved_trait_members(
    entry: dict[str, JSONValue],
    catalog: dict[str, set[str]],
    captured: set[str],
    unresolved: list[JSONValue],
) -> list[dict[str, JSONValue]]:
    """Expand public trait aliases or record the gap; `resolved_trait_members(row, catalog, names, gaps)`."""
    canonical = implementation_trait(entry)
    if canonical is None or canonical.split("::", 1)[0] not in captured:
        return [entry]
    aliases = catalog.get(canonical, set())
    if not aliases:
        unresolved.append(f"unreachable-trait:{canonical}:{entry.get('path')}")
        return []
    return [renamed_trait_member(entry, alias) for alias in sorted(aliases)]


def resolve_public_trait_paths(crates: list[dict[str, JSONValue]]) -> None:
    """Withhold inaccessible trait implementations; `resolve_public_trait_paths(crates)`."""
    catalog = trait_export_catalog(crates)
    captured = {string_value(crate.get("crate"), "crate") for crate in crates}
    for crate in crates:
        unresolved = array_value(crate.get("unresolved_links"), "unresolved links")
        entries = array_value(crate.get("declarations"), "declarations")
        resolved_entries = [
            resolved
            for value in entries
            for resolved in resolved_trait_members(
                object_value(value, "declaration"), catalog, captured, unresolved
            )
        ]
        crate["declarations"] = list(
            {str(row["declaration_sha256"]): row for row in resolved_entries}.values()
        )
        crate["unresolved_links"] = sorted(set(str(item) for item in unresolved))
