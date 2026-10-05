"""Keep flat copy contracts attached to exact pinned native declarations."""

from __future__ import annotations

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

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

    def test_four_copy_selectors_are_unique_and_keep_generic_limits(self) -> None:
        self.assertEqual(len(self.audit["bindings"]), 4)
        for binding in self.audit["bindings"]:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            self.assertEqual(self.rows[selector]["rust_items"], binding["rust_items"])
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
