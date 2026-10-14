"""Named process fakes prove original status/env/output contracts without builds or graphics."""

from __future__ import annotations

import contextlib
import io
from pathlib import Path
import subprocess
import tempfile
from typing import TextIO
import unittest
from unittest.mock import patch

from release_verification_plan import GateCommand, VerificationGate
from verify_release_gate import NativeGateExecutor, gate_arguments, run_gate, main
from test_release_verification_plan import ShaderWorkspaceFixture


class RecordingGateExecutor:
    """Records argv/environment; configured status models the actual child."""

    def __init__(self, statuses: tuple[int, ...]) -> None:
        self.statuses = iter(statuses)
        self.calls: list[tuple[GateCommand, Path, dict[str, str]]] = []

    def run(self, command: GateCommand, root: Path, environment: dict[str, str]) -> int:
        """Record without launching external work; e.g. fake.run(command, root, env)."""
        self.calls.append((command, root, dict(environment)))
        return next(self.statuses)


class NamedChildProcess:
    """Models native subprocess output and a nonzero child exit."""

    def __init__(self) -> None:
        self.calls: list[tuple[str, ...]] = []

    def __call__(
        self,
        argv: tuple[str, ...],
        *,
        cwd: Path,
        env: dict[str, str],
        stdout: TextIO | None = None,
    ) -> subprocess.CompletedProcess[str]:
        self.calls.append(argv)
        if stdout is not None:
            stdout.write("actual performance output\n")
        return subprocess.CompletedProcess(argv, 17)


class GateRunnerTests(unittest.TestCase):
    def test_failure_stops_without_reinterpreting_argv_or_overriding_vvl(self) -> None:
        commands = (
            GateCommand(("cargo", "test", "space argument")),
            GateCommand(("never",)),
        )
        fake = RecordingGateExecutor((101,))
        inherited = {
            "VK_LAYER_PATH": "/verified/layer",
            "LD_LIBRARY_PATH": "/verified/lib",
        }
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            status = run_gate(
                VerificationGate("test", commands), Path("/root"), inherited, fake
            )
        self.assertEqual(status, 101)
        self.assertEqual(len(fake.calls), 1)
        self.assertEqual(fake.calls[0][2], inherited)
        self.assertIn("'space argument'", output.getvalue())

    def test_env_override_is_command_local_and_all_success_commands_execute(
        self,
    ) -> None:
        commands = (
            GateCommand(("first",), (("RUSTDOCFLAGS", "-D warnings"),)),
            GateCommand(("second",)),
        )
        fake = RecordingGateExecutor((0, 0))
        inherited = {"RUSTDOCFLAGS": "original"}
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(
                run_gate(
                    VerificationGate("test", commands), Path("."), inherited, fake
                ),
                0,
            )
        self.assertEqual(fake.calls[0][2]["RUSTDOCFLAGS"], "-D warnings")
        self.assertEqual(fake.calls[1][2]["RUSTDOCFLAGS"], "original")
        self.assertEqual(inherited["RUSTDOCFLAGS"], "original")

    def test_signal_status_uses_shell_convention(self) -> None:
        fake = RecordingGateExecutor((-11,))
        with contextlib.redirect_stdout(io.StringIO()):
            status = run_gate(
                VerificationGate("test", (GateCommand(("child",)),)),
                Path("."),
                {},
                fake,
            )
        self.assertEqual(status, 139)

    def test_native_executor_retains_stdout_path_and_nonzero_status(self) -> None:
        child = NamedChildProcess()
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch("verify_release_gate.subprocess.run", child):
                command = GateCommand(
                    ("performance",), stdout_path=Path("target/result.log")
                )
                self.assertEqual(NativeGateExecutor().run(command, root, {}), 17)
                self.assertEqual(
                    NativeGateExecutor().run(GateCommand(("plain",)), root, {}), 17
                )
            self.assertEqual(
                (root / "target/result.log").read_text(), "actual performance output\n"
            )
        self.assertEqual(child.calls, [("performance",), ("plain",)])

    def test_cli_root_source_and_actual_selected_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = ShaderWorkspaceFixture(Path(temporary))
            fake = RecordingGateExecutor((0, 0))
            with patch("verify_release_gate.NativeGateExecutor", return_value=fake):
                with contextlib.redirect_stdout(io.StringIO()):
                    argv = [
                        "--gate",
                        "inventory",
                        "--root",
                        str(fixture.root),
                        "--source",
                        "../OCCT",
                    ]
                    self.assertEqual(main(argv), 0)
            self.assertEqual(fake.calls[0][0].argv[-1], "../OCCT")
            self.assertEqual(fake.calls[0][1], fixture.root)
        self.assertIsNone(gate_arguments(["--gate", "rust-tests"]).source)

    def test_cli_default_source_and_unknown_gate_rejection(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            fixture = ShaderWorkspaceFixture(Path(temporary))
            fake = RecordingGateExecutor((0, 0))
            with patch("verify_release_gate.NativeGateExecutor", return_value=fake):
                with contextlib.redirect_stdout(io.StringIO()):
                    main(["--gate", "inventory", "--root", str(fixture.root)])
            self.assertEqual(
                fake.calls[0][0].argv[-1], str(fixture.root / "target/occt-source")
            )
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(
            SystemExit
        ) as raised:
            gate_arguments(["--gate", "missing"])
        self.assertEqual(raised.exception.code, 2)


if __name__ == "__main__":
    unittest.main()
