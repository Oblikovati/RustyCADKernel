"""Keep Martensite point cloud capture families in the full verification matrix."""
import unittest
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_point_cloud_cases import POINT_CLOUD_MODES, point_cloud_pixels
from tkv3d_corpus_ais_point_cloud import append_point_cloud_requests


class PointCloudCaseTests(unittest.TestCase):
    def test_capture_modes_dispatch_and_remain_in_full_corpus(self) -> None:
        self.assertEqual(capture_modes("point-cloud"), POINT_CLOUD_MODES)
        self.assertEqual(len(POINT_CLOUD_MODES), 16)
        self.assertTrue(set(POINT_CLOUD_MODES).issubset(capture_modes("all")))
        for mode in POINT_CLOUD_MODES:
            self.assertEqual(capture_executable(Path("examples/capture"), mode), Path("examples/ais_point_cloud_capture"))

    def test_family_is_selectable_and_numeric_requests_are_independent(self) -> None:
        args = parse_arguments(["--source", "OCCT", "--build", "build", "--output", "receipt.json", "--case-family", "point-cloud"])
        self.assertEqual(args.case_family, "point-cloud")
        rows: list[str] = []
        append_point_cloud_requests(rows)
        self.assertEqual(len(rows), 77)
        self.assertEqual(len(set(rows)), 77)
        self.assertEqual(rows[0], "v3_point_cloud_alias")
        for row in rows[5:]:
            tokens = row.split()
            self.assertEqual(int(tokens[4]), len(tokens[5:]))

    def test_geometry_and_subset_roles_reject_false_empty_captures(self) -> None:
        background = bytes([0, 0, 255, 255])
        for mode in POINT_CLOUD_MODES:
            expected = "passed" if mode == "cloud-erased" else "failed"
            self.assertEqual(point_cloud_pixels(1, 1, background, mode)["status"], expected)
        for mode, rgba in [("cloud-subset-selected", [255,255,0,255]), ("cloud-subset-detected", [0,255,255,255]), ("cloud-points", [84,57,7,255])]:
            self.assertEqual(point_cloud_pixels(1,1,bytes(rgba),mode)["status"], "passed")
        self.assertEqual(point_cloud_pixels(1,1,bytes([84,57,7,255]),"cloud-subset-detected")["status"], "failed")
        with self.assertRaises(ValueError):
            point_cloud_pixels(0,1,b"", "cloud-erased")
