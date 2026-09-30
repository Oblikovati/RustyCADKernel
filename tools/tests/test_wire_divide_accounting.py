"""Wire candidates cannot silently lose their documented native crashes."""

import unittest
from unittest.mock import patch
from tkshhealing_corpus_wire_divide import wire_divide_candidates, wire_divide_partition
from native_crash_accounting import partition_crash_requests
from tkshhealing_wire_divide_crashes import NATIVE_CRASH_HASHES
from verify_wire_divide import verify_wire_divide, wire_divide_oracle


class WireDivideAccountingTests(unittest.TestCase):
    def test_partition_retains_every_candidate(self) -> None:
        exact, crashes = wire_divide_partition()
        self.assertEqual((len(exact), len(crashes)), (699, 2))
        self.assertCountEqual(exact + crashes, wire_divide_candidates())

    def test_missing_crash_request_is_rejected(self) -> None:
        candidates = wire_divide_candidates()[:-1]
        with self.assertRaisesRegex(ValueError, "expected"):
            partition_crash_requests(candidates, NATIVE_CRASH_HASHES)

    def test_verifier_requires_null_object_replay(self) -> None:
        with patch("verify_wire_divide.verify_crash_oracle") as verify:
            verify_wire_divide()
        self.assertEqual(verify.call_args.args[-1], "Standard_NullObject")
        self.assertIs(verify.call_args.args[1], wire_divide_partition)

    def test_oracle_keeps_toolkit_libraries(self) -> None:
        oracle = wire_divide_oracle()
        self.assertEqual(oracle.toolkit, "TKShHealing-WireDivide")
        self.assertIn("TKShHealing", oracle.libraries)
        self.assertEqual(oracle.corpus(), "")


if __name__ == "__main__":
    unittest.main()
