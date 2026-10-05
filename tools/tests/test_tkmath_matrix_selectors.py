"""Keep matrix arithmetic selectors bound to unique compiler declarations."""

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes


ROOT = Path(__file__).resolve().parents[2]


class MatrixSelectorInventoryTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        inventory = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
        with gzip.open(inventory, "rt") as stream:
            cls.declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.owners = {
            header["path"]: {owner["toolkit"] for owner in header.get("owners", [])}
            for header in manifest["headers"]
        }
        cls.by_name, _, _ = cpp_indexes(cls.declarations, cls.owners)
        receipt = ROOT / "porting/audits/tkmath-matrix-selector-repair.json"
        cls.receipt = json.loads(receipt.read_text())
        cls.expected = cls.receipt["declarations"]
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.parity_selectors = {row["upstream_id"] for row in parity["api"]}

    def test_all_matrix_arithmetic_selectors_are_unique(self) -> None:
        inventory = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
        expected_hash = self.receipt["inputs"]["declaration_inventory_sha256"]
        self.assertEqual(hashlib.sha256(inventory.read_bytes()).hexdigest(), expected_hash)
        for expected in self.expected:
            selector = expected["new_selector"]
            self.assertIn(selector, self.parity_selectors)
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", expected["new_selector"])
            self.assertEqual([row["id"] for row in candidates], [expected["compiler_id"]])
