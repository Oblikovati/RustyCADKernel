"""Resolve each OSD error method through its concrete compiler trait implementation."""

from __future__ import annotations

import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]


class OsdErrorTraitBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["OsdErrorTraitBindingsTest"]) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.rows = {row["upstream_id"]: row for row in parity["api"]}
        cls.audit = json.loads(
            (ROOT / "porting/audits/osd-error-trait-bindings.json").read_text()
        )

    def test_twenty_four_native_methods_have_concrete_trait_pairs(self) -> None:
        self.assertEqual(len(self.audit["bindings"]), 24)
        for binding in self.audit["bindings"]:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            self.assertEqual(self.rows[selector]["rust_items"], binding["rust_items"])
            self.assertIn("OsdErrorAccess::", binding["rust_items"][0])
            self.assertIn(
                "::<rustycad_tkernel::osd::OsdErrorAccess>::", binding["rust_items"][1]
            )

    def test_all_compiled_trait_targets_match_the_current_source(self) -> None:
        with gzip.open(ROOT / "porting/api-inventory/rust-api.json.gz", "rt") as stream:
            inventory = json.load(stream)
        crate = next(
            row for row in inventory["crates"] if row["crate"] == "rustycad_tkernel"
        )
        for target in self.audit["compiled_targets"]:
            matches = [
                row for row in crate["declarations"] if row["path"] == target["path"]
            ]
            self.assertEqual(len(matches), 1, target["path"])
            digest = hashlib.sha256(
                (ROOT / target["source_file"]).read_bytes()
            ).hexdigest()
            self.assertEqual(matches[0]["source_sha256"], digest)
            self.assertEqual(target["source_sha256"], digest)
            self.assertTrue(matches[0]["signature"]["has_body"])


if __name__ == "__main__":
    unittest.main()
