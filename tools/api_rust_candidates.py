"""Resolve the Rust side of a ledger row against the normalized rustdoc inventory."""

from __future__ import annotations

from collections import defaultdict
import re

from api_inventory.inputs import JSONValue
from api_rust_declaration_selectors import selected_rust_candidates
from api_rust_kind import select_rust_kind
from api_rust_reference_targets import nominal_reference_owner


def row_list(document: dict[str, JSONValue], field: str) -> list[dict[str, JSONValue]]:
    rows = document.get(field)
    if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
        raise ValueError(f"JSON field {field!r} must be an array of objects")
    return rows


def candidate_status(candidates: list[dict[str, JSONValue]]) -> str:
    if not candidates:
        return "missing"
    return "unique" if len(candidates) == 1 else "ambiguous"


def rust_crates(
    document: dict[str, JSONValue],
) -> dict[str, dict[str, list[dict[str, JSONValue]]]]:
    result = {}
    for crate in row_list(document, "crates"):
        crate_name = crate.get("crate")
        if not isinstance(crate_name, str):
            continue
        paths: dict[str, list[dict[str, JSONValue]]] = defaultdict(list)
        for item in crate.get("declarations", []):
            if isinstance(item, dict) and isinstance(item.get("path"), str):
                paths[item["path"]].append(item)
        result[crate_name.replace("-", "_")] = dict(paths)
    return result


def rust_candidates(
    rust_item: JSONValue, crates: dict[str, dict[str, list[dict[str, JSONValue]]]]
) -> tuple[str, list[dict[str, JSONValue]]]:
    if not isinstance(rust_item, str) or not rust_item:
        return "no_rust_target_declared", []
    if bool_primitive_ufcs_target(rust_item):
        return bool_primitive_candidates(rust_item, crates)
    prefix, separator = rust_target_prefix(rust_item)
    if not separator or prefix not in crates:
        return "unknown_rust_crate", []
    candidates = crates[prefix].get(rust_item, [])
    status = candidate_status(candidates)
    if status == "unique" and not rust_source_is_pinned(candidates[0]):
        return "unverifiable_source", candidates
    return status, candidates


def bool_primitive_ufcs_target(target: str) -> bool:
    return target.startswith("<bool as ") and ">::" in target


def bool_primitive_candidates(
    target: str, crates: dict[str, dict[str, list[dict[str, JSONValue]]]]
) -> tuple[str, list[dict[str, JSONValue]]]:
    matches = [
        item
        for crate_name, paths in crates.items()
        for item in paths.get(target, [])
        if item.get("foreign_impl_self_primitive") == "bool"
        and normalized_origin(item) == crate_name
    ]
    status = candidate_status(matches)
    if status == "unique" and not bool_source_is_pinned(matches[0]):
        return "unverifiable_source", matches
    return status, matches


def bool_source_is_pinned(item: dict[str, JSONValue]) -> bool:
    anchor_file = item.get("foreign_impl_anchor_source_file")
    anchor_hash = item.get("foreign_impl_anchor_source_sha256")
    return (
        rust_source_is_pinned(item)
        and isinstance(anchor_file, str)
        and bool(anchor_file)
        and isinstance(anchor_hash, str)
        and re.fullmatch(r"[0-9a-f]{64}", anchor_hash) is not None
    )


def normalized_origin(item: dict[str, JSONValue]) -> str | None:
    origin = item.get("foreign_impl_origin_crate")
    return origin.replace("-", "_") if isinstance(origin, str) else None


def rust_target_prefix(target: str) -> tuple[str, bool]:
    if target.startswith("<"):
        owner, separator, _rest = target[1:].partition(" as ")
        prefix, path_separator, _tail = nominal_reference_owner(owner).partition("::")
        return prefix, bool(separator and path_separator)
    prefix, separator, _path = target.partition("::")
    return prefix, bool(separator)


def row_rust_candidates(
    row: dict[str, JSONValue],
    crates: dict[str, dict[str, list[dict[str, JSONValue]]]],
) -> tuple[str, list[dict[str, JSONValue]]]:
    """Require all accessor targets; e.g. `row_rust_candidates(row, crates)`."""
    if "rust_kind" in row:
        selected = select_rust_kind(crates, row["rust_kind"])
        if selected is None:
            return "invalid_rust_kind", []
        crates = selected
    return rust_target_candidates(row, crates)


def rust_target_candidates(
    row: dict[str, JSONValue],
    crates: dict[str, dict[str, list[dict[str, JSONValue]]]],
) -> tuple[str, list[dict[str, JSONValue]]]:
    """Resolve path-only or explicit declarations; `rust_target_candidates(row, crates)`."""
    if "rust_items" not in row:
        if "rust_declaration_selectors" in row:
            target = row.get("rust_item")
            paths = [target] if isinstance(target, str) else []
            return selected_rust_candidates(row, paths, crates, rust_candidates)
        return rust_candidates(row.get("rust_item"), crates)
    targets = paired_rust_targets(row)
    if targets is None:
        return "invalid_rust_targets", []
    if "rust_declaration_selectors" in row:
        return selected_rust_candidates(row, targets, crates, rust_candidates)
    matches = [rust_candidates(target, crates) for target in targets]
    status = next((status for status, _ in matches if status != "unique"), "unique")
    return status, [item for _, candidates in matches for item in candidates]


def paired_rust_targets(row: dict[str, JSONValue]) -> list[str] | None:
    """Validate an ordered target set; e.g. `paired_rust_targets(row)`."""
    targets = row.get("rust_items")
    if not isinstance(targets, list) or not targets:
        return None
    if any(not isinstance(target, str) or not target for target in targets):
        return None
    if len(set(targets)) != len(targets):
        return None
    primary = row.get("rust_item")
    if "rust_item" in row and primary != targets[0]:
        return None
    return targets


def rust_source_is_pinned(item: dict[str, JSONValue]) -> bool:
    source_file = item.get("source_file")
    source_hash = item.get("source_sha256")
    return (
        isinstance(source_file, str)
        and bool(source_file)
        and item.get("source_status") is None
        and isinstance(source_hash, str)
        and re.fullmatch(r"[0-9a-f]{64}", source_hash) is not None
    )
