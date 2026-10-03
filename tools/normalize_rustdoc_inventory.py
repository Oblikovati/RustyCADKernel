#!/usr/bin/env python3
"""Normalize pinned rustdoc JSON into a reachable Rust API inventory."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import sys
from collections import Counter
from collections.abc import Mapping, Sequence
from api_inventory_capture import capture_files, index_records, verify_artifact
from api_inventory_json import JSONValue, array_value, declaration_digest, integer_value, object_value
from api_inventory_json import read_json, sha256_bytes, string_value
from api_inventory_reexports import resolve_cross_crate_reexports, source_hashes
from rustdoc_impl_ownership import inherent_members, unresolved_impl_owner_links
ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_CAPTURE = ROOT / "target/api-proof/rustdoc-json"
DEFAULT_OUTPUT = ROOT / "target/api-proof/rust-api-inventory.json"


def item_index(document: Mapping[str, JSONValue]) -> dict[str, dict[str, JSONValue]]:
    """Build the item table used to resolve reachability and member links."""
    raw = object_value(document.get("index"), "rustdoc index")
    return {key: object_value(item, f"rustdoc index item {key}") for key, item in raw.items()}


def item_inner(item: Mapping[str, JSONValue]) -> tuple[str, dict[str, JSONValue]]:
    inner = object_value(item.get("inner"), "rustdoc item inner")
    if len(inner) != 1:
        raise ValueError("rustdoc item must contain exactly one kind")
    kind, value = next(iter(inner.items()))
    if isinstance(value, str):
        return kind, {"source": value}
    return kind, object_value(value, f"rustdoc item {kind}")


def stable_path(path_table: Mapping[str, JSONValue], item_id: str, fallback: str) -> str:
    raw = path_table.get(item_id)
    if not isinstance(raw, dict):
        return fallback
    parts = raw.get("path")
    if not isinstance(parts, list) or not all(isinstance(part, str) for part in parts):
        return fallback
    return "::".join(parts)


def stable_signature(value: JSONValue, path_table: Mapping[str, JSONValue], unresolved: list[str]) -> JSONValue:
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


def file_hash(source_root: pathlib.Path, filename: str | None) -> tuple[str | None, str | None]:
    if filename is None:
        return None, "missing_span"
    path = (source_root / filename).resolve()
    if not path.is_relative_to(source_root.resolve()) or not path.is_file():
        return None, "source_file_missing"
    return sha256_bytes(path.read_bytes()), None


def package_digest(workspace: pathlib.Path, manifest_path: str) -> str:
    package_root = workspace / pathlib.Path(*pathlib.PurePosixPath(manifest_path).parent.parts)
    paths = sorted(path for path in package_root.rglob("*") if path.is_file())
    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.relative_to(workspace).as_posix().encode())
        digest.update(b"\0")
        digest.update(sha256_bytes(path.read_bytes()).encode())
        digest.update(b"\n")
    return digest.hexdigest()


def record_entry(path: str, item_id: str, item: Mapping[str, JSONValue], source_root: pathlib.Path,
                 paths: Mapping[str, JSONValue], unresolved: list[str],
                 impl_context: JSONValue | None = None) -> dict[str, JSONValue]:
    kind, body = item_inner(item)
    safe_body = definition_body(kind, body)
    if kind == "macro":
        source = string_value(safe_body.get("source"), "macro source")
        safe_body = {"source_sha256": sha256_bytes(source.encode()), "source_bytes": len(source.encode())}
    signature = stable_signature(safe_body, paths, unresolved)
    digest, missing = file_hash(source_root, span_filename(item))
    entry = {"path": path, "kind": kind, "signature": signature, "source_file": span_filename(item),
             "source_sha256": digest, "source_status": missing}
    if impl_context is not None:
        entry["impl_context"] = impl_context
    entry["declaration_sha256"] = declaration_digest(entry)
    return entry


def definition_body(kind: str, body: Mapping[str, JSONValue]) -> dict[str, JSONValue]:
    collection_keys = {"module": {"items"}, "struct": {"impls", "kind"}, "union": {"impls", "kind"},
                       "enum": {"impls", "variants"}, "trait": {"impls", "items"}}
    removed = collection_keys.get(kind, {"impls"})
    output = {key: value for key, value in body.items() if key not in removed}
    if kind in {"struct", "union"}:
        output["shape"] = shape_summary(body.get("kind"), "fields")
    if kind == "enum":
        output["variant_count"] = len(array_value(body.get("variants"), "enum variants"))
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


def member_entries(item: Mapping[str, JSONValue], path: str, items: Mapping[str, dict[str, JSONValue]],
                   paths: Mapping[str, JSONValue], source_root: pathlib.Path,
                   unresolved: list[str]) -> list[dict[str, JSONValue]]:
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
        members.append(record_entry(member_path, str(member_id), member, source_root, paths, unresolved))
        members.extend(member_entries(member, member_path, items, paths, source_root, unresolved))
    return members


def member_is_exposed(parent_kind: str, member: Mapping[str, JSONValue]) -> bool:
    member_kind, _ = item_inner(member)
    if parent_kind == "enum" and member_kind == "variant":
        return True
    if parent_kind in {"trait", "variant"}:
        return True
    return member.get("visibility") == "public"


def member_name(parent_kind: str, member: Mapping[str, JSONValue], position: int) -> str:
    name = member.get("name")
    if isinstance(name, str):
        return name
    return f"tuple_field_{position}" if parent_kind == "variant" else f"anonymous_{position}"


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


def visible_impl_items(impl: Mapping[str, JSONValue], path: str, items: Mapping[str, dict[str, JSONValue]],
                       paths: Mapping[str, JSONValue], source_root: pathlib.Path,
                       unresolved: list[str]) -> list[dict[str, JSONValue]]:
    output = []
    trait = impl.get("trait")
    trait_path = trait_impl_path(trait, paths, unresolved)
    context_value = {"for": impl.get("for"), "generics": impl.get("generics"), "trait": trait}
    context = stable_signature(context_value, paths, unresolved)
    for member_id in array_value(impl.get("items"), "impl.items"):
        member = items.get(str(member_id))
        if member is None or (trait is None and member.get("visibility") != "public"):
            continue
        name = string_value(member.get("name"), "impl member name")
        member_path = f"{path}::{name}" if trait_path is None else f"{path}::<{trait_path}>::{name}"
        output.append(record_entry(member_path, str(member_id), member, source_root,
                                   paths, unresolved, context))
    return output


def trait_impl_path(trait: JSONValue | None, paths: Mapping[str, JSONValue],
                    unresolved: list[str]) -> str | None:
    if trait is None:
        return None
    normalized = stable_signature(trait, paths, unresolved)
    if not isinstance(normalized, dict):
        return None
    return string_value(normalized.get("resolved_item_path", normalized.get("path")), "trait path")


def make_reachable_inventory(document: Mapping[str, JSONValue], source_root: pathlib.Path,
                             crate_name: str) -> dict[str, JSONValue]:
    items = item_index(document)
    paths = object_value(document.get("paths"), "rustdoc paths")
    root_id = str(integer_value(document.get("root"), "rustdoc root"))
    root_item = items.get(root_id)
    if root_item is None:
        raise ValueError(f"rustdoc root item {root_id} is absent")
    root_name = string_value(root_item.get("name"), "crate root name")
    if root_name != crate_name:
        raise ValueError(f"rustdoc root {root_name} does not match {crate_name}")
    state = WalkState(items, paths, source_root)
    walk_module(root_item, root_name, state, set())
    state.unresolved.extend(
        unresolved_impl_owner_links(state.foreign_impl_links, state.owned_impl_ids)
    )
    return {"crate": crate_name, "declarations": state.entries,
            "unclassified_kinds": {"counts": unsupported_counts(items),
                                    "scope": "raw index item kinds not classified by this normalizer"},
            "unresolved_links": sorted(set(state.unresolved)), "source_files": source_hashes(state.entries)}


class WalkState:
    def __init__(self, items: Mapping[str, dict[str, JSONValue]], paths: Mapping[str, JSONValue],
                 source_root: pathlib.Path) -> None:
        self.items = items
        self.paths = paths
        self.source_root = source_root
        self.entries: list[dict[str, JSONValue]] = []
        self.unresolved: list[str] = []
        self.seen: set[tuple[str, str]] = set()
        self.foreign_impl_links: dict[str, set[str]] = {}
        self.owned_impl_ids: set[str] = set()


def walk_module(item: Mapping[str, JSONValue], path: str, state: WalkState,
                active: set[str], emit: bool = True) -> None:
    item_id = str(integer_value(item.get("id"), "module id"))
    if item_id in active:
        return
    kind, body = item_inner(item)
    if kind != "module":
        return
    key = (item_id, path)
    if key in state.seen:
        return
    state.seen.add(key)
    if emit:
        state.entries.append(record_entry(path, item_id, item, state.source_root, state.paths, state.unresolved))
    child_active = active | {item_id}
    for child_id in array_value(body.get("items"), "module.items"):
        child = state.items.get(str(child_id))
        if child is not None:
            expose_child(child, path, child_active, state)


def expose_child(item: Mapping[str, JSONValue], parent: str, active: set[str], state: WalkState) -> None:
    if item.get("visibility") != "public":
        return
    kind, body = item_inner(item)
    item_id = str(integer_value(item.get("id"), "item id"))
    name = item_name(item, kind, body)
    path = f"{parent}::{name}"
    if kind == "use":
        expose_use(item, body, path, active, state)
        return
    if kind == "module":
        walk_module(item, path, state, active)
        return
    key = (item_id, path)
    if key in state.seen:
        return
    state.seen.add(key)
    state.entries.append(record_entry(path, item_id, item, state.source_root, state.paths, state.unresolved))
    state.entries.extend(member_entries(item, path, state.items, state.paths, state.source_root, state.unresolved))
    state.entries.extend(
        inherent_members(item, path, state, item_inner, visible_impl_items)
    )


def item_name(item: Mapping[str, JSONValue], kind: str, body: Mapping[str, JSONValue]) -> str:
    if kind == "use":
        return string_value(body.get("name"), "re-export name")
    return string_value(item.get("name"), f"public {kind} name")


def expose_use(item: Mapping[str, JSONValue], body: Mapping[str, JSONValue], path: str,
               active: set[str], state: WalkState) -> None:
    target = str(integer_value(body.get("id"), "re-export target id"))
    source = string_value(body.get("source"), "re-export source")
    canonical_target = stable_path(state.paths, target, source)
    target_path = reexport_target_path(target, source, state.paths)
    reexport = reexport_metadata(item, source, body.get("is_glob") is True, state)
    reexport["canonical_target_path"] = canonical_target
    target_item = state.items.get(target)
    if target_item is None:
        reexport_entry(path, "unresolved_reexport", target_path, reexport, state, target)
        return
    target_kind, _ = item_inner(target_item)
    if target_kind == "module":
        reexport_entry(path, "reexport_module", target_path, reexport, state)
        walk_module(target_item, path, state, active, emit=False)
        return
    expose_alias_target(target_item, path, target, reexport, state)


def reexport_target_path(target: str, source: str, paths: Mapping[str, JSONValue]) -> str:
    if source.startswith("rustycad_"):
        return source
    return stable_path(paths, target, source)


def reexport_metadata(item: Mapping[str, JSONValue], source: str, is_glob: bool,
                      state: WalkState) -> dict[str, JSONValue]:
    filename = span_filename(item)
    digest, status = file_hash(state.source_root, filename)
    return {"source": source, "is_glob": is_glob, "source_file": filename,
            "source_sha256": digest, "source_status": status}


def reexport_entry(path: str, kind: str, target_path: str, metadata: Mapping[str, JSONValue],
                   state: WalkState, unresolved_id: str | None = None) -> None:
    entry: dict[str, JSONValue] = {"path": path, "kind": kind, "target_path": target_path,
                                   "reexport": dict(metadata)}
    if unresolved_id is not None or not target_path:
        target = unresolved_id or target_path
        state.unresolved.append(target)
        entry["unresolved_target"] = target
    entry["declaration_sha256"] = declaration_digest(entry)
    state.entries.append(entry)


def expose_alias_target(item: Mapping[str, JSONValue], path: str, target: str,
                        reexport: Mapping[str, JSONValue], state: WalkState) -> None:
    key = (target, path)
    if key in state.seen:
        return
    state.seen.add(key)
    entry = record_entry(path, target, item, state.source_root, state.paths, state.unresolved)
    entry["reexport"] = dict(reexport)
    entry["declaration_sha256"] = declaration_digest(entry)
    state.entries.append(entry)
    state.entries.extend(member_entries(item, path, state.items, state.paths, state.source_root, state.unresolved))
    state.entries.extend(
        inherent_members(item, path, state, item_inner, visible_impl_items)
    )


def unsupported_counts(items: Mapping[str, dict[str, JSONValue]]) -> dict[str, int]:
    counts: Counter[str] = Counter()
    supported = {"module", "use", "struct", "enum", "union", "trait", "function", "type_alias",
                 "constant", "static", "macro", "proc_macro", "primitive", "assoc_const", "assoc_type",
                 "impl", "variant", "struct_field"}
    for item in items.values():
        kind, _ = item_inner(item)
        if kind not in supported:
            counts[kind] += 1
    return dict(sorted(counts.items()))


def crate_inventory(capture: pathlib.Path, record: Mapping[str, JSONValue], workspace: pathlib.Path,
                    expected: int) -> dict[str, JSONValue]:
    document = verify_artifact(capture, record, expected)
    crate = string_value(record.get("library_target"), "library_target")
    manifest_path = string_value(record.get("manifest_path"), "manifest_path")
    actual_package_digest = package_digest(workspace, manifest_path)
    expected_package_digest = string_value(record.get("package_inputs_sha256"), "package_inputs_sha256")
    if actual_package_digest != expected_package_digest:
        raise ValueError(f"captured source inputs differ for {crate}; use the matching workspace revision")
    result = make_reachable_inventory(document, workspace, crate)
    result["artifact_sha256"] = record.get("artifact_sha256")
    result["package_source_sha256"] = record.get("package_inputs_sha256")
    result["raw_indexed_items"] = record.get("indexed_items")
    result["feature_selection"] = record.get("feature_selection")
    result["available_features"] = record.get("available_features")
    return result


def normalized_manifest(capture: pathlib.Path, workspace: pathlib.Path) -> dict[str, JSONValue]:
    manifest, manifest_path = capture_files(capture)
    config = object_value(read_json(ROOT / "porting/api-proof-toolchain.json"), "toolchain config")
    format_version = integer_value(config.get("rustdoc_json_format_version"), "rustdoc_json_format_version")
    if manifest.get("rustdoc_json_format_version") not in (None, format_version):
        raise ValueError("capture format does not match pinned rustdoc JSON version 61")
    records = index_records(manifest)
    crates = [crate_inventory(capture, records[name], workspace, format_version) for name in sorted(records)]
    resolve_cross_crate_reexports(crates)
    return {"schema_version": 1, "inventory_kind": "reachable-rust-public-surface",
            "cross_language_completeness_claim": False, "rustdoc_format_version": format_version,
            "visibility_policy": "public-reachable-including-doc-hidden-v1",
            "capture_manifest_sha256": sha256_bytes(manifest_path.read_bytes()),
            "capture_workspace_revision": capture_revision(manifest),
            "capture_feature_policy": manifest.get("feature_policy"),
            "cargo_arguments": manifest.get("cargo_arguments"),
            "capture_inputs": selected_capture_inputs(manifest), "crates": crates,
            "notes": ["Only reachable public Rust surfaces are listed.",
                      "Unsupported rustdoc kinds and unresolved links are explicit per crate.",
                      "This inventory is not an OCCT-to-Rust parity proof."]}


def capture_revision(manifest: Mapping[str, JSONValue]) -> str:
    """Return the source revision recorded alongside the raw artifacts."""
    inputs = object_value(manifest.get("inputs"), "capture inputs")
    return string_value(inputs.get("workspace_revision"), "workspace_revision")


def selected_capture_inputs(manifest: Mapping[str, JSONValue]) -> dict[str, JSONValue]:
    inputs = object_value(manifest.get("inputs"), "capture inputs")
    keys = ("workspace_revision", "toolchain_config_sha256", "cargo_metadata_sha256",
            "workspace_inputs_sha256", "workspace_file_count", "tool_versions")
    return {key: inputs[key] for key in keys if key in inputs}


def emit_inventory(capture: pathlib.Path, workspace: pathlib.Path, output: pathlib.Path) -> None:
    result = normalized_manifest(capture, workspace)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=pathlib.Path, default=DEFAULT_CAPTURE)
    parser.add_argument("--workspace", type=pathlib.Path, default=ROOT)
    parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_OUTPUT)
    return parser


def main() -> int:
    args = argument_parser().parse_args()
    emit_inventory(args.capture, args.workspace, args.output)
    print(f"Wrote reachable Rust API inventory: {args.output}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
