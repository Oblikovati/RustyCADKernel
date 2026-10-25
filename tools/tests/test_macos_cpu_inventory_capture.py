"""Test transport receipts without a native build, network request or baseline patch."""
from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "ci"))
from capture_macos_cpu_inventory import (
    BASE_SHA, PRODUCT_SHA, InventoryCommandCapture, capture_inventory_evidence,
    inventory_capture_environment, inventory_checks_command, inventory_observations,
    inventory_tracked_seal,
)


class FrozenInventoryProcesses(InventoryCommandCapture):
    """A named fixed product/compiler fixture, with no external process execution."""
    command_status = 1
    compiler_host = "aarch64-apple-darwin"
    mutate_source = False

    def checked(self, arguments: list[str]) -> bytes:
        self.records.append(dict(argv=arguments, returncode=0))
        if arguments[:3] == ["git", "rev-parse", "HEAD"]:
            return (PRODUCT_SHA + "\n").encode()
        if arguments[:2] == ["git", "merge-base"]:
            return (BASE_SHA + "\n").encode()
        if arguments[:3] == ["git", "ls-files", "--stage"]:
            return b"100644 abc 0\tinput.rs\0"
        if arguments[0] == "rustc":
            return ("rustc 1.99.0 fixture\nhost: " + self.compiler_host + "\n").encode()
        return b""

    def run(self, arguments: list[str]) -> subprocess.CompletedProcess[bytes]:
        self.records.append(dict(argv=arguments, returncode=self.command_status))
        destination = self.product / "target/unsafe-inventory"
        destination.mkdir(parents=True)
        report = dict(target=self.compiler_host, provenance=dict(compiler="rustc 1.99.0 fixture"),
                      diagnostics=dict(workspace_summary=[dict(observed=1)]))
        (destination / "latest.json").write_text(json.dumps(report))
        (destination / "compiler.stdout.jsonl").write_text('{"reason":"build-finished","success":true}\n')
        (destination / "compiler.stderr.txt").write_bytes(b"actual comparator output\r\n")
        if self.mutate_source:
            (self.product / "input.rs").write_text("changed")
        return subprocess.CompletedProcess(arguments, self.command_status, b"", b"")


class UnexpectedPassingProcesses(FrozenInventoryProcesses):
    command_status = 0


class MutatingInventoryProcesses(FrozenInventoryProcesses):
    mutate_source = True


class WrongHostInventoryProcesses(FrozenInventoryProcesses):
    compiler_host = "x86_64-apple-darwin"


