"""Named lifecycle fakes preserve the exact current gate and post-gate red evidence."""

from contextlib import ExitStack
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import diagnose_current_vulkan as current
from diagnose_release_renderer import RendererDiagnosticProcess


class NamedCurrentLifecycle:
    def __init__(self) -> None:
        self.events: list[str] = []
        self.arguments: list[str] = []

    def source(self, probe: RendererDiagnosticProcess) -> None:
        self.events.append("pin")

    def clean(self, probe: RendererDiagnosticProcess, label: str) -> None:
        self.events.append(label)

    def gate(self, probe: RendererDiagnosticProcess, arguments: list[str]) -> int:
        self.events.append("exact-first-gpu-command")
        self.arguments = arguments
        return 101

    def observe(self, probe: RendererDiagnosticProcess, label: str) -> None:
        self.events.append(label)

    def inputs(
        self, probe: RendererDiagnosticProcess, label: str
    ) -> dict[str, dict[str, str]]:
        self.events.append(label)
        return {"source": {"frozen.rs": "same"}}

    def feature(self, probe: RendererDiagnosticProcess) -> None:
        self.events.append("features")

    def build(
        self,
        probe: RendererDiagnosticProcess,
        captures: tuple[tuple[str, str, str], ...],
    ) -> list[object]:
        self.events.append("capture-build")
        return []

    def capture(
        self,
        probe: RendererDiagnosticProcess,
        artifacts: list[object],
        filters: tuple[str, ...],
        captures: tuple[tuple[str, str, str], ...],
    ) -> None:
        self.events.append("strict-and-raw")

    def bookend(self, probe: RendererDiagnosticProcess) -> None:
        self.events.append("cold-binary-after")

    def observations(self) -> dict[str, object]:
        return {
            "require_clean_source": self.clean,
            "record_driver_snapshot": self.observe,
            "record_renderer_inputs": self.inputs,
            "record_feature_graph": self.feature,
            "build_artifacts": self.build,
            "replay_and_capture": self.capture,
            "record_cold_binary_after": self.bookend,
        }


class CurrentVulkanTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.probe = RendererDiagnosticProcess(
            self.root, self.root, "immutable-current"
        )

    def tearDown(self) -> None:
        self.directory.cleanup()

    def verdict(
        self, revision: str = "immutable-current", exit_code: int = 101
    ) -> None:
        (self.root / "cold-verdict.json").write_text(
            json.dumps({"revision": revision, "exit_code": exit_code})
        )

    def test_first_gate_matches_exact_current_ci_arguments(self) -> None:
        self.assertEqual(
            current.current_vulkan_arguments(),
            ["cargo", "test", "-p", "martensite-vulkan", "--all-targets", "--locked"],
        )

    def test_gate_has_no_observation_or_renderer_before_exact_command(self) -> None:
        fake = NamedCurrentLifecycle()
        with ExitStack() as stack:
            for name, method in {
                "record_source": fake.source,
                "require_clean_source": fake.clean,
                "run_cold_gate": fake.gate,
            }.items():
                stack.enter_context(patch.object(current, name, method))
            current.current_vulkan_gate(self.probe)
        self.assertEqual(
            fake.events,
            [
                "pin",
                "gate-source-before",
                "exact-first-gpu-command",
                "gate-source-after",
            ],
        )
        self.assertEqual(fake.arguments, current.current_vulkan_arguments())

    def test_missing_original_verdict_blocks_observation(self) -> None:
        with self.assertRaisesRegex(
            ValueError, "cold-verdict.*expected completed original"
        ):
            current.require_current_gate(self.probe)

    def test_wrong_immutable_revision_blocks_observation(self) -> None:
        self.verdict("wrong")
        with self.assertRaisesRegex(
            ValueError, "wrong.*expected immutable immutable-current"
        ):
            current.require_current_gate(self.probe)

    def test_red_original_verdict_allows_evidence_without_replacement(self) -> None:
        self.verdict(exit_code=-11)
        fake = NamedCurrentLifecycle()
        with ExitStack() as stack:
            for name, method in fake.observations().items():
                stack.enter_context(patch.object(current, name, method))
            current.current_vulkan_observations(self.probe)
        self.assertIn("strict-and-raw", fake.events)
        self.assertLess(
            fake.events.index("driver-before"), fake.events.index("capture-build")
        )
        self.assertEqual(
            json.loads((self.root / "cold-verdict.json").read_text())["exit_code"], -11
        )


if __name__ == "__main__":
    unittest.main()
