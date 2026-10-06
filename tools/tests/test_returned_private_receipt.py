"""Replay bounded native evidence and reject identity/hash/visibility corruption."""

from __future__ import annotations

import copy
import pathlib
import unittest

from api_inventory.inputs import read_object
from verify_returned_private_reachability import (
    DEFAULT_RECEIPT,
    ROOT,
    read_capture,
    replay,
    verify_capture_pair,
    verify_files,
)


class ReturnedReceiptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.receipt = read_object(ROOT / DEFAULT_RECEIPT)
        source = pathlib.Path(cls.receipt["capture_source_root"])
        cls.baseline = read_capture(
            ROOT / cls.receipt["baseline_capture"], source, cls.receipt["header"]
        )
        cls.current = read_capture(
            ROOT / cls.receipt["current_capture"], source, cls.receipt["header"]
        )

    def test_current_receipt_replays(self) -> None:
        result = replay(ROOT, ROOT / DEFAULT_RECEIPT)
        self.assertEqual(result["primary_declarations"], 181)
        self.assertEqual(result["api_candidates"], 57)
        self.assertEqual(result["returned_type_edges"], 7)
        self.assertFalse(result["coverage_credit"])
        self.assertFalse(result["api_denominator_complete"])

    def test_changed_attested_hash_is_rejected(self) -> None:
        receipt = copy.deepcopy(self.receipt)
        receipt["attested_files"][0]["sha256"] = "wrong"
        with self.assertRaisesRegex(ValueError, "expected SHA256"):
            verify_files(ROOT, receipt)

    def test_primary_visibility_change_is_rejected(self) -> None:
        current = copy.deepcopy(self.current)
        identifier = self.receipt["reached_declarations"][0]["id"]
        current.declarations[identifier]["api_visible"] = True
        with self.assertRaisesRegex(ValueError, "exact baseline/current equality"):
            verify_capture_pair(self.baseline, current, self.receipt)

    def test_missing_member_is_rejected(self) -> None:
        current = copy.deepcopy(self.current)
        edge = next(
            row for row in current.returned_type_reachability if "members" in row
        )
        edge["members"].pop()
        with self.assertRaisesRegex(ValueError, "expected 12 exact identities"):
            verify_capture_pair(self.baseline, current, self.receipt)

    def test_duplicate_producer_is_rejected(self) -> None:
        current = copy.deepcopy(self.current)
        edge = next(
            row for row in current.returned_type_reachability if "members" in row
        )
        current.returned_type_reachability.append(copy.deepcopy(edge))
        current.summaries[0]["returned_type_reachability_events"] = 8
        with self.assertRaisesRegex(ValueError, "expected one exact edge"):
            verify_capture_pair(self.baseline, current, self.receipt)

    def test_failed_compiler_diagnostic_is_rejected(self) -> None:
        current = copy.deepcopy(self.current)
        current.diagnostics.append({"event": "diagnostic", "severity": "fatal"})
        with self.assertRaisesRegex(ValueError, "expected no error or fatal"):
            verify_capture_pair(self.baseline, current, self.receipt)

    def test_truncated_capture_is_rejected(self) -> None:
        current = copy.deepcopy(self.current)
        current.returned_type_reachability.pop()
        with self.assertRaisesRegex(ValueError, "expected no merge gaps"):
            verify_capture_pair(self.baseline, current, self.receipt)


if __name__ == "__main__":
    unittest.main()
