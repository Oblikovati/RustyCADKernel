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
from api_inventory_json import (
    JSONValue,
    array_value,
    declaration_digest,
    integer_value,
    object_value,
)
from api_inventory_json import read_json, sha256_bytes, string_value
from api_inventory_reexports import resolve_cross_crate_reexports, source_hashes
from api_tool_hashes import stage_hashes
from rustdoc_impl_ownership import inherent_members, unresolved_impl_owner_links
from rustdoc_foreign_impls import resolve_foreign_impl_candidates
from rustdoc_trait_paths import resolve_public_trait_paths
from rustdoc_declarations import (
    file_hash,
    item_index,
    item_inner,
    member_entries,
    record_entry,
    span_filename,
    stable_path,
    visible_impl_items,
)

ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_CAPTURE = ROOT / "target/api-proof/rustdoc-json"
DEFAULT_OUTPUT = ROOT / "target/api-proof/rust-api-inventory.json"


def package_digest(workspace: pathlib.Path, manifest_path: str) -> str:
    package_root = workspace / pathlib.Path(
        *pathlib.PurePosixPath(manifest_path).parent.parts
    )
    paths = sorted(path for path in package_root.rglob("*") if path.is_file())
    digest = hashlib.sha256()
    for path in paths:
        digest.update(path.relative_to(workspace).as_posix().encode())
        digest.update(b"\0")
        digest.update(sha256_bytes(path.read_bytes()).encode())
        digest.update(b"\n")
    return digest.hexdigest()


def make_reachable_inventory(
    document: Mapping[str, JSONValue], source_root: pathlib.Path, crate_name: str
) -> dict[str, JSONValue]:
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
    return {
        "crate": crate_name,
        "declarations": state.entries,
        "unclassified_kinds": {
            "counts": unsupported_counts(items),
            "scope": "raw index item kinds not classified by this normalizer",
        },
        "unresolved_links": sorted(set(state.unresolved)),
        "source_files": source_hashes(state.entries),
        "foreign_impl_candidates": state.foreign_impl_candidates,
    }


class WalkState:
    def __init__(
        self,
        items: Mapping[str, dict[str, JSONValue]],
        paths: Mapping[str, JSONValue],
        source_root: pathlib.Path,
    ) -> None:
        self.items = items
        self.paths = paths
        self.source_root = source_root
        self.entries: list[dict[str, JSONValue]] = []
        self.unresolved: list[str] = []
        self.seen: set[tuple[str, str]] = set()
        self.foreign_impl_links: dict[str, set[str]] = {}
        self.foreign_impl_candidates: list[dict[str, JSONValue]] = []
        self.owned_impl_ids: set[str] = set()


def walk_module(
    item: Mapping[str, JSONValue],
    path: str,
    state: WalkState,
    active: set[str],
    emit: bool = True,
) -> None:
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
        state.entries.append(
            record_entry(
                path, item_id, item, state.source_root, state.paths, state.unresolved
            )
        )
    child_active = active | {item_id}
    for child_id in array_value(body.get("items"), "module.items"):
        child = state.items.get(str(child_id))
        if child is not None:
            expose_child(child, path, child_active, state)


def expose_child(
    item: Mapping[str, JSONValue], parent: str, active: set[str], state: WalkState
) -> None:
    if item.get("visibility") != "public":
        return
    kind, body = item_inner(item)
    item_id = str(integer_value(item.get("id"), "item id"))
    name = item_name(item, kind, body)
    path = f"{parent}::{name}"
    if kind == "use":
        expose_use(
            item, body, parent if body.get("is_glob") is True else path, active, state
        )
        return
    if kind == "module":
        walk_module(item, path, state, active)
        return
    key = (item_id, path)
    if key in state.seen:
        return
    state.seen.add(key)
    state.entries.append(
        record_entry(
            path, item_id, item, state.source_root, state.paths, state.unresolved
        )
    )
    state.entries.extend(
        member_entries(
            item, path, state.items, state.paths, state.source_root, state.unresolved
        )
    )
    state.entries.extend(
        inherent_members(item, path, state, item_inner, visible_impl_items)
    )


