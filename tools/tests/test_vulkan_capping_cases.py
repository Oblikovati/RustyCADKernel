"""Keep section rendering fixtures in every native camera and device profile."""

import unittest
from vulkan_capping_cases import CAPPING_MODES
from verify_vulkan import CASES


class CappingCaptureCasesTests(unittest.TestCase):
    def test_section_matrix_keeps_disabled_and_instance_controls(self) -> None:
        self.assertEqual(len(CAPPING_MODES), len(set(CAPPING_MODES)))
        self.assertEqual(len(CAPPING_MODES) * len(CASES), 174)
        self.assertTrue(
            {
                "cap-open",
                "cap-off",
                "cap-disabled",
                "cap-shadow",
                "cap-chain",
                "cap-chain-disabled-child",
                "cap-overlap",
                "cap-overlap-object",
                "cap-pers-local",
                "cap-pers-global",
                "cap-group-pers",
                "cap-mirrored",
                "cap-mirror-uncut",
            }
            <= set(CAPPING_MODES)
        )
