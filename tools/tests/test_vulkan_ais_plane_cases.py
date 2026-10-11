"""The targeted family remains part of default renderer verification."""

import unittest
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_ais_plane_cases import AIS_PLANE_MODES, QUAD_MODES, layer_quad_pixels

MODES = (*AIS_PLANE_MODES, *QUAD_MODES)


class AisPlaneCaseTests(unittest.TestCase):
    def test_modes_dispatch_and_remain_in_full_corpus(self) -> None:
        self.assertEqual(capture_modes("ais-plane"), MODES)
        self.assertEqual(len(MODES), 28)
        self.assertTrue(set(MODES).issubset(capture_modes("all")))
        for mode in MODES:
            self.assertEqual(
                capture_executable(Path("examples/capture"), mode),
                Path(
                    "examples/ais_plane_capture"
                    if mode.startswith("aisplane-")
                    else "examples/quadrangle_capture"
                ),
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
                "ais-plane",
            ]
        )
        self.assertEqual(options.case_family, "ais-plane")

    def test_added_layer_roles_reject_the_default_only_capture(self) -> None:
        for mode, pixel in [
            ("quad-layers", [187, 187, 0, 255]),
            ("quad-clear-depth", [255, 0, 0, 255]),
        ]:
            self.assertEqual(
                layer_quad_pixels(1, 1, bytes(pixel), mode)["status"], "passed"
            )
            self.assertEqual(
                layer_quad_pixels(1, 1, bytes([0, 187, 187, 255]), mode)["status"],
                "failed",
            )
        with self.assertRaisesRegex(ValueError, "expected positive dimensions"):
            layer_quad_pixels(1, 1, b"", "quad-layers")


if __name__ == "__main__":
    unittest.main()
