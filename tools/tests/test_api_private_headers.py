"""Private-header API rows are removed from the ledger and count for nothing in reports."""

import unittest

from api_inventory_conflicts import (
    all_default_arguments,
    expand_default_arguments,
    material_conflicts,
)
from api_private_headers import (
    inventory_public_sources,
    is_private_header_row,
    public_header_paths,
    split_private_header_rows,
)
from remove_private_header_api_rows import removal_report, removed_rows

PUBLIC_LXX = "src/FoundationClasses/TKernel/NCollection/NCollection_UtfString.lxx"
PRIVATE = "src/ModelingData/TKG3d/GeomHash/GeomHash_PointHasher.pxx"
HEADER = "src/ModelingData/TKG3d/Geom/Geom_Line.hxx"


def row(source: str, upstream_id: str = "X") -> dict:
    return {"upstream_id": upstream_id, "toolkit": "TKG3d", "source": source, "status": "implemented"}


class PrivateHeaderTest(unittest.TestCase):
    def test_only_unlisted_private_suffixes_are_private(self) -> None:
        public = {PUBLIC_LXX, HEADER}
        self.assertTrue(is_private_header_row(row(PRIVATE), public))
        self.assertFalse(is_private_header_row(row(PUBLIC_LXX), public))
        self.assertFalse(is_private_header_row(row("src/A/Unlisted.hxx"), public))
        self.assertFalse(is_private_header_row({"source": None}, public))

    def test_split_keeps_order(self) -> None:
        rows = [row(HEADER, "a"), row(PRIVATE, "b"), row(HEADER, "c")]
        public, private = split_private_header_rows(rows, {HEADER})
        self.assertEqual([item["upstream_id"] for item in public], ["a", "c"])
        self.assertEqual([item["upstream_id"] for item in private], ["b"])

    def test_manifest_without_headers_is_an_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected a list"):
            public_header_paths({"headers": None})


class RemovalScriptTest(unittest.TestCase):
    def test_private_fragment_with_observed_declarations_is_kept(self) -> None:
        gxx = "src/ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_Approx.gxx"
        parity = {"api": [row(gxx, "ApproxInt_Approx::Perform")]}
        observed = [{"source": gxx, "api_visible": True}, {"source": PRIVATE, "api_visible": False}]
        self.assertEqual(removed_rows(parity, {"headers": []}, observed), [])
        self.assertEqual(inventory_public_sources(observed), {gxx})

    def test_removes_private_rows_in_place_and_reports_them(self) -> None:
        parity = {"api": [row(HEADER, "keep"), row(PRIVATE, "GeomHash_PointHasher::operator()")]}
        removed = removed_rows(parity, {"headers": [{"path": HEADER}]}, [])
        self.assertEqual([item["upstream_id"] for item in parity["api"]], ["keep"])
        self.assertEqual(
            removal_report(removed),
            [f"TKG3d\timplemented\t{PRIVATE}\tGeomHash_PointHasher::operator()"],
        )

    def test_malformed_ledger_is_an_error(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected a list of objects"):
            removed_rows({"api": [1]}, {"headers": []}, [])


class SpellingOnlyConflictTest(unittest.TestCase):
    def setUp(self) -> None:
        self.template = {"kind": "ClassTemplate", "qualified_name": "math_VectorBase",
                         "template_signature": "template <typename __template_param_0 = double> "}

    def conflict(self, identity: str, first: str, other: str, field: str = "signature") -> dict:
        return {"id": identity, "field": field, "first": first, "other": other}

    def test_defaults_are_read_only_when_every_parameter_has_one(self) -> None:
        self.assertEqual(all_default_arguments("template <typename A = int, int N = 2> "), ["int", "2"])
        self.assertIsNone(all_default_arguments("template <typename A, int N = 2> "))

    def test_empty_argument_list_expands_to_defaults(self) -> None:
        text = expand_default_arguments("class math_VectorBase<>", {"math_VectorBase": "double"})
        self.assertEqual(text, "class math_VectorBase<double>")

    def test_default_spelling_conflict_is_not_material(self) -> None:
        benign = self.conflict("vec", "class math_VectorBase<double>", "class math_VectorBase<>")
        self.assertEqual(material_conflicts([benign], [self.template]), [])

    def test_other_conflicts_keep_the_whole_identity_withheld(self) -> None:
        benign = self.conflict("vec", "class math_VectorBase<double>", "class math_VectorBase<>")
        real = self.conflict("vec", "access_marker", "local_scope", "exclusion_reason")
        changed = self.conflict("other", "class math_VectorBase<float>", "class math_VectorBase<>")
        kept = material_conflicts([benign, real, changed], [self.template])
        self.assertEqual(kept, [benign, real, changed])


if __name__ == "__main__":
    unittest.main()
