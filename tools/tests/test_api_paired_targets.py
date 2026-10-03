"""A C++ mutable-reference mapping requires every declared Rust accessor."""
import unittest
from api_inventory.inputs import JSONValue
from api_crosswalk import paired_rust_targets, row_rust_candidates, match_api_row

GETTER = "rustycad_tkmath::Tool::mode"
SETTER = "rustycad_tkmath::Tool::set_mode"
HEADER = "src/FoundationClasses/TKMath/Tool.hxx"


def fixture_crates(paths: list[str]) -> dict[str, dict[str, list[dict[str, JSONValue]]]]:
    return {"rustycad_tkmath": {
        path: [{"path": path, "source_file": "crates/tkmath/src/lib.rs",
                "source_sha256": "a" * 64}]
        for path in paths
    }}


def fixture_row() -> dict[str, JSONValue]:
    return {"upstream_id": "Tool::Mode()", "toolkit": "TKMath",
            "status": "implemented", "source": HEADER,
            "rust_item": GETTER, "rust_items": [GETTER, SETTER]}


def fixture_cpp() -> dict[str, list[dict[str, JSONValue]]]:
    return {"Tool::Mode": [{"id": "method", "usr": "c:@method",
        "qualified_name": "Tool::Mode", "kind": "CXXMethod",
        "signature": "int &()", "parameter_types": [], "written_parameter_types": [],
        "source": HEADER, "api_visible": True, "_toolkits": ["TKMath"], "sites": []}]}


class PairedRustTargetsTests(unittest.TestCase):
    def test_both_accessor_targets_are_required(self) -> None:
        crates = fixture_crates([GETTER, SETTER])
        status, rows = row_rust_candidates(fixture_row(), crates)
        self.assertEqual(status, "unique")
        self.assertEqual([row["path"] for row in rows], [GETTER, SETTER])

    def test_missing_setter_does_not_resolve_as_implemented(self) -> None:
        row = match_api_row(fixture_row(), fixture_cpp(), fixture_crates([GETTER]), {HEADER})
        self.assertEqual(row["rust_status"], "missing")
        self.assertEqual(row["resolution"], "missing_rust_target")
        self.assertEqual(row["declared_rust_paths"], [GETTER, SETTER])

    def test_pair_is_one_cpp_claim_with_two_rust_matches(self) -> None:
        row = match_api_row(fixture_row(), fixture_cpp(), fixture_crates([GETTER, SETTER]), {HEADER})
        self.assertEqual(row["resolution"], "implemented")
        self.assertEqual(row["cpp_ids"], ["method"])
        self.assertEqual(row["rust_paths"], [GETTER, SETTER])

    def test_unpinned_companion_prevents_resolution(self) -> None:
        crates = fixture_crates([GETTER, SETTER])
        crates["rustycad_tkmath"][SETTER][0]["source_sha256"] = "invalid"
        self.assertEqual(row_rust_candidates(fixture_row(), crates)[0], "unverifiable_source")

    def test_ambiguous_companion_prevents_resolution(self) -> None:
        crates = fixture_crates([GETTER, SETTER])
        crates["rustycad_tkmath"][SETTER] *= 2
        self.assertEqual(row_rust_candidates(fixture_row(), crates)[0], "ambiguous")

    def test_invalid_lists_fail_closed(self) -> None:
        for targets in [[], [GETTER, GETTER], [GETTER, None], [GETTER, ""], "not-list"]:
            with self.subTest(targets=targets):
                row = fixture_row()
                row["rust_items"] = targets
                self.assertIsNone(paired_rust_targets(row))
                self.assertEqual(row_rust_candidates(row, {})[0], "invalid_rust_targets")

    def test_conflicting_primary_is_rejected(self) -> None:
        for primary in [SETTER, None, 0, ""]:
            with self.subTest(primary=primary):
                row = fixture_row()
                row["rust_item"] = primary
                self.assertIsNone(paired_rust_targets(row))

    def test_list_only_and_single_target_rows_remain_supported(self) -> None:
        row = fixture_row()
        del row["rust_item"]
        self.assertEqual(paired_rust_targets(row), [GETTER, SETTER])
        row = {"rust_item": GETTER}
        self.assertEqual(row_rust_candidates(row, fixture_crates([GETTER]))[0], "unique")


if __name__ == "__main__":
    unittest.main()
