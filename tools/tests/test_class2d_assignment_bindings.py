"""Keep both classifier assignment alias domains in the API evidence."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]


class Class2dAssignmentBindingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))

    def test_copy_and_move_include_self_assignment(self) -> None:
        audit = json.loads(
            (ROOT / "porting/audits/cslib-class2d-self-assignment.json").read_text()
        )
        parity = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        rows = {row["upstream_id"]: row for row in parity}
        for binding in audit["cpp_bindings"]:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKMath", self.by_name)
            self.assertEqual(status, "unique")
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            self.assertEqual(rows[selector]["status"], "implemented")
            self.assertEqual(rows[selector]["rust_items"], binding["rust_items"])
