"""Named process/package fakes retain both diagnostic cohorts and unchanged strict gates."""

import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import diagnose_source_archives as pair
import renderer_archive_inputs as inputs
from diagnose_release_renderer import digest_file
import renderer_archive_stage as staging
from diagnose_release_renderer import DiagnosticBinary, RendererDiagnosticProcess
from release_inventory import ReleaseCrate


class NamedPairProcess(RendererDiagnosticProcess):
    def __init__(self, root: Path, output: Path) -> None:
        super().__init__(root, output, "frozen-revision")
        self.calls: list[tuple[str, list[str]]] = []
        self.replies: dict[str, subprocess.CompletedProcess[str]] = {}

    def run(self, label: str, arguments: list[str]) -> subprocess.CompletedProcess[str]:
        self.calls.append((label, arguments))
        stdout = self.revision if arguments == ["git", "rev-parse", "HEAD"] else ""
        result = self.replies.get(
            label, subprocess.CompletedProcess(arguments, 0, stdout, "")
        )
        (self.output / f"{label}.stdout").write_text(result.stdout)
        (self.output / f"{label}.stderr").write_text(result.stderr)
        return result


class NamedPairOperations:
    def __init__(self, binary: Path) -> None:
        self.binary = binary
        self.filters: tuple[str, ...] = ()
        self.captures: tuple[tuple[str, str, str], ...] = ()

    def record(
        self, probe: RendererDiagnosticProcess, label: str = ""
    ) -> dict[str, dict[str, str]]:
        return {"source": {"lib.rs": "same"}}

    def build(
        self,
        probe: RendererDiagnosticProcess,
        captures: tuple[tuple[str, str, str], ...],
    ) -> list[DiagnosticBinary]:
        self.captures = captures
        return [DiagnosticBinary("toolkit", str(self.binary), digest_file(self.binary))]

    def replay(
        self,
        probe: RendererDiagnosticProcess,
        artifacts: list[DiagnosticBinary],
        filters: tuple[str, ...],
        captures: tuple[tuple[str, str, str], ...],
    ) -> None:
        self.filters = filters
        for name in filters:
            probe.run(f"strict-{name}", [artifacts[0].path, name, "--nocapture"])


class NamedCanonicalStage:
    def __init__(self) -> None:
        self.calls: list[str] = []
        self.crates = [
            ReleaseCrate(f"package-{index}", "0.2.4", f"crates/p{index}")
            for index in range(31)
        ]

    def inventory(
        self, root: Path, commands: staging.SourceArchiveCommands
    ) -> list[ReleaseCrate]:
        self.calls.append("inventory")
        return self.crates

    def support(self, root: Path, commands: staging.SourceArchiveCommands) -> None:
        self.calls.append("test-support")

    def provenance(
        self,
        probe: RendererDiagnosticProcess,
        archives: Path,
        crates: list[ReleaseCrate],
    ) -> None:
        self.calls.append("provenance")

    def stage(
        self,
        archives: Path,
        destination: Path,
        crates: list[ReleaseCrate],
        lockfile: Path,
    ) -> None:
        self.calls.append("canonical-stage-workspace")


class NamedPairedLifecycle:
    def __init__(self) -> None:
        self.calls: list[str] = []

    def pin(self, probe: RendererDiagnosticProcess) -> None:
        self.calls.append("pin")

    def clean(self, probe: RendererDiagnosticProcess, label: str) -> None:
        self.calls.append(label)

    def driver(self, probe: RendererDiagnosticProcess) -> None:
        self.calls.append("driver")

    def package(self, probe: RendererDiagnosticProcess, stage: Path) -> None:
        self.calls.append("canonical-stage")

    def cohort(self, probe: RendererDiagnosticProcess) -> dict[str, dict[str, str]]:
        self.calls.append(probe.output.name)
        return {"source": {"lib.rs": "same"}}


class SourceArchiveTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.binary = self.root / "frozen-toolkit"
        self.binary.write_bytes(b"frozen")
        self.probe = NamedPairProcess(self.root, self.root)

    def tearDown(self) -> None:
        self.directory.cleanup()

    def expected_pair_sequence(self) -> list[str]:
        return [
            "pin",
            "source-clean-before",
            "driver",
            "canonical-stage",
            "source",
            "stage",
            "source-clean-after",
        ]

    def test_pair_orchestration_runs_both_roots_with_one_injected_pin(self) -> None:
        fake = NamedPairedLifecycle()
        with patch.object(pair, "record_source", fake.pin), patch.object(
            pair, "require_clean_source", fake.clean
        ):
            with patch.object(pair, "record_driver", fake.driver), patch.object(
                pair, "prepare_archive_stage", fake.package
            ), patch.object(pair, "run_renderer_cohort", fake.cohort):
                pair.paired_renderer_diagnostic(
                    self.root, self.root / "paired", "injected-pin"
                )
        self.assertEqual(fake.calls, self.expected_pair_sequence())
        payload = json.loads(
            (self.root / "paired/source-stage-comparison.json").read_text()
        )
        self.assertEqual(payload["revision"], "injected-pin")

    def test_package_command_checks_root_and_preserves_red_exit(self) -> None:
        commands = staging.SourceArchiveCommands(self.probe)
        self.probe.replies["package-command-001"] = subprocess.CompletedProcess(
            [], 17, "", "red"
        )
        with self.assertRaisesRegex(ValueError, "exit=17.*expected exit=0"):
            commands.run(["cargo", "package"], self.root)
        with self.assertRaisesRegex(ValueError, "expected pinned root"):
            commands.run(["cargo"], self.root / "other")
        self.assertEqual(len(self.probe.calls), 1)

    def test_canonical_stage_keeps_31_packages_and_test_support(self) -> None:
        fake = NamedCanonicalStage()
        with patch.object(staging, "release_inventory", fake.inventory), patch.object(
            staging, "package_test_support", fake.support
        ):
            with patch.object(
                staging, "record_archive_provenance", fake.provenance
            ), patch.object(staging, "stage_workspace", fake.stage):
                crates = staging.prepare_archive_stage(self.probe, self.root / "stage")
        self.assertEqual(len(crates), 31)
        self.assertEqual(
            fake.calls,
            ["inventory", "test-support", "provenance", "canonical-stage-workspace"],
        )
        self.assertIn("--exclude-lockfile", self.probe.calls[1][1])
        self.assertEqual(
            self.probe.calls[0][1], ["python3", "tools/toolkit_test_suites.py"]
        )

    def test_wrong_package_count_is_rejected_before_packaging(self) -> None:
        fake = NamedCanonicalStage()
        fake.crates.pop()
        with patch.object(staging, "release_inventory", fake.inventory):
            with self.assertRaisesRegex(ValueError, "count=30.*expected exactly 31"):
                staging.prepare_archive_stage(self.probe, self.root / "stage")
        self.assertEqual(self.probe.calls, [])

    def test_dirty_source_is_rejected(self) -> None:
        pair.require_clean_source(self.probe, "clean")
        self.probe.replies["dirty"] = subprocess.CompletedProcess(
            [], 0, " M shader.frag", ""
        )
        with self.assertRaisesRegex(ValueError, "shader.frag.*expected clean"):
            pair.require_clean_source(self.probe, "dirty")

    def test_input_comparison_retains_changes_without_waiving_tests(self) -> None:
        pair.write_pair_comparison(
            self.root,
            "pin",
            {"source": {"lib.rs": "old"}},
            {"source": {"lib.rs": "new"}},
        )
        receipt = json.loads((self.root / "source-stage-comparison.json").read_text())
        self.assertTrue(receipt["release_guards_unchanged"])
        self.assertEqual(receipt["input_differences"]["source"]["changed"], ["lib.rs"])
        probe = pair.diagnostic_probe(self.root, self.root / "output", "pin")
        self.assertEqual(probe.revision, "pin")

    def test_actual_feature_calls_keep_default_and_all_selected_graphs(self) -> None:
        inputs.record_feature_graph(self.probe)
        self.assertEqual(
            [name for name, _ in self.probe.calls],
            [
                "features-default",
                "feature-tree-default",
                "features-all",
                "feature-tree-all",
            ],
        )
        self.assertNotIn("--all-features", self.probe.calls[0][1])
        self.assertIn("--all-features", self.probe.calls[2][1])
        self.assertTrue((self.root / "features-default.json").is_file())
        self.assertTrue((self.root / "features-all.json").is_file())
        hashes = json.loads((self.root / "feature-graphs-sha256.json").read_text())
        self.assertEqual(len(hashes), 4)

    def test_feature_failure_stays_a_required_provenance_error(self) -> None:
        self.probe.replies["features-default"] = subprocess.CompletedProcess(
            [], 11, "", "red"
        )
        with self.assertRaisesRegex(ValueError, "exit=11.*expected Cargo"):
            inputs.record_feature_graph(self.probe)
        self.probe.replies["feature-tree-all"] = subprocess.CompletedProcess(
            [], 12, "", "red"
        )
        with self.assertRaisesRegex(ValueError, "exit=12.*expected selected"):
            inputs.record_selected_feature_tree(self.probe, "all", ["--all-features"])

    def test_red_strict_filters_still_run_full_toolkit(self) -> None:
        fake = NamedPairOperations(self.binary)
        for name in pair.PAIR_FILTERS:
            self.probe.replies[f"strict-{name}"] = subprocess.CompletedProcess(
                [], 101, "", "red"
            )
        with patch.object(pair, "record_renderer_inputs", fake.record), patch.object(
            pair, "record_feature_graph", fake.record
        ):
            with patch.object(pair, "build_artifacts", fake.build), patch.object(
                pair, "replay_and_capture", fake.replay
            ):
                with patch.object(pair, "record_driver_snapshot", fake.record):
                    pair.run_renderer_cohort(self.probe)
        self.assertEqual(fake.filters, pair.PAIR_FILTERS)
        self.assertEqual(fake.captures, pair.PAIR_CAPTURES)
        self.assertEqual(
            self.probe.calls[0], ("full-toolkit", [str(self.binary), "--nocapture"])
        )


if __name__ == "__main__":
    unittest.main()
