"""Inventory reports reject missing, changed or truncated compressed artifacts."""

from __future__ import annotations

import io
import json
import unittest
from pathlib import Path
from unittest.mock import patch

from api_inventory.inputs import JSONObject
from report_returned_type_inventory import read_table


class FakeInventoryArtifacts:
    """Named inventory I/O fake for artifact authentication failures."""

    def __init__(self) -> None:
        self.rows: list[JSONObject] = [{"id": "row"}]
        self.recorded_rows = 1
        self.actual_hash = "bound"
        self.omit_artifacts = False

    def manifest(self, path: Path) -> JSONObject:
        if self.omit_artifacts:
            return {}
        return {
            "artifacts": {
                "declarations": {
                    "path": "rows.gz",
                    "sha256": "bound",
                    "rows": self.recorded_rows,
                }
            }
        }

    def digest(self, path: Path) -> str:
        return self.actual_hash

    def open(self, path: Path, mode: str) -> io.StringIO:
        return io.StringIO("".join(json.dumps(row) + "\n" for row in self.rows))


class InventoryArtifactTests(unittest.TestCase):
    def setUp(self) -> None:
        self.fake = FakeInventoryArtifacts()
        for target, method in [
            ("read_object", self.fake.manifest),
            ("digest", self.fake.digest),
            ("gzip.open", self.fake.open),
        ]:
            patcher = patch(
                "report_returned_type_inventory." + target, side_effect=method
            )
            patcher.start()
            self.addCleanup(patcher.stop)

    def test_recorded_table_is_read(self) -> None:
        self.assertEqual(read_table(Path("/inventory"), "declarations"), self.fake.rows)

    def test_missing_manifest_entry_is_rejected(self) -> None:
        self.fake.omit_artifacts = True
        with self.assertRaisesRegex(TypeError, "expected manifest artifact object"):
            read_table(Path("/inventory"), "declarations")

    def test_changed_digest_is_rejected(self) -> None:
        self.fake.actual_hash = "changed"
        with self.assertRaisesRegex(ValueError, "expected SHA256"):
            read_table(Path("/inventory"), "declarations")

    def test_boolean_row_count_is_rejected(self) -> None:
        self.fake.recorded_rows = True
        with self.assertRaisesRegex(ValueError, "expected True rows, got 1"):
            read_table(Path("/inventory"), "declarations")

    def test_row_count_mismatch_is_rejected(self) -> None:
        self.fake.recorded_rows = 2
        with self.assertRaisesRegex(ValueError, "expected 2 rows, got 1"):
            read_table(Path("/inventory"), "declarations")


if __name__ == "__main__":
    unittest.main()
