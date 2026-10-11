"""Focused lighting runs retain all native configurations and strict parity gates."""

import unittest
from pathlib import Path
from vulkan_capping_cases import CAPPING_MODES
from verify_vulkan import CASES, capture_modes, configuration, parse_arguments
from vulkan_lighting_cases import LIGHTING_MODES, LIT_LINE_MODES


class LightingCaptureCasesTests(unittest.TestCase):
    def test_lighting_family_contains_mutations_and_projection_union(self) -> None:
        self.assertEqual(capture_modes("lighting"), LIGHTING_MODES)
        self.assertEqual(len(LIGHTING_MODES), len(set(LIGHTING_MODES)))
        self.assertEqual(len(LIGHTING_MODES) * len(CASES), 126)
        self.assertTrue(set(LIGHTING_MODES) <= set(capture_modes("all")))
        self.assertTrue(
            {
                "lit-normal-updated",
                "lit-light-off",
                "lit-clipped",
                "lit-headlight-rotated",
                "lit-distinct-back",
            }
            <= set(LIGHTING_MODES)
        )
        with self.assertRaisesRegex(
            ValueError,
            "expected all, lighting, capping, camera-animation, object-animation or group-flipping or text-label",
        ):
            capture_modes("unknown")

    def test_command_configuration_keeps_explicit_family(self) -> None:
        args = parse_arguments(
            [
                "--source",
                "source",
                "--build",
                "build",
                "--output",
                "result.json",
                "--case-family",
                "lighting",
            ]
        )
        config = configuration(args)
        self.assertEqual(config.case_family, "lighting")
        self.assertEqual(config.profiles, ("lavapipe", "host"))
        self.assertEqual(config.samples, 100)
        self.assertEqual(config.captures, Path("result.captures").resolve())


class CappingFamilySelectionTests(unittest.TestCase):
    def test_capping_family_contains_all_native_lighting_algorithms(self) -> None:
        self.assertEqual(capture_modes("capping"), CAPPING_MODES)
        self.assertTrue(
            {"cap-lit-facet", "cap-lit-gouraud", "cap-lit-phong"} <= set(CAPPING_MODES)
        )


class LitLineFamilyTests(unittest.TestCase):
    def test_nodal_line_family_preserves_mutations_and_fallbacks(self) -> None:
        self.assertEqual(capture_modes("lit-lines"), LIT_LINE_MODES)
        self.assertEqual(len(LIT_LINE_MODES), 10)
        for suffix in (
            "gouraud",
            "phong",
            "facet",
            "no-normals",
            "patterned",
            "clipped",
            "normal-updated",
            "material-updated",
            "light-updated",
        ):
            with self.subTest(suffix=suffix):
                self.assertIn(f"lit-line-{suffix}", LIT_LINE_MODES)


class LitLineShaderVariantTests(unittest.TestCase):
    def test_lit_line_shaders_are_recompiled_for_both_clip_states(self) -> None:
        from verify_vulkan import shader_variants, shader_variant_name

        variants = shader_variants("lighting.frag")
        self.assertIn((False, False, True), variants)
        self.assertIn((True, False, True), variants)
        self.assertEqual(len(variants), 6)
        self.assertEqual(
            shader_variants("unlit.vert"), ((False, False, False), (True, False, False))
        )
        self.assertEqual(
            shader_variant_name("lighting.vert", False, False, True),
            "lighting.line.vert.spv",
        )
        self.assertEqual(
            shader_variant_name("lighting.frag", True, False, True),
            "lighting.line.clipped.frag.spv",
        )
