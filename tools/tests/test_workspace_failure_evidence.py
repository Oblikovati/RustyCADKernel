"""Actual workspace failure graph and original executable remain authoritative."""

from __future__ import annotations

import argparse
from contextlib import ExitStack
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import diagnose_current_vulkan as current
import renderer_workspace_evidence as evidence
from release_verification_plan import rust_gates
from diagnose_release_renderer import (
    DiagnosticBinary,
    RendererDiagnosticProcess,
    digest_file,
)
from diagnose_source_archives import PAIR_CAPTURES, PAIR_FILTERS


class WorkspaceCompilerFake(RendererDiagnosticProcess):
    """Named compiler transport returns real local bytes and foreign same-name targets."""

    def __init__(self, directory: Path) -> None:
        super().__init__(directory, directory, "a" * 40)
        self.calls: list[tuple[str, list[str]]] = []
        self.exit_code = 0
        self.extra = False
        self.records: list[str] = []
        for name, _, _ in PAIR_CAPTURES:
            executable = directory / name
            executable.write_bytes(name.encode())
            self.records.append(
                self.artifact(name, executable, directory / "crates/vulkan/Cargo.toml")
            )

    def artifact(self, name: str, executable: Path, manifest: Path) -> str:
        return json.dumps(
            {
                "reason": "compiler-artifact",
                "manifest_path": str(manifest),
                "target": {"name": name},
                "executable": str(executable),
            }
        )

    def run(self, label: str, arguments: list[str]) -> subprocess.CompletedProcess[str]:
        self.calls.append((label, arguments))
        return subprocess.CompletedProcess(
            arguments, self.exit_code, "\n".join(self.records), ""
        )


class WorkspaceLifecycleFake:
    """Named lifecycle records ordering and original strict executable identity."""

    def __init__(self, original: DiagnosticBinary) -> None:
        self.events: list[str] = []
        self.original = original
        self.strict_binary = ""
        self.strict_cwd = Path()

    def strict(
        self, probe: RendererDiagnosticProcess, binary: str, filters: tuple[str, ...]
    ) -> None:
        self.events.append("original-strict-before-build")
        self.strict_binary = binary
        self.strict_cwd = probe.root

    def captures(self, probe: RendererDiagnosticProcess) -> list[DiagnosticBinary]:
        self.events.append("workspace-capture-build")
        return []

    def raw(
        self,
        probe: RendererDiagnosticProcess,
        binaries: dict[str, str],
        captures: tuple[tuple[str, str, str], ...],
    ) -> None:
        self.events.append("workspace-raw")
        return None

    def bookkeeping(self, *arguments: object) -> None:
        self.events.append("bookkeeping")
        return None


class WorkspaceEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.probe = WorkspaceCompilerFake(self.root)
        self.original = self.root / "original-toolkit"
        self.original.write_bytes(b"original failing toolkit")
        self.log = self.root / "original-gate.log"
        self.log.write_text(
            f"Running tests/toolkit.rs ({self.original})\n{PAIR_FILTERS[0]} ... FAILED\n"
        )
        records = [{"path": str(self.original), "sha256": digest_file(self.original)}]
        (self.root / "cold-executed-binaries.json").write_text(json.dumps(records))

    def test_workspace_profile_binds_exact_alias_and_refuses_single_package_header(
        self,
    ) -> None:
        self.assertEqual(
            current.gate_profile_arguments("workspace"),
            ["cargo", "test-all", "--no-fail-fast"],
        )
        contents = f"Martensite original gate revision: {self.probe.revision}\n"
        contents += "Martensite original gate command: cargo test-all --no-fail-fast\n"
        current.validate_original_log(self.probe, contents, "workspace")
        with self.assertRaisesRegex(ValueError, "original identity"):
            current.validate_original_log(self.probe, contents, "vulkan")
        with self.assertRaisesRegex(ValueError, "profile"):
            current.gate_profile_arguments("unbound")
        current.write_imported_verdict(self.probe, 101, "workspace")
        self.assertEqual(
            json.loads((self.root / "cold-verdict.json").read_text())["profile"],
            "workspace",
        )

    def test_original_toolkit_selected_once_by_executed_test_namespace(self) -> None:
        other = self.root / "foreign-toolkit"
        other.write_bytes(b"foreign")
        self.log.write_text(
            f"Running tests/toolkit.rs ({other})\nforeign::test ... ok\n"
            + self.log.read_text()
        )
        selected = evidence.original_workspace_toolkit(self.probe)
        self.assertEqual(selected.path, str(self.original))
        self.assertEqual(selected.sha256, digest_file(self.original))
        self.original.write_bytes(b"replaced")
        with self.assertRaisesRegex(ValueError, "unchanged executed bytes"):
            evidence.original_workspace_toolkit(self.probe)

    def test_missing_and_duplicate_original_process_are_refused(self) -> None:
        original = self.log.read_text()
        for contents in ("no executed toolkit", original + original):
            self.log.write_text(contents)
            with self.assertRaisesRegex(ValueError, "exactly one executed toolkit"):
                evidence.original_workspace_toolkit(self.probe)

    def test_captures_use_workspace_allfeatures_and_separate_target_directory(
        self,
    ) -> None:
        artifacts = evidence.workspace_capture_artifacts(self.probe)
        arguments = self.probe.calls[0][1]
        self.assertIn("--workspace", arguments)
        self.assertIn("--all-features", arguments)
        self.assertNotIn("-p", arguments)
        self.assertEqual(
            arguments[arguments.index("--target-dir") + 1],
            str(self.root / "workspace-build"),
        )
        self.assertEqual(
            {item.name for item in artifacts}, {name for name, _, _ in PAIR_CAPTURES}
        )
        self.assertEqual(self.original.read_bytes(), b"original failing toolkit")

    def test_foreign_same_name_compiler_outputs_never_replace_vulkan(self) -> None:
        foreign = self.root / "foreign"
        foreign.write_bytes(b"foreign")
        self.probe.records.append(
            self.probe.artifact(
                PAIR_CAPTURES[0][0], foreign, self.root / "crates/other/Cargo.toml"
            )
        )
        artifacts = evidence.workspace_capture_artifacts(self.probe)
        self.assertNotIn(str(foreign), [artifact.path for artifact in artifacts])
        self.probe.records.append(self.probe.records[0])
        with self.assertRaisesRegex(ValueError, "expected exact"):
            evidence.workspace_capture_artifacts(self.probe)

    def test_failed_or_missing_workspace_capture_build_is_not_success(self) -> None:
        self.probe.exit_code = 101
        with self.assertRaisesRegex(ValueError, "exit=101"):
            evidence.workspace_capture_artifacts(self.probe)
        self.probe.exit_code, self.probe.records = 0, []
        with self.assertRaisesRegex(ValueError, "expected exact"):
            evidence.workspace_capture_artifacts(self.probe)

    def test_graph_records_entire_workspace_and_rejects_failed_metadata(self) -> None:
        evidence.workspace_feature_graph(self.probe)
        self.assertIn("--workspace", self.probe.calls[0][1])
        self.assertTrue(
            all("--all-features" in arguments for _, arguments in self.probe.calls)
        )
        self.probe.exit_code = 1
        with self.assertRaisesRegex(
            ValueError, "expected completed workspace feature graph"
        ):
            evidence.workspace_feature_graph(self.probe)

    def patch_replay_lifecycle(
        self, stack: ExitStack, fake: WorkspaceLifecycleFake
    ) -> None:
        replacements = {
            "replay_strict_filters": fake.strict,
            "workspace_capture_artifacts": fake.captures,
            "replay_raw_captures": fake.raw,
        }
        for name in (
            "record_binary_bookend",
            "record_buffer_hashes",
            "record_cold_binary_after",
        ):
            replacements[name] = fake.bookkeeping
        for name, replacement in replacements.items():
            stack.enter_context(patch.object(evidence, name, replacement))

    def test_original_strict_replay_precedes_any_capture_build(self) -> None:
        original = evidence.original_workspace_toolkit(self.probe)
        fake = WorkspaceLifecycleFake(original)
        with ExitStack() as stack:
            self.patch_replay_lifecycle(stack, fake)
            evidence.workspace_replay(self.probe)
        self.assertEqual(
            fake.events[:3],
            [
                "original-strict-before-build",
                "workspace-capture-build",
                "workspace-raw",
            ],
        )
        self.assertEqual(fake.strict_binary, str(self.original))
        self.assertEqual(fake.strict_cwd, self.root / "crates/vulkan")

    def test_package_cwd_probe_shares_original_command_history(self) -> None:
        replay = evidence.PackageReplayProbe(self.probe)
        self.assertIs(replay.results, self.probe.results)
        self.assertEqual(replay.root, self.root / "crates/vulkan")
        self.assertEqual(replay.output, self.probe.output)
        with patch("diagnose_release_renderer.run_observed_command") as run:
            run.return_value = subprocess.CompletedProcess(
                ["toolkit"], 101, "original red", ""
            )
            replay.run("strict-original", ["toolkit"])
        self.assertEqual(run.call_args.args[1], self.root / "crates/vulkan")
        history = json.loads((self.root / "commands.json").read_text())
        self.assertEqual(history["commands"][0]["exit_code"], 101)
        self.assertEqual(len(self.probe.results), 1)

    def saved_command(self) -> dict[str, object]:
        return {
            "label": "source-head",
            "arguments": ["git", "rev-parse", "HEAD"],
            "exit_code": 0,
            "elapsed_seconds": 0.1,
        }

    def test_record_to_observe_history_restores_before_appending(self) -> None:
        current.restore_command_history(self.probe)
        record = self.saved_command()
        path = self.root / "commands.json"
        path.write_text(
            json.dumps({"revision": self.probe.revision, "commands": [record]})
        )
        current.restore_command_history(self.probe)
        self.assertEqual(self.probe.results[0].label, "source-head")
        self.probe.results.append(
            current.recorded_command(
                {**record, "label": "strict-red", "exit_code": 101}
            )
        )
        self.probe.persist()
        self.assertEqual(
            [item["label"] for item in json.loads(path.read_text())["commands"]],
            ["source-head", "strict-red"],
        )

    def test_command_history_rejects_another_revision(self) -> None:
        path = self.root / "commands.json"
        path.write_text(json.dumps({"revision": "wrong", "commands": []}))
        with self.assertRaisesRegex(ValueError, "expected revision"):
            current.restore_command_history(self.probe)

    def test_malformed_command_history_is_not_recast_as_evidence(self) -> None:
        record = {
            "label": "gate",
            "arguments": ["cargo"],
            "exit_code": 101,
            "elapsed_seconds": 1.0,
        }
        for invalid in (
            None,
            {},
            {**record, "arguments": "cargo"},
            {**record, "exit_code": True},
            {**record, "elapsed_seconds": -1},
        ):
            with self.assertRaisesRegex(ValueError, "expected"):
                current.recorded_command(invalid)

    def test_wrong_original_profile_blocks_workspace_observation(self) -> None:
        current.write_imported_verdict(self.probe, 101, "vulkan")
        with self.assertRaisesRegex(ValueError, "expected workspace"):
            evidence.workspace_observations(self.probe)
        self.assertEqual(self.probe.calls, [])

    def test_workspace_dispatch_uses_workspace_observer_and_rejects_new_gate(
        self,
    ) -> None:
        parser = argparse.ArgumentParser()
        with patch.object(evidence, "workspace_observations") as observe:
            current.dispatch_workspace_phase(self.probe, "observe", parser)
        observe.assert_called_once_with(self.probe)
        with self.assertRaises(SystemExit):
            current.dispatch_workspace_phase(self.probe, "gate", parser)

    def rust_workflow(self) -> str:
        root = Path(__file__).resolve().parents[2]
        contents = (root / ".github/workflows/ci.yml").read_text()
        return contents.split("  rust:", 1)[1].split("  coverage:", 1)[0]

    def test_diagnostic_scope_cannot_narrow_reusable_release_owners(self) -> None:
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/ci.yml").read_text()
        dispatch = workflow.split("  workflow_dispatch:", 1)[1].split(
            "  workflow_call:", 1
        )[0]
        reusable = workflow.split("  workflow_call:", 1)[1].split("permissions:", 1)[0]
        self.assertIn("options: [all, rust]", dispatch)
        self.assertIn("default: all", dispatch)
        self.assertIn("source_ref:", dispatch)
        self.assertNotIn("check_scope:", reusable)
        condition = "if: ${{ github.event_name != 'workflow_dispatch' || inputs.check_scope != 'rust' }}"
        self.assertEqual(workflow.count(condition), 5)
        self.assertNotIn(condition, self.rust_workflow())

    def test_workflow_preserves_original_workspace_exit_and_failure_only_tools(
        self,
    ) -> None:
        workflow = self.rust_workflow()
        runner = "python3 target/verification-tools/tools/verify_release_gate.py --root . --gate rust-tests"
        self.assertIn(runner + " 2>&1 | tee -a target/rust-gate/gate.log", workflow)
        self.assertEqual(
            rust_gates()[1].commands[0].argv, ("cargo", "test-all", "--no-fail-fast")
        )
        self.assertIn("gate_status=${PIPESTATUS[0]}", workflow)
        self.assertIn('exit "$gate_status"', workflow)
        self.assertLess(
            workflow.index(runner + " 2>&1"),
            workflow.index("path: diagnostic-tools"),
        )
        self.assertIn("--phase observe --profile workspace", workflow)
        self.assertIn("always() && steps.rust_gate.outcome == 'failure'", workflow)
        self.assertNotIn("continue-on-error", workflow)


if __name__ == "__main__":
    unittest.main()
