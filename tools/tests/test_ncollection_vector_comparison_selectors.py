"""Bind vector comparison selectors to exact declarations."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes

ROOT = Path(__file__).resolve().parents[2]
ROWS = (
    (
        "NCollection_Vec2::IsEqual(const NCollection_Vec2<Element_t> &) const noexcept",
        "d2be3fe5cbb3e9ca62d20769951a860b3f5f65e111bdc5a7cb12bc90ca7ecdff",
        "rustycad_tkernel::ncollection::Vec2::is_equal",
    ),
    (
        "NCollection_Vec2::operator==(const NCollection_Vec2<Element_t> &) const noexcept",
        "9b94bb4debe68a6b05e55b940fb5859eca1ae4cc74932633aa166cb68afbd0e8",
        "rustycad_tkernel::ncollection::Vec2::<core::cmp::PartialEq>::eq",
    ),
    (
        "NCollection_Vec2::operator!=(const NCollection_Vec2<Element_t> &) const noexcept",
        "87ce66c57d5b0fc146ba0b64b2f4bc38f6fdacd26872e67c7a838d52be4ca5b5",
        "rustycad_tkernel::ncollection::Vec2::<core::cmp::PartialEq>::ne",
    ),
    (
        "NCollection_Vec3::IsEqual(const NCollection_Vec3<Element_t> &) const noexcept",
        "606b858c5d4a00f85b4f90fb30679e222c8dd23e84aae7f6e8c2568808c27c15",
        "rustycad_tkernel::ncollection::Vec3::is_equal",
    ),
    (
        "NCollection_Vec3::operator==(const NCollection_Vec3<Element_t> &) const noexcept",
        "cf14b7e8a3296ceb34fdc54747f1304a2da76c687334d62594a92d4d9c23cc67",
        "rustycad_tkernel::ncollection::Vec3::<core::cmp::PartialEq>::eq",
    ),
    (
        "NCollection_Vec3::operator!=(const NCollection_Vec3<Element_t> &) const noexcept",
        "bd74fcd108abf8219a921261447dd875872c77784e24044c524aff53c5470f04",
        "rustycad_tkernel::ncollection::Vec3::<core::cmp::PartialEq>::ne",
    ),
    (
        "NCollection_Vec4::IsEqual(const NCollection_Vec4<Element_t> &) const noexcept",
        "2c709053114afdf66d6b364d93224c6277bf5e312e41413d815579620a05f5a1",
        "rustycad_tkernel::ncollection::Vec4::is_equal",
    ),
    (
        "NCollection_Vec4::operator==(const NCollection_Vec4<Element_t> &) const noexcept",
        "afe8c8ba2b404fd56a82e2fe60133a35910e38a6a3e1f8f0aca958f7af478c99",
        "rustycad_tkernel::ncollection::Vec4::<core::cmp::PartialEq>::eq",
    ),
    (
        "NCollection_Vec4::operator!=(const NCollection_Vec4<Element_t> &) const noexcept",
        "447655af81f0293242af81e0788fad582d8a578f5b7f5fd11ef8c1c7df1b7b88",
        "rustycad_tkernel::ncollection::Vec4::<core::cmp::PartialEq>::ne",
    ),
)


class VectorComparisonSelectorTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        declarations_path = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
        with gzip.open(declarations_path, "rt") as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        owners = {
            header["path"]: {owner["toolkit"] for owner in header.get("owners", [])}
            for header in manifest["headers"]
        }
        cls.by_name, _, _ = cpp_indexes(declarations, owners)
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.parity_rows = {row["upstream_id"]: row for row in parity["api"]}

    def test_comparison_selectors_are_exact(self) -> None:
        for selector, compiler_id, rust_item in ROWS:
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [compiler_id])
            self.assertEqual(self.parity_rows[selector]["rust_item"], rust_item)
