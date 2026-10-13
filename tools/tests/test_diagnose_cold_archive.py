"""Named fake stages prove cold ordering and preservation of red package verdicts."""

from contextlib import ExitStack
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import diagnose_cold_archive as cold
from diagnose_release_renderer import RendererDiagnosticProcess


class NamedColdProcess(RendererDiagnosticProcess):
    def __init__(self, root: Path, output: Path, exit_code: int) -> None:
        super().__init__(root, output, "immutable-sha")
        self.exit_code = exit_code
        self.arguments: list[str] = []

    def run(self, label: str, arguments: list[str]) -> subprocess.CompletedProcess[str]:
        self.arguments = arguments
        return subprocess.CompletedProcess(
            arguments, self.exit_code, "raw strict output", ""
        )


class NamedColdLifecycle:
    def __init__(self) -> None:
        self.events: list[str] = []

    def source(self, probe: RendererDiagnosticProcess) -> None:
        self.events.append("pin")

    def clean(self, probe: RendererDiagnosticProcess, label: str) -> None:
        self.events.append(label)

    def package(self, probe: RendererDiagnosticProcess, stage: Path) -> None:
        self.events.append("canonical-31-archives")

    def gate(self, probe: RendererDiagnosticProcess) -> int:
        self.events.append("exact-cold-all-targets")
        return 101

    def observe(self, probe: RendererDiagnosticProcess) -> dict[str, dict[str, str]]:
        self.events.append("driver-observation-strict-replays-raw-captures")
        return {}

    def bookend(self, probe: RendererDiagnosticProcess) -> None:
        self.events.append("cold-binary-after")

    def bindings(self) -> dict[str, object]:
        return {
            "record_source": self.source,
            "require_clean_source": self.clean,
            "prepare_archive_stage": self.package,
            "run_cold_gate": self.gate,
            "run_renderer_cohort": self.observe,
            "record_cold_binary_after": self.bookend,
        }


class ColdArchiveTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)

    def tearDown(self) -> None:
        self.directory.cleanup()

    def test_exact_release_command_has_no_filter_serialization_or_json_override(
        self,
    ) -> None:
        self.assertEqual(
            cold.cold_package_arguments(self.root),
            [
                "cargo",
                "test",
                "--package",
                "martensite-vulkan",
                "--locked",
                "--target-dir",
                str(self.root / "target"),
                "--all-targets",
            ],
        )

    def test_red_gate_preserves_actual_exit_and_still_runs_post_gate_evidence(
        self,
    ) -> None:
        fake = NamedColdLifecycle()
        with ExitStack() as stack:
            for name, method in fake.bindings().items():
                stack.enter_context(patch.object(cold, name, method))
            cold.cold_archive_diagnostic(self.root, self.root / "evidence", "pin")
        self.assertEqual(
            fake.events,
            [
                "pin",
                "source-clean-before",
                "canonical-31-archives",
                "exact-cold-all-targets",
                "driver-observation-strict-replays-raw-captures",
                "cold-binary-after",
                "source-clean-after",
            ],
        )

    def test_real_red_verdict_retains_negative_exit_without_retries(self) -> None:
        probe = NamedColdProcess(self.root, self.root, -11)
        with patch.object(
            cold, "record_renderer_inputs", return_value={}
        ), patch.object(cold, "record_environment"):
            self.assertEqual(cold.run_cold_gate(probe), -11)
        verdict = json.loads((self.root / "cold-verdict.json").read_text())
        self.assertEqual(verdict["exit_code"], -11)
        self.assertTrue(verdict["release_guards_unchanged"])
        self.assertEqual(probe.arguments, cold.cold_package_arguments(self.root))

    def test_injected_gate_arguments_preserve_the_actual_original_command(self) -> None:
        probe = NamedColdProcess(self.root, self.root, 101)
        arguments = [
            "cargo",
            "test",
            "-p",
            "martensite-vulkan",
            "--all-targets",
            "--locked",
        ]
        with patch.object(
            cold, "record_renderer_inputs", return_value={}
        ), patch.object(cold, "record_environment"):
            self.assertEqual(cold.run_cold_gate(probe, arguments), 101)
        self.assertEqual(probe.arguments, arguments)
        self.assertEqual(
            json.loads((self.root / "cold-verdict.json").read_text())["exit_code"], 101
        )

    def test_cargo_executables_are_hashed_before_later_builds(self) -> None:
        binary = self.root / "toolkit-frozen"
        binary.write_bytes(b"executed")
        probe = NamedColdProcess(self.root, self.root, 0)
        cold.retain_cold_executables(
            probe, "Running tests/toolkit.rs (toolkit-frozen)\n"
        )
        receipt = json.loads((self.root / "cold-executed-binaries.json").read_text())
        self.assertEqual(
            receipt, [{"path": str(binary), "sha256": cold.digest_file(binary)}]
        )

    def test_cold_binary_mutation_is_retained_as_build_context_evidence(self) -> None:
        binary = self.root / "toolkit-frozen"
        binary.write_bytes(b"executed")
        probe = NamedColdProcess(self.root, self.root, 0)
        cold.retain_cold_executables(
            probe, "Running tests/toolkit.rs (toolkit-frozen)\n"
        )
        cold.record_cold_binary_after(probe)
        self.assertTrue(
            json.loads((self.root / "cold-binary-bookends.json").read_text())["equal"]
        )
        binary.write_bytes(b"rebuilt")
        cold.record_cold_binary_after(probe)
        self.assertFalse(
            json.loads((self.root / "cold-binary-bookends.json").read_text())["equal"]
        )

    def test_missing_executed_binary_is_a_typed_provenance_failure(self) -> None:
        probe = NamedColdProcess(self.root, self.root, 0)
        with self.assertRaisesRegex(
            ValueError, "missing.*expected existing Cargo test binary"
        ):
            cold.retain_cold_executables(probe, "Running tests/toolkit.rs (missing)\n")


if __name__ == "__main__":
    unittest.main()
