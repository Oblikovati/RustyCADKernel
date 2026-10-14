"""Keep native OCCT builds local while preserving Martensite CI test gates."""

from release_verification_plan import verification_gates
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class CiOracleBoundaryTests(unittest.TestCase):
    def test_ci_keeps_tooling_and_rust_tests_without_native_oracle_builds(self) -> None:
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        self.assertIn("verify_release_gate.py --root . --gate tools", workflow)
        gates = {gate.name: gate for gate in verification_gates(ROOT, Path("../OCCT"))}
        self.assertEqual(
            gates["tools"].commands[0].argv,
            ("python3", "tools/run_tool_tests.py", "--jobs", "2"),
        )
        self.assertIn("cargo test-all --no-fail-fast", workflow)
        for native_command in (
            "verify_tkernel.py",
            "verify_toolkit.py",
            "tkernel-oracle:",
        ):
            with self.subTest(native_command=native_command):
                self.assertNotIn(native_command, workflow)

    def test_release_reuses_the_same_ci_boundary(self) -> None:
        workflow = (ROOT / ".github/workflows/release.yml").read_text()
        self.assertIn("uses: ./.github/workflows/ci.yml", workflow)
        self.assertIn("needs: [resolve, owner-checks]", workflow)
        self.assertIn("tools/release_packages.py", workflow)


if __name__ == "__main__":
    unittest.main()
