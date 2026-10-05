"""Pin Map's borrowed insertion overloads and fresh compiled adapter paths."""

from __future__ import annotations

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners
from normalize_rustdoc_inventory import make_reachable_inventory

ROOT = Path(__file__).resolve().parents[2]
PREFIX = "rustycad_tkernel::ncollection::Map::"


class MapBorrowedInsertionBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["MapBorrowedInsertionBindingsTest"]) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as source:
            declarations = [json.loads(line) for line in source]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.rows = {row["upstream_id"]: row for row in parity["api"]}
        audit = json.loads(
            (ROOT / "porting/audits/ncollection-map-borrowed-add.json").read_text()
        )
        cls.bindings = audit["bindings"]

    def test_borrowed_overloads_resolve_uniquely_with_partial_limits(self) -> None:
        self.assertEqual(len(self.bindings), 2)
        for binding in self.bindings:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            self.assertEqual(self.rows[selector]["rust_item"], binding["rust_item"])
            self.assertEqual(self.rows[selector]["status"], "partial")

    def test_compiled_adapters_are_public_and_match_the_current_source(self) -> None:
        capture = ROOT / "porting/runtime-evidence/map-borrowed-add-rustdoc.json.gz"
        with gzip.open(capture, "rt") as source:
            inventory = make_reachable_inventory(
                json.load(source), ROOT, "rustycad_tkernel"
            )
        filename = "crates/tkernel/src/ncollection/map/borrowed.rs"
        expected = hashlib.sha256((ROOT / filename).read_bytes()).hexdigest()
        for method in ("add_ref", "added_ref"):
            rows = [
                row
                for row in inventory["declarations"]
                if row["path"] == PREFIX + method
            ]
            self.assertEqual(len(rows), 1, method)
            self.assertEqual(rows[0]["kind"], "function")
            self.assertEqual(rows[0]["source_file"], filename)
            self.assertEqual(rows[0]["source_sha256"], expected)


if __name__ == "__main__":
    unittest.main()
