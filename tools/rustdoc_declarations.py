"""Render rustdoc declarations and members independently of public reachability."""

from __future__ import annotations

import pathlib
from collections.abc import Mapping
from api_inventory_json import JSONValue, array_value, declaration_digest, object_value
from api_inventory_json import sha256_bytes, string_value


def item_index(document: Mapping[str, JSONValue]) -> dict[str, dict[str, JSONValue]]:
    """Build the item table used to resolve reachability and member links."""
    raw = object_value(document.get("index"), "rustdoc index")
    return {
        key: object_value(item, f"rustdoc index item {key}")
        for key, item in raw.items()
    }


def item_inner(item: Mapping[str, JSONValue]) -> tuple[str, dict[str, JSONValue]]:
    inner = object_value(item.get("inner"), "rustdoc item inner")
    if len(inner) != 1:
        raise ValueError("rustdoc item must contain exactly one kind")
    kind, value = next(iter(inner.items()))
    if isinstance(value, str):
        return kind, {"source": value}
    return kind, object_value(value, f"rustdoc item {kind}")


def stable_path(
    path_table: Mapping[str, JSONValue], item_id: str, fallback: str
) -> str:
    raw = path_table.get(item_id)
    if not isinstance(raw, dict):
        return fallback
    parts = raw.get("path")
    if not isinstance(parts, list) or not all(isinstance(part, str) for part in parts):
        return fallback
    return "::".join(parts)


def stable_signature(
    value: JSONValue, path_table: Mapping[str, JSONValue], unresolved: list[str]
) -> JSONValue:
    if isinstance(value, list):
        return [stable_signature(item, path_table, unresolved) for item in value]
    if not isinstance(value, dict):
        return value
    output: dict[str, JSONValue] = {}
    for key, item in value.items():
        if key == "id" and isinstance(item, (str, int)):
            path_value = value.get("path")
            fallback = path_value if isinstance(path_value, str) else ""
            stable = stable_path(path_table, str(item), fallback)
            if not stable:
                unresolved.append(str(item))
                output["unresolved_item_id"] = str(item)
            else:
                output["resolved_item_path"] = stable
        else:
            output[key] = stable_signature(item, path_table, unresolved)
    return output


def span_filename(item: Mapping[str, JSONValue]) -> str | None:
    span = item.get("span")
    if not isinstance(span, dict):
        return None
    filename = span.get("filename")
    return filename if isinstance(filename, str) else None


def file_hash(
    source_root: pathlib.Path, filename: str | None
) -> tuple[str | None, str | None]:
    if filename is None:
        return None, "missing_span"
    path = (source_root / filename).resolve()
    if not path.is_relative_to(source_root.resolve()) or not path.is_file():
        return None, "source_file_missing"
    return sha256_bytes(path.read_bytes()), None


def record_entry(
    path: str,
    item_id: str,
    item: Mapping[str, JSONValue],
    source_root: pathlib.Path,
    paths: Mapping[str, JSONValue],
    unresolved: list[str],
    impl_context: JSONValue | None = None,
) -> dict[str, JSONValue]:
    kind, body = item_inner(item)
    safe_body = definition_body(kind, body)
    if kind == "macro":
        source = string_value(safe_body.get("source"), "macro source")
        safe_body = {
            "source_sha256": sha256_bytes(source.encode()),
            "source_bytes": len(source.encode()),
        }
    signature = stable_signature(safe_body, paths, unresolved)
    digest, missing = file_hash(source_root, span_filename(item))
    entry = {
        "path": path,
        "kind": kind,
        "signature": signature,
        "source_file": span_filename(item),
        "source_sha256": digest,
        "source_status": missing,
    }
    if impl_context is not None:
        entry["impl_context"] = impl_context
    if kind == "trait":
        entry["canonical_trait_path"] = stable_path(paths, item_id, path)
    if kind in {"struct", "enum", "union", "type_alias"}:
        entry["canonical_type_path"] = stable_path(paths, item_id, path)
    entry["declaration_sha256"] = declaration_digest(entry)
    return entry


def definition_body(kind: str, body: Mapping[str, JSONValue]) -> dict[str, JSONValue]:
    collection_keys = {
        "module": {"items"},
        "struct": {"impls", "kind"},
        "union": {"impls", "kind"},
        "enum": {"impls", "variants"},
        "trait": {"impls", "items"},
    }
    removed = collection_keys.get(kind, {"impls"})
    output = {key: value for key, value in body.items() if key not in removed}
    if kind in {"struct", "union"}:
        output["shape"] = shape_summary(body.get("kind"), "fields")
    if kind == "enum":
        output["variant_count"] = len(
            array_value(body.get("variants"), "enum variants")
        )
    if kind == "variant":
        output["shape"] = shape_summary(body.get("kind"), "fields")
    return output