class InventoryProductFixture:
    """Materializes a small named source tree before the simulated checkout command."""
    @staticmethod
    def prepare(product: Path) -> None:
        """Provide owned source/config fixtures; call prepare(product) before capture."""
        (product / "input.rs").write_text("original")
        (product / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.99.0"\n')
        directory = product / "docs/features/safe-toolkits/compiler-baselines"
        directory.mkdir(parents=True)
        baseline = dict(target="aarch64-apple-darwin", compiler="rustc 1.99.0 fixture", summary=[])
        (directory / "aarch64-apple-darwin.json").write_text(json.dumps(baseline))


class PreparedInventoryProcesses(FrozenInventoryProcesses):
    def __init__(self, product: Path, output: Path, environment: dict[str, str]) -> None:
        """Bind a named on-disk fixture; instantiate with fresh isolated directories."""
        super().__init__(product, output, environment)
        InventoryProductFixture.prepare(product)


class PreparedPassingProcesses(PreparedInventoryProcesses, UnexpectedPassingProcesses):
    command_status = 0


class PreparedMutatingProcesses(PreparedInventoryProcesses, MutatingInventoryProcesses):
    mutate_source = True


class PreparedWrongHostProcesses(PreparedInventoryProcesses, WrongHostInventoryProcesses):
    compiler_host = "x86_64-apple-darwin"


class GitlinkInventoryProcesses(PreparedInventoryProcesses):
    fixture_commit = "1" * 40

    def checked(self, arguments: list[str]) -> bytes:
        """Return one named gitlink identity; delegate other recorded fixture queries."""
        if "-C" in arguments and arguments[-2:] == ["rev-parse", "HEAD"]:
            return (self.fixture_commit + "\n").encode()
        return super().checked(arguments)


class MalformedReportProcesses(PreparedInventoryProcesses):
    def run(self, arguments: list[str]) -> subprocess.CompletedProcess[bytes]:
        """Keep a raw malformed report to exercise failed retention, without replacing it."""
        result = super().run(arguments)
        (self.product / "target/unsafe-inventory/latest.json").write_bytes(b"not-json")
        return result


class RecordedInventoryProgram:
    @staticmethod
    def command(status: int) -> list[str]:
        """Build a controlled byte/status probe; call command(7) to verify failure retention."""
        program = "import sys;sys.stdout.buffer.write(b'raw\\r\\n');sys.exit(" + str(status) + ")"
        return [sys.executable, "-c", program]


class MacInventoryCaptureTests(unittest.TestCase):
    def capture_fixture(self, process_type: type[InventoryCommandCapture]) -> tuple[int, dict[str, object]]:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            status = capture_inventory_evidence(root / "product", root / "evidence", {}, "arm64", process_type)
            receipt = json.loads((root / "evidence/receipt.json").read_text())
            return status, receipt

    def test_comparator_failure_is_retained_without_app_suites_or_acceptance(self) -> None:
        status, receipt = self.capture_fixture(PreparedInventoryProcesses)
        self.assertEqual(status, 1)
        self.assertEqual(receipt["command_returncode"], 1)
        self.assertTrue(receipt["reports"]["all_three_present"])
        self.assertTrue(receipt["observations"]["compiler_success"])
        self.assertEqual(receipt["before"], receipt["after"])
        self.assertFalse(receipt["expectations_accepted"])
        self.assertEqual(receipt["observations"]["observed_baseline"]["verdict"], "mismatch")
        self.assertIn("--checks-only", receipt["command"])

    def test_unexpected_gate_success_does_not_admit_baselines(self) -> None:
        status, receipt = self.capture_fixture(PreparedPassingProcesses)
        self.assertEqual(receipt["command_returncode"], 0)
        self.assertEqual(status, 1)
        self.assertFalse(receipt["expectations_accepted"])
        self.assertEqual(receipt["observations"]["observed_baseline"]["verdict"], "mismatch")
        self.assertIn("--checks-only", receipt["command"])

    def test_changed_source_and_wrong_host_remain_explicit_failures(self) -> None:
        for fixture in (PreparedMutatingProcesses, PreparedWrongHostProcesses):
            status, receipt = self.capture_fixture(fixture)
            self.assertEqual(status, 1)
            self.assertIn("capture_error", receipt)
            self.assertFalse(receipt["expectations_accepted"])

    def test_credentials_are_removed_from_each_cargo_environment(self) -> None:
        source = dict(GH_TOKEN="secret", GITHUB_TOKEN="secret", OTHER_TOKEN="secret",
                      MARTENSITE_RELEASE_HISTORY_SHA256="pin", RUSTFLAGS="old", PATH="bin")
        actual = inventory_capture_environment(source, Path("private-build"))
        self.assertFalse(any("TOKEN" in key for key in actual))
        self.assertNotIn("RUSTFLAGS", actual)
        self.assertNotIn("MARTENSITE_RELEASE_HISTORY_SHA256", actual)
        self.assertEqual(actual["MARTENSITE_CI_MODE"], "pr")
        self.assertEqual(actual["GITHUB_EVENT_NAME"], "workflow_dispatch")

    def test_command_uses_existing_supported_checks_only_driver(self) -> None:
        self.assertEqual(inventory_checks_command("aarch64-apple-darwin"), [
            "cargo", "+1.99.0", "xtask", "ci", "run", "--target", "aarch64-apple-darwin", "--checks-only"])
        with tempfile.TemporaryDirectory() as directory:
            self.assertFalse(inventory_observations(Path(directory))["report_present"])


    def test_raw_process_status_and_bytes_are_preserved(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            commands = InventoryCommandCapture(root, root, os.environ)
            result = commands.run(RecordedInventoryProgram.command(7))
            self.assertEqual(result.returncode, 7)
            self.assertEqual((root / "command-000.stdout.log").read_bytes(), b"raw\r\n")
            with self.assertRaisesRegex(RuntimeError, "returned 3; expected zero"):
                commands.checked(RecordedInventoryProgram.command(3))

    def test_gitlinks_symlinks_and_unmerged_entries_are_explicit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            commands = GitlinkInventoryProcesses(root, root, {})
            name, seal = inventory_tracked_seal(commands, ("160000 " + "1" * 40 + " 0\texternal/fixture").encode())
            self.assertEqual((name, seal), ("external/fixture", "gitlink:" + "1" * 40))
            (root / "link").symlink_to("/unreadable-target-is-not-followed")
            _, seal = inventory_tracked_seal(commands, b"120000 abc 0\tlink")
            self.assertEqual(seal, hashlib.sha256(b"/unreadable-target-is-not-followed").hexdigest())
            with self.assertRaisesRegex(ValueError, "expected zero"):
                inventory_tracked_seal(commands, b"100644 abc 1\tinput.rs")

    def test_malformed_reports_retain_actual_failure_and_never_gain_acceptance(self) -> None:
        status, receipt = self.capture_fixture(MalformedReportProcesses)
        self.assertEqual((status, receipt["command_returncode"]), (1, 1))
        self.assertTrue(receipt["reports"]["all_three_present"])
        self.assertIn("retention_error", receipt)
        self.assertFalse(receipt["expectations_accepted"])

    def test_workflow_keeps_old_default_and_an_opt_in_checks_only_capture(self) -> None:
        workflow = Path(__file__).resolve().parents[2] / ".github/workflows/native-platform-oracle.yml"
        text = workflow.read_text()
        self.assertIn("default: rust-controls", text)
        self.assertIn("runner: [macos-latest, macos-26-intel]", text)
        self.assertIn("run: python3 tools/ci/capture_macos_cpu_rust.py", text)
        self.assertIn("if: inputs.capture == 'compiler-inventory'", text)
        self.assertIn("persist-credentials: false", text)
        self.assertIn("target/macos-cpu-pipe/compiler-inventory/", text)


if __name__ == "__main__":
    unittest.main()
