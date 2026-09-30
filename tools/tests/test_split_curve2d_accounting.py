"""Every 2D curve split request remains visible in exact or crash accounting."""

import hashlib
import unittest
from native_crash_accounting import partition_crash_requests
from tkshhealing_corpus_split_curve2d import split2d_partition, SPLIT2D_CRASH_SHA256


class SplitCurve2dAccountingTests(unittest.TestCase):
    def test_all_requests_are_accounted(self) -> None:
        exact, crashes = split2d_partition()
        self.assertEqual((len(exact), len(crashes)), (530, 70))
        self.assertTrue(set(exact).isdisjoint(crashes))
        self.assertEqual(
            {hashlib.sha256(row.encode()).hexdigest() for row in crashes},
            SPLIT2D_CRASH_SHA256,
        )

    def test_missing_native_crash_is_rejected(self) -> None:
        exact, crashes = split2d_partition()
        with self.assertRaisesRegex(ValueError, "inventory"):
            partition_crash_requests(exact + crashes[:-1], SPLIT2D_CRASH_SHA256)


if __name__ == "__main__":
    unittest.main()
