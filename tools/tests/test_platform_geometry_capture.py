"""Native captures must never originate from the Rust test's output."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('platform_geometry', Path(__file__).resolve().parents[1] / 'ci/capture_platform_geometry.py')
GEOMETRY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEOMETRY)


class FakeGeometryProcesses:
    """Capture the native build commands and write a named executable."""
    def __init__(self, root: Path):
        self.root = root
        self.commands: list[list[str]] = []

    def run(self, command: list[str], **kwargs):
        self.commands.append(command)
        if command[1] == '--build':
            folder = Path(command[2])
            folder.mkdir(parents=True, exist_ok=True)
            (folder / 'native-torus').write_bytes(b'native-only')

    def check_output(self, command: list[str], **kwargs) -> bytes:
        self.commands.append(command)
        return b'independent native rows\n'


class PlatformGeometryCaptureTests(unittest.TestCase):
    def test_native_probe_links_pinned_libraries_and_returns_only_native_bytes(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            native = root / 'target/native-platform'
            native.mkdir(parents=True)
            for name in 'TKBO TKMesh TKPrim TKShHealing TKTopAlgo TKGeomAlgo TKBRep TKGeomBase TKG3d TKG2d TKMath TKernel'.split():
                (native / f'lib{name}.so').write_bytes(b'')
            process = FakeGeometryProcesses(root)
            with patch.object(GEOMETRY.platform, 'system', return_value='Linux'), patch.object(GEOMETRY.subprocess, 'run', process.run), patch.object(GEOMETRY.subprocess, 'check_output', process.check_output):
                result = GEOMETRY.capture_probe(root, root / 'capture', 'torus', root / 'native.cxx')
            self.assertEqual(result, b'independent native rows\n')
            project = (root / 'capture/torus/CMakeLists.txt').read_text()
            self.assertIn('libTKBO.so', project)
            self.assertIn('native.cxx', project)
            self.assertNotIn('cargo', project)
            self.assertEqual(len(process.commands), 3)

    def test_missing_rust_marker_cannot_silently_change_a_test(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / 'crates/tkbo/tests/torus.rs'
            source.parent.mkdir(parents=True)
            source.write_text('untouched test')
            with self.assertRaisesRegex(ValueError, 'marker count 0; expected one'):
                GEOMETRY.capture_rust(root, root, 'tkbo', 'torus', 'missing marker', 'print')
            self.assertEqual(source.read_text(), 'untouched test')


if __name__ == '__main__':
    unittest.main()
