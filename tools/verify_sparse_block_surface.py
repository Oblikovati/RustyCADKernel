#!/usr/bin/env python3
"""Replay bounded Sparse Block bindings and refreshed historical base-body evidence."""

from __future__ import annotations

import argparse
import json
from collections.abc import Callable
from pathlib import Path

from api_binding_replay import binding_equal, binding_object
from api_binding_transaction import replay_binding_transaction
from api_coverage_report import read_json, read_jsonl
from api_inventory.inputs import JSONObject, digest, object_rows, text_field
from api_inventory.merge import declaration_identity
from verify_returned_private_reachability import verify_files

ROOT = Path(__file__).resolve().parents[1]
RECEIPT = Path("porting/audits/sparse-block-native-surface-v1.json")
RUST_KINDS = {
    "CXXRecord": "struct",
    "Typedef": "type_alias",
    "Field": "struct_field",
    "CXXMethod": "function",
    "CXXConstructor": "function",
}


def verify_block_native(record: JSONObject, declarations: list[JSONObject]) -> None:
    """Require the exact captured declaration without changing visibility; e.g. `verify_block_native(record, rows)`."""
    expected = binding_object(record, "native_declaration")
    identifier = text_field(expected, "id")
    binding_equal(declaration_identity(expected), identifier, "exact native ID")
    matches = [row for row in declarations if row.get("id") == identifier]
    binding_equal(matches, [expected], "native declaration")
    binding_equal(expected.get("api_visible"), False, "native primary visibility")
    if not text_field(expected, "qualified_name").startswith(
        "NCollection_SparseArrayBase::Block"
    ):
        raise ValueError(f"native {identifier}: expected bounded Sparse Block scope")


def verify_block_rust(
    record: JSONObject,
    declarations: list[JSONObject],
    source_hash: Callable[[str], str],
) -> None:
    """Require one current compiler declaration; e.g. `verify_block_rust(record, rust, digest)`."""
    expected = binding_object(record, "rust_declaration")
    matches = [
        row
        for row in declarations
        if row.get("path") == expected.get("path")
        and row.get("kind") == expected.get("kind")
    ]
    binding_equal(matches, [expected], "unique exact Rust declaration")
    native = binding_object(record, "native_declaration")
    binding_equal(
        expected.get("kind"), RUST_KINDS.get(text_field(native, "kind")), "Rust kind"
    )
    verify_block_function(expected)
    verify_compiler_source(expected, source_hash)


def verify_compiler_source(
    expected: JSONObject, source_hash: Callable[[str], str]
) -> None:
    """Reject stale declaration or reexport files; e.g. `verify_compiler_source(row, digest)`."""
    binding_equal(
        source_hash(text_field(expected, "source_file")),
        expected.get("source_sha256"),
        "current Rust source",
    )
    reexport = expected.get("reexport")
    if isinstance(reexport, dict):
        binding_equal(
            source_hash(text_field(reexport, "source_file")),
            reexport.get("source_sha256"),
            "current Rust export",
        )


def verify_block_function(declaration: JSONObject) -> None:
    """Withhold function declaration-only matches; e.g. `verify_block_function(compiler_row)`."""
    if declaration.get("kind") != "function":
        return
    binding_equal(
        binding_object(declaration, "signature").get("has_body"),
        True,
        "Rust function body",
    )
    context = binding_object(declaration, "impl_context")
    verify_block_impl(context)


def verify_block_impl(context: JSONObject) -> None:
    """Reject unrelated and specialized implementations; e.g. `verify_block_impl(context)`."""
    binding_equal(context.get("trait"), None, "inherent block implementation")
    binding_equal(
        context.get("generics"),
        {"params": [], "where_predicates": []},
        "block impl generics",
    )
    owner = binding_object(context, "for")
    resolved = binding_object(owner, "resolved_path")
    binding_equal(
        resolved.get("resolved_item_path"),
        "rustycad_tkernel::ncollection::sparse_array_base::block::SparseArrayBlock",
        "block impl owner",
    )


def verify_block_records(
    records: list[JSONObject],
    native: list[JSONObject],
    rust: list[JSONObject],
    source_hash: Callable[[str], str],
) -> int:
    """Reject duplicates and automatic parity credit; e.g. `verify_block_records(records, native, rust, digest)`."""
    seen: set[str] = set()
    for record in records:
        identifier = text_field(binding_object(record, "native_declaration"), "id")
        if identifier in seen:
            raise ValueError(f"native {identifier}: expected one scoped binding")
        binding_equal(record.get("status"), "partial", "binding status")
        binding_equal(record.get("implemented_credit"), False, "implemented credit")
        verify_block_native(record, native)
        verify_block_rust(record, rust, source_hash)
        seen.add(identifier)
    return len(seen)


def replay_sparse_block_surface(root: Path, receipt_path: Path) -> JSONObject:
    """Authenticate the fresh receipt and both binding tables; e.g. `replay_sparse_block_surface(root, receipt)`."""
    receipt = read_json(root / receipt_path)
    verify_files(root, receipt)
    count = replay_binding_transaction(root, receipt_path)
    records, native, rust = binding_inventory_rows(root, receipt)
    block_count = verify_block_records(
        records, native, rust, lambda path: digest(root / path)
    )
    binding_equal(
        block_count,
        receipt.get("expected_block_bindings"),
        "complete bounded Block binding count",
    )
    return {
        "refreshed_base_bindings": count,
        "block_bindings": block_count,
        "implemented_credit_added": 0,
    }


def binding_inventory_rows(
    root: Path, receipt: JSONObject
) -> tuple[list[JSONObject], list[JSONObject], list[JSONObject]]:
    """Load native and Rust snapshots selected by the receipt; e.g. `binding_inventory_rows(root, receipt)`."""
    records = read_jsonl(root / text_field(receipt, "block_bindings_path"))
    native = read_jsonl(root / "porting/api-inventory/occt-declarations.jsonl.gz")
    inventory = read_json(root / text_field(receipt, "rust_inventory_path"))
    crates = object_rows(inventory.get("crates"), "Rust crates")
    rust = [
        row
        for crate in crates
        for row in object_rows(crate.get("declarations"), "Rust declarations")
    ]
    return records, native, rust


def main() -> None:
    """Use --receipt for an explicitly selected source snapshot."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", type=Path, default=RECEIPT)
    print(
        json.dumps(
            replay_sparse_block_surface(ROOT, parser.parse_args().receipt),
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
