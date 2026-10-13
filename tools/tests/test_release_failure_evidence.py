"""Cold red receipts survive stage cleanup without changing release acceptance."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from release_failure_commands import LOG_LIMIT, ReleaseDiagnosticCommands
from release_failure_inputs import (
    declared_driver_files,
    driver_manifest_identity,
    resolved_library_probe,
    failed_executables,
    file_identity,
    host_metadata,
    library_metadata,
    stage_hashes,
)
from release_inventory import SystemCommands
from release_packages import build_release, parse_release_arguments
from test_release_packages import FakeCommands, ReleaseFixture


class ColdFailingCommands(FakeCommands):
    """First extracted test fails; creates the real reported executable before failing."""

    def check_isolated_workspace(self, root: Path) -> None:
        super().check_isolated_workspace(root)
        executable = root / "target/debug/deps/cold-test"
        executable.parent.mkdir(parents=True)
        executable.write_bytes(b"fake failed executable")
        self.failed_stage = root
        stdout = "test exact_renderer_contract ... FAILED\n"
        stderr = (
            f"process didn't exit successfully: `{executable}` (exit status: 101)\n"
        )
        self.failure = subprocess.CalledProcessError(
            101, ["cargo", "test"], stdout, stderr
        )
        raise self.failure


class NamedLibraryProbe:
    """No driver loading: deterministic resolved-library metadata for a fake executable."""

    def __init__(self, library: Path) -> None:
        # The probe owns no real process or graphics driver state.
        self.library = library
        self.calls: list[list[str]] = []

    def run(
        self, arguments: list[str], **options: object
    ) -> subprocess.CompletedProcess[str]:
        self.calls.append(arguments)
        return subprocess.CompletedProcess(
            arguments, 0, f"libtest.so => {self.library} (0x1)\n", ""
        )


class OutputCommands:
    """Named external command fake records successful stderr as well as stdout."""

    last_stderr = "successful warning\n"

    def run(self, arguments: list[str], root: Path) -> str:
        self.arguments = arguments
        self.root = root
        return "successful result\n"


class UnwritableDiagnosticCommands(ReleaseDiagnosticCommands):
    """Named receipt I/O refusal must never hide the original test failure."""

    def record(
        self, arguments: list[str], root: Path, stdout: str, stderr: str, exit_code: int
    ) -> None:
        # Simulate a full diagnostic volume, independently of the cold test runner.
        raise OSError("diagnostic volume full")


class MissingLibraryProbe:
    """Named absent ldd command remains diagnostic rather than replacing the red."""

    def run(
        self, arguments: list[str], **options: object
    ) -> subprocess.CompletedProcess[str]:
        # A runner without ELF tooling still keeps source/executable identity receipts.
        raise FileNotFoundError("ldd unavailable")


class ReleaseFailureEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.fixture = ReleaseFixture(self.root)
        self.library = self.root / "libtest.so"
        self.library.write_bytes(b"resolved library bytes")

    def test_cold_failure_identity_survives_stage_cleanup_without_sealing(self) -> None:
        runner = ColdFailingCommands(self.fixture)
        probe = NamedLibraryProbe(self.library)
        with patch("release_failure_inputs.subprocess.run", probe.run):
            with self.assertRaises(subprocess.CalledProcessError) as failure:
                build_release(self.root, "v0.1.0", runner)
        self.assertIs(failure.exception, runner.failure)
        self.assertFalse(runner.failed_stage.exists())
        self.assertFalse((self.root / "target/releases/v0.1.0").exists())
        self.assertEqual(sum(call[:2] == ["cargo", "test"] for call in runner.calls), 1)
        folder = next((self.root / "target/release-diagnostics").glob("*/commands"))
        self.assert_receipts(folder, runner)
        self.assertEqual(probe.calls[0][0], "ldd")

    def assert_receipts(self, folder: Path, runner: ColdFailingCommands) -> None:
        command = json.loads((folder / "command-0001.json").read_text())
        self.assertEqual(command["exit"], 101)
        self.assertEqual(command["cwd"], str(runner.failed_stage))
        self.assertIn("FAILED", (folder / "command-0001-stdout.log").read_text())
        receipt = json.loads((folder / "failure-inputs.json").read_text())
        inputs = {entry["path"]: entry["sha256"] for entry in receipt["inputs"]}
        self.assertIn("Cargo.lock", inputs)
        expected = hashlib.sha256(b"exact oracle result\n").hexdigest()
        self.assertEqual(inputs["tools/oracle/check.expected"], expected)
        executable = receipt["failed_executables"][0]
        self.assertEqual(
            executable["identity"]["sha256"],
            hashlib.sha256(b"fake failed executable").hexdigest(),
        )
        self.assertFalse(any(path.startswith("target/") for path in inputs))
        self.assertFalse(any(path.name == "cold-test" for path in folder.rglob("*")))

    def test_successful_command_preserves_stderr_and_bounds_receipt_streams(
        self,
    ) -> None:
        runner = OutputCommands()
        logged = ReleaseDiagnosticCommands(runner, self.root / "logs")
        self.assertEqual(logged.run(["cold-test"], self.root), "successful result\n")
        self.assertEqual(
            (logged.output / "command-0001-stderr.log").read_text(), runner.last_stderr
        )
        logged.record(["large"], self.root, "x" * (LOG_LIMIT + 1), "", 0)
        receipt = json.loads((logged.output / "command-0001.json").read_text())
        self.assertTrue(receipt["streams"]["stdout"]["truncated"])
        self.assertEqual(
            (logged.output / "command-0001-stdout.log").stat().st_size, LOG_LIMIT
        )

    def test_hashing_excludes_root_build_but_keeps_source_target_fixture(self) -> None:
        stage = self.root / "stage"
        (stage / "target").mkdir(parents=True)
        (stage / "crates/target").mkdir(parents=True)
        (stage / "target/cache").write_bytes(b"build")
        source = stage / "crates/target/fixture"
        source.write_bytes(b"source")
        self.assertEqual(
            stage_hashes(stage), [file_identity(source, "crates/target/fixture")]
        )

    def test_failed_executable_parser_rejects_foreign_paths_and_uses_last_running(
        self,
    ) -> None:
        binary = self.root / "target/debug/deps/failure"
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"binary")
        self.assertEqual(
            failed_executables(self.root, f"Running unit ({binary})\n"), [binary]
        )
        foreign = self.root.parent / "foreign-test"
        self.assertEqual(
            failed_executables(
                self.root, f"process didn't exit successfully: `{foreign}`"
            ),
            [],
        )

    def test_whitelisted_environment_and_library_metadata_do_not_probe_graphics(
        self,
    ) -> None:
        probe = NamedLibraryProbe(self.library)
        with patch.dict(
            os.environ,
            {"SECRET_TOKEN": "never-upload", "VK_DRIVER_FILES": "declared.json"},
        ):
            self.assertEqual(
                host_metadata()["environment"]["VK_DRIVER_FILES"], "declared.json"
            )
            self.assertNotIn("SECRET_TOKEN", host_metadata()["environment"])
        with patch("release_failure_inputs.subprocess.run", probe.run):
            receipt = library_metadata(self.library)
        self.assertEqual(
            receipt["resolved_libraries"][0]["sha256"],
            hashlib.sha256(self.library.read_bytes()).hexdigest(),
        )
        self.assertEqual(probe.calls, [["ldd", str(self.library)]])

    def test_system_boundary_retains_stderr_and_original_failure(self) -> None:
        runner = SystemCommands()
        with self.assertRaises(subprocess.CalledProcessError) as failure:
            runner.run(
                [
                    "python3",
                    "-c",
                    "import sys;print('out');print('err',file=sys.stderr);sys.exit(7)",
                ],
                self.root,
            )
        self.assertEqual(failure.exception.returncode, 7)
        self.assertEqual(failure.exception.output, "out\n")
        self.assertEqual(failure.exception.stderr, "err\n")

    def test_original_failure_survives_diagnostic_volume_refusal(self) -> None:
        runner = ColdFailingCommands(self.fixture)
        logged = UnwritableDiagnosticCommands(runner, self.root / "refused-logs")
        stage = self.root / "stage"
        (stage / "crates/kernel").mkdir(parents=True)
        (stage / "crates/kernel/Cargo.toml").write_text("manifest")
        (stage / "tools/oracle").mkdir(parents=True)
        (stage / "tools/oracle/check.expected").write_text("fixture")
        with self.assertRaises(subprocess.CalledProcessError) as failure:
            logged.run(["cargo", "test"], stage)
        self.assertIs(failure.exception, runner.failure)

    def test_declared_driver_hashes_are_read_without_loading_and_bad_json_is_diagnostic(
        self,
    ) -> None:
        manifest = self.root / "icd.json"
        manifest.write_text(json.dumps({"ICD": {"library_path": str(self.library)}}))
        with patch.dict(os.environ, {"VK_DRIVER_FILES": str(manifest)}):
            receipt = declared_driver_files()[0]
        self.assertEqual(
            receipt["library"], file_identity(self.library, str(self.library))
        )
        manifest.write_text("invalid JSON")
        self.assertIn("error", driver_manifest_identity(manifest))
        manifest.write_text(json.dumps({"ICD": {"library_path": "missing.so"}}))
        self.assertEqual(
            driver_manifest_identity(manifest)["unresolved_library"], "missing.so"
        )

    def test_absent_library_tool_cannot_replace_original_failure(self) -> None:
        probe = MissingLibraryProbe()
        with patch("release_failure_inputs.subprocess.run", probe.run):
            receipt = resolved_library_probe(self.library)
            metadata = library_metadata(self.library)
        self.assertEqual(receipt.returncode, -1)
        self.assertEqual(metadata["resolved_libraries"], [])
        self.assertIn("unavailable", metadata["stderr"])

    def test_cli_defaults_to_tool_checkout_and_selects_explicit_kernel_root(
        self,
    ) -> None:
        default = parse_release_arguments(["--tag", "v0.2.3"])
        self.assertEqual(default.root, Path(__file__).resolve().parents[2])
        self.assertEqual(default.tag, "v0.2.3")
        selected = parse_release_arguments(
            ["--tag", "v0.2.2", "--root", str(self.root)]
        )
        self.assertEqual(selected.root, self.root.resolve())
        self.assertEqual(selected.tag, "v0.2.2")
        self.assertNotEqual(selected.root, default.root)


if __name__ == "__main__":
    unittest.main()