def item_name(
    item: Mapping[str, JSONValue], kind: str, body: Mapping[str, JSONValue]
) -> str:
    if kind == "use":
        return string_value(body.get("name"), "re-export name")
    return string_value(item.get("name"), f"public {kind} name")


def expose_use(
    item: Mapping[str, JSONValue],
    body: Mapping[str, JSONValue],
    path: str,
    active: set[str],
    state: WalkState,
) -> None:
    target = str(integer_value(body.get("id"), "re-export target id"))
    source = string_value(body.get("source"), "re-export source")
    canonical_target = stable_path(state.paths, target, source)
    target_path = reexport_target_path(target, source, state.paths)
    reexport = reexport_metadata(item, source, body.get("is_glob") is True, state)
    reexport["canonical_target_path"] = canonical_target
    target_item = state.items.get(target)
    if target_item is None:
        reexport_entry(
            path, "unresolved_reexport", target_path, reexport, state, target
        )
        return
    target_kind, _ = item_inner(target_item)
    if target_kind == "module":
        if body.get("is_glob") is not True:
            reexport_entry(path, "reexport_module", target_path, reexport, state)
        walk_module(target_item, path, state, active, emit=False)
        return
    expose_alias_target(target_item, path, target, reexport, state)


def reexport_target_path(
    target: str, source: str, paths: Mapping[str, JSONValue]
) -> str:
    if source.startswith("rustycad_"):
        return source
    return stable_path(paths, target, source)


def reexport_metadata(
    item: Mapping[str, JSONValue], source: str, is_glob: bool, state: WalkState
) -> dict[str, JSONValue]:
    filename = span_filename(item)
    digest, status = file_hash(state.source_root, filename)
    return {
        "source": source,
        "is_glob": is_glob,
        "source_file": filename,
        "source_sha256": digest,
        "source_status": status,
    }


def reexport_entry(
    path: str,
    kind: str,
    target_path: str,
    metadata: Mapping[str, JSONValue],
    state: WalkState,
    unresolved_id: str | None = None,
) -> None:
    entry: dict[str, JSONValue] = {
        "path": path,
        "kind": kind,
        "target_path": target_path,
        "reexport": dict(metadata),
    }
    if unresolved_id is not None or not target_path:
        target = unresolved_id or target_path
        state.unresolved.append(target)
        entry["unresolved_target"] = target
    entry["declaration_sha256"] = declaration_digest(entry)
    state.entries.append(entry)


def expose_alias_target(
    item: Mapping[str, JSONValue],
    path: str,
    target: str,
    reexport: Mapping[str, JSONValue],
    state: WalkState,
) -> None:
    key = (target, path)
    if key in state.seen:
        return
    state.seen.add(key)
    entry = record_entry(
        path, target, item, state.source_root, state.paths, state.unresolved
    )
    entry["reexport"] = dict(reexport)
    entry["declaration_sha256"] = declaration_digest(entry)
    state.entries.append(entry)
    state.entries.extend(
        member_entries(
            item, path, state.items, state.paths, state.source_root, state.unresolved
        )
    )
    state.entries.extend(
        inherent_members(item, path, state, item_inner, visible_impl_items)
    )


def unsupported_counts(items: Mapping[str, dict[str, JSONValue]]) -> dict[str, int]:
    counts: Counter[str] = Counter()
    supported = {
        "module",
        "use",
        "struct",
        "enum",
        "union",
        "trait",
        "function",
        "type_alias",
        "constant",
        "static",
        "macro",
        "proc_macro",
        "primitive",
        "assoc_const",
        "assoc_type",
        "impl",
        "variant",
        "struct_field",
    }
    for item in items.values():
        kind, _ = item_inner(item)
        if kind not in supported:
            counts[kind] += 1
    return dict(sorted(counts.items()))


