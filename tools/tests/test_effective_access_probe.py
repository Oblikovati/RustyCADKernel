"""Native access result validation rejects unrelated failures and false passing records."""

from __future__ import annotations

import subprocess
import unittest

from build_effective_api_obligations import verify_control_record, verify_function_body
from probe_effective_api_access import EXPECTED_STDOUT, require_probe_result


class NativeAccessProbeTests(unittest.TestCase):
    def test_accept_positive_and_intended_access_failures(self) -> None:
        require_probe_result("positive", subprocess.CompletedProcess([], 0, "", ""))
        for control, category in [
            ("sparse_private_name", "private"),
            ("protected_type", "protected"),
            ("abstract_construction", "abstract"),
        ]:
            require_probe_result(
                control, subprocess.CompletedProcess([], 1, "", category)
            )

    def test_reject_wrong_compiler_outcomes(self) -> None:
        for control, status, diagnostic in [
            ("positive", 1, "error"),
            ("private_storage", 0, "private"),
            ("protected_type", 1, "missing header"),
        ]:
            with self.assertRaisesRegex(ValueError, "expected intended"):
                require_probe_result(
                    control, subprocess.CompletedProcess([], status, "", diagnostic)
                )

    def test_record_requires_execution_and_exact_output(self) -> None:
        row = {
            "control": "positive",
            "exit_code": 0,
            "expected_outcome_observed": True,
            "execution": {"exit_code": 0, "stdout": EXPECTED_STDOUT},
        }
        verify_control_record(row)
        for changes in [
            {"execution": None},
            {"execution": {"exit_code": 0, "stdout": "wrong"}},
            {"expected_outcome_observed": False},
        ]:
            with self.assertRaises(ValueError):
                verify_control_record({**row, **changes})

    def test_reject_bool_exit_code(self) -> None:
        with self.assertRaisesRegex(ValueError, "integer exit"):
            verify_control_record(
                {
                    "control": "private_storage",
                    "exit_code": True,
                    "expected_outcome_observed": True,
                }
            )

    def test_rust_body_requires_exact_inherent_context(self) -> None:
        for row in [
            {"kind": "function", "signature": {"has_body": False}},
            {
                "kind": "function",
                "signature": {"has_body": True},
                "impl_context": {"trait": {"path": "Copy"}},
            },
            {
                "kind": "function",
                "signature": {"has_body": True},
                "impl_context": {"trait": None, "for": "wrong"},
            },
        ]:
            with self.assertRaises(ValueError):
                verify_function_body(row)


if __name__ == "__main__":
    unittest.main()
