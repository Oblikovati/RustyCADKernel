"""Keep default storage callback mappings bound to compiler-visible trait methods."""

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]


class StorageDefaultCallbackBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))

    def test_existing_methods_use_exact_trait_paths(self) -> None:
        audit = json.loads(
            (ROOT / "porting/audits/storage-default-callback-bindings.json").read_text()
        )
        rows = {
            row["upstream_id"]: row
            for row in json.loads((ROOT / "porting/parity.json").read_text())["api"]
        }
        self.assertEqual(len(audit["bindings"]), 4)
        for binding in audit["bindings"]:
            self.assert_binding(binding, rows[binding["selector"]])

    def assert_binding(self, binding: dict[str, str], row: dict[str, str]) -> None:
        status, candidates = cpp_candidates(
            binding["selector"], "TKernel", self.by_name
        )
        self.assertEqual(status, "unique")
        self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
        self.assertEqual(row["rust_item"], binding["rust_path"])
        self.assertEqual(row["status"], binding["previous_status"])
        source = (ROOT / binding["source_file"]).read_bytes()
        self.assertEqual(hashlib.sha256(source).hexdigest(), binding["source_sha256"])
