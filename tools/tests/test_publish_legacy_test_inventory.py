"""Test exact legacy test mapping and unverified discovery accounting."""
from __future__ import annotations

import unittest

from publish_legacy_test_inventory import report_document


class LegacyTestInventoryTest(unittest.TestCase):
    def test_exact_source_name_toolkit_matching_avoids_name_collisions(self) -> None:
        inventory = fixture_inventory([
            declaration("src/A/TKA/GTests/Shared_Test.cxx", "Shared.Case"),
            declaration("src/B/TKB/GTests/Shared_Test.cxx", "Shared.Case", "TEST_F"),
        ])
        parity = {"tests": [mapping("TKA", "src/A/TKA/GTests/Shared_Test.cxx", "Shared.Case")]}
        records, summary = report_document(inventory, parity, "inventory-hash", "parity-hash")
        gtests = by_kind(records, "gtest_declaration")
        self.assertEqual([row["mapping_status"] for row in gtests], ["mapped", "unmapped"])
        self.assertIsNone(gtests[1]["mapped_rust_test"])
        self.assertEqual(summary["by_toolkit"]["TKA"]["mapped"], 1)
        self.assertEqual(summary["by_toolkit"]["TKB"]["unmapped"], 1)

    def test_duplicate_parity_rows_are_explicit_and_not_marked_green(self) -> None:
        source = "src/A/TKA/GTests/A_Test.cxx"
        candidate = declaration(source, "A.Case")
        row = mapping("TKA", source, "A.Case")
        records, summary = report_document(
            fixture_inventory([candidate]), {"tests": [row, dict(row, rust_test="another") ]},
            "inventory-hash", "parity-hash",
        )
        gtest = by_kind(records, "gtest_declaration")[0]
        self.assertEqual(gtest["mapping_status"], "duplicate_parity_mapping")
        self.assertIsNone(gtest["mapped_rust_test"])
        self.assertEqual(len(gtest["duplicate_mapping_rows"]), 2)
        self.assertEqual(gtest["runtime_status"], "unverified")
        self.assertFalse(summary["runtime_proof"])

    def test_duplicate_declarations_unknown_mapping_and_unmapped_are_explicit(self) -> None:
        source = "src/A/TKA/GTests/A_Test.cxx"
        candidate = declaration(source, "A.Case")
        unknown = mapping("TKB", "src/B/TKB/GTests/Missing_Test.cxx", "Missing.Case")
        inventory = fixture_inventory([])
        inventory["toolkits"][0]["gtest_declaration_candidates"] = [candidate, dict(candidate)]
        records, summary = report_document(
            inventory, {"tests": [unknown]}, "inventory-hash", "parity-hash",
        )
        gtests = by_kind(records, "gtest_declaration")
        anomalies = by_kind(records, "mapping_anomaly")
        self.assertTrue(all(row["mapping_status"] == "duplicate_declaration_identity" for row in gtests))
        self.assertTrue(any(row["reason"] == "unknown_parity_mapping" for row in anomalies))
        self.assertEqual(summary["gtest"]["unknown_parity_mappings"], 1)
        self.assertEqual(summary["gtest"]["duplicate_declaration_identities"], 2)

    def test_draw_and_regression_discovery_never_claims_runtime_passes(self) -> None:
        inventory = fixture_inventory([])
        inventory["draw_case_candidates"] = ["tests/blend/case/A1"]
        inventory["upstream_test_files"] = ["tests/blend/case/A1", "tests/blend/begin"]
        records, summary = report_document(inventory, {"tests": []}, "i", "p")
        discovered = by_kind(records, "draw_case_candidate") + by_kind(records, "regression_file")
        self.assertEqual(len(by_kind(records, "draw_case_candidate")), 1)
        self.assertEqual(len(by_kind(records, "regression_file")), 2)
        self.assertTrue(all(row["discovery_status"] == "discovered_unverified" for row in discovered))
        self.assertTrue(all(row["runtime_status"] == "unverified" for row in discovered))
        self.assertEqual(summary["draw_cases"]["discovered"], 1)
        self.assertEqual(summary["regression_files"]["discovered"], 2)
        self.assertFalse(summary["runtime_proof"])

    def test_translated_parity_status_is_mapping_only_not_runtime_proof(self) -> None:
        source = "src/A/TKA/GTests/A_Test.cxx"
        row = mapping("TKA", source, "A.Case")
        row["status"] = "translated"
        records, summary = report_document(
            fixture_inventory([declaration(source, "A.Case")]), {"tests": [row]}, "i", "p"
        )
        result = by_kind(records, "gtest_declaration")[0]
        self.assertEqual(result["mapped_rust_test"]["status"], "translated")
        self.assertEqual(result["runtime_status"], "unverified")
        self.assertFalse(summary["runtime_proof"])


def fixture_inventory(candidates: list[dict]) -> dict:
    return {
        "schema_version": 1,
        "revision": "occt-revision",
        "toolkits": [{"name": "TKA", "gtest_declaration_candidates": candidates[:1]},
                     {"name": "TKB", "gtest_declaration_candidates": candidates[1:]}],
        "draw_case_candidates": [],
        "upstream_test_files": [],
    }


def declaration(source: str, name: str, macro: str = "TEST") -> dict:
    return {"source": source, "name": name, "macro": macro}


def mapping(toolkit: str, source: str, name: str) -> dict:
    return {"toolkit": toolkit, "source": source, "upstream_id": name,
            "rust_package": f"rustycad-{toolkit.lower()}", "rust_test": name.replace(".", "__"),
            "status": "translated"}


def by_kind(records: list[dict], kind: str) -> list[dict]:
    return [row for row in records if row["kind"] == kind]


if __name__ == "__main__":
    unittest.main()
