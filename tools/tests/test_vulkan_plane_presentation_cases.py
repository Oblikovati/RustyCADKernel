"""The targeted family remains part of default renderer verification."""

import unittest
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_plane_presentation_cases import PLANE_PRESENTATION_MODES


class PlanePresentationCaseTests(unittest.TestCase):
    def test_modes_dispatch_and_remain_in_full_corpus(self) -> None:
        self.assertEqual(capture_modes("plane-presentations"), PLANE_PRESENTATION_MODES)
        self.assertEqual(len(PLANE_PRESENTATION_MODES), 6)
        self.assertTrue(set(PLANE_PRESENTATION_MODES).issubset(capture_modes("all")))
        for mode in PLANE_PRESENTATION_MODES:
            self.assertEqual(
                capture_executable(Path("examples/capture"), mode),
                Path("examples/plane_presentation_capture"),
            )

    def test_targeted_family_is_explicitly_selectable(self) -> None:
        options = parse_arguments(
            [
                "--source",
                "OCCT",
                "--build",
                "build",
                "--output",
                "result.json",
                "--case-family",
                "plane-presentations",
            ]
        )
        self.assertEqual(options.case_family, "plane-presentations")


if __name__ == "__main__":
    unittest.main()
