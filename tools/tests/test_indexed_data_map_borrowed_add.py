from __future__ import annotations

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from normalize_rustdoc_inventory import make_reachable_inventory

ROOT = Path(__file__).resolve().parents[2]
SELECTOR = "NCollection_IndexedDataMap::Add(const TheKeyType&,const TheItemType&)"
RUST_TARGET = "rustycad_tkernel::ncollection::IndexedDataMap::add_ref"
CPP_INVENTORY = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
RUSTDOC_CAPTURE = (
    ROOT / "porting/runtime-evidence/indexed-data-map-borrowed-add-rustdoc.json.gz"
)


def matching_cpp_declarations() -> list[dict[str, object]]:
    with gzip.open(CPP_INVENTORY, "rt", encoding="utf-8") as source:
        return [
            row
            for line in source
            if line.strip()
            for row in [json.loads(line)]
            if row.get("qualified_name") == "NCollection_IndexedDataMap::Add"
            and row.get("signature")
            == "int (const type-parameter-0-0 &, const type-parameter-0-1 &)"
        ]


class IndexedDataMapBorrowedAddMappingTests(unittest.TestCase):
    def test_legacy_selector_resolves_once_to_partial_row(self) -> None:
        declarations = matching_cpp_declarations()
        self.assertEqual(len(declarations), 1)
        self.assertEqual(
            declarations[0]["id"],
            "995e70f21b7807ced267919d9ca8d9eba57088eeb5826d7b60eac2a5301f450a",
        )
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        rows = [row for row in parity["api"] if row.get("upstream_id") == SELECTOR]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["rust_item"], RUST_TARGET)
        self.assertEqual(rows[0]["status"], "partial")

    def test_rustdoc_capture_resolves_public_path_once(self) -> None:
        with gzip.open(RUSTDOC_CAPTURE, "rt", encoding="utf-8") as source:
            document = read_json_from_text(source.read())
        inventory = make_reachable_inventory(document, ROOT, "rustycad_tkernel")
        rows = [
            row for row in inventory["declarations"] if row.get("path") == RUST_TARGET
        ]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["kind"], "function")
        self.assertEqual(
            rows[0]["source_file"],
            "crates/tkernel/src/ncollection/indexed_data_map/binding.rs",
        )
        source = rows[0]["source_file"]
        self.assertEqual(
            rows[0]["source_sha256"],
            hashlib.sha256((ROOT / source).read_bytes()).hexdigest(),
        )


def read_json_from_text(source: str) -> dict[str, object]:
    value = json.loads(source)
    if not isinstance(value, dict):
        raise ValueError("rustdoc capture must contain a JSON object")
    return value


if __name__ == "__main__":
    unittest.main()
