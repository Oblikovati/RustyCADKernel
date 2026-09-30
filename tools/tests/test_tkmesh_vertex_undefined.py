"""Invalid native mesh-index candidates remain visible and never become exact passes."""
import json
import subprocess
import unittest
from pathlib import Path
from unittest.mock import patch
import verify_tkmesh_vertex_undefined as accounting


class RecordedOutputPath:
    """Named in-memory evidence sink for report validation."""
    def __init__(self, contents: dict[str, str], name: str = "") -> None:
        self.contents = contents
        self.name = name

    def __truediv__(self, name: str) -> "RecordedOutputPath":
        return RecordedOutputPath(self.contents, name)

    def write_text(self, text: str) -> int:
        self.contents[self.name] = text
        return len(text)


class UndefinedIndexProbePair:
    """Native unchecked output and a deterministic translated bounds failure."""
    def __init__(self, native_code: int = 0, rust_code: int = 0,
                 exception: str = "Standard_OutOfRange") -> None:
        self.native_code = native_code
        self.rust_code = rust_code
        self.exception = exception

    def __call__(self, command: list[str], *, input: str, text: bool,
                 capture_output: bool, timeout: int) -> subprocess.CompletedProcess[str]:
        native = command[0] == "native"
        return subprocess.CompletedProcess(command, self.native_code if native else self.rust_code,
                                           stdout="OK 123\n" if native else self.exception + "\n")


class RecordedReplay:
    """Named replay dispatcher recording the declared exception category."""
    def __init__(self) -> None:
        self.inputs: list[str] = []

    def __call__(self, native: Path, rust: Path, request: str,
                 exception: str = "Standard_OutOfRange") -> dict[str, str | int]:
        self.inputs.append(request)
        return {"input": request, "rust_exception": exception}


class UndefinedVertexAccountingTests(unittest.TestCase):
    def test_inventory_keeps_all_indices_and_operations(self) -> None:
        self.assertEqual(len(accounting.candidates()), 9)
        self.assertEqual(len(set(accounting.candidates())), 9)
        self.assertIn("mesh_vertex_invalid 2 2", accounting.candidates())
        self.assertEqual(accounting.empty_defined_corpus(), "")

    def test_existing_probe_selection_preserves_standard_gate_artifacts(self) -> None:
        native, rust = accounting.vertex_probes(accounting.MESH_ORACLE, Path("unused"), True)
        self.assertEqual(native, accounting.MESH_ORACLE.evidence / "native-probe")
        self.assertEqual(rust, accounting.MESH_ORACLE.rust_probe)

    def test_uninitialized_slot_remains_divergent_even_without_native_crash(self) -> None:
        with patch.object(accounting.subprocess, "run", new=UndefinedIndexProbePair()):
            result = accounting.replay_uninitialized(Path("native"), Path("rust"), "request")
        self.assertEqual(result["native_returncode"], 0)
        self.assertEqual(result["rust_exception"], "Standard_OutOfRange")
        self.assertIn("divergence", result["classification"])

    def test_unexpected_process_outcomes_fail_accounting(self) -> None:
        for twin in [UndefinedIndexProbePair(2), UndefinedIndexProbePair(0, -11),
                     UndefinedIndexProbePair(exception="Standard_ConstructionError")]:
            with patch.object(accounting.subprocess, "run", new=twin):
                with self.assertRaises(ValueError):
                    accounting.replay_uninitialized(Path("native"), Path("rust"), "request")

    def test_dispatch_and_report_do_not_claim_exact_success(self) -> None:
        crashes, uninitialized = RecordedReplay(), RecordedReplay()
        with patch.object(accounting, "replay_crash", new=crashes):
            with patch.object(accounting, "replay_uninitialized", new=uninitialized):
                records = accounting.replay_candidates(Path("native"), Path("rust"))
        self.assertEqual(len(crashes.inputs), 6)
        self.assertEqual(len(uninitialized.inputs), 3)
        contents: dict[str, str] = {}
        accounting.write_report(RecordedOutputPath(contents), records)
        report = json.loads(contents["report.json"])
        self.assertFalse(report["complete_parity"])
        self.assertEqual(report["exact_passed"], 0)
        self.assertEqual(report["candidates"], 9)
        self.assertEqual(contents["inputs.txt"].splitlines(), accounting.candidates())


if __name__ == "__main__":
    unittest.main()
