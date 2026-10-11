"""Point alpha tooling retains strict routing, provenance and native coverage gates."""
import hashlib
import json
import unittest
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_point_alpha_cases import POINT_ALPHA_MODES
from vulkan_source_receipt import source_hashes

ROOT = Path(__file__).resolve().parents[2]


class PointAlphaCaseTests(unittest.TestCase):
    def test_family_routes_to_raw_arrays_and_is_in_complete_corpus(self) -> None:
        self.assertEqual(capture_modes("point-alpha"), POINT_ALPHA_MODES)
        self.assertEqual(len(set(POINT_ALPHA_MODES)), 21)
        self.assertTrue(set(POINT_ALPHA_MODES).issubset(capture_modes("all")))
        for mode in POINT_ALPHA_MODES:
            self.assertEqual(capture_executable(Path("examples/graphic3d_capture"), mode), Path("examples/graphic3d_capture"))
        args = parse_arguments(["--source", "OCCT", "--build", "build", "--output", "receipt.json", "--case-family", "point-alpha"])
        self.assertEqual(args.case_family, "point-alpha")

    def test_committed_assets_are_device_bound_repeatable_and_fingerprinted(self) -> None:
        folder = ROOT / "crates/vulkan/tests/fixtures/point-alpha"
        receipt = json.loads((folder / "provenance.json").read_text())
        self.assertEqual(receipt["upstream_revision"], "3d097a0328e71b826377d4814ab05ec3c3d23871")
        self.assertTrue(receipt["repeatable"])
        self.assertEqual([p["name"] for p in receipt["profiles"]], ["lavapipe", "host"])
        for profile in receipt["profiles"]:
            self.assertEqual(len(profile["assets"]), 42)
            self.assert_profile_assets(folder, profile)
        self.assertIn("tools/oracle/martensite_vulkan_point_alpha.cxx", source_hashes())

    def assert_profile_assets(self, folder: Path, profile: dict[str, object]) -> None:
        assets = profile["assets"]
        self.assertIsInstance(assets, dict)
        for name, fingerprint in assets.items():
            payload = (folder / profile["name"] / name).read_bytes()
            self.assertEqual(len(payload), 64 * 64 * 4)
            self.assertEqual(hashlib.sha256(payload).hexdigest(), fingerprint)
