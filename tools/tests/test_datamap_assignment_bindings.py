"""Pin the exact DataMap assignment declarations and compiled Rust adapters."""

from __future__ import annotations

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners
from normalize_rustdoc_inventory import make_reachable_inventory

ROOT = Path(__file__).resolve().parents[2]
COPY_TYPE = "const NCollection_DataMap<TheKeyType, TheItemType, Hasher> &"
MOVE_TYPE = "NCollection_DataMap<TheKeyType, TheItemType, Hasher> &&"
PREFIX = "rustycad_tkernel::ncollection::DataMap::"
ASSIGNMENT_BINDINGS = (
    (
        f"NCollection_DataMap::Assign({COPY_TYPE})",
        "a256da3d2787a69761138522360112c85ee87b1a2c1f241d1bf41bcce940c46e",
        "assign_returning",
    ),
    (
        f"NCollection_DataMap::operator=({COPY_TYPE})",
        "3841024661195064f7dac70b8cc31404f25303889652c11701bdf3eb6cfd8cc7",
        "assign_returning",
    ),
    (
        f"NCollection_DataMap::operator=({MOVE_TYPE})",
        "f64f3b01f3906c156b6829a7518551ae41c774c73af0b98ef15ae56672b542b0",
        "move_from_returning",
    ),
)


class DataMapAssignmentBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["DataMapAssignmentBindingsTest"]) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as source:
            declarations = [json.loads(line) for line in source]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.by_selector = {row["upstream_id"]: row for row in parity["api"]}

    def test_assignment_selectors_bind_unique_declarations_with_explicit_alias_adapter(
        self,
    ) -> None:
        for selector, compiler_id, method in ASSIGNMENT_BINDINGS:
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [compiler_id])
            row = self.by_selector[selector]
            self.assertEqual(row["rust_item"], PREFIX + method)
            self.assertEqual(
                row["rust_items"], [PREFIX + method, PREFIX + "assign_self"]
            )
            self.assertEqual(row["status"], "partial")

    def test_compiled_assignment_adapters_are_public_and_source_bound(self) -> None:
        capture = ROOT / "porting/runtime-evidence/datamap-assignment-rustdoc.json.gz"
        with gzip.open(capture, "rt") as stream:
            inventory = make_reachable_inventory(
                json.load(stream), ROOT, "rustycad_tkernel"
            )
        source = "crates/tkernel/src/ncollection/data_map/assignment.rs"
        expected = hashlib.sha256((ROOT / source).read_bytes()).hexdigest()
        for method in ("assign_returning", "assign_self", "move_from_returning"):
            rows = [
                row
                for row in inventory["declarations"]
                if row["path"] == PREFIX + method
            ]
            self.assertEqual(len(rows), 1, method)
            self.assertEqual(rows[0]["kind"], "function")
            self.assertEqual(rows[0]["source_file"], source)
            self.assertEqual(rows[0]["source_sha256"], expected)


if __name__ == "__main__":
    unittest.main()
