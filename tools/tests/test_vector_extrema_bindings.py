"""Keep primitive extrema mappings attached to exact compiler declarations."""

from __future__ import annotations

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]


class VectorExtremaBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["VectorExtremaBindingsTest"]) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.rows = {row["upstream_id"]: row for row in parity["api"]}
        cls.audit = json.loads(
            (ROOT / "porting/audits/vector-extrema-bindings.json").read_text()
        )

    def test_all_six_selectors_resolve_to_the_pinned_declarations(self) -> None:
        self.assertEqual(len(self.audit["bindings"]), 6)
        for binding in self.audit["bindings"]:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            self.assertEqual(self.rows[selector]["rust_item"], binding["rust_item"])


if __name__ == "__main__":
    unittest.main()
