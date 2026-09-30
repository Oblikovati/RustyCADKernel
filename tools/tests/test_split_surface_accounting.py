"""Surface sweep crashes must remain accounted, separate from exact passes."""

import hashlib
import unittest
from native_crash_accounting import partition_crash_requests
from tkshhealing_corpus_split_surface import (
    split_surface_partition,
    SPLIT_SURFACE_CRASH_SHA256,
)


class SplitSurfaceAccountingTests(unittest.TestCase):
    def test_all_candidates_are_preserved(self) -> None:
        exact, crashes = split_surface_partition()
        self.assertEqual((len(exact), len(crashes)), (634, 16))
        self.assertTrue(set(exact).isdisjoint(crashes))
        self.assertEqual(
            {hashlib.sha256(row.encode()).hexdigest() for row in crashes},
            SPLIT_SURFACE_CRASH_SHA256,
        )

    def test_missing_crash_request_is_rejected(self) -> None:
        exact, crashes = split_surface_partition()
        with self.assertRaisesRegex(ValueError, "inventory"):
            partition_crash_requests(exact + crashes[1:], SPLIT_SURFACE_CRASH_SHA256)

    def test_unknown_requests_remain_in_exact_gate(self) -> None:
        exact, crashes = partition_crash_requests(["unclassified"], frozenset())
        self.assertEqual(exact, ["unclassified"])
        self.assertEqual(crashes, [])


if __name__ == "__main__":
    unittest.main()
