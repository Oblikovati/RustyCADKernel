"""Typed divergence accounting keeps its historical default and accepts explicit categories."""
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch
from native_crash_accounting import replay_crash, validate_crash


class NativeCrashRustFailure:
    """Named subprocess twin for deterministic native faults and Rust failures."""
    def __init__(self, exception: str) -> None:
        self.exception = exception
        self.calls: list[list[str]] = []

    def __call__(self, command: list[str], *, input: str, text: bool,
                 capture_output: bool, timeout: int) -> subprocess.CompletedProcess[str]:
        self.calls.append(command)
        code = -11 if command[0] == "native" else 0
        output = "" if code else self.exception + "\n"
        return subprocess.CompletedProcess(command, code, stdout=output, stderr="")


class ExpectedExceptionTests(unittest.TestCase):
    def test_default_construction_error_remains_accepted(self) -> None:
        validate_crash(-11, 0, "Standard_ConstructionError\n")
        with self.assertRaises(ValueError):
            validate_crash(-11, 0, "Standard_OutOfRange\n")

    def test_explicit_out_of_range_is_not_construction_error(self) -> None:
        validate_crash(-11, 0, "Standard_OutOfRange\n", "Standard_OutOfRange")
        with self.assertRaises(ValueError):
            validate_crash(-11, 0, "Standard_ConstructionError\n", "Standard_OutOfRange")

    def test_wrong_exit_codes_and_partial_tokens_remain_rejected(self) -> None:
        for native, rust, output in [(0, 0, "Standard_OutOfRange"),
                                     (-11, -11, "Standard_OutOfRange"),
                                     (-11, 0, "notStandard_OutOfRange")]:
            with self.assertRaises(ValueError):
                validate_crash(native, rust, output, "Standard_OutOfRange")

    def test_replay_records_selected_exception_without_changing_default(self) -> None:
        for exception in ["Standard_ConstructionError", "Standard_OutOfRange"]:
            twin = NativeCrashRustFailure(exception)
            with patch("native_crash_accounting.subprocess.run", new=twin):
                record = replay_crash(Path("native"), Path("rust"), "request", exception)
            self.assertEqual(record["rust_exception"], exception)
            self.assertEqual(record["native_returncode"], -11)
            self.assertEqual(twin.calls, [["native"], ["rust"]])


if __name__ == "__main__":
    unittest.main()
