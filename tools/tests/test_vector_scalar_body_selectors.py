"""Keep generic and boolean scalar bodies explicit in the six existing mappings."""

from __future__ import annotations

import copy
import gzip
import hashlib
import json
import unittest
from pathlib import Path

from api_crosswalk import row_rust_candidates, rust_crates

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = Path(__file__).parent / "fixtures/vector-scalar-body-selectors.json.gz"
PATHS = {
    f"rustycad_tkernel::ncollection::Vec{size}::{method}"
    for size in (2, 3, 4)
    for method in ("multiply", "multiplied")
}


class VectorScalarBodySelectorTests(unittest.TestCase):
    def setUp(self) -> None:
        with gzip.open(FIXTURE, "rt") as stream:
            self.crates = rust_crates(json.load(stream))
        ledger = json.loads((ROOT / "porting/parity.json").read_text())
        self.rows = [row for row in ledger["api"] if row.get("rust_item") in PATHS]

    def test_six_mappings_select_twelve_actual_bodies(self) -> None:
        self.assertEqual(len(self.rows), 6)
        for row in self.rows:
            with self.subTest(upstream=row["upstream_id"]):
                status, bodies = row_rust_candidates(row, self.crates)
                self.assertEqual(status, "unique")
                self.assertEqual(len(bodies), 2)
                self.assertTrue(all(body["signature"]["has_body"] for body in bodies))
                contexts = [
                    body["impl_context"]["generics"]["params"] for body in bodies
                ]
                self.assertEqual(
                    sorted(bool(context) for context in contexts), [False, True]
                )

    def test_fixture_sources_remain_exact(self) -> None:
        for path in PATHS:
            for body in self.crates["rustycad_tkernel"][path]:
                digest = hashlib.sha256(
                    (ROOT / body["source_file"]).read_bytes()
                ).hexdigest()
                self.assertEqual(digest, body["source_sha256"])

    def test_removing_selection_restores_real_ambiguity(self) -> None:
        for row in self.rows:
            unselected = {
                key: value
                for key, value in row.items()
                if key != "rust_declaration_selectors"
            }
            status, bodies = row_rust_candidates(unselected, self.crates)
            self.assertEqual((status, len(bodies)), ("ambiguous", 2))

    def test_wrong_body_digest_cannot_resolve(self) -> None:
        for row in self.rows:
            altered = copy.deepcopy(row)
            altered["rust_declaration_selectors"][0]["declaration_sha256"] = "0" * 64
            self.assertEqual(
                row_rust_candidates(altered, self.crates)[0], "missing_rust_selection"
            )

    def test_wrong_specialization_cannot_resolve(self) -> None:
        for row in self.rows:
            altered = copy.deepcopy(row)
            selectors = altered["rust_declaration_selectors"]
            selectors[0]["impl_context"] = selectors[1]["impl_context"]
            self.assertEqual(
                row_rust_candidates(altered, self.crates)[0], "missing_rust_selection"
            )

    def test_duplicate_selection_is_rejected(self) -> None:
        for row in self.rows:
            altered = copy.deepcopy(row)
            altered["rust_declaration_selectors"] *= 2
            self.assertEqual(
                row_rust_candidates(altered, self.crates)[0], "invalid_rust_selectors"
            )


if __name__ == "__main__":
    unittest.main()
