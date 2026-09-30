"""Crash accounting rejects changed native/Rust behavior and missing candidate inputs."""

import unittest
from native_crash_accounting import validate_crash, partition_crash_requests
from tkshhealing_corpus_transfer_proj import transfer_projection_partition
from tkshhealing_transfer_projection_crashes import NATIVE_CRASH_HASHES


class TransferProjectionAccounting(unittest.TestCase):
    def test_candidate_partition_preserves_all_requests(self) -> None:
        exact, crashes = transfer_projection_partition()
        self.assertEqual((len(exact), len(crashes)), (380, 2))
        self.assertTrue(
            all("sh_transfer_parameters_proj" in row for row in exact + crashes)
        )

    def test_null_object_is_explicitly_required_for_projection(self) -> None:
        validate_crash(-11, 0, "1 0 Standard_NullObject 1 0", "Standard_NullObject")
        with self.assertRaises(ValueError):
            validate_crash(-11, 0, "Standard_ConstructionError", "Standard_NullObject")

    def test_existing_splitter_failure_requirement_is_unchanged(self) -> None:
        validate_crash(-11, 0, "Standard_ConstructionError")
        with self.assertRaises(ValueError):
            validate_crash(-11, 0, "Standard_NullObject")

    def test_native_success_and_rust_crash_cannot_pass(self) -> None:
        for native, rust in [(0, 0), (-6, 0), (-11, -11)]:
            with self.assertRaises(ValueError):
                validate_crash(
                    native, rust, "Standard_NullObject", "Standard_NullObject"
                )

    def test_missing_crash_cannot_disappear(self) -> None:
        exact, _ = transfer_projection_partition()
        with self.assertRaises(ValueError):
            partition_crash_requests(exact, NATIVE_CRASH_HASHES)


if __name__ == "__main__":
    unittest.main()
