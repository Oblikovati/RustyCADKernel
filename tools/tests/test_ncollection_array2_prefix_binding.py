"""Check Array2 prefix and Init declarations against the pinned compiler inventory."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]
ROWS = (
    (
        "NCollection_Array2(NCollection_Array2::pointer,const size_t,const size_t)",
        "702822f81a31e600b7883b9bbc5679ad680a647e13b73388e547b56df0b4d8fd",
        "NCollection_Array2(pointer,size_t,size_t)",
        "rustycad_tkernel::ncollection::Array2::from_buffer",
    ),
    (
        "NCollection_Array2::Init(NCollection_Array2::const_reference)",
        "aa6064b6be6ecf62190be44bb2e418f1912f1c57bfd9cf64be05d1bb7e7314a3",
        "NCollection_Array2::Init(const_reference)",
        "rustycad_tkernel::ncollection::Array2::init",
    ),
)


class Array2PrefixBindingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.parity_rows = {row["upstream_id"]: row for row in parity["api"]}

    def test_array2_prefix_and_fill_selectors_are_unique(self) -> None:
        for selector, compiler_id, parity_id, rust_item in ROWS:
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [compiler_id])
            self.assertEqual(self.parity_rows[parity_id]["rust_item"], rust_item)
