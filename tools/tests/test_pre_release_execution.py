"""Named command fixtures verify original exits and durable pre-tag evidence."""

import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from typing import BinaryIO

from pre_release_execution import (
    CandidateEnvironmentCommands,
    bounded_ci_environment,
    command_acceptance,
    execute_recorded_command,
    gate_execution_environment,
    native_gate_environment,
    preserve_command_output,
    preserve_failure_diagnostics,
    require_cargo_tool_versions,
    require_native_gate_tools,
    verify_recorded_gate,
    verify_workflow_gates,
)
from release_verification_plan import GateCommand, VerificationGate


class PreReleaseFilesFixture:
    """Own small files used by recorded command tests."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.evidence = root / "evidence"
        self.evidence.mkdir()
        self.log = self.evidence / "command.log"


class FakePreReleaseProcess:
    """Record process arguments and write distinct stdout/stderr bytes."""

    def __init__(self, status: int = 0) -> None:
        self.status = status
        self.calls: list[tuple[tuple[str, ...], Path, dict[str, str]]] = []

    def run(
        self,
        argv: tuple[str, ...] | list[str],
        cwd: Path,
        env: dict[str, str],
        stdout: BinaryIO | int | None = None,
        stderr: BinaryIO | int | None = None,
        text: bool = False,
        capture_output: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        self.calls.append((tuple(argv), cwd, env))
        if "llvm-cov" in argv:
            self.coverage_outputs(cwd)
        if stdout is not None and not isinstance(stdout, int):
            stdout.write(b"actual stdout\n")
        if stderr is not None and not isinstance(stderr, int):
            stderr.write(b"actual stderr\n")
        return subprocess.CompletedProcess(
            argv, self.status, "stdout result", "stderr result"
        )

    def coverage_outputs(self, root: Path) -> None:
        reports = (root / "target/lcov.info", root / "target/nextest/ci/junit.xml")
        for report in reports:
            report.parent.mkdir(parents=True, exist_ok=True)
            report.write_text("named coverage fixture\n")


class FakeCargoVersionReads:
    """Return declared tool versions without installing or invoking Cargo."""

    def __init__(self, wrong: bool = False) -> None:
        self.wrong = wrong
        self.calls: list[list[str]] = []

    def read(self, argv: list[str], cwd: Path, env: dict[str, str], text: bool) -> str:
        self.calls.append(argv)
        versions = {"nextest": "0.9.148", "llvm-cov": "0.9.1", "deny": "0.20.2"}
        return "cargo-" + argv[1] + " " + ("0.0.0" if self.wrong else versions[argv[1]])


class FakeNativeGateTools:
    """Resolve named prerequisite tools without relying on host installations."""

    def __init__(self, missing: str = "") -> None:
        self.missing = missing
        self.paths: list[str | None] = []

    def lookup(self, name: str, mode: int, path: str | None) -> str | None:
        self.paths.append(path)
        return None if name == self.missing else "/fixture/bin/" + name


class PreReleaseExecutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.files = PreReleaseFilesFixture(Path(self.temporary.name))

    def test_original_command_exit_and_overrides_are_preserved(self) -> None:
        process = FakePreReleaseProcess(37)
        command = GateCommand(("cargo", "doc"), (("RUSTDOCFLAGS", "-D warnings"),))
        with patch("pre_release_execution.subprocess.run", new=process.run):
            status = execute_recorded_command(
                command, self.files.root, {"VK_LAYER_PATH": "verified"}, self.files.log
            )
        self.assertEqual(status, 37)
        self.assertEqual(
            process.calls[0][2],
            {"VK_LAYER_PATH": "verified", "RUSTDOCFLAGS": "-D warnings"},
        )
        recorded = command_acceptance(
            command, self.files.log, status, self.files.evidence
        )
        self.assertEqual(
            recorded["sha256"], hashlib.sha256(self.files.log.read_bytes()).hexdigest()
        )
        self.assertEqual(recorded["exit_code"], 37)

    def test_performance_stdout_belongs_to_candidate_not_current_directory(
        self,
    ) -> None:
        process = FakePreReleaseProcess()
        command = GateCommand(("renderer",), stdout_path=Path("target/performance.log"))
        with patch("pre_release_execution.subprocess.run", new=process.run):
            execute_recorded_command(command, self.files.root, {}, self.files.log)
        preserve_command_output(command, self.files.root, self.files.evidence)
        self.assertEqual(
            (self.files.evidence / "outputs/performance.log").read_bytes(),
            b"actual stdout\n",
        )

    def test_first_failed_gate_command_stops_later_commands_and_retains_prefix(
        self,
    ) -> None:
        process = FakePreReleaseProcess(37)
        gate = VerificationGate(
            "rust-prechecks", (GateCommand(("formatter",)), GateCommand(("lint",)))
        )
        with patch("pre_release_execution.subprocess.run", new=process.run):
            with self.assertRaises(subprocess.CalledProcessError) as failure:
                verify_recorded_gate(gate, self.files.root, {}, self.files.evidence)
        self.assertEqual(failure.exception.returncode, 37)
        self.assertEqual(len(process.calls), 1)
        receipt = json.loads((self.files.evidence / "rust-prechecks.json").read_text())
        self.assertEqual((receipt["exit_code"], len(receipt["commands"])), (37, 1))

    def test_failed_performance_output_survives_checkout_disposal(self) -> None:
        process = FakePreReleaseProcess(37)
        command = GateCommand(("renderer",), stdout_path=Path("target/performance.log"))
        gate = VerificationGate("vulkan-performance", (command,))
        with patch("pre_release_execution.subprocess.run", new=process.run):
            with self.assertRaises(subprocess.CalledProcessError) as failure:
                verify_recorded_gate(gate, self.files.root, {}, self.files.evidence)
        self.assertEqual(failure.exception.returncode, 37)
        self.assertEqual(
            (self.files.evidence / "outputs/performance.log").read_bytes(),
            b"actual stdout\n",
        )

    def test_all_planned_commands_are_executed_in_declared_order(self) -> None:
        process = FakePreReleaseProcess()
        shaders = self.files.root / "crates/vulkan/shaders"
        shaders.mkdir(parents=True)
        (shaders / "test.spv").write_bytes(b"fixture")
        with patch("pre_release_execution.subprocess.run", new=process.run):
            outcomes = verify_workflow_gates(
                self.files.root, Path("oracle"), {}, self.files.evidence
            )
        self.assertEqual(len(outcomes), 11)
        self.assertEqual(outcomes[0]["name"], "rust-prechecks")
        self.assertEqual(outcomes[-1]["name"], "tools")
        self.assertTrue(all(outcome["exit_code"] == 0 for outcome in outcomes))

    def test_candidate_commands_retain_both_streams_and_propagate_original_error(
        self,
    ) -> None:
        process = FakePreReleaseProcess(37)
        boundary = CandidateEnvironmentCommands(
            {"VK_LAYER_PATH": "verified"}, self.files.evidence
        )
        with patch("pre_release_execution.subprocess.run", new=process.run):
            with self.assertRaises(subprocess.CalledProcessError) as failure:
                boundary.run(["cargo", "test"], self.files.root)
        self.assertEqual(failure.exception.returncode, 37)
        self.assertEqual(boundary.last_stderr, "stderr result")
        self.assertEqual(
            (self.files.evidence / "package-000.stdout.log").read_text(),
            "stdout result",
        )

    def test_versions_match_workflow_pins_and_wrong_tool_cannot_pass(self) -> None:
        versions = FakeCargoVersionReads()
        with patch("pre_release_execution.subprocess.check_output", new=versions.read):
            require_cargo_tool_versions(self.files.root, {})
        self.assertEqual(len(versions.calls), 3)
        versions.wrong = True
        with patch("pre_release_execution.subprocess.check_output", new=versions.read):
            with self.assertRaisesRegex(ValueError, "expected"):
                require_cargo_tool_versions(self.files.root, {})

    def test_ci_policy_bounds_profiles_and_preserves_loader(self) -> None:
        selected = bounded_ci_environment(
            {
                "VK_LAYER_PATH": "verified",
                "CARGO_TARGET_DIR": "other-source",
                "CARGO_LLVM_COV_TARGET_DIR": "other-coverage",
                "VK_ICD_FILENAMES": "native",
                "LLVM_TARGET_FEATURES": "other-cpu",
            }
        )
        self.assertEqual(selected["VK_LAYER_PATH"], "verified")
        self.assertEqual(selected["LLVM_PROFILE_FILE_NAME"], "martensite-%4m.profraw")
        self.assertEqual(selected["GALLIUM_OVERRIDE_CPU_CAPS"], "sse2")
        for variable in (
            "CARGO_TARGET_DIR",
            "CARGO_LLVM_COV_TARGET_DIR",
            "VK_ICD_FILENAMES",
            "LLVM_TARGET_FEATURES",
        ):
            self.assertNotIn(variable, selected)

    def test_cold_driver_cache_is_independent_of_workspace_consumers(self) -> None:
        environments = {
            name: gate_execution_environment(
                VerificationGate(name, ()), self.files.root, {}
            )
            for name in ("rust-tests", "coverage", "vulkan-cold", "vulkan-performance")
        }
        cold = environments["vulkan-cold"]["MESA_SHADER_CACHE_DIR"]
        self.assertNotEqual(cold, environments["rust-tests"]["MESA_SHADER_CACHE_DIR"])
        self.assertNotEqual(cold, environments["coverage"]["MESA_SHADER_CACHE_DIR"])
        self.assertEqual(
            cold, environments["vulkan-performance"]["MESA_SHADER_CACHE_DIR"]
        )

    def test_inherited_alias_and_toolchain_cannot_replace_ci_checks(self) -> None:
        overrides = {
            "CARGO_ALIAS_LINT": "version",
            "CARGO_ALIAS_TEST_ALL": "version",
            "RUSTUP_TOOLCHAIN": "nightly",
            "CARGO_BUILD_TARGET": "other",
            "RUSTFLAGS": "--cfg bypass",
            "RUSTDOCFLAGS": "--cfg bypass",
            "CARGO_ENCODED_RUSTFLAGS": "--cfg bypass",
        }
        selected = bounded_ci_environment(overrides)
        for variable in overrides:
            self.assertNotIn(variable, selected)

    def test_partial_reports_and_package_failure_receipts_are_durable(self) -> None:
        reports = (
            "target/lcov.info",
            "target/nextest/ci/junit.xml",
            "target/release-diagnostics/failed-command.json",
        )
        for relative in reports:
            source = self.files.root / relative
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_bytes(b"original failure evidence")
        preserve_failure_diagnostics(self.files.root, self.files.evidence)
        for relative in (
            "outputs/lcov.info",
            "outputs/junit.xml",
            "package-diagnostics/failed-command.json",
        ):
            self.assertEqual(
                (self.files.evidence / relative).read_bytes(),
                b"original failure evidence",
            )

    def test_native_prerequisites_fail_before_expensive_verification(self) -> None:
        tools = FakeNativeGateTools()
        require_native_gate_tools({"PATH": "/fixture"}, tools.lookup)
        self.assertEqual(tools.paths, ["/fixture"] * 6)
        missing = FakeNativeGateTools("xvfb-run")
        with self.assertRaisesRegex(ValueError, "xvfb-run.*expected"):
            require_native_gate_tools({"PATH": "/fixture"}, missing.lookup)

    def test_distribution_cache_is_selected_without_replacing_system_path(self) -> None:
        cache = self.files.root / "target/native-ci-tools/usr/bin"
        cache.mkdir(parents=True)
        tools = FakeNativeGateTools()
        with patch("pre_release_execution.shutil.which", new=tools.lookup):
            environment = native_gate_environment(self.files.root, {"PATH": "/system"})
        self.assertEqual(environment["PATH"], str(cache) + ":/system")

    def test_empty_gate_cannot_create_success_receipt(self) -> None:
        with self.assertRaisesRegex(ValueError, "at least one"):
            verify_recorded_gate(
                VerificationGate("rust-prechecks", ()),
                self.files.root,
                {},
                self.files.evidence,
            )
        self.assertFalse((self.files.evidence / "rust-prechecks.json").exists())


if __name__ == "__main__":
    unittest.main()
