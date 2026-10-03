"""Resolve public re-export paths between captured workspace crates."""

from __future__ import annotations

from collections.abc import Mapping, Sequence

from api_inventory_json import (
    JSONValue,
    array_value,
    declaration_digest,
    object_value,
    string_value,
)


def resolve_cross_crate_reexports(crates: list[dict[str, JSONValue]]) -> None:
    catalog = path_catalog(crates)
    for crate in crates:
        declarations = array_value(crate.get("declarations"), "declarations")
        crate["declarations"] = expand_reexports(crate, declarations, catalog)
        crate["unresolved_links"] = unresolved_ids(crate["declarations"])
        crate["source_files"] = source_hashes(crate["declarations"])


def path_catalog(
    crates: Sequence[Mapping[str, JSONValue]],
) -> dict[str, list[dict[str, JSONValue]]]:
    catalog: dict[str, list[dict[str, JSONValue]]] = {}
    for crate in crates:
        name = string_value(crate.get("crate"), "crate name")
        for value in array_value(crate.get("declarations"), "declarations"):
            entry = object_value(value, "declaration")
            path = string_value(entry.get("path"), "declaration path")
            catalog.setdefault(path, []).append({"crate": name, "entry": entry})
    return catalog


def expand_reexports(
    crate: Mapping[str, JSONValue],
    declarations: Sequence[JSONValue],
    catalog: Mapping[str, list[dict[str, JSONValue]]],
) -> list[dict[str, JSONValue]]:
    crate_name = string_value(crate.get("crate"), "crate name")
    output: list[dict[str, JSONValue]] = []
    for value in declarations:
        entry = object_value(value, "declaration")
        output.extend(expand_one_reexport(crate_name, entry, catalog))
    return output


def expand_one_reexport(
    crate_name: str,
    entry: Mapping[str, JSONValue],
    catalog: Mapping[str, list[dict[str, JSONValue]]],
) -> list[dict[str, JSONValue]]:
    kind = string_value(entry.get("kind"), "declaration kind")
    if kind not in {"unresolved_reexport", "reexport_module"}:
        return [dict(entry)]
    return resolve_reexport_entry(crate_name, entry, catalog)


def resolve_reexport_entry(
    crate_name: str,
    entry: Mapping[str, JSONValue],
    catalog: Mapping[str, list[dict[str, JSONValue]]],
) -> list[dict[str, JSONValue]]:
    target = string_value(entry.get("target_path"), "re-export target path")
    metadata = entry.get("reexport")
    alternatives = reexport_target_paths(target, metadata)
    owner = alternatives[0].split("::", 1)[0]
    if owner == crate_name:
        return [dict(entry)]
    for candidate_path in alternatives:
        candidates = external_candidates(owner, candidate_path, catalog)
        if not candidates:
            continue
        if isinstance(metadata, dict) and metadata.get("is_glob") is True:
            candidates = [
                row for row in candidates if row.get("path") != candidate_path
            ]
        return [
            remap_reexport(candidate, candidate_path, entry) for candidate in candidates
        ]
    return [dict(entry)]


def reexport_target_paths(target: str, metadata: JSONValue | None) -> list[str]:
    paths = [target]
    if isinstance(metadata, dict):
        canonical = metadata.get("canonical_target_path")
        if isinstance(canonical, str) and canonical not in paths:
            paths.append(canonical)
    return paths


def external_candidates(
    owner: str, target: str, catalog: Mapping[str, list[dict[str, JSONValue]]]
) -> list[dict[str, JSONValue]]:
    prefix = f"{target}::"
    candidates = []
    for path, records in catalog.items():
        if path != target and not path.startswith(prefix):
            continue
        candidates.extend(
            record["entry"]
            for record in records
            if record.get("crate") == owner and not unresolved_kind(record["entry"])
        )
    return candidates


def unresolved_kind(entry: Mapping[str, JSONValue]) -> bool:
    if entry.get("kind") in {"unresolved_reexport", "reexport_module"}:
        return True
    return False


def remap_reexport(
    source: Mapping[str, JSONValue], target: str, alias: Mapping[str, JSONValue]
) -> dict[str, JSONValue]:
    source_path = string_value(source.get("path"), "source declaration path")
    alias_path = string_value(alias.get("path"), "alias declaration path")
    mapped = dict(source)
    mapped["path"] = alias_path + source_path[len(target) :]
    mapped["reexport_origin_path"] = source_path
    if source_path == target and isinstance(alias.get("reexport"), dict):
        mapped["reexport"] = alias["reexport"]
    mapped["declaration_sha256"] = declaration_digest(mapped)
    return mapped


def unresolved_ids(declarations: Sequence[JSONValue]) -> list[str]:
    found: set[str] = set()
    for entry in declarations:
        collect_unresolved(entry, found)
    return sorted(found)


def collect_unresolved(value: JSONValue, found: set[str]) -> None:
    if isinstance(value, list):
        for item in value:
            collect_unresolved(item, found)
        return
    if not isinstance(value, dict):
        return
    collect_unresolved_fields(value, found)
    for child in value.values():
        collect_unresolved(child, found)


def collect_unresolved_fields(value: Mapping[str, JSONValue], found: set[str]) -> None:
    for key in ("unresolved_item_id", "unresolved_target"):
        target = value.get(key)
        if isinstance(target, (str, int)):
            found.add(str(target))


def source_hashes(
    entries: Sequence[Mapping[str, JSONValue]],
) -> list[dict[str, JSONValue]]:
    records: dict[str, str] = {}
    for entry in entries:
        add_source_hash(records, entry)
        reexport = entry.get("reexport")
        if isinstance(reexport, dict):
            add_source_hash(records, reexport)
    return [
        {"path": filename, "sha256": records[filename]} for filename in sorted(records)
    ]


def add_source_hash(records: dict[str, str], entry: Mapping[str, JSONValue]) -> None:
    filename, digest = entry.get("source_file"), entry.get("source_sha256")
    if isinstance(filename, str) and isinstance(digest, str):
        records[filename] = digest
