"""Bind borrowed Array1/Array2 SetValue to exact compiler declarations."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]
ROWS = (
    (
        "NCollection_Array1::SetValue(int,const NCollection_Array1::value_type&)",
        "26a0fb19212cab88e4a6ec79a5d954a4ebeea29921ac14ef4a17f7fd9c01b30c",
        "NCollection_Array1::SetValue(int,const value_type&)",
        "rustycad_tkernel::ncollection::Array1::set_value_ref",
    ),
    (
        "NCollection_Array2::SetValue(int,int,const TheItemType&)",
        "d6b4a993b415266d4a853fc447e23f9b6b0f016c897d5d0e861c33695a227677",
        "NCollection_Array2::SetValue(int,int,const TheItemType&)",
        "rustycad_tkernel::ncollection::Array2::set_value_ref",
    ),
)


class ArrayBorrowedSetValueBindingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        with gzip.open(ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt") as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.parity_rows = {row["upstream_id"]: row for row in parity["api"]}

    def test_borrowed_setvalue_rows_match_unique_compiler_ids(self) -> None:
        for selector, compiler_id, parity_id, rust_item in ROWS:
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [compiler_id])
            self.assertEqual(self.parity_rows[parity_id]["rust_item"], rust_item)
