"""Guard exact native verdicts and full-field comparisons in the diagnostic."""

from __future__ import annotations
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import capture_contract as capture


class AuthoredVerdict:
    """Name each complete authored test verdict and its external stream."""

    def __init__(self, root: Path, text: str, status: int) -> None:
        self.root = root
        self.stream = root / "test.stdout.log"
        self.stream.write_text(text)
        self.receipt: dict[str, object] = {
            "stdout": self.stream.name,
            "returncode": status,
        }


class AuthoredMeshCapture:
    """Return controlled records while retaining independently expected bytes."""

    def __init__(self, records: bytes) -> None:
        self.records = records

    def __call__(
        self, output: Path, label: str, command: list[str], environment: dict[str, str]
    ) -> dict[str, object]:
        stream = output / (label + ".stdout.log")
        stream.write_bytes(self.records)
        return {"stdout": stream.name, "returncode": 0}


class ContractCaptureTests(unittest.TestCase):
    def test_one_real_passing_test_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            verdict = AuthoredVerdict(
                Path(directory),
                "test result: ok. 1 passed; 0 failed; 0 ignored; 3 filtered out;",
                0,
            )
            capture.require_test_verdict(verdict.root, verdict.receipt, True)

    def test_one_real_original_failure_is_retained(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            verdict = AuthoredVerdict(
                Path(directory),
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 3 filtered out;",
                101,
            )
            capture.require_test_verdict(verdict.root, verdict.receipt, False)

    def test_zero_ignored_or_bad_exit_cannot_count_as_success(self) -> None:
        rows = [
            ("0 passed; 0 failed; 0 ignored;", 0),
            ("0 passed; 0 failed; 1 ignored;", 0),
            ("1 passed; 0 failed; 0 ignored;", 1),
        ]
        with tempfile.TemporaryDirectory() as directory:
            for summary, status in rows:
                verdict = AuthoredVerdict(
                    Path(directory), "test result: ok. " + summary, status
                )
                with self.assertRaises(ValueError):
                    capture.require_test_verdict(verdict.root, verdict.receipt, True)

    def test_full_records_match_in_two_fresh_captures(self) -> None:
        expected = (capture.HERE / "portable-cylinder.expected").read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(capture, "capture", AuthoredMeshCapture(expected)):
                results = capture.capture_records(
                    Path(directory), ["cargo"], {}, "fixed"
                )
                self.assertEqual(len(results), 2)

    def test_matching_hash_rows_do_not_hide_a_changed_coordinate(self) -> None:
        expected = (capture.HERE / "portable-cylinder.expected").read_bytes()
        changed = expected.replace(b"4014000000000000", b"4014000000000001", 1)
        self.assertNotEqual(expected, changed)
        self.assertEqual(hash_rows(expected), hash_rows(changed))
        with tempfile.TemporaryDirectory() as directory:
            with patch.object(capture, "capture", AuthoredMeshCapture(changed)):
                with self.assertRaises(ValueError):
                    capture.capture_records(Path(directory), ["cargo"], {}, "fixed")


def hash_rows(records: bytes) -> list[bytes]:
    """Keep unchanged hashes in the adversarial full-record test; `hash_rows(records)`."""
    return [row for row in records.splitlines() if row.startswith(b"hash ")]


if __name__ == "__main__":
    unittest.main()
