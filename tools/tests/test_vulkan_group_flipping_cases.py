"""The targeted family remains part of default renderer verification."""
import unittest
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_group_flipping_cases import GROUP_FLIPPING_MODES

class GroupFlippingCaseTests(unittest.TestCase):
    def test_modes_dispatch_and_remain_in_full_corpus(self) -> None:
        self.assertEqual(capture_modes("group-flipping"), GROUP_FLIPPING_MODES)
        self.assertEqual(len(GROUP_FLIPPING_MODES), 9)
        self.assertTrue(set(GROUP_FLIPPING_MODES).issubset(capture_modes("all")))
        for mode in GROUP_FLIPPING_MODES:
            self.assertEqual(capture_executable(Path("examples/capture"),mode),
                             Path("examples/group_flipping_capture"))

    def test_targeted_family_is_explicitly_selectable(self) -> None:
        options = parse_arguments(["--source","OCCT","--build","build",
            "--output","result.json","--case-family","group-flipping"])
        self.assertEqual(options.case_family,"group-flipping")

if __name__ == "__main__":
    unittest.main()
