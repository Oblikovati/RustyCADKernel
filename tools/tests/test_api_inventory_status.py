"""A historical snapshot never becomes current after source or evidence edits."""

from pathlib import Path
import json
import tempfile
import unittest

from api_inventory.inputs import JSONObject, digest
from api_inventory_status import compiler_status_lines
from api_tool_hashes import IMPLEMENTATION_FILES, implementation_hashes


class CompilerSnapshotFixture:
    """Named filesystem fixture containing only deterministic local evidence."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.create_tool_inputs()
        self.folder = root / "porting/api-inventory"
        self.folder.mkdir(parents=True)
        self.report: JSONObject = {
            "input_sha256": self.input_hashes(),
            "rust_capture_inputs": {"workspace_inputs_sha256": "source"},
            "tool_implementation_sha256": implementation_hashes(root),
            "artifacts": self.archive_hashes(),
        }
        (self.folder / "completeness.json").write_text(json.dumps(self.report))

    def create_tool_inputs(self) -> None:
        for paths in IMPLEMENTATION_FILES.values():
            for relative in paths:
                path = self.root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(f"fixture:{relative}")

    def input_hashes(self) -> JSONObject:
        paths = {
            "parity": self.root / "porting/parity.json",
            "cpp": self.folder / "occt-declarations.jsonl.gz",
            "cpp_manifest": self.folder / "occt-manifest.json",
            "headers": self.root / "porting/api-header-manifest.json",
        }
        for path in paths.values():
            path.write_text("{}")
        table = self.folder / "occt-macros.jsonl.gz"
        table.write_text("{}")
        paths["cpp_manifest"].write_text(
            json.dumps({"artifacts": {"macros": {"sha256": digest(table)}}})
        )
        return {key: digest(path) for key, path in paths.items()}

    def archive_hashes(self) -> JSONObject:
        artifacts: JSONObject = {}
        for name in ("rust-api", "api-crosswalk"):
            path = self.folder / f"{name}.json.gz"
            path.write_text("{}")
            artifacts[name] = {"sha256": digest(path)}
        return artifacts


class CompilerStatusTests(unittest.TestCase):
    def test_missing_snapshot_stays_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            self.assertIn(
                "UNKNOWN", compiler_status_lines(Path(directory), "fixture")[0]
            )

    def test_source_and_ledger_changes_mark_snapshot_stale(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            CompilerSnapshotFixture(root)
            self.assertIn(
                "fingerprints current", compiler_status_lines(root, "source")[0]
            )
            self.assertIn("STALE", compiler_status_lines(root, "changed")[0])
            (root / "porting/parity.json").write_text("changed")
            self.assertIn("STALE", compiler_status_lines(root, "source")[0])

    def test_modified_or_missing_evidence_is_stale(self) -> None:
        names = [
            "occt-declarations.jsonl.gz",
            "occt-manifest.json",
            "occt-macros.jsonl.gz",
            "rust-api.json.gz",
            "api-crosswalk.json.gz",
        ]
        for name in names:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture = CompilerSnapshotFixture(root)
                (fixture.folder / name).write_text("changed")
                self.assertIn("STALE", compiler_status_lines(root, "source")[0])
                (fixture.folder / name).unlink()
                self.assertIn("STALE", compiler_status_lines(root, "source")[0])

    def test_legacy_snapshot_without_tool_metadata_is_stale(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = CompilerSnapshotFixture(root)
            fixture.report.pop("tool_implementation_sha256")
            (fixture.folder / "completeness.json").write_text(json.dumps(fixture.report))
            self.assertIn("STALE", compiler_status_lines(root, "source")[0])

    def test_tool_edit_or_removal_makes_published_summary_stale(self) -> None:
        for remove in (False, True):
            with self.subTest(remove=remove), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                fixture = CompilerSnapshotFixture(root)
                tool = root / "tools/rustdoc_declarations.py"
                if remove:
                    tool.unlink()
                else:
                    tool.write_text("changed")
                self.assertIn("STALE", compiler_status_lines(root, "source")[0])
