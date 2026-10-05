"""Cargo receipts must include all test binaries, doctests, and command failures."""

from __future__ import annotations

import unittest
from unittest.mock import patch
from pathlib import Path
import io

from cargo_test_log_summary import (
    cargo_log_arguments,
    main,
    read_cargo_test_log,
    summarize_cargo_test_log,
)


class FakeCargoLogFile(io.StringIO):
    """Named file double; reading tests do not touch the filesystem."""


class CargoTestLogSummaryTests(unittest.TestCase):
    def test_multiple_binaries_and_final_doctest_group_are_totaled(self) -> None:
        log = (
            "test result: ok. 100 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out;\n"
            "test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n"
            "Doc-tests example\n"
            "test result: ok. 245 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out;\n"
        )
        summary = summarize_cargo_test_log(log, 0)
        self.assertEqual(
            (summary.passed, summary.ignored, summary.result_groups), (395, 5, 3)
        )
        self.assertTrue(summary.command_succeeded)

    def test_failed_group_cannot_be_hidden_by_later_success(self) -> None:
        log = (
            "test result: FAILED. 4 passed; 1 failed; 0 ignored;\n"
            "test result: ok. 2 passed; 0 failed; 0 ignored;\n"
        )
        summary = summarize_cargo_test_log(log, 0)
        self.assertEqual(
            (summary.passed, summary.failed, summary.failed_result_groups), (6, 1, 1)
        )
        self.assertFalse(summary.command_succeeded)

    def test_aborted_command_with_successful_prefix_is_not_green(self) -> None:
        summary = summarize_cargo_test_log(
            "test result: ok. 3 passed; 0 failed; 0 ignored;", 101
        )
        self.assertFalse(summary.command_succeeded)
        self.assertEqual(summary.command_exit_code, 101)

    def test_failure_status_matters_even_with_zero_failed_count(self) -> None:
        summary = summarize_cargo_test_log(
            "test result: FAILED. 0 passed; 0 failed; 0 ignored;", 0
        )
        self.assertFalse(summary.command_succeeded)

    def test_colored_zero_case_summary_remains_explicit(self) -> None:
        summary = summarize_cargo_test_log(
            "\x1b[32mtest result: ok.\x1b[0m 0 passed; 0 failed; 0 ignored;", 0
        )
        self.assertEqual((summary.passed, summary.result_groups), (0, 1))
        self.assertTrue(summary.command_succeeded)

    def test_log_without_summaries_is_rejected(self) -> None:
        with self.assertRaisesRegex(
            ValueError, "0 result groups; expected at least one"
        ):
            summarize_cargo_test_log("Finished compilation", 0)

    def test_plain_log_uses_named_filesystem_double(self) -> None:
        with patch.object(
            Path, "read_text", return_value=FakeCargoLogFile("plain").read()
        ):
            self.assertEqual(read_cargo_test_log(Path("tests.log")), "plain")

    def test_gzip_log_uses_named_filesystem_double(self) -> None:
        with patch(
            "cargo_test_log_summary.gzip.open",
            return_value=FakeCargoLogFile("archived"),
        ):
            self.assertEqual(read_cargo_test_log(Path("tests.log.gz")), "archived")

    def test_command_line_requires_explicit_exit_code(self) -> None:
        with patch("sys.argv", ["summary", "tests.log", "--exit-code", "101"]):
            self.assertEqual(cargo_log_arguments(), (Path("tests.log"), 101))

    def test_cli_reports_success_failure_and_missing_summary(self) -> None:
        for text, exit_code, expected in [
            ("test result: ok. 1 passed; 0 failed; 0 ignored;", 0, 0),
            ("test result: ok. 1 passed; 0 failed; 0 ignored;", 101, 1),
            ("no summaries", 0, 2),
        ]:
            with self.subTest(exit_code=exit_code, expected=expected):
                self.assert_cli_exit(text, exit_code, expected)

    def assert_cli_exit(self, text: str, exit_code: int, expected: int) -> None:
        with (
            patch("sys.argv", ["summary", "tests.log", "--exit-code", str(exit_code)]),
            patch.object(Path, "read_text", return_value=FakeCargoLogFile(text).read()),
            patch("sys.stdout", new_callable=io.StringIO),
            patch("sys.stderr", new_callable=io.StringIO),
        ):
            self.assertEqual(main(), expected)


if __name__ == "__main__":
    unittest.main()
