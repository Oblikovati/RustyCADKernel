"""Rubber-band family cannot omit functional scenes or hide absent filling."""
import unittest
from pathlib import Path
from tkv3d_corpus_ais_rubber_band import append_rubber_band_requests
from vulkan_capture_families import capture_modes
from vulkan_case_dispatch import capture_executable
from vulkan_rubber_band_cases import RUBBER_BAND_MODES, rubber_band_pixels
from verify_vulkan import parse_arguments


class RubberBandCasesTests(unittest.TestCase):
    def test_every_mode_reaches_the_capture_executable(self) -> None:
        self.assertEqual(capture_modes("rubber-band"), RUBBER_BAND_MODES)
        self.assertTrue(set(RUBBER_BAND_MODES).issubset(capture_modes("all")))
        for mode in RUBBER_BAND_MODES:
            self.assertEqual(capture_executable(Path("examples/capture"), mode), Path("examples/ais_rubber_band_capture"))
        args = parse_arguments(["--source", "OCCT", "--build", "build", "--output", "receipt", "--case-family", "rubber-band"])
        self.assertEqual(args.case_family, "rubber-band")

    def test_native_requests_are_independent_and_complete(self) -> None:
        rows: list[str] = []
        append_rubber_band_requests(rows)
        self.assertEqual(len(rows), 2520)
        self.assertEqual(len(set(rows)), 2520)
        for row in rows:
            tokens = row.split()
            self.assertEqual(int(tokens[5]), len(tokens[7:]))

    def test_blank_or_missing_filling_is_rejected(self) -> None:
        for mode in RUBBER_BAND_MODES:
            self.assertEqual(rubber_band_pixels(1, 1, bytes([0,0,255,255]), mode)["status"], "failed")
        self.assertEqual(rubber_band_pixels(1, 1, bytes([255,255,255,255]), "rubber-filled")["status"], "failed")
        self.assertEqual(rubber_band_pixels(1, 1, bytes([255,0,0,255]), "rubber-filled")["status"], "passed")
        self.assertEqual(rubber_band_pixels(1, 1, bytes([255,255,255,255]), "rubber-open")["status"], "passed")
        with self.assertRaises(ValueError):
            rubber_band_pixels(0, 1, b"", "rubber-open")
