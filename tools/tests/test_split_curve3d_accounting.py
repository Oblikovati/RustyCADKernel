"""Native crashes remain explicit divergences, never exact comparison passes."""

import hashlib
import unittest
from tkshhealing_corpus_split_curve3d import split3d_partition, NATIVE_CRASH_SHA256
from verify_split_curve3d import validate_crash


class SplitCurve3dAccountingTests(unittest.TestCase):
    def test_every_candidate_is_accounted_once(self) -> None:
        exact, crashes = split3d_partition()
        self.assertEqual((len(exact), len(crashes)), (332, 16))
        self.assertEqual(
            {hashlib.sha256(row.encode()).hexdigest() for row in crashes},
            NATIVE_CRASH_SHA256,
        )
        self.assertTrue(set(exact).isdisjoint(crashes))

    def test_native_crash_and_typed_failure_are_distinct(self) -> None:
        validate_crash(-11, 0, "0 Standard_ConstructionError 2")
        with self.assertRaisesRegex(ValueError, "reclassify"):
            validate_crash(0, 0, "Standard_ConstructionError")

    def test_rust_crash_or_changed_failure_rejected(self) -> None:
        for code, output in [(-11, ""), (0, ""), (0, "Standard_NullObject")]:
            with self.subTest(code=code, output=output):
                with self.assertRaisesRegex(ValueError, "expected 0"):
                    validate_crash(-11, code, output)

    def test_different_native_crash_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected -11"):
            validate_crash(-6, 0, "Standard_ConstructionError")


if __name__ == "__main__":
    unittest.main()