def crate_inventory(
    capture: pathlib.Path,
    record: Mapping[str, JSONValue],
    workspace: pathlib.Path,
    expected: int,
) -> dict[str, JSONValue]:
    document = verify_artifact(capture, record, expected)
    crate = string_value(record.get("library_target"), "library_target")
    manifest_path = string_value(record.get("manifest_path"), "manifest_path")
    actual_package_digest = package_digest(workspace, manifest_path)
    expected_package_digest = string_value(
        record.get("package_inputs_sha256"), "package_inputs_sha256"
    )
    if actual_package_digest != expected_package_digest:
        raise ValueError(
            f"captured source inputs differ for {crate}; use the matching workspace revision"
        )
    result = make_reachable_inventory(document, workspace, crate)
    result["artifact_sha256"] = record.get("artifact_sha256")
    result["package_source_sha256"] = record.get("package_inputs_sha256")
    result["raw_indexed_items"] = record.get("indexed_items")
    result["feature_selection"] = record.get("feature_selection")
    result["available_features"] = record.get("available_features")
    return result


def normalized_manifest(
    capture: pathlib.Path, workspace: pathlib.Path
) -> dict[str, JSONValue]:
    manifest, manifest_path = capture_files(capture)
    config = object_value(
        read_json(ROOT / "porting/api-proof-toolchain.json"), "toolchain config"
    )
    format_version = integer_value(
        config.get("rustdoc_json_format_version"), "rustdoc_json_format_version"
    )
    if manifest.get("rustdoc_json_format_version") not in (None, format_version):
        raise ValueError("capture format does not match pinned rustdoc JSON version 61")
    records = index_records(manifest)
    crates = [
        crate_inventory(capture, records[name], workspace, format_version)
        for name in sorted(records)
    ]
    resolve_cross_crate_reexports(crates)
    resolve_foreign_impl_candidates(crates)
    resolve_public_trait_paths(crates)
    return {
        "schema_version": 1,
        "inventory_kind": "reachable-rust-public-surface",
        "cross_language_completeness_claim": False,
        "rustdoc_format_version": format_version,
        "visibility_policy": "public-reachable-from-private-item-capture-v1",
        "rustdoc_item_policy": manifest.get("rustdoc_item_policy"),
        "capture_manifest_sha256": sha256_bytes(manifest_path.read_bytes()),
        "capture_workspace_revision": capture_revision(manifest),
        "tool_implementation_sha256": {
            "rust_normalizer": stage_hashes(ROOT, "rust_normalizer")
        },
        "capture_feature_policy": manifest.get("feature_policy"),
        "cargo_arguments": manifest.get("cargo_arguments"),
        "capture_inputs": selected_capture_inputs(manifest),
        "crates": crates,
        "notes": [
            "Private rustdoc items support alias resolution but are not emitted unless publicly reachable.",
            "Only reachable public Rust surfaces are listed.",
            "Unsupported rustdoc kinds and unresolved links are explicit per crate.",
            "This inventory is not an OCCT-to-Rust parity proof.",
        ],
    }


def capture_revision(manifest: Mapping[str, JSONValue]) -> str:
    """Return the source revision recorded alongside the raw artifacts."""
    inputs = object_value(manifest.get("inputs"), "capture inputs")
    return string_value(inputs.get("workspace_revision"), "workspace_revision")


def selected_capture_inputs(manifest: Mapping[str, JSONValue]) -> dict[str, JSONValue]:
    inputs = object_value(manifest.get("inputs"), "capture inputs")
    keys = (
        "workspace_revision",
        "toolchain_config_sha256",
        "cargo_metadata_sha256",
        "workspace_inputs_sha256",
        "workspace_file_count",
        "tool_versions",
    )
    return {key: inputs[key] for key in keys if key in inputs}


def emit_inventory(
    capture: pathlib.Path, workspace: pathlib.Path, output: pathlib.Path
) -> None:
    result = normalized_manifest(capture, workspace)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


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
