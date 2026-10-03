"""A historical snapshot never becomes current after source or ledger edits."""

from pathlib import Path
import json
import tempfile
import unittest

from api_inventory.inputs import digest
from api_inventory_status import compiler_status_lines


class CompilerStatusTests(unittest.TestCase):
    def test_missing_snapshot_stays_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            self.assertIn(
                "UNKNOWN", compiler_status_lines(Path(directory), "fixture")[0]
            )

    def test_source_and_ledger_changes_mark_snapshot_stale(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "porting/api-inventory"
            folder.mkdir(parents=True)
            ledger = root / "porting/parity.json"
            ledger.write_text("{}")
            report = {
                "input_sha256": {"parity": digest(ledger)},
                "rust_capture_inputs": {"workspace_inputs_sha256": "source"},
            }
            (folder / "completeness.json").write_text(json.dumps(report))
            self.assertIn(
                "fingerprints current", compiler_status_lines(root, "source")[0]
            )
            self.assertIn("STALE", compiler_status_lines(root, "changed")[0])
            ledger.write_text("changed")
            self.assertIn("STALE", compiler_status_lines(root, "source")[0])
