"""Porting-only capture integrity; Rust tests retain the behavioral regressions."""
import gzip
import hashlib
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
PROBES = {
    'torus': ('bop_torus', 'bop_algo_bop_torus'),
    'cells_builder': ('cells_builder', 'bop_algo_cells_builder'),
    'edges_to_wires': ('edges_to_wires', 'bop_algo_edges_to_wires'),
    'defeaturing': ('defeaturing', 'brep_algo_api_defeaturing'),
}


class NativeOracleCaptureFixture:
    """Read the committed native receipt and its separate Rust diagnostics."""

    def __init__(self, host: str) -> None:
        self.host = host
        self.evidence = ROOT / 'porting/runtime-evidence/release-ci-2026-10-11' / f'{host}-full-geometry'
        self.sha256: dict[str, str] = json.loads((self.evidence / 'receipt.json').read_text())['sha256']

    def native_bytes(self, filename: str) -> bytes:
        return (self.evidence / filename).read_bytes()

    def rust_diagnostic(self, target: str) -> str:
        return gzip.decompress((self.evidence / f'{target}.log.gz').read_bytes()).decode('utf-8').replace('\r\n', '\n')


class HostGeometryAssetTests(unittest.TestCase):
    def test_boolean_assets_retain_raw_native_bytes_and_receipt_hashes(self) -> None:
        for host in ('macos', 'windows'):
            fixture = NativeOracleCaptureFixture(host)
            for name, (probe, _) in PROBES.items():
                asset = (ROOT / f'crates/tkbo/tests/fixtures/tkbo_{probe}.{host}.expected').read_bytes()
                self.assertEqual(asset, fixture.native_bytes(f'{name}.expected'))
                self.assertEqual(hashlib.sha256(asset).hexdigest(), fixture.sha256[f'{name}.expected'])

    def test_retained_rust_rows_match_independent_native_rows_exactly(self) -> None:
        for host in ('macos', 'windows'):
            fixture = NativeOracleCaptureFixture(host)
            for name, (_, target) in PROBES.items():
                log = fixture.rust_diagnostic(target)
                rows = log.split('MARTENSITE_SNAPSHOT_BEGIN\n', 1)[1].split('MARTENSITE_SNAPSHOT_END', 1)[0]
                native = fixture.native_bytes(f'{name}.expected').decode('utf-8').replace('\r\n', '\n')
                self.assertEqual(rows, native, f'{host} {name}')

    def test_all_cylinder_face_assets_retain_native_node_and_triangle_hashes(self) -> None:
        for host in ('macos', 'windows'):
            fixture = NativeOracleCaptureFixture(host)
            asset = (ROOT / f'crates/tkmesh/tests/fixtures/tkmesh_cylinder_nodes.{host}.expected').read_bytes()
            self.assertEqual(asset, fixture.native_bytes('cylinder.expected'))
            self.assertEqual(hashlib.sha256(asset).hexdigest(), fixture.sha256['cylinder.expected'])
            native = re.findall(r'^hash (\d+) ([0-9a-f]+) ([0-9a-f]+)$', asset.decode().replace('\r\n', '\n'), re.M)
            actual = re.findall(r'MARTENSITE_CYLINDER_FACE=(\d+) NODES=([0-9a-f]+) TRIANGLES=([0-9a-f]+)', fixture.rust_diagnostic('brep_mesh_planar_face_discret'))
            self.assertEqual(len(native), 3)
            self.assertEqual(actual, native)


if __name__ == '__main__':
    unittest.main()
