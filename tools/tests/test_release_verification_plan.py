"""The shared gate plan retains the actual workflow commands without heavy execution."""

from __future__ import annotations

from pathlib import Path
import tempfile
import unittest

from release_verification_plan import (
    GATE_NAMES,
    VerificationGate,
    verification_gates,
    shader_commands,
    xvfb_example,
)


class ShaderWorkspaceFixture:
    """Owned source fixtures exercise deterministic shader discovery."""

    def __init__(self, root: Path) -> None:
        self.root = root
        folder = root / "crates/vulkan/shaders"
        folder.mkdir(parents=True)
        for name in ("unlit.vert.spv", "unlit.frag.spv", "extra.frag.spv"):
            (folder / name).write_bytes(b"attributed test shader")

    def gates(self) -> dict[str, VerificationGate]:
        """Build the real contract over owned files; e.g. fixture.gates()."""
        return {
            gate.name: gate for gate in verification_gates(self.root, Path("../OCCT"))
        }


class VerificationPlanTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.fixture = ShaderWorkspaceFixture(Path(self.temporary.name))
        self.gates = self.fixture.gates()

    def test_complete_gate_names_and_nonempty_argv(self) -> None:
        self.assertEqual(tuple(self.gates), GATE_NAMES)
        for gate in self.gates.values():
            self.assertTrue(gate.commands)
            self.assertTrue(
                all(
                    command.argv and isinstance(command.argv, tuple)
                    for command in gate.commands
                )
            )

    def test_original_workspace_and_cold_test_commands(self) -> None:
        self.assertEqual(
            self.gates["rust-tests"].commands[0].argv,
            ("cargo", "test-all", "--no-fail-fast"),
        )
        self.assertEqual(
            self.gates["vulkan-cold"].commands[0].argv,
            ("cargo", "test", "-p", "martensite-vulkan", "--all-targets", "--locked"),
        )
        self.assertEqual(
            [c.argv[-1] for c in self.gates["rust-prechecks"].commands],
            ["tools/toolkit_test_suites.py", "tools/check_rust_format.py", "lint"],
        )

    def test_docs_environment_and_coverage_exclusion(self) -> None:
        docs = self.gates["rust-docs"].commands[0]
        self.assertEqual(docs.env, (("RUSTDOCFLAGS", "-D warnings"),))
        self.assertEqual(
            docs.argv,
            ("cargo", "doc", "--workspace", "--all-features", "--no-deps", "--locked"),
        )
        argv = self.gates["coverage"].commands[0].argv
        self.assertEqual(argv[argv.index("--profile") + 1], "ci")
        self.assertEqual(
            argv[argv.index("-E") + 1],
            "not test(=bvh_bug30655::BVH_PairDistanceTest__Bug30655_SphereDistance)",
        )
        self.assertEqual(argv[-3:], ("--lcov", "--output-path", "target/lcov.info"))

    def test_all_shader_variants_and_native_controls_remain(self) -> None:
        commands = self.gates["vulkan-shaders"].commands
        self.assertEqual(len(commands), 7)
        self.assertEqual(commands[0].argv[-1], "crates/vulkan/shaders/extra.frag.spv")
        self.assertEqual(commands[-4].argv[0], "glslangValidator")
        self.assertEqual(commands[-1].argv[-1], "crates/vulkan/shaders/unlit.frag.spv")
        with self.assertRaisesRegex(ValueError, "expected nonempty"):
            shader_commands(self.fixture.root / "missing")

    def test_inventory_source_and_performance_log_are_explicit(self) -> None:
        self.assertEqual(self.gates["inventory"].commands[0].argv[-1], "../OCCT")
        performance = self.gates["vulkan-performance"].commands[0]
        self.assertEqual(
            performance.stdout_path, Path("target/renderer-performance.log")
        )
        self.assertEqual(performance.argv, xvfb_example("renderer_performance", True))
        self.assertEqual(len(self.gates["vulkan-acceptance"].commands), 2)

    def test_workflow_calls_every_shared_gate_and_preserves_diagnostic_pipes(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        workflow = (root / ".github/workflows/ci.yml").read_text()
        for name in GATE_NAMES:
            self.assertIn(
                f"tools/verify_release_gate.py --root . --gate {name}", workflow
            )
        self.assertIn(
            "--gate rust-tests 2>&1 | tee -a target/rust-gate/gate.log", workflow
        )
        self.assertIn(
            "--gate vulkan-cold 2>&1 | tee -a target/vulkan-gate/gate.log", workflow
        )
        self.assertEqual(workflow.count("gate_status=${PIPESTATUS[0]}"), 2)
        self.assertEqual(workflow.count("path: target/verification-tools"), 6)
        self.assertEqual(workflow.count("ref: ${{ github.workflow_sha }}"), 8)
        self.assertEqual(
            workflow.count("ref: ${{ inputs.source_ref || github.sha }}"), 6
        )


if __name__ == "__main__":
    unittest.main()
