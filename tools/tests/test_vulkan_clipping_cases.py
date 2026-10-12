"""The native matrix exercises every retained clipping fixture on both device profiles."""
from pathlib import Path
import re
import unittest
from vulkan_clipping_cases import CLIPPING_MODES
from verify_vulkan import CASES

class ClippingCaptureCasesTests(unittest.TestCase):
    def test_rust_fixture_and_python_matrix_have_identical_modes(self) -> None:
        root = Path(__file__).resolve().parents[2]
        source = (root / 'crates/vulkan/tests/support/clipping_scene.rs').read_text()
        declared = source.split('pub const MODES:', 1)[1].split('];', 1)[0]
        self.assertEqual(tuple(re.findall(r'"(clip-[a-z0-9-]+)"', declared)), CLIPPING_MODES)
        self.assertEqual(len(CLIPPING_MODES), len(set(CLIPPING_MODES)))
        self.assertEqual(len(CASES) * len(CLIPPING_MODES), 174)

    def test_ownership_chains_live_changes_and_all_primitive_families_are_in_matrix(self) -> None:
        self.assertTrue({'clip-local', 'clip-override', 'clip-empty-override',
                         'clip-intersection', 'clip-union', 'clip-two-chains',
                         'clip-head-off', 'clip-child-off', 'clip-equation-updated',
                         'clip-cleared', 'clip-group-transform', 'clip-lines',
                         'clip-points', 'clip-outside', 'clip-inside',
                         'clip-derived-source', 'clip-derived-alias-clear'} <= set(CLIPPING_MODES))

class ClippingFontPolicyTests(unittest.TestCase):
    def test_white_glyphs_ignore_fragment_planes_but_obey_structure_culling(self) -> None:
        from vulkan_clipping_cases import font_plane_policy
        baseline = bytes([255, 80, 80, 255, 255, 0, 0, 255])
        cut = bytes([80, 80, 255, 255, 0, 0, 255, 255])
        empty = bytes([0, 0, 255, 255] * 2)
        result = font_plane_policy(baseline, cut, cut, empty)
        self.assertEqual(result['status'], 'passed')
        self.assertEqual(result['baseline_glyph_pixels'], 1)
        for buffers in ((empty, empty, empty, empty),
                        (baseline, empty, cut, empty),
                        (baseline, cut, empty, empty),
                        (baseline, cut, cut, baseline)):
            self.assertEqual(font_plane_policy(*buffers)['status'], 'failed')

    def test_incomplete_rgba_buffers_cannot_pass_the_policy_diagnostic(self) -> None:
        from vulkan_clipping_cases import font_plane_policy
        for buffers in ((b'',) * 4, (bytes(3),) * 4,
                        (bytes(4), bytes(8), bytes(4), bytes(4))):
            with self.assertRaisesRegex(ValueError, 'equal nonempty complete pixels'):
                font_plane_policy(*buffers)
