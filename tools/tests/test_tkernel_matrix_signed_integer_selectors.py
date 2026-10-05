"""Bind Mat3/Mat4 integer methods to unique pinned OCCT declarations."""

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes

ROOT = Path(__file__).resolve().parents[2]


class SignedIntegerMatrixSelectorTest(unittest.TestCase):
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
        audit_path = (
            ROOT / "porting/audits/tkernel-matrix-signed-integer-selectors.json"
        )
        cls.audit = json.loads(audit_path.read_text())
        cls.rows = {
            row["upstream_id"]: row
            for row in json.loads((ROOT / "porting/parity.json").read_text())["api"]
        }

    def test_inventory_and_header_inputs_match_audit(self) -> None:
        inventory = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
        self.assertEqual(
            hashlib.sha256(inventory.read_bytes()).hexdigest(),
            self.audit["inputs"]["declaration_inventory_sha256"],
        )
        manifest = ROOT / "porting/api-header-manifest.json"
        self.assertEqual(
            hashlib.sha256(manifest.read_bytes()).hexdigest(),
            self.audit["inputs"]["header_manifest_sha256"],
        )

    def test_each_matrix_method_has_one_exact_compiler_declaration(self) -> None:
        declarations = self.audit["declarations"]
        self.assertEqual(len(declarations), 9)
        for expected in declarations:
            self.assert_selector(expected)

    def test_mat3_output_selectors_require_the_template_argument(self) -> None:
        for selector in self.audit["previously_unresolved"]:
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual((status, candidates), ("missing", []), selector)

    def assert_selector(self, expected: dict[str, str]) -> None:
        selector = expected["selector"]
        status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
        self.assertEqual(status, "unique", selector)
        self.assertEqual([item["id"] for item in candidates], [expected["compiler_id"]])
        self.assertEqual(candidates[0]["signature"], expected["compiler_signature"])
        self.assertEqual(candidates[0]["source"], expected["source"])
        self.assert_ledger_binding(selector, expected)

    def assert_ledger_binding(self, selector: str, expected: dict[str, str]) -> None:
        row = self.rows[selector]
        self.assertEqual(row["selector_id"], expected["compiler_id"])
        self.assertEqual(row["source"], expected["source"])
        self.assertEqual(row["source_sha256"], expected["source_sha256"])
