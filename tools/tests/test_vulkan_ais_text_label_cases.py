"""The targeted family remains part of default renderer verification."""

import unittest
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_ais_text_label_cases import AIS_TEXT_LABEL_MODES


class AisTextLabelCaseTests(unittest.TestCase):
    def test_modes_dispatch_and_remain_in_full_corpus(self) -> None:
        self.assertEqual(capture_modes("text-label"), AIS_TEXT_LABEL_MODES)
        self.assertEqual(len(AIS_TEXT_LABEL_MODES), 6)
        self.assertTrue(set(AIS_TEXT_LABEL_MODES).issubset(capture_modes("all")))
        for mode in AIS_TEXT_LABEL_MODES:
            self.assertEqual(
                capture_executable(Path("examples/capture"), mode),
                Path("examples/ais_text_label_capture"),
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
                "text-label",
            ]
        )
        self.assertEqual(options.case_family, "text-label")


if __name__ == "__main__":
    unittest.main()
