"""Trihedron verification retains functional, native and timing obligations."""
import unittest
from pathlib import Path
from verify_vulkan import capture_modes, parse_arguments
from vulkan_case_dispatch import capture_executable
from vulkan_ais_trihedron_cases import AIS_TRIHEDRON_MODES, trihedron_pixels
from tkv3d_corpus_ais_trihedron import append_trihedron_requests


class TrihedronCaseTests(unittest.TestCase):
    def test_capture_family_routes_every_mode_and_is_in_full_corpus(self) -> None:
        self.assertEqual(capture_modes("ais-trihedron"), AIS_TRIHEDRON_MODES)
        self.assertTrue(set(AIS_TRIHEDRON_MODES).issubset(capture_modes("all")))
        for mode in AIS_TRIHEDRON_MODES:
            self.assertEqual(capture_executable(Path("examples/capture"),mode),Path("examples/ais_trihedron_capture"))
        args=parse_arguments(["--source","OCCT","--build","build","--output","receipt.json","--case-family","ais-trihedron"])
        self.assertEqual(args.case_family,"ais-trihedron")

    def test_native_rows_are_independent_and_cover_cached_selection(self) -> None:
        rows: list[str]=[]
        append_trihedron_requests(rows)
        self.assertEqual(len(rows),48)
        self.assertEqual(len(set(rows)),48)
        for row in rows:
            tokens=row.split()
            self.assertEqual(int(tokens[2]),len(tokens[4:]))
        self.assertTrue(any(" 14 " in row for row in rows))

    def test_roles_reject_blank_geometry_and_unchanged_selection(self) -> None:
        for mode in AIS_TRIHEDRON_MODES:
            self.assertEqual(trihedron_pixels(1,1,bytes([0,0,255,255]),mode,bytes([0,0,255,255]))["status"],"failed")
        self.assertEqual(trihedron_pixels(1,1,bytes([84,57,7,255]),"tri-wire")["status"],"passed")
        ordinary=bytes([255,255,0,255])
        self.assertEqual(trihedron_pixels(1,1,ordinary,"tri-axis",ordinary)["status"],"failed")
        self.assertEqual(trihedron_pixels(1,1,bytes([204,204,204,255]),"tri-selected",ordinary)["status"],"passed")
        self.assertEqual(trihedron_pixels(1,1,bytes([150,150,150,255]),"tri-shaded-axis",ordinary)["status"],"passed")
        self.assertEqual(trihedron_pixels(1,1,ordinary,"tri-shaded-origin",ordinary)["status"],"failed")
        with self.assertRaises(ValueError):
            trihedron_pixels(1,1,ordinary,"tri-selected")
        with self.assertRaises(ValueError):
            trihedron_pixels(0,1,b"","tri-axis")
