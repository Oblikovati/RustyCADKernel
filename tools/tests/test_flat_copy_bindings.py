"""Keep flat copy contracts attached to exact pinned native declarations."""

from __future__ import annotations

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners
from api_inventory.inputs import JSONValue

ROOT = Path(__file__).resolve().parents[2]


class FlatCopyBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["FlatCopyBindingsTest"]) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.rows = {row["upstream_id"]: row for row in parity["api"]}
        cls.audit = json.loads(
            (ROOT / "porting/audits/flat-copy-contracts.json").read_text()
        )
        cls.hash_audit = json.loads(
            (ROOT / "porting/audits/flat-hash-contracts.json").read_text()
        )

        cls.borrow_audit = json.loads(
            (ROOT / "porting/audits/flat-borrowed-insertion.json").read_text()
        )

    def test_four_copy_selectors_are_unique_and_keep_generic_limits(self) -> None:
        self.assert_native_binding_contracts(self.audit["bindings"], 4)

    def test_twenty_eight_hash_selectors_keep_generic_limits(self) -> None:
        self.assert_native_binding_contracts(self.hash_audit["bindings"], 28)

    def test_all_eight_variadic_insertion_selectors_preserve_the_pack(self) -> None:
        packs = [row for row in self.hash_audit["bindings"] if "..." in row["selector"]]
        self.assertEqual(len(packs), 8)
        for binding in packs:
            self.assertTrue(binding["selector"].endswith("Args &&...)"))

    def test_fourteen_borrowed_selectors_keep_generic_limits(self) -> None:
        self.assert_native_binding_contracts(self.borrow_audit["bindings"], 14)

    def test_borrowed_retargeting_preserves_historical_owned_bindings(self) -> None:
        targets = {row["rust_item"] for row in self.borrow_audit["bindings"]}
        retargets = [
            row for row in self.hash_audit["bindings"] if "original_rust_item" in row
        ]
        self.assertEqual(len(targets), 14)
        self.assertEqual({row["rust_item"] for row in retargets}, targets)
        for row in retargets:
            self.assertNotEqual(row["rust_item"], row["original_rust_item"])
            self.assertEqual(
                row["binding_retarget_audit"],
                "porting/audits/flat-borrowed-insertion.json",
            )

    def assert_native_binding_contracts(
        self, bindings: list[dict[str, JSONValue]], count: int
    ) -> None:
        self.assertEqual(len(bindings), count)
        for binding in bindings:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            key = "rust_items" if "rust_items" in binding else "rust_item"
            self.assertEqual(self.rows[selector][key], binding[key])
            self.assertEqual(self.rows[selector]["status"], "partial")

    def test_both_assignment_bindings_cover_return_and_self_entry_points(self) -> None:
        assignments = [
            binding
            for binding in self.audit["bindings"]
            if "::operator=" in binding["selector"]
        ]
        self.assertEqual(len(assignments), 2)
        for binding in assignments:
            self.assertTrue(binding["rust_items"][0].endswith("::assign_returning"))
            self.assertTrue(binding["rust_items"][1].endswith("::assign_self"))
            self.assertTrue(binding["rust_items"][2].endswith("::clone_from"))


if __name__ == "__main__":
    unittest.main()
