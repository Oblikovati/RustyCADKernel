#!/usr/bin/env python3
"""Martensite lit-marker regression independent of deferred point raster placement.

Example: python verify_lit_palette.py --receipt renderer.json --captures captures
"""
import argparse
from collections import Counter
import gzip
import json
from pathlib import Path
import unittest


def marker_palette(rgba: bytes) -> Counter[tuple[int, ...]]:
    """Count exact visible RGBA values; marker_palette(bytes([255,0,0,255]))."""
    if not rgba or len(rgba) % 4:
        raise ValueError(f"received RGBA length {len(rgba)}; expected nonempty four-byte pixels")
    palette = Counter(tuple(rgba[i:i + 4]) for i in range(0, len(rgba), 4))
    palette.pop((0, 0, 255, 255), None)
    return palette


def load_receipt(path: Path) -> dict[str, object]:
    """Read the retained capture receipt; load_receipt(Path('renderer.json.gz'))."""
    text = gzip.decompress(path.read_bytes()) if path.suffix == '.gz' else path.read_bytes()
    return json.loads(text)


def compare_lit_case(captures: Path, profile: str, case: dict[str, object]) -> dict[str, object]:
    """Check exact colors/counts without relocating pixels; compare_lit_case(root, 'host', row)."""
    palettes = [marker_palette((captures / profile / (Path(case[key]).name + '.rgba')).read_bytes())
                for key in ('native_capture', 'ported_capture')]
    return {'profile': profile, 'case': case['case'], 'passed': palettes[0] == palettes[1],
            'native_palette': sorted(palettes[0].items()), 'vulkan_palette': sorted(palettes[1].items())}


def verify_lit_receipt(receipt: dict[str, object], captures: Path) -> list[dict[str, object]]:
    """Require all 60 paired lighting cases; verify_lit_receipt(receipt, Path('captures'))."""
    rows = []
    profiles = receipt['profiles']
    assert {profile['profile'] for profile in profiles} == {'lavapipe', 'host'}
    for profile in profiles:
        lit = [row for row in profile['cases'] if 'cloud-lit-' in row['case']]
        assert len(lit) == 30, f"received {len(lit)} lit cases; expected 5 modes x 6 configurations"
        rows.extend(compare_lit_case(captures, profile['profile'], case) for case in lit)
    assert all(row['passed'] for row in rows), 'native/Vulkan exact marker colors or counts differ'
    return rows


class MarkerPaletteRegressionTests(unittest.TestCase):
    def test_pixel_reordering_preserves_exact_color_counts(self) -> None:
        first = bytes([255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255])
        second = bytes([0, 255, 0, 255, 255, 0, 0, 255, 0, 0, 255, 255])
        self.assertEqual(marker_palette(first), marker_palette(second))

    def test_white_ambient_fixture_error_and_invalid_shapes_are_rejected(self) -> None:
        self.assertNotEqual(marker_palette(bytes([84, 57, 7, 255])), marker_palette(bytes([255, 208, 126, 255])))
        for rgba in (b'', b'123'):
            with self.assertRaises(ValueError):
                marker_palette(rgba)


def main() -> None:
    """Run the independent functional regression; main() uses explicit artifact paths."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--receipt', type=Path, required=True)
    parser.add_argument('--captures', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(MarkerPaletteRegressionTests)
    assert unittest.TextTestRunner().run(suite).wasSuccessful()
    rows = verify_lit_receipt(load_receipt(args.receipt), args.captures)
    args.output.write_text(json.dumps(rows, indent=2) + '\n')
    print(f'PASS: {len(rows)} exact native/Vulkan lit-marker color/count comparisons; strict raster gates unchanged')


if __name__ == '__main__':
    main()
