"""Keep the borrowed DataMap Bind target tied to its exact native declaration."""

import gzip
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from normalize_rustdoc_inventory import make_reachable_inventory

SELECTOR = "NCollection_DataMap::Bind(const TheKeyType&,const TheItemType&)"
CPP_ID = "a5f48fb286110250b400b7c10f88e6e51ac50f96f25230b85c37f876df86ac12"
RUST_TARGET = "rustycad_tkernel::ncollection::DataMap::bind_ref"


class DataMapBorrowedBindMappingTest(unittest.TestCase):
    def test_existing_const_bind_row_points_to_borrowed_adapter(self) -> None:
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        rows = [row for row in parity["api"] if row["upstream_id"] == SELECTOR]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["rust_item"], RUST_TARGET)
        self.assertEqual(rows[0]["status"], "partial")
        self.assertEqual(len(read_cpp_rows(CPP_ID)), 1)
        self.assertEqual(len(read_rust_rows(RUST_TARGET)), 1)


def read_cpp_rows(cpp_id: str) -> list[dict[str, object]]:
    matches: list[dict[str, object]] = []
    with gzip.open(
        ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
    ) as stream:
        for line in stream:
            row: dict[str, object] = json.loads(line)
            if row["id"] == cpp_id:
                matches.append(row)
    return matches


def read_rust_rows(rust_path: str) -> list[dict[str, object]]:
    capture = (
        ROOT / "porting/runtime-evidence/datamap-borrowed-bind-tkernel-rustdoc.json.gz"
    )
    with gzip.open(capture, "rt") as stream:
        raw = json.load(stream)
    inventory = make_reachable_inventory(raw, ROOT, "rustycad_tkernel")
    return [item for item in inventory["declarations"] if item["path"] == rust_path]
