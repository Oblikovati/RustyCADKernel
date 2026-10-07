"""Rules for GTests of a C++-only language property (status not_applicable_to_rust)."""

from __future__ import annotations

import unittest

from legacy_not_applicable import not_applicable_case, not_applicable_rows
from legacy_pending_dependency import runnable_rows

SHA = "a" * 64
REASON = "The test asserts std::is_move_constructible<T> is false; every Rust value can be moved."


def row(**fields: object) -> dict[str, object]:
    """A not-applicable test row; fields replace or add keys."""
    base = {"upstream_id": "Suite.Case", "status": "not_applicable_to_rust", "source": "src/X/GTests/X_Test.cxx",
            "source_sha256": SHA, "reason": REASON}
    base.update(fields)
    return base


class NotApplicableRowTest(unittest.TestCase):
    def test_valid_row(self) -> None:
        case = not_applicable_case(row())
        self.assertEqual((case.identity, case.reason), ("Suite.Case", REASON))

    def test_short_or_missing_reason_is_refused(self) -> None:
        for reason in (None, "", "C++ only"):
            with self.assertRaisesRegex(ValueError, "reason"):
                not_applicable_case(row(reason=reason))

    def test_rust_side_is_refused(self) -> None:
        with self.assertRaisesRegex(ValueError, "rust_test"):
            not_applicable_case(row(rust_test="case"))

    def test_native_approval_is_refused(self) -> None:
        with self.assertRaisesRegex(ValueError, "native_expected_failure"):
            not_applicable_case(row(native_expected_failure="x"))

    def test_unpinned_source_is_refused(self) -> None:
        with self.assertRaisesRegex(ValueError, "source_sha256"):
            not_applicable_case(row(source_sha256="abc"))

    def test_rows_without_rust_side_are_not_runnable(self) -> None:
        rows = [row(), {"upstream_id": "A.B", "status": "pending_dependency"}, {"upstream_id": "C.D", "status": "translated"}]
        self.assertEqual([r["upstream_id"] for r in runnable_rows(rows)], ["C.D"])
        self.assertEqual(list(not_applicable_rows(rows)), ["Suite.Case"])


if __name__ == "__main__":
    unittest.main()
