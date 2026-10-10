"""Regression tests ensure renderer evidence rejects corrupt or misleading captures."""

import struct
import unittest
from inventory_vulkan_baseline import rendering_controls
from vulkan_capture_compare import compare_bytes


class VulkanCaptureComparisonTests(unittest.TestCase):
    def test_exact_color_and_tolerated_depth(self) -> None:
        result = compare_bytes(
            "pixel",
            1,
            bytes(4),
            bytes(4),
            struct.pack("=f", 0.25),
            struct.pack("=f", 0.2500001),
        )
        self.assertEqual(result.color_mismatch_pixels, 0)
        self.assertLess(result.maximum_depth_delta, 1e-6)

    def test_truncation_and_empty_captures_fail(self) -> None:
        for pixels, color in [(1, bytes(3)), (0, b"")]:
            with self.subTest(pixels=pixels), self.assertRaisesRegex(
                ValueError, "complete RGBA/depth"
            ):
                compare_bytes("bad", pixels, color, color, color, color)

    def test_color_or_depth_mismatch_fails(self) -> None:
        for color, depth in [(bytes([1, 0, 0, 0]), 0.25), (bytes(4), 0.5)]:
            with self.subTest(color=color, depth=depth), self.assertRaisesRegex(
                ValueError, "identical RGBA"
            ):
                compare_bytes(
                    "bad",
                    1,
                    bytes(4),
                    color,
                    struct.pack("=f", 0.25),
                    struct.pack("=f", depth),
                )

    def test_nan_infinity_and_out_of_range_depth_fail(self) -> None:
        for value in [float("nan"), float("inf"), -0.1, 1.1]:
            with self.subTest(value=value), self.assertRaisesRegex(
                ValueError, "finite depth"
            ):
                compare_bytes(
                    "bad",
                    1,
                    bytes(4),
                    bytes(4),
                    struct.pack("=f", value),
                    struct.pack("=f", 0.25),
                )

    def test_baseline_extracts_fields_with_provenance_not_methods(self) -> None:
        header = "public: //! @name stereo\n  unsigned int Resolution; //!< note\n  NCollection_Mat4<float> Left;\n  void Draw();\n"
        fields = rendering_controls(header)
        self.assertEqual([f["name"] for f in fields], ["Resolution", "Left"])
        self.assertEqual([f["line"] for f in fields], [2, 3])
        self.assertTrue(all(f["status"] == "pending" for f in fields))


if __name__ == "__main__":
    unittest.main()
