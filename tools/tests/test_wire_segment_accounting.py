"""Wire-segment candidates remain fully accounted between parity and crash gaps."""

import unittest
from unittest.mock import patch

from tkshhealing_corpus_wire_segment import wire_segment_candidates
from tkshhealing_wire_segment_crashes import NATIVE_CRASH_HASHES
from tkshhealing_wire_segment_crashes import wire_segment_partition
from native_crash_accounting import partition_crash_requests
from verify_wire_segment import verify_wire_segment, wire_segment_oracle


class WireSegmentAccountingTests(unittest.TestCase):
    def test_partition_covers_every_candidate(self) -> None:
        exact, crashes = wire_segment_partition()
        self.assertEqual((len(exact), len(crashes)), (176, 2))
        self.assertCountEqual(exact + crashes, wire_segment_candidates())

    def test_missing_crash_candidate_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected"):
            partition_crash_requests(wire_segment_candidates()[:-1], NATIVE_CRASH_HASHES)

    def test_verifier_requires_native_null_object_failure(self) -> None:
        with patch("verify_wire_segment.verify_crash_oracle") as verify:
            verify_wire_segment()
        self.assertEqual(verify.call_args.args[-1], "Standard_NullObject")
        self.assertIs(verify.call_args.args[1], wire_segment_partition)

    def test_oracle_uses_tkshhealing_libraries(self) -> None:
        oracle = wire_segment_oracle()
        self.assertEqual(oracle.toolkit, "TKShHealing-WireSegment")
        self.assertIn("TKShHealing", oracle.libraries)
        self.assertEqual(oracle.corpus(), "")


if __name__ == "__main__":
    unittest.main()
