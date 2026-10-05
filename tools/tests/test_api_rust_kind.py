"""Declaration-kind selectors must resolve collisions without hiding missing evidence."""

import unittest
from api_crosswalk import row_rust_candidates
from api_inventory.inputs import JSONValue
from api_rust_kind import RustCrateIndex, select_rust_kind

VECTOR_X = "rustycad_tkernel::ncollection::Vec2::x"


def vector_kind_fixture() -> RustCrateIndex:
    entries = [
        {
            "path": VECTOR_X,
            "kind": kind,
            "source_file": "vector.rs",
            "source_sha256": "a" * 64,
        }
        for kind in ["function", "struct_field"]
    ]
    return {"rustycad_tkernel": {VECTOR_X: entries}}


class RustKindSelectorTests(unittest.TestCase):
    def test_absent_selector_preserves_ambiguity(self) -> None:
        status, entries = row_rust_candidates(
            {"rust_item": VECTOR_X}, vector_kind_fixture()
        )
        self.assertEqual((status, len(entries)), ("ambiguous", 2))

    def test_explicit_kind_selects_only_requested_declaration(self) -> None:
        for kind in ["function", "struct_field"]:
            row = {"rust_item": VECTOR_X, "rust_kind": kind}
            status, entries = row_rust_candidates(row, vector_kind_fixture())
            self.assertEqual(status, "unique")
            self.assertEqual([entry["kind"] for entry in entries], [kind])

    def test_invalid_kind_is_rejected(self) -> None:
        for kind in [None, "", "invented", 4, [], {}]:
            row: dict[str, JSONValue] = {"rust_item": VECTOR_X, "rust_kind": kind}
            self.assertIsNone(select_rust_kind(vector_kind_fixture(), kind))
            self.assertEqual(
                row_rust_candidates(row, vector_kind_fixture()),
                ("invalid_rust_kind", []),
            )

    def test_missing_kind_does_not_fall_back_to_other_kind(self) -> None:
        crates = vector_kind_fixture()
        crates["rustycad_tkernel"][VECTOR_X].pop()
        row = {"rust_item": VECTOR_X, "rust_kind": "struct_field"}
        self.assertEqual(row_rust_candidates(row, crates), ("missing", []))

    def test_duplicate_same_kind_remains_ambiguous(self) -> None:
        crates = vector_kind_fixture()
        crates["rustycad_tkernel"][VECTOR_X] *= 2
        row = {"rust_item": VECTOR_X, "rust_kind": "function"}
        self.assertEqual(row_rust_candidates(row, crates)[0], "ambiguous")

    def test_selected_source_must_still_be_pinned(self) -> None:
        crates = vector_kind_fixture()
        crates["rustycad_tkernel"][VECTOR_X][0]["source_sha256"] = "invalid"
        row = {"rust_item": VECTOR_X, "rust_kind": "function"}
        self.assertEqual(row_rust_candidates(row, crates)[0], "unverifiable_source")

    def test_paired_targets_all_obey_kind(self) -> None:
        row = {"rust_items": [VECTOR_X, VECTOR_X + "_missing"], "rust_kind": "function"}
        self.assertEqual(row_rust_candidates(row, vector_kind_fixture())[0], "missing")

    def test_filter_does_not_modify_compiler_inventory(self) -> None:
        crates = vector_kind_fixture()
        select_rust_kind(crates, "function")
        self.assertEqual(len(crates["rustycad_tkernel"][VECTOR_X]), 2)
