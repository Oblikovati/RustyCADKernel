"""Native captures must never originate from the Rust test's output."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location('platform_geometry', Path(__file__).resolve().parents[1] / 'ci/capture_platform_geometry.py')
GEOMETRY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GEOMETRY)


class FakeGeometryProcesses:
    """Capture the native build commands and write a named executable."""
    def __init__(self, root: Path) -> None:
        self.root = root
        self.commands: list[list[str]] = []

    def run(self, command: list[str], *, check: bool) -> None:
        self.commands.append(command)
        if command[1] == '--build':
            folder = Path(command[2])
            folder.mkdir(parents=True, exist_ok=True)
            (folder / f'native-{folder.parent.name}').write_bytes(b'native-only')

    def check_output(self, command: list[str], *, env: dict[str, str]) -> bytes:
        self.commands.append(command)
        return b'independent native rows\n'


class NativeGeometryBuildFixture:
    def __init__(self, root: Path) -> None:
        self.native = root / 'target/native-platform'
        self.include = self.native / 'inc'
        self.include.mkdir(parents=True)
        (self.include / 'BRepAlgoAPI_Fuse.hxx').write_bytes(b'native header')
        for name in 'TKBO TKMesh TKPrim TKShHealing TKTopAlgo TKGeomAlgo TKBRep TKGeomBase TKG3d TKG2d TKMath TKernel'.split():
            (self.native / f'lib{name}.so').write_bytes(b'')


class FakeRustGeometryProcesses:
    def __init__(self, source: Path) -> None:
        self.source = source

    def run(self, command: list[str], **options: object) -> subprocess.CompletedProcess[str]:
        assert 'MARTENSITE_CYLINDER_FACE' in self.source.read_text()
        return subprocess.CompletedProcess(command, 101)


class FakeBooleanSnapshotCaptures:
    def __init__(self) -> None:
        self.native_names: list[str] = []

    def capture_probe(self, root: Path, output: Path, name: str, source: Path) -> bytes:
        self.native_names.append(name)
        return b'independent native snapshot\n'

    def capture_rust(self, root: Path, output: Path, package: str, target: str, marker: str, addition: str) -> int:
        assert 'MARTENSITE_SNAPSHOT_BEGIN' in addition
        return 101


class PlatformGeometryCaptureTests(unittest.TestCase):
    def test_native_probe_links_pinned_libraries_and_returns_only_native_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            fixture = NativeGeometryBuildFixture(root)
            process = FakeGeometryProcesses(root)
            with patch.object(GEOMETRY.platform, 'system', return_value='Linux'), patch.object(GEOMETRY.subprocess, 'run', process.run), patch.object(GEOMETRY.subprocess, 'check_output', process.check_output):
                result = GEOMETRY.capture_probe(root, root / 'capture', 'torus', root / 'native.cxx')
            self.assertEqual(result, b'independent native rows\n')
            project = (root / 'capture/torus/CMakeLists.txt').read_text()
            self.assertIn('libTKBO.so', project)
            self.assertIn('native.cxx', project)
            self.assertIn(f'"{fixture.include.as_posix()}"', project)
            self.assertNotIn('cargo', project)
            self.assertEqual(len(process.commands), 3)

    def test_cylinder_capture_does_not_require_boolean_libraries(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            fixture = NativeGeometryBuildFixture(root)
            (fixture.native / 'libTKBO.so').unlink()
            process = FakeGeometryProcesses(root)
            with patch.object(GEOMETRY.platform, 'system', return_value='Linux'), patch.object(GEOMETRY.subprocess, 'run', process.run), patch.object(GEOMETRY.subprocess, 'check_output', process.check_output):
                result = GEOMETRY.capture_probe(root, root / 'capture', 'cylinder', root / 'native.cxx')
            self.assertEqual(result, b'independent native rows\n')
            self.assertNotIn('TKBO', (root / 'capture/cylinder/CMakeLists.txt').read_text())

    def test_all_face_capture_retains_failure_and_restores_original_assertions(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            source = root / 'crates/tkmesh/tests/brep_mesh_planar_face_discret.rs'
            source.parent.mkdir(parents=True)
            original = '    let mut faces = Explorer::new();\n'
            source.write_text(original)
            process = FakeRustGeometryProcesses(source)
            with patch.object(GEOMETRY.subprocess, 'run', process.run):
                status = GEOMETRY.capture_cylinder_faces(root, root)
            self.assertEqual(status, 101)
            self.assertEqual(source.read_text(), original)

    def test_boolean_snapshots_keep_native_bytes_and_all_original_verdicts(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            processes = FakeBooleanSnapshotCaptures()
            with patch.object(GEOMETRY, 'capture_probe', processes.capture_probe), patch.object(GEOMETRY, 'capture_rust', processes.capture_rust):
                statuses = GEOMETRY.capture_boolean_snapshots(root, root)
            self.assertEqual(len(statuses), 4)
            self.assertEqual(set(statuses.values()), {101})
            for name in processes.native_names:
                self.assertEqual((root / f'{name}.expected').read_bytes(), b'independent native snapshot\n')

    def test_probe_contraction_policy_is_explicit_and_rejects_unknown_values(self) -> None:
        for policy in ('off', 'on', 'fast'):
            with patch.dict(GEOMETRY.os.environ, {'MARTENSITE_FP_CONTRACT': policy}):
                self.assertEqual(GEOMETRY.probe_compiler_flags(), f'-ffp-contract={policy}')
        with patch.dict(GEOMETRY.os.environ, {'MARTENSITE_FP_CONTRACT': 'invalid'}):
            with self.assertRaisesRegex(ValueError, "invalid.*expected off"):
                GEOMETRY.probe_compiler_flags()

    def test_missing_rust_marker_cannot_silently_change_a_test(self) -> None:
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
