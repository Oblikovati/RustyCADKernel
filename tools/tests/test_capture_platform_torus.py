"""Diagnostic capture preserves native rows and restores the Rust test after failure."""

from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from ci import capture_platform_torus as capture


class OracleCaptureFixture:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.output = root / "target/platform-torus"
        self.output.mkdir(parents=True)
        self.source = root / "crates/tkbo/tests/bop_algo_bop_torus.rs"
        self.source.parent.mkdir(parents=True)
        self.original = '    let expected = include_str!("fixture");\n'
        self.source.write_text(self.original)
        library = root / "target/native-platform/mac/clang/lib/libTKBO.dylib"
        library.parent.mkdir(parents=True)
        library.touch()


class FakeOracleProcesses:
    def __init__(self, fixture: OracleCaptureFixture) -> None:
        self.fixture = fixture
        self.crash = False
        self.commands: list[list[str]] = []

    def run(
        self, command: list[str], **options: object
    ) -> subprocess.CompletedProcess[str]:
        self.commands.append(command)
        if command[0] == "cargo":
            assert "MARTENSITE_TORUS_BEGIN" in self.fixture.source.read_text()
            if self.crash:
                raise OSError("fake process launch failed")
            return subprocess.CompletedProcess(command, 101)
        return subprocess.CompletedProcess(command, 0)

    def check_output(self, command: list[str], **options: object) -> bytes:
        self.commands.append(command)
        return b"done 1 warnings 0\n"


class PlatformOracleCaptureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.fixture = OracleCaptureFixture(Path(self.temporary.name))
        self.processes = FakeOracleProcesses(self.fixture)

    def test_native_rows_are_captured_without_normalization(self) -> None:
        with patch.object(capture.subprocess, "run", self.processes.run), patch.object(
            capture.subprocess, "check_output", self.processes.check_output
        ):
            executable = capture.capture_native(self.fixture.root, self.fixture.output)
        self.assertEqual(executable, self.fixture.output / "native-torus")
        self.assertEqual(
            (self.fixture.output / "native.expected").read_bytes(),
            b"done 1 warnings 0\n",
        )
        self.assertIn("-O2", self.processes.commands[0])
        self.assertTrue(
            any(arg.endswith("libTKBO.dylib") for arg in self.processes.commands[0])
        )

    def test_failed_rust_assertion_is_retained_and_source_restored(self) -> None:
        with patch.object(capture.subprocess, "run", self.processes.run):
            status = capture.capture_rust(self.fixture.root, self.fixture.output)
        self.assertEqual(status, 101)
        self.assertEqual(self.fixture.source.read_text(), self.fixture.original)

    def test_process_launch_failure_also_restores_source(self) -> None:
        self.processes.crash = True
        with patch.object(
            capture.subprocess, "run", self.processes.run
        ), self.assertRaises(OSError):
            capture.capture_rust(self.fixture.root, self.fixture.output)
        self.assertEqual(self.fixture.source.read_text(), self.fixture.original)

    def test_changed_fixture_contract_is_rejected_before_mutation(self) -> None:
        self.fixture.source.write_text("unexpected fixture\n")
        with self.assertRaisesRegex(ValueError, "expected exactly one"):
            capture.capture_rust(self.fixture.root, self.fixture.output)
        self.assertEqual(self.fixture.source.read_text(), "unexpected fixture\n")
