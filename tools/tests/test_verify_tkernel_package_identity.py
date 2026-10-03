"""Package-aware TKernel execution keeps target/test strings unambiguous."""

from __future__ import annotations

import pathlib
import unittest
from unittest.mock import patch

from verify_tkernel import run_rust_mappings


class CargoPackageCommandFake:
    """Return the mapped test event for each requested crate and target."""

    def __init__(self) -> None:
        self.calls: list[tuple[str, str]] = []

    def __call__(self, command, _log, *, env):
        package = command[command.index("-p") + 1]
        target = command[command.index("--test") + 1]
        self.calls.append((package, target))
        return "test shared_case ... ok\n"


class TKernelPackageIdentityTest(unittest.TestCase):
    def test_same_target_and_test_run_once_per_package(self) -> None:
        expected = {
            ("rustycad-tkernel", "shared_target", "shared_case"),
            ("rustycad-tkgeomalgo", "shared_target", "shared_case"),
        }
        fake = CargoPackageCommandFake()
        with patch("verify_tkernel.run", side_effect=fake):
            passed = run_rust_mappings(expected, {}, pathlib.Path("target/runtime"))
        self.assertEqual(
            set(fake.calls),
            {
                ("rustycad-tkernel", "shared_target"),
                ("rustycad-tkgeomalgo", "shared_target"),
            },
        )
        self.assertEqual(passed, expected)

    def test_missing_package_test_does_not_become_a_pass(self) -> None:
        expected = {("rustycad-tkernel", "target", "expected_case")}
        with patch("verify_tkernel.run", return_value="test other_case ... ok\n"):
            with self.assertRaisesRegex(RuntimeError, "rustycad-tkernel/target"):
                run_rust_mappings(expected, {}, pathlib.Path("target/runtime"))


if __name__ == "__main__":
    unittest.main()
