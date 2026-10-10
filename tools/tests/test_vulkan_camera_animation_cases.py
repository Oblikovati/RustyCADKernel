"""The camera family retains native controls and joins the full renderer corpus."""

import unittest
from verify_vulkan import capture_modes, parse_arguments
from vulkan_camera_animation_cases import CAMERA_ANIMATION_MODES
from tkv3d_corpus_animation_camera import append_camera_requests
from verify_animation_camera import corpus


class CameraAnimationCaseTests(unittest.TestCase):
    def test_capture_family_is_selectable_and_included_in_full_matrix(self) -> None:
        self.assertEqual(capture_modes("camera-animation"), CAMERA_ANIMATION_MODES)
        self.assertEqual(len(CAMERA_ANIMATION_MODES), 8)
        self.assertTrue(set(CAMERA_ANIMATION_MODES).issubset(capture_modes("all")))
        options = parse_arguments(
            [
                "--source",
                "OCCT",
                "--build",
                "build",
                "--output",
                "receipt.json",
                "--case-family",
                "camera-animation",
            ]
        )
        self.assertEqual(options.case_family, "camera-animation")

    def test_numeric_corpus_is_repeatable_and_covers_missing_endpoints(self) -> None:
        rows: list[str] = []
        append_camera_requests(rows)
        self.assertEqual(len(rows), 594)
        self.assertEqual(len(set(rows)), 594)
        self.assertIn("v3_animation_camera 7 0.0 0.0 0.0", rows)
        again: list[str] = []
        append_camera_requests(again)
        self.assertEqual(rows, again)
        self.assertEqual(corpus(), "\n".join(rows) + "\n")
