"""Original cold failure remains red while post-gate diagnostics are retained."""

from __future__ import annotations

import argparse
from contextlib import ExitStack
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import diagnose_current_vulkan as current
from diagnose_release_renderer import RendererDiagnosticProcess

ROOT = Path(__file__).resolve().parents[2]


class ImportedGateMetadataFake:
    """Named source inspector which can fail after the original verdict is saved."""

    def __init__(self) -> None:
        self.events: list[str] = []
        self.fail = False

    def source(self, probe: RendererDiagnosticProcess) -> None:
        self.events.append("source")
        if self.fail:
            raise ValueError("source metadata failure: expected pinned source")

    def clean(self, probe: RendererDiagnosticProcess, label: str) -> None:
        self.events.append(label)
        return None

    def gate(self, probe: RendererDiagnosticProcess) -> None:
        self.events.append("unexpected-gate")
        raise AssertionError("original cargo gate must never rerun")


class VulkanFailureEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        self.probe = RendererDiagnosticProcess(self.directory, self.directory, "a" * 40)
        self.executable = self.directory / "original-toolkit"
        self.executable.write_bytes(b"original executable")
        self.log = self.directory / "gate.log"
        self.log.write_text(
            f"Martensite original gate revision: {'a' * 40}\n"
            "Martensite original gate command: cargo test -p martensite-vulkan --all-targets --locked\n"
            f"Running tests/toolkit.rs ({self.executable})\nFAILED original\n"
        )
        self.status = self.directory / "gate.exit"
        self.status.write_text("101\n")
        self.fake = ImportedGateMetadataFake()

    def import_gate(self) -> None:
        with ExitStack() as stack:
            stack.enter_context(
                patch.object(current, "record_source", self.fake.source)
            )
            stack.enter_context(
                patch.object(current, "require_clean_source", self.fake.clean)
            )
            stack.enter_context(
                patch.object(current, "current_vulkan_gate", self.fake.gate)
            )
            current.record_existing_gate(self.probe, self.log, self.status)

    def test_original_exit_and_binary_retained_without_gpu_gate_rerun(self) -> None:
        self.import_gate()
        verdict = json.loads((self.directory / "cold-verdict.json").read_text())
        self.assertEqual(verdict["exit_code"], 101)
        self.assertEqual(verdict["arguments"], current.current_vulkan_arguments())
        self.assertTrue(verdict["imported_original_gate"])
        self.assertEqual(
            (self.directory / "original-gate.log").read_bytes(), self.log.read_bytes()
        )
        binaries = json.loads(
            (self.directory / "cold-executed-binaries.json").read_text()
        )
        self.assertEqual(binaries[0]["path"], str(self.executable))
        self.assertEqual(self.fake.events, ["source", "imported-source"])

    def test_metadata_failure_cannot_replace_original_red_verdict(self) -> None:
        self.fake.fail = True
        with self.assertRaisesRegex(ValueError, "source metadata failure"):
            self.import_gate()
        verdict = json.loads((self.directory / "cold-verdict.json").read_text())
        self.assertEqual(verdict["exit_code"], 101)
        self.assertTrue((self.directory / "original-gate.log").is_file())

    def test_invalid_status_never_invents_completed_gate(self) -> None:
        for value in ("missing", "-1", "256", "0; exit 0"):
            self.status.write_text(value)
            with self.assertRaisesRegex(ValueError, "expected an integer status"):
                self.import_gate()
        self.assertFalse((self.directory / "cold-verdict.json").exists())

    def test_unbound_or_wrong_command_log_cannot_create_verdict(self) -> None:
        original = self.log.read_text()
        for contents in (
            "unbound output",
            original.replace("a" * 40, "b" * 40),
            original.replace("--locked", "--all-features"),
        ):
            self.log.write_text(contents)
            with self.assertRaisesRegex(ValueError, "original identity"):
                self.import_gate()
        self.assertFalse((self.directory / "cold-verdict.json").exists())

    def test_record_dispatch_requires_both_paths(self) -> None:
        parser = argparse.ArgumentParser()
        args = argparse.Namespace(phase="record", gate_log=None, gate_exit=self.status)
        with self.assertRaises(SystemExit):
            current.dispatch_phase(self.probe, args, parser)
        args.gate_log = self.log
        with patch.object(current, "record_existing_gate") as record:
            current.dispatch_phase(self.probe, args, parser)
        record.assert_called_once_with(self.probe, self.log, self.status, "vulkan")

    def test_failure_workflow_cannot_observe_before_original_gate(self) -> None:
        workflow = (
            (ROOT / ".github/workflows/ci.yml")
            .read_text()
            .split("  vulkan-runtime:", 1)[1]
        )
        self.assertLess(
            workflow.index("cargo test -p"), workflow.index("path: diagnostic-tools")
        )
        self.assertLess(
            workflow.index("--phase record"), workflow.index("install -y vulkan-tools")
        )
        self.assertLess(
            workflow.index("install -y vulkan-tools"), workflow.index("--phase observe")
        )
        self.assertIn("ref: ${{ github.workflow_sha }}", workflow)
        self.assertIn("always() && steps.vulkan_gate.outcome == 'failure'", workflow)
        self.assertNotIn("continue-on-error", workflow)

    def gate_script(self) -> str:
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        block = workflow.split("        id: vulkan_gate\n", 1)[1].split(
            "      # Current", 1
        )[0]
        script = block.split("        run: |\n", 1)[1]
        return "\n".join(
            line.removeprefix("          ") for line in script.splitlines()
        )

    def fake_gate_environment(self) -> dict[str, str]:
        for name, contents in {
            "cargo": 'printf "%s\\n" "$*"; echo original-error >&2; exit 101',
            "git": f"echo {'a' * 40}",
        }.items():
            binary = self.directory / name
            binary.write_text(f"#!/bin/sh\n{contents}\n")
            binary.chmod(0o755)
        return {**os.environ, "PATH": f'{self.directory}:{os.environ["PATH"]}'}

    def stage_current_gate_helper(self) -> None:
        """Use actual orchestration over the isolated fake source, as older-tag CI does."""
        helper = self.directory / "target/verification-tools/tools"
        helper.mkdir(parents=True)
        for name in ("release_verification_plan.py", "verify_release_gate.py"):
            (helper / name).write_bytes((ROOT / "tools" / name).read_bytes())
        shader = self.directory / "crates/vulkan/shaders/unlit.vert.spv"
        shader.parent.mkdir(parents=True)
        shader.write_bytes(b"test-only source variant")

    def test_actual_shell_gate_retains_exact_command_and_failure_exit(self) -> None:
        self.stage_current_gate_helper()
        environment = self.fake_gate_environment()
        result = subprocess.run(
            ["bash", "-e", "-c", self.gate_script()],
            cwd=self.directory,
            env=environment,
            text=True,
            capture_output=True,
        )
        self.assertEqual(result.returncode, 101)
        self.assertEqual(
            (self.directory / "target/vulkan-gate/gate.exit").read_text(), "101\n"
        )
        log = (self.directory / "target/vulkan-gate/gate.log").read_text()
        self.assertIn("test -p martensite-vulkan --all-targets --locked", log)
        self.assertIn("original-error", log)


if __name__ == "__main__":
    unittest.main()
