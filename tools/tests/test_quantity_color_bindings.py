"""Keep color bindings attached to exact native and public Rust declarations."""

from __future__ import annotations

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners
from tkernel_d1_review import kept_partial_rows, review_outcome

ROOT = Path(__file__).resolve().parents[2]
# Rows the 2026-10-15 D1 review kept partial (class-A gaps).
D1_KEPT = kept_partial_rows(ROOT)


class QuantityColorBindingsTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls: type["QuantityColorBindingsTest"]) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))
        parity = json.loads((ROOT / "porting/parity.json").read_text())
        cls.rows = {row["upstream_id"]: row for row in parity["api"]}
        cls.audit = json.loads(
            (ROOT / "porting/audits/quantity-color-bindings.json").read_text()
        )

    def test_seven_bindings_resolve_without_erasing_behavior_limits(self) -> None:
        self.assertEqual(len(self.audit["bindings"]), 7)
        for binding in self.audit["bindings"]:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            self.assertEqual(self.rows[selector]["rust_items"], binding["rust_items"])
            row = self.rows[selector]
            if binding["status"] == "partial":
                # The D1 review (tkernel_d1_review) may have closed a partial binding.
                self.assertIn(review_outcome(row, D1_KEPT), ("implemented", "partial"))
            else:
                self.assertEqual(row["status"], binding["status"])

    def test_rgb_accessor_is_a_borrowed_public_method(self) -> None:
        selector = "Quantity_Color::Rgb() const noexcept"
        self.assertEqual(
            self.rows[selector]["rust_item"],
            "rustycad_tkernel::quantity::Color::rgb_ref",
        )
        self.assertEqual(review_outcome(self.rows[selector], D1_KEPT), "implemented")


if __name__ == "__main__":
    unittest.main()
