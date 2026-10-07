#!/usr/bin/env python3
"""Replay exact inferred OSD interface bindings; no implemented credit is granted."""

from __future__ import annotations

import argparse
import gzip
import json
from pathlib import Path

from api_inventory.inputs import (
    JSONObject,
    digest,
    object_rows,
    read_object,
    text_field,
)
from normalize_rustdoc_inventory import make_reachable_inventory
from verify_returned_private_reachability import verify_files

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = "porting/audits/osd-iterator-interface-v1.json"
NATIVE_FIELDS = (
    "id",
    "qualified_name",
    "signature",
    "kind",
    "access",
    "ancestor_access",
    "api_visible",
    "exclusion_reason",
)

SCOPED_METHODS = {
    "OSD_Parallel::IteratorInterface": ("ParallelIteratorInterface", "inferred_trait"),
    "OSD_Parallel::IteratorInterface::~IteratorInterface": (
        "destroy_parallel_iterator",
        "owned_delete_body",
    ),
    "OSD_Parallel::IteratorInterface::IsEqual": (
        "ParallelIteratorInterface::is_equal",
        "abstract_dispatch",
    ),
    "OSD_Parallel::IteratorInterface::Increment": (
        "ParallelIteratorInterface::increment",
        "abstract_dispatch",
    ),
    "OSD_Parallel::IteratorInterface::Clone": (
        "ParallelIteratorInterface::clone_box",
        "abstract_dispatch",
    ),
}


def verify_scope(binding: JSONObject) -> None:
    """Reject methods borrowed from another obligation; e.g. `verify_scope(binding)`."""
    pair = SCOPED_METHODS.get(binding["native"]["qualified_name"])
    actual = (
        binding["rust"]["path"].removeprefix("rustycad_tkernel::osd::"),
        binding["evidence_mode"],
    )
    if actual != pair:
        raise ValueError(f"scoped binding {actual!r}: expected {pair!r}")


def compiler_rows(root: Path, receipt: JSONObject) -> list[JSONObject]:
    """Re-normalize raw rustdoc rather than trusting hand-entered rows; e.g. `compiler_rows(ROOT, receipt)`."""
    with gzip.open(root / text_field(receipt, "rustdoc_capture"), "rt") as stream:
        document = json.load(stream)
    if document["format_version"] != 61:
        raise ValueError(f"rustdoc format {document['format_version']}: expected 61")
    return make_reachable_inventory(document, root, "rustycad_tkernel")["declarations"]


def read_rows(path: Path) -> list[JSONObject]:
    """Read exact compressed compiler rows; e.g. `read_rows(native_path)`."""
    with gzip.open(path, "rt") as stream:
        return [
            row
            for row in map(json.loads, stream)
            if row.get("qualified_name") in SCOPED_METHODS
        ]


def verify_binding(
    binding: JSONObject, native: list[JSONObject], rust: list[JSONObject], root: Path
) -> None:
    """Reject wrong IDs, abstract/body confusion or stale bytes; e.g. `verify_binding(row, cpp, rs, root)`."""
    verify_scope(binding)
    expected = binding["native"]
    matches = [row for row in native if row["id"] == expected["id"]]
    actual = [{key: row[key] for key in NATIVE_FIELDS} for row in matches]
    if actual != [expected] or expected["api_visible"] is not False:
        raise ValueError(
            f"native {expected!r}: expected one unchanged private declaration"
        )
    selected = [row for row in rust if row["path"] == binding["rust"]["path"]]
    if selected != [binding["rust"]]:
        raise ValueError(
            f"Rust {binding['rust']['path']}: expected one exact compiler declaration"
        )
    verify_rust_shape(binding, selected[0], root)


def verify_rust_shape(binding: JSONObject, rust: JSONObject, root: Path) -> None:
    """Keep required trait declarations separate from bodies; e.g. `verify_rust_shape(binding, row, root)`."""
    mode = binding["evidence_mode"]
    has_body = rust["signature"].get("has_body")
    expected = {
        "abstract_dispatch": ("function", False),
        "owned_delete_body": ("function", True),
        "inferred_trait": ("trait", None),
    }
    if (rust["kind"], has_body) != expected.get(mode) or binding["status"] != "partial":
        raise ValueError(
            f"Rust shape {(rust['kind'], has_body, mode)!r}: expected bounded partial interface evidence"
        )
    for record in [rust, *([rust["reexport"]] if "reexport" in rust else [])]:
        if digest(root / text_field(record, "source_file")) != record["source_sha256"]:
            raise ValueError(
                f"source {record['source_file']}: expected compiler source hash"
            )


def replay(root: Path, receipt_path: Path) -> JSONObject:
    """Replay the new OSD subset while retaining the historical overlay; e.g. `replay(ROOT, path)`."""
    receipt = read_object(receipt_path)
    verify_files(root, receipt)
    bindings = object_rows(receipt["bindings"], "bindings")
    if len(bindings) != 5 or len({row["native"]["id"] for row in bindings}) != 5:
        raise ValueError(
            f"binding count {len(bindings)}: expected five distinct native declarations"
        )
    native = read_rows(root / "porting/api-inventory/occt-declarations.jsonl.gz")
    with gzip.open(root / text_field(receipt, "rust_inventory"), "rt") as stream:
        rust = json.load(stream)["crates"][0]["declarations"]
    captured = compiler_rows(root, receipt)
    for row in bindings:
        verify_binding(row, native, rust, root)
        verify_binding(row, native, captured, root)
    return {
        "partial_bindings": 5,
        "required_abstract_methods": 3,
        "owned_delete_bodies": 1,
        "inferred_traits": 1,
        "implementation_credit": False,
        "api_denominator_complete": False,
    }


def main() -> None:
    """Choose current or historical receipt explicitly; e.g. `--receipt current.json`."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, default=ROOT / RECEIPT)
    arguments = parser.parse_args()
    print(json.dumps(replay(ROOT, arguments.receipt), sort_keys=True))


if __name__ == "__main__":
    main()
