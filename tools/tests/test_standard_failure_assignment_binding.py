"""Keep base exception assignment bound to its native declaration."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]
SELECTOR = "Standard_Failure::operator=(const Standard_Failure&)"
COMPILER_ID = "b94cc1f21b3fd499940427c345ffce659a5a091901b8a6c02d53c6508582151a"


class FailureAssignmentBindingTest(unittest.TestCase):
    def setUp(self) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        self.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))

    def test_base_assignment_uses_explicit_rust_operation(self) -> None:
        status, candidates = cpp_candidates(SELECTOR, "TKernel", self.by_name)
        self.assertEqual(status, "unique")
        self.assertEqual([row["id"] for row in candidates], [COMPILER_ID])
        rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        matches = [row for row in rows if row["upstream_id"] == SELECTOR]
        self.assertEqual(len(matches), 1)
        self.assertEqual(
            matches[0]["rust_item"], "rustycad_tkernel::standard::Failure::assign_base"
        )
        self.assertEqual(matches[0]["status"], "partial")

    def test_base_copy_slices_derived_identity(self) -> None:
        selector = "Standard_Failure(const Standard_Failure&)"
        compiler_id = "c72444b328de71bc61f432cb717bba784d9c25ac7ea8ee0965162d3a0f96c909"
        status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
        self.assertEqual(status, "unique")
        self.assertEqual([row["id"] for row in candidates], [compiler_id])
        rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        matches = [row for row in rows if row["upstream_id"] == selector]
        self.assertEqual(len(matches), 1)
        self.assertEqual(
            matches[0]["rust_item"], "rustycad_tkernel::standard::Failure::copy_base"
        )
        self.assertEqual(matches[0]["status"], "partial")
