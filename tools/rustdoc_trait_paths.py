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
    if path.startswith("<"):
        return renamed_ufcs_trait_member(entry, path, trait)
    owner, separator, tail = path.rpartition("::<")
    _, member_separator, member = tail.rpartition(">::")
    if not separator or not member_separator:
        raise ValueError(
            f"implementation path {path!r}; expected Owner::<Trait>::member"
        )
    application = tail.rpartition(">::")[0]
    current_trait, separator, arguments = application.partition("<")
    suffix = f"<{arguments}" if separator else ""
    if not current_trait:
        raise ValueError(f"implementation path {path!r}; expected a trait application")
    renamed = dict(entry)
    renamed["path"] = f"{owner}::<{trait}{suffix}>::{member}"
    renamed["declaration_sha256"] = declaration_digest(renamed)
    return renamed


def renamed_ufcs_trait_member(
    entry: dict[str, JSONValue], path: str, trait: str
) -> dict[str, JSONValue]:
    prefix, separator, member = path[1:].rpartition(">::")
    owner, trait_separator, trait_application = prefix.partition(" as ")
    canonical = implementation_trait(entry)
    provenance = (
        "reference_impl" if "reference_impl_trait_path" in entry else "foreign_impl"
    )
    current = entry.get(f"{provenance}_trait_path")
    recorded = entry.get(f"{provenance}_trait_canonical")
    if not separator or not trait_separator or canonical is None:
        raise ValueError(
            f"implementation path {path!r}; expected <Self as Trait>::member"
        )
    current_trait, separator, suffix = trait_application.partition("<")
    suffix = f"<{suffix}" if separator else ""
    if current_trait != current or canonical != recorded:
        raise ValueError(
            f"implementation path {path!r}; expected recorded foreign trait identity"
        )
    renamed = dict(entry)
    renamed["path"] = f"<{owner} as {trait}{suffix}>::{member}"
    renamed[f"{provenance}_trait_path"] = trait
    renamed["declaration_sha256"] = declaration_digest(renamed)
    return renamed


def resolved_trait_members(
    entry: dict[str, JSONValue],
    catalog: dict[str, set[str]],
    captured: set[str],
    unresolved: list[JSONValue],
) -> list[dict[str, JSONValue]]:
    """Expand public trait aliases or record a reachability gap."""
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
