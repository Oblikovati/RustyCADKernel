"""All specialized scene families resolve to their own capture probes."""

import unittest
from pathlib import Path
from vulkan_case_dispatch import CAPTURE_EXECUTABLES, capture_executable


class CaptureDispatchTests(unittest.TestCase):
    def test_family_probes_keep_the_build_directory(self) -> None:
        base = Path("/isolated/examples/graphic3d_capture")
        for prefix, name in CAPTURE_EXECUTABLES.items():
            self.assertEqual(
                capture_executable(base, prefix + "case"), base.with_name(name)
            )
        self.assertEqual(capture_executable(base, "default"), base)
