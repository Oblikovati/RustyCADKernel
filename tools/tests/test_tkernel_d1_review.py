"""The 2026-10-15 D1 review outcomes stay consistent with the ledger and its audit."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from tkernel_d1_review import (
    EXCLUDED_MARK,
    IMPLEMENTED_MARK,
    kept_partial_rows,
    review_outcome,
    supersedes,
)

ROOT = Path(__file__).resolve().parents[2]


class ReviewOutcomeTest(unittest.TestCase):
    def test_each_outcome_needs_its_mark_or_audit_entry(self) -> None:
        kept = frozenset({"Kept"})
        kept_row = {"upstream_id": "Kept", "status": "partial"}
        self.assertEqual(review_outcome(kept_row, kept), "partial")
        implemented = {"upstream_id": "X", "status": "implemented", "notes": "n " + IMPLEMENTED_MARK}
        self.assertEqual(review_outcome(implemented, kept), "implemented")
        excluded = {"upstream_id": "X", "status": "excluded", "notes": EXCLUDED_MARK + ", ...)"}
        self.assertEqual(review_outcome(excluded, kept), "excluded")
        for row in (
            {"upstream_id": "Other", "status": "partial"},
            {"upstream_id": "X", "status": "implemented", "notes": "no review"},
            {"upstream_id": "X", "status": "excluded", "notes": "Excluded: other reason"},
        ):
            with self.assertRaisesRegex(ValueError, "expected a D1-reviewed row"):
                review_outcome(row, kept)

    def test_supersedes_keeps_every_other_field_and_the_old_note(self) -> None:
        proposed = {"upstream_id": "X", "rust_item": "a::b", "status": "partial", "notes": "old"}
        reviewed = dict(proposed, status="implemented", notes="old " + IMPLEMENTED_MARK + " (plan)")
        self.assertTrue(supersedes(reviewed, proposed))
        self.assertFalse(supersedes(dict(reviewed, rust_item="a::c"), proposed))
        self.assertFalse(supersedes(dict(reviewed, notes=IMPLEMENTED_MARK), proposed))
        self.assertFalse(supersedes(dict(proposed), proposed))


class LedgerAgreesWithAuditTest(unittest.TestCase):
    def test_kept_rows_are_partial_and_reviewed_rows_carry_their_marks(self) -> None:
        kept = kept_partial_rows(ROOT)
        rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        tkernel = [row for row in rows if row["toolkit"] == "TKernel"]
        partial = {row["upstream_id"] for row in tkernel if row["status"] == "partial"}
        self.assertEqual(partial, set(kept))
        reviewed = [row for row in tkernel if IMPLEMENTED_MARK in row.get("notes", "")]
        self.assertTrue(all(review_outcome(row, kept) == "implemented" for row in reviewed))
        audit_path = ROOT / "porting/audits/tkernel-d1-partial-review-2026-10-15.json"
        decisions = json.loads(audit_path.read_text())["decisions"]
        implemented = decisions["implemented_class_b_or_cpp_only_residue"]
        self.assertEqual(len(reviewed), implemented)


if __name__ == "__main__":
    unittest.main()
