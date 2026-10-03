"""Cargo freshness must not reuse a different toolkit's same-named example."""

import json
from pathlib import Path
import unittest
from unittest.mock import patch
from toolkit_numeric import NumericOracle, ROOT, build_rust_probe


class FakeCargoFilesystem:
    """Named fake for artifact publication, removal and Cargo execution."""

    def __init__(self) -> None:
        self.files: dict[Path, str] = {
            ROOT / "target/debug/examples/numeric_probe": "wrong-toolkit"
        }
        self.commands: list[list[str]] = []
        self.fail_build = False
        self.target_directory = ROOT / "target/debug/examples"

    def remove(self, path: Path, *, missing_ok: bool) -> None:
        assert missing_ok
        self.files.pop(path, None)

    def run(self, command: list[str], log: Path) -> str:
        self.commands.append(command)
        shared = self.target_directory / command[6]
        if self.fail_build:
            raise RuntimeError("fake Cargo build failed")
        # Cargo may keep a fresh target; unique target names keep package identity.
        if shared not in self.files:
            self.files[shared] = command[4]
        artifact = {
            "reason": "compiler-artifact",
            "target": {"name": command[6], "kind": ["example"]},
            "executable": str(shared),
        }
        return (
            json.dumps(artifact)
            + "\n"
            + json.dumps(
                {
                    "reason": "build-finished",
                    "success": True,
                }
            )
        )

    def copy(self, source: Path, destination: Path) -> None:
        self.files[destination] = self.files[source]


def empty_corpus() -> str:
    return ""


class ToolkitProbeBinaryTests(unittest.TestCase):
    def test_selected_package_replaces_stale_example_and_keeps_private_snapshot(
        self,
    ) -> None:
        fake = FakeCargoFilesystem()
        first = NumericOracle("First", "", [], "rustycad-first", empty_corpus)
        second = NumericOracle("Second", "", [], "rustycad-second", empty_corpus)
        with patch("toolkit_numeric.run", fake.run), patch(
            "toolkit_numeric.shutil.copy2", fake.copy
        ), patch.object(Path, "unlink", autospec=True, side_effect=fake.remove):
            build_rust_probe(first, first.evidence)
            build_rust_probe(second, second.evidence)
        self.assertEqual(fake.commands[0][6], "rustycad-first-numeric-probe")
        self.assertEqual(fake.commands[1][6], "rustycad-second-numeric-probe")
        self.assertEqual(fake.files[first.rust_probe], "rustycad-first")
        self.assertEqual(fake.files[second.rust_probe], "rustycad-second")

    def test_failed_build_removes_previous_toolkit_snapshot(self) -> None:
        fake = FakeCargoFilesystem()
        oracle = NumericOracle("First", "", [], "rustycad-first", empty_corpus)
        fake.files[oracle.rust_probe] = "outdated-success"
        fake.fail_build = True
        with patch("toolkit_numeric.run", fake.run), patch(
            "toolkit_numeric.shutil.copy2", fake.copy
        ), patch.object(Path, "unlink", autospec=True, side_effect=fake.remove):
            with self.assertRaisesRegex(RuntimeError, "fake Cargo build failed"):
                build_rust_probe(oracle, oracle.evidence)
        self.assertNotIn(oracle.rust_probe, fake.files)

    def test_custom_target_directory_uses_reported_executable(self) -> None:
        fake = FakeCargoFilesystem()
        fake.target_directory = Path("/isolated-target/debug/examples")
        oracle = NumericOracle("First", "", [], "rustycad-first", empty_corpus)
        stale = ROOT / "target/debug/examples" / oracle.rust_example
        fake.files[stale] = "stale-worktree-build"
        with patch("toolkit_numeric.run", fake.run), patch(
            "toolkit_numeric.shutil.copy2", fake.copy
        ), patch.object(Path, "unlink", autospec=True, side_effect=fake.remove):
            build_rust_probe(oracle, oracle.evidence)
        self.assertEqual(fake.files[oracle.rust_probe], "rustycad-first")
        self.assertIn("--message-format=json", fake.commands[0])