def shape_summary(value: JSONValue | None, field_name: str) -> JSONValue:
    if not isinstance(value, dict) or not value:
        return value
    shape_name, shape_value = next(iter(value.items()))
    if isinstance(shape_value, dict):
        fields = shape_value.get(field_name, [])
        summary: dict[str, JSONValue] = {"kind": shape_name}
        if isinstance(fields, list):
            summary["field_count"] = len(fields)
        stripped = shape_value.get("has_stripped_fields")
        if isinstance(stripped, bool):
            summary["has_stripped_fields"] = stripped
        return summary
    if isinstance(shape_value, list):
        return {"kind": shape_name, "field_count": len(shape_value)}
    return {"kind": shape_name}


def member_entries(
    item: Mapping[str, JSONValue],
    path: str,
    items: Mapping[str, dict[str, JSONValue]],
    paths: Mapping[str, JSONValue],
    source_root: pathlib.Path,
    unresolved: list[str],
) -> list[dict[str, JSONValue]]:
    kind, body = item_inner(item)
    ids = member_ids(kind, body)
    members = []
    for position, member_id in enumerate(ids):
        if not isinstance(member_id, (str, int)):
            continue
        member = items.get(str(member_id))
        if member is None:
            continue
        if not member_is_exposed(kind, member):
            continue
        name = member_name(kind, member, position)
        member_path = f"{path}::{name}"
        members.append(
            record_entry(
                member_path, str(member_id), member, source_root, paths, unresolved
            )
        )
        members.extend(
            member_entries(member, member_path, items, paths, source_root, unresolved)
        )
    return members


def member_is_exposed(parent_kind: str, member: Mapping[str, JSONValue]) -> bool:
    member_kind, _ = item_inner(member)
    if parent_kind == "enum" and member_kind == "variant":
        return True
    if parent_kind in {"trait", "variant"}:
        return True
    return member.get("visibility") == "public"


def member_name(
    parent_kind: str, member: Mapping[str, JSONValue], position: int
) -> str:
    name = member.get("name")
    if isinstance(name, str):
        return name
    return (
        f"tuple_field_{position}"
        if parent_kind == "variant"
        else f"anonymous_{position}"
    )


def member_ids(kind: str, body: Mapping[str, JSONValue]) -> list[JSONValue]:
    if kind == "struct" or kind == "union":
        shape = body.get("kind")
        if isinstance(shape, dict):
            fields = next(iter(shape.values()), {})
            if isinstance(fields, dict):
                return array_value(fields.get("fields", []), "fields")
    if kind == "enum":
        return array_value(body.get("variants", []), "variants")
    if kind == "trait":
        return array_value(body.get("items", []), "trait items")
    if kind == "variant":
        variant = body.get("kind")
        if isinstance(variant, dict):
            shape = next(iter(variant.values()), {})
            if isinstance(shape, dict):
                return array_value(shape.get("fields", []), "variant fields")
            if isinstance(shape, list):
                return [field for field in shape if field is not None]
    return []


def visible_impl_items(
    impl: Mapping[str, JSONValue],
    path: str,
    items: Mapping[str, dict[str, JSONValue]],
    paths: Mapping[str, JSONValue],
    source_root: pathlib.Path,
    unresolved: list[str],
) -> list[dict[str, JSONValue]]:
    output = []
    trait = impl.get("trait")
    trait_path = trait_impl_path(trait, paths, unresolved)
    context_value = {
        "for": impl.get("for"),
        "generics": impl.get("generics"),
        "trait": trait,
    }
    context = stable_signature(context_value, paths, unresolved)
    for member_id in array_value(impl.get("items"), "impl.items"):
        member = items.get(str(member_id))
        if member is None or (trait is None and member.get("visibility") != "public"):
            continue
        name = string_value(member.get("name"), "impl member name")
        member_path = (
            f"{path}::{name}"
            if trait_path is None
            else f"{path}::<{trait_path}>::{name}"
        )
        output.append(
            record_entry(
                member_path,
                str(member_id),
                member,
                source_root,
                paths,
                unresolved,
                context,
            )
        )
    return output


def trait_impl_path(
    trait: JSONValue | None, paths: Mapping[str, JSONValue], unresolved: list[str]
) -> str | None:
    if trait is None:
        return None
    normalized = stable_signature(trait, paths, unresolved)
    if not isinstance(normalized, dict):
        return None
    return string_value(
        normalized.get("resolved_item_path", normalized.get("path")), "trait path"
    )
