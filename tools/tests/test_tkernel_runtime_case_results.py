"""Test exact native/Rust result serialization for legacy mappings."""

from __future__ import annotations

import unittest

from api_inventory.inputs import JSONObject
from tkernel_mapping import runtime_case_results


class RuntimeCaseResultTest(unittest.TestCase):
    def test_expanded_mapping_retains_declaration_and_both_case_ids(self) -> None:
        """Typed expansions preserve both identities; e.g. two indexed native cases."""
        mapping = mapping_row()
        native = {"Typed/0": "passed", "Typed/1": "upstream_skipped"}
        rust = {("suite_test", "typed_0"), ("suite_test", "typed_1")}
        cases = runtime_case_results([mapping], native, rust)
        self.assertEqual(len(cases), 2)
        self.assertEqual(cases[1]["native_status"], "upstream_skipped")
        self.assertEqual(cases[1]["rust_status"], "passed")
        self.assertEqual(cases[1]["upstream_id"], "TypedSuite.Case")
        self.assertEqual(cases[1]["source_sha256"], "0" * 64)

    def test_missing_rust_case_is_explicitly_unpassed(self) -> None:
        """Missing test events never become a pass; e.g. empty Rust results."""
        cases = runtime_case_results(
            [mapping_row()], {"Typed/0": "passed", "Typed/1": "passed"}, set()
        )
        self.assertTrue(all(case["rust_status"] == "missing" for case in cases))


def mapping_row() -> JSONObject:
    return {
        "toolkit": "TKA",
        "source": "src/Typed.cxx",
        "upstream_id": "TypedSuite.Case",
        "rust_target": "suite_test",
        "rust_test": "case_test",
        "source_sha256": "0" * 64,
        "runtime_cases": [
            {"upstream_case": "Typed/0", "rust_test": "typed_0"},
            {"upstream_case": "Typed/1", "rust_test": "typed_1"},
        ],
    }


if __name__ == "__main__":
    unittest.main()
