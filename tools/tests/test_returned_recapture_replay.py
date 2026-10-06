"""Negative checks for declaration identity, semantic projection and denominator claims."""

from __future__ import annotations

import unittest

from api_inventory.merge import declaration_identity
from report_returned_type_inventory import COMPARISON_FIELDS
from verify_returned_type_recapture import verify_comparison_row, verify_summary


class RecaptureReplayTests(unittest.TestCase):
    def setUp(self) -> None:
        self.declaration = {
            "usr": "native",
            "kind": "Function",
            "signature": "void ()",
            "identity_signature": "void ()",
            "template_signature": "",
            "api_visible": True,
        }
        projection = {field: self.declaration.get(field) for field in COMPARISON_FIELDS}
        self.row = {
            "id": declaration_identity(self.declaration),
            "state": "unchanged",
            "current": projection,
            "published": dict(projection),
        }
        self.counts = {
            "declarations": 1,
            "api_declarations": 1,
            "headers_by_status": {"parsed": 1},
            "unsupported_records": 0,
        }
        self.summary = {
            "primary_id_comparison": {"unchanged": 1},
            "coverage_credit": False,
            "api_denominator_complete": False,
        }

    def test_exact_row_and_summary_pass_without_coverage_credit(self) -> None:
        verify_comparison_row(self.row, self.declaration)
        verify_summary({"global_expected": self.counts}, self.counts, self.summary)

    def test_wrong_identity_is_rejected(self) -> None:
        self.row["id"] = "wrong"
        with self.assertRaisesRegex(
            ValueError, "expected recomputed exact native identity"
        ):
            verify_comparison_row(self.row, self.declaration)

    def test_changed_signature_or_visibility_is_rejected(self) -> None:
        for field, value in [("signature", "int ()"), ("api_visible", False)]:
            row = {**self.row, "published": {**self.row["published"], field: value}}
            with (
                self.subTest(field=field),
                self.assertRaisesRegex(ValueError, "unchanged current and published"),
            ):
                verify_comparison_row(row, self.declaration)

    def test_changed_counts_or_promoted_credit_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "declarations: expected"):
            verify_summary(
                {"global_expected": {**self.counts, "declarations": 2}},
                self.counts,
                self.summary,
            )
        for field in ["coverage_credit", "api_denominator_complete"]:
            with (
                self.subTest(field=field),
                self.assertRaisesRegex(ValueError, "expected false credit"),
            ):
                verify_summary(
                    {"global_expected": self.counts},
                    self.counts,
                    {**self.summary, field: True},
                )
        with self.assertRaisesRegex(ValueError, "only unchanged historical identities"):
            verify_summary(
                {"global_expected": self.counts},
                self.counts,
                {**self.summary, "primary_id_comparison": {"changed": 1}},
            )


if __name__ == "__main__":
    unittest.main()
