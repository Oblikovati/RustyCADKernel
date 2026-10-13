"""Fixture generation preserves actual native parameter contracts and exact argument order."""

import unittest
from vulkan_oit_cases import (
    OitCaptureCase,
    oit_alpha_fallback_cases,
    oit_interaction_cases,
    oit_capping_cases,
)


class OitCaseTests(unittest.TestCase):
    def test_source_receipt_retains_every_native_oit_producer(self) -> None:
        from vulkan_source_receipt import RECEIPT_ORACLE_PATHS

        stems = (
            "layers.hxx",
            "layer_lifetime.cxx",
            "transition.cxx",
            "alpha_scope.cxx",
            "draw_trace.hxx",
            "capability.cxx",
            "capture.cxx",
            "fixture.hxx",
        )
        self.assertTrue(
            all(
                "tools/oracle/martensite_oit_" + stem in RECEIPT_ORACLE_PATHS
                for stem in stems
            )
        )

    def test_capture_arguments_share_native_and_rust_order(self) -> None:
        case = OitCaptureCase("peeling", 4, 0.75, 8, True, False, "clipped")
        self.assertEqual(
            case.arguments("/tmp/fixture"),
            ["peeling", "/tmp/fixture", "4", "0.75", "8", "1", "0", "clipped"],
        )

    def test_interactions_cover_each_real_shader_and_order_path(self) -> None:
        cases = oit_interaction_cases()
        self.assertEqual(len(cases), 168)
        self.assertEqual(len(set(cases)), 168)
        self.assertEqual(
            {case.fixture for case in cases},
            {
                "two",
                "clipped",
                "texture-nearest",
                "texture-phong",
                "texture-gouraud",
                "texture-facet",
                "texture-no-modulate",
            },
        )

    def test_no_blended_fill_controls_keep_sample_request(self) -> None:
        cases = oit_alpha_fallback_cases()
        self.assertEqual(len(cases), 24)
        self.assertTrue(all(case.samples == 4 for case in cases))
        self.assertEqual(len([case for case in cases if case.method == "peeling"]), 8)

    def test_caps_cover_actual_material_stencil_and_sampled_policies(self) -> None:
        cases = oit_capping_cases()
        self.assertEqual(len(cases), 96)
        self.assertEqual(len(set(cases)), 96)
        self.assertTrue(all(case.fixture.startswith("cap-") for case in cases))
        self.assertTrue(all(not case.reverse for case in cases))


if __name__ == "__main__":
    unittest.main()
