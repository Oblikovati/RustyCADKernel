"""Native alpha captures remain independent fixtures and retain strict family routing."""
import hashlib
import json
from pathlib import Path
import unittest
from verify_vulkan import parse_arguments
from vulkan_capture_families import capture_modes
from vulkan_line_cases import LINE_ALPHA_MODES

ROOT = Path(__file__).resolve().parents[2]

class LineAlphaCasesTests(unittest.TestCase):
    def test_alpha_family_retains_every_legacy_mode_and_full_matrix_membership(self) -> None:
        self.assertEqual(capture_modes("line-alpha"), LINE_ALPHA_MODES)
        self.assertTrue(set(LINE_ALPHA_MODES).issubset(capture_modes("all")))
        args = parse_arguments(["--source", "OCCT", "--build", "build", "--output", "receipt", "--case-family", "line-alpha"])
        self.assertEqual(args.case_family, "line-alpha")
        self.assertEqual(len(LINE_ALPHA_MODES), 11)

    def test_committed_native_buffers_match_their_device_provenance(self) -> None:
        folder = ROOT / "crates/vulkan/tests/fixtures/line-alpha"
        proof = json.loads((folder / "provenance.json").read_text())
        self.assertTrue(proof["local_only"])
        self.assertEqual(set(proof["profiles"]), {"lavapipe", "host"})
        for profile, row in proof["profiles"].items():
            self.assertEqual(len(row["assets"]), 22)
            for name, digest in row["assets"].items():
                payload = (folder / profile / name).read_bytes()
                self.assertEqual(len(payload), 64 * 64 * 4)
                self.assertEqual(hashlib.sha256(payload).hexdigest(), digest)
