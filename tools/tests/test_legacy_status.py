"""Regression coverage for declaration counts shown by the porting status CLI."""

from __future__ import annotations

import unittest

from api_inventory.inputs import JSONObject
from legacy_status import legacy_status_lines


def status_inventory(*names: str) -> JSONObject:
    """Use the same names in two toolkits to detect cross-toolkit matching."""
    return {
        "toolkits": [
            {
                "name": toolkit,
                "gtest_declaration_candidates": [
                    {"source": "src/Shared_Test.cxx", "name": name, "macro": "TEST"}
                    for name in names
                ],
            }
            for toolkit in ("TKA", "TKB")
        ]
    }


def status_mapping(name: str, status: str = "translated") -> JSONObject:
    """Provide a declaration mapping with explicit pending/C++-only evidence."""
    return {
        "toolkit": "TKA",
        "source": "src/Shared_Test.cxx",
        "upstream_id": name,
        "status": status,
        "blocked_by": ["UnportedClass"],
        "reason": "C++ type-trait assertion has no Rust equivalent",
    }


class LegacyStatusTest(unittest.TestCase):
    def test_pending_and_cpp_only_rows_never_inflate_mapped_counts(self) -> None:
        ledger: JSONObject = {
            "tests": [
                status_mapping("Suite.Passed"),
                status_mapping("Suite.Pending", "pending_dependency"),
                status_mapping("Suite.Cpp", "not_applicable_to_rust"),
            ]
        }
        lines = legacy_status_lines(
            status_inventory("Suite.Passed", "Suite.Pending", "Suite.Cpp"), ledger
        )
        self.assertEqual(
            lines[0],
            "GoogleTest total: 1/6 GoogleTest declarations mapped; "
            "3 unmapped; 1 pending dependencies; 1 not applicable to Rust; 0 mapping anomalies",
        )
        self.assertEqual(
            lines[1],
            "TKA: 1/3 GoogleTest declarations mapped; "
            "0 unmapped; 1 pending dependencies; 1 not applicable to Rust; 0 mapping anomalies",
        )
        self.assertEqual(
            lines[2],
            "TKB: 0/3 GoogleTest declarations mapped; "
            "3 unmapped; 0 pending dependencies; 0 not applicable to Rust; 0 mapping anomalies",
        )

    def test_duplicate_mappings_are_visible_and_receive_no_mapping_credit(self) -> None:
        row = status_mapping("Suite.Case")
        lines = legacy_status_lines(
            status_inventory("Suite.Case"), {"tests": [row, row]}
        )
        self.assertIn("0/2 GoogleTest declarations mapped; 1 unmapped", lines[0])
        self.assertIn("1 mapping anomalies", lines[0])
        self.assertIn("TKA: 0/1 GoogleTest declarations mapped", lines[1])
        self.assertIn("1 mapping anomalies", lines[1])

    def test_duplicate_declarations_are_visible_and_receive_no_mapping_credit(
        self,
    ) -> None:
        lines = legacy_status_lines(
            status_inventory("Suite.Case", "Suite.Case"),
            {"tests": [status_mapping("Suite.Case")]},
        )
        self.assertIn("0/4 GoogleTest declarations mapped", lines[0])
        self.assertIn("4 mapping anomalies", lines[0])
        self.assertIn("2 mapping anomalies", lines[1])
        self.assertIn("2 mapping anomalies", lines[2])

    def test_unknown_mapping_is_visible_even_for_an_unknown_toolkit(self) -> None:
        row = status_mapping("Missing.Case")
        for toolkit in ("TKA", "UnknownToolkit"):
            with self.subTest(toolkit=toolkit):
                row["toolkit"] = toolkit
                lines = legacy_status_lines(status_inventory(), {"tests": [row]})
                self.assertIn("0/0 GoogleTest declarations mapped", lines[0])
                self.assertIn("1 mapping anomalies", lines[0])
                self.assertIn("completion remain unproven", lines[-1])

    def test_matching_name_in_wrong_source_is_unmapped_with_an_anomaly(self) -> None:
        row = status_mapping("Suite.Case")
        row["source"] = "src/Wrong_Test.cxx"
        lines = legacy_status_lines(status_inventory("Suite.Case"), {"tests": [row]})
        self.assertIn("0/2 GoogleTest declarations mapped; 2 unmapped", lines[0])
        self.assertIn("1 mapping anomalies", lines[1])

    def test_all_mapped_declarations_do_not_establish_runtime_or_full_completion(
        self,
    ) -> None:
        first = status_mapping("Suite.Case")
        second = {**first, "toolkit": "TKB"}
        lines = legacy_status_lines(
            status_inventory("Suite.Case"), {"tests": [first, second]}
        )
        self.assertIn("2/2 GoogleTest declarations mapped; 0 unmapped", lines[0])
        self.assertIn(
            "runtime parity and full API/DRAW/configuration completion remain unproven",
            lines[-1],
        )


if __name__ == "__main__":
    unittest.main()
