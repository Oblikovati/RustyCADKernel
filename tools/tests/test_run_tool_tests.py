"""The per-process tool test runner finds modules and reports failed processes."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from run_tool_tests import failed_modules, test_modules


class RunToolTestsTest(unittest.TestCase):
    def test_modules_are_sorted_and_filtered(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            for name in ("test_b.py", "test_a.py", "helper.py"):
                Path(folder, name).write_text("")
            self.assertEqual(test_modules(Path(folder), "test_*.py"), ["test_a.py", "test_b.py"])

    def test_killed_and_failed_processes_are_failures(self) -> None:
        results = [("test_a.py", 0, ""), ("test_b.py", 1, "FAILED"), ("test_c.py", -9, "")]
        self.assertEqual([name for name, _, _ in failed_modules(results)], ["test_b.py", "test_c.py"])


if __name__ == "__main__":
    unittest.main()
