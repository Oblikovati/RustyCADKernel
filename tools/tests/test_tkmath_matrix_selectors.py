"""Keep matrix arithmetic selectors bound to unique compiler declarations."""

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes


ROOT = Path(__file__).resolve().parents[2]
MAP_DECLARATIONS = (
    (
        "NCollection_Mat3::Map(Element_t*)",
        "5e585163596ae7b4ae99893e33d04cc3f5a3924208df7dcaf46888b0aa143281",
    ),
    (
        "NCollection_Mat3::Map(const Element_t*)",
        "d08d3990fe85e9a8ca758ffd72072e2c3f830e605288f88e703da5c8dbc74fd2",
    ),
    (
        "NCollection_Mat4::Map(Element_t*)",
        "3460dbd18af0689c543a4636660cddbde649ca7f0a73c48b700637b0af1079b5",
    ),
    (
        "NCollection_Mat4::Map(const Element_t*)",
        "03389096a229d7a2e0b9f394b0a8fefbc41c7e2d23e37849aed35c4339cb8092",
    ),
)


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

    def test_map_overloads_match_exact_compiler_declarations(self) -> None:
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        parity_rows = {row["upstream_id"]: row for row in parity["api"]}
        for selector, compiler_id in MAP_DECLARATIONS:
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [compiler_id])
            self.assertEqual(parity_rows[selector]["status"], "partial")
