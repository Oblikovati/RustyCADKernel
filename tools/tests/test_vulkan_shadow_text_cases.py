"""Shadow text uses the retained font path and keeps strict native comparisons."""
import unittest
import hashlib
import json
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_text_cases import SHADOW_TEXT_MODES

class ShadowTextCaseTests(unittest.TestCase):
    def test_shadow_family_routes_to_retained_text_without_relaxing_comparison(self) -> None:
        self.assertEqual(capture_modes("shadow-text"), SHADOW_TEXT_MODES)
        self.assertEqual(len(set(SHADOW_TEXT_MODES)), 9)
        self.assertTrue(set(SHADOW_TEXT_MODES).issubset(capture_modes("all")))
        for mode in SHADOW_TEXT_MODES:
            self.assertEqual(capture_executable(Path("examples/graphic3d_capture"), mode), Path("examples/text_capture"))
        args = parse_arguments(["--source", "OCCT", "--build", "build", "--output", "receipt.json", "--case-family", "shadow-text"])
        self.assertEqual(args.case_family, "shadow-text")

    def test_native_shadow_assets_are_repeatable_device_bound_and_fingerprinted(self) -> None:
        folder = Path(__file__).resolve().parents[2] / "crates/vulkan/tests/fixtures/shadow-text"
        receipt = json.loads((folder / "provenance.json").read_text())
        self.assertEqual(receipt["upstream_revision"], "3d097a0328e71b826377d4814ab05ec3c3d23871")
        self.assertTrue(receipt["repeatable"])
        self.assertEqual(receipt["extent"], [96, 48])
        for profile in receipt["profiles"]:
            self.assertEqual(len(profile["assets"]), 6)
            self.assertTrue(profile["device"]["name"])
            for name, fingerprint in profile["assets"].items():
                payload = (folder / profile["name"] / name).read_bytes()
                self.assertEqual(len(payload), 96 * 48 * 4)
                self.assertEqual(hashlib.sha256(payload).hexdigest(), fingerprint)
