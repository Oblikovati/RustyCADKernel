import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import diagnose_release_renderer as diagnostic


class NamedDiagnosticFake(diagnostic.RendererDiagnosticProcess):
    def __init__(
        self,
        root: Path,
        output: Path,
        replies: dict[str, subprocess.CompletedProcess[str]],
    ) -> None:
        super().__init__(root, output)
        self.replies = replies
        self.calls = []

    def run(self, label: str, arguments: list[str]) -> subprocess.CompletedProcess[str]:
        self.calls.append((label, arguments))
        return self.replies.get(
            label, subprocess.CompletedProcess(arguments, 0, "", "")
        )


class DiagnosticContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.binary = self.root / "binary"
        self.binary.write_bytes(b"frozen")
        self.output = self.root / "output"
        self.output.mkdir()

    def tearDown(self) -> None:
        self.directory.cleanup()

    def fake(
        self, replies: dict[str, subprocess.CompletedProcess[str]] | None = None
    ) -> NamedDiagnosticFake:
        return NamedDiagnosticFake(self.root, self.output, replies or {})

    def artifact(self, name: str) -> str:
        return json.dumps(
            {
                "reason": "compiler-artifact",
                "target": {"name": name},
                "executable": str(self.binary),
            }
        )

    def test_digest_and_artifact_selection(self) -> None:
        self.assertEqual(
            diagnostic.digest_file(self.binary), hashlib.sha256(b"frozen").hexdigest()
        )
        selected = diagnostic.cargo_executables("not-json\n" + self.artifact("toolkit"))
        self.assertEqual(selected[0].name, "toolkit")
        self.assertEqual(diagnostic.cargo_executables("{}"), [])

    def test_injected_revision_is_not_labeled_as_the_older_release(self) -> None:
        expected = "4c884b0e01435243f4c23ee7eda93aaff9029641"
        probe = diagnostic.RendererDiagnosticProcess(self.root, self.output, expected)
        probe.persist()
        receipt = json.loads((self.output / "commands.json").read_text())
        self.assertEqual(receipt["revision"], expected)
        self.assertNotEqual(receipt["revision"], diagnostic.REVISION)

    def test_real_process_preserves_red_exit(self) -> None:
        probe = diagnostic.RendererDiagnosticProcess(self.root, self.output)
        result = probe.run("red", ["python3", "-c", "raise SystemExit(7)"])
        self.assertEqual(result.returncode, 7)
        self.assertEqual(
            json.loads((self.output / "commands.json").read_text())["commands"][0][
                "exit_code"
            ],
            7,
        )

    def test_source_pin_and_hashes(self) -> None:
        replies = {
            "source-head": subprocess.CompletedProcess([], 0, diagnostic.REVISION, ""),
            "source-files": subprocess.CompletedProcess([], 0, "binary\0", ""),
        }
        diagnostic.record_source(self.fake(replies))
        self.assertIn(
            "binary", json.loads((self.output / "source-sha256.json").read_text())
        )
        with self.assertRaisesRegex(ValueError, "expected immutable"):
            diagnostic.record_source(self.fake())

    def loader_fake(self) -> NamedDiagnosticFake:
        reply = subprocess.CompletedProcess(
            [], 0, f" libsample.so (libc6) => {self.binary}\n", ""
        )
        return self.fake({"loader-cache": reply})

    def test_absolute_and_relative_library_paths(self) -> None:
        probe = self.loader_fake()
        actual = diagnostic.resolve_driver_library(
            probe, self.root / "icd.json", self.binary
        )
        self.assertEqual(actual, self.binary)
        actual = diagnostic.resolve_driver_library(
            probe, self.root / "icd.json", Path("folder/lib.so")
        )
        self.assertEqual(actual, self.root / "folder/lib.so")

    def test_soname_cache_and_missing_library(self) -> None:
        probe = self.loader_fake()
        with patch.dict(os.environ, {"LD_LIBRARY_PATH": ""}):
            actual = diagnostic.resolve_driver_library(
                probe, self.root / "icd.json", Path("libsample.so")
            )
            self.assertEqual(actual, self.binary)
            with self.assertRaisesRegex(ValueError, "expected resolvable"):
                diagnostic.resolve_driver_library(
                    probe, self.root / "icd.json", Path("missing.so")
                )

    def test_library_environment_precedes_cache(self) -> None:
        with patch.dict(os.environ, {"LD_LIBRARY_PATH": str(self.root)}):
            actual = diagnostic.resolve_driver_library(
                self.loader_fake(), self.root / "icd.json", Path("binary")
            )
            self.assertEqual(actual, self.binary)

    def test_driver_environment_and_libraries(self) -> None:
        icd = self.root / "icd.json"
        icd.write_text(json.dumps({"ICD": {"library_path": str(self.binary)}}))
        with patch.dict(
            os.environ,
            {
                "VK_DRIVER_FILES": str(icd),
                "VK_LAYER_PATH": str(self.root),
                "LD_LIBRARY_PATH": str(self.root),
            },
        ):
            diagnostic.record_driver(self.fake())
        hashes = json.loads((self.output / "libraries-sha256.json").read_text())
        self.assertIn(str(self.binary), hashes)
        self.assertIn(
            "VK_DRIVER_FILES",
            json.loads((self.output / "environment.json").read_text()),
        )

    def artifact_fake(self) -> NamedDiagnosticFake:
        toolkit = subprocess.CompletedProcess([], 0, self.artifact("toolkit"), "")
        captures = "\n".join(self.artifact(name) for name, _, _ in diagnostic.CAPTURES)
        return self.fake(
            {
                "build-toolkit": toolkit,
                "build-captures": subprocess.CompletedProcess([], 0, captures, ""),
            }
        )

    def test_all_strict_failures_still_produce_all_captures(self) -> None:
        probe = self.artifact_fake()
        artifacts = diagnostic.build_artifacts(probe)
        for name in diagnostic.FILTERS:
            probe.replies[f"strict-{name}"] = subprocess.CompletedProcess(
                [], 101, "", "strict red"
            )
        diagnostic.replay_and_capture(probe, artifacts)
        self.assertEqual(
            sum(label.startswith("strict-") for label, _ in probe.calls), 5
        )
        self.assertEqual(
            sum(label.startswith("capture-") for label, _ in probe.calls), 5
        )
        self.assertEqual(
            (self.output / "binaries-before.json").read_text(),
            (self.output / "binaries-after.json").read_text(),
        )

    def test_binary_mutation_is_rejected_after_the_after_receipt_is_saved(self) -> None:
        artifacts = [
            diagnostic.DiagnosticBinary(
                "toolkit", str(self.binary), diagnostic.digest_file(self.binary)
            )
        ]
        self.binary.write_bytes(b"changed")
        with self.assertRaisesRegex(
            ValueError, "changed diagnostic binaries.*expected unchanged"
        ):
            diagnostic.record_binary_bookend(self.fake(), artifacts)
        self.assertTrue((self.output / "binaries-after.json").is_file())

    def test_failed_build_and_missing_toolkit_are_rejected(self) -> None:
        probe = self.artifact_fake()
        probe.replies["build-toolkit"] = subprocess.CompletedProcess([], 1, "", "")
        with self.assertRaisesRegex(ValueError, "expected successful"):
            diagnostic.build_artifacts(probe)
        with self.assertRaisesRegex(ValueError, "expected compiled toolkit"):
            diagnostic.replay_and_capture(probe, [])


if __name__ == "__main__":
    unittest.main()
