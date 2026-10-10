"""Focused lighting runs retain all native configurations and strict parity gates."""

import unittest
from pathlib import Path
from verify_vulkan import CASES, capture_modes, configuration, parse_arguments
from vulkan_lighting_cases import LIGHTING_MODES


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
        with self.assertRaisesRegex(ValueError, "expected all or lighting"):
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
