from __future__ import annotations

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from normalize_rustdoc_inventory import make_reachable_inventory

ROOT = Path(__file__).resolve().parents[2]
SELECTOR = "NCollection_IndexedMap::Add(const TheKeyType&)"
RUST_TARGET = "rustycad_tkernel::ncollection::IndexedMap::add_ref"
CPP_INVENTORY = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
RUSTDOC_CAPTURE = (
    ROOT / "porting/runtime-evidence/indexed-map-borrowed-add-rustdoc.json.gz"
)


def cpp_candidates() -> list[dict[str, object]]:
    with gzip.open(CPP_INVENTORY, "rt", encoding="utf-8") as source:
        return [
            row
            for line in source
            if line.strip()
            for row in [json.loads(line)]
            if row.get("qualified_name") == "NCollection_IndexedMap::Add"
            and row.get("signature") == "int (const type-parameter-0-0 &)"
        ]


def rustdoc_inventory() -> dict[str, object]:
    with gzip.open(RUSTDOC_CAPTURE, "rt", encoding="utf-8") as source:
        document = json.load(source)
    return make_reachable_inventory(document, ROOT, "rustycad_tkernel")


class IndexedMapBorrowedAddMappingTests(unittest.TestCase):
    def test_native_selector_resolves_to_partial_row(self) -> None:
        declarations = cpp_candidates()
        self.assertEqual(len(declarations), 1)
        self.assertEqual(
            declarations[0]["id"],
            "3139d3af57aa3ab1c74eac365b9dadf2a17aafb33f22513aa3682d8df3d4cc4a",
        )
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        rows = [row for row in parity["api"] if row.get("upstream_id") == SELECTOR]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["rust_item"], RUST_TARGET)
        self.assertEqual(rows[0]["status"], "partial")

    def test_rustdoc_resolves_public_method_once(self) -> None:
        declarations = rustdoc_inventory()["declarations"]
        rows = [row for row in declarations if row.get("path") == RUST_TARGET]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["kind"], "function")
        source = rows[0]["source_file"]
        self.assertEqual(source, "crates/tkernel/src/ncollection/indexed_map.rs")
        self.assertEqual(
            rows[0]["source_sha256"],
            hashlib.sha256((ROOT / source).read_bytes()).hexdigest(),
        )


if __name__ == "__main__":
    unittest.main()
