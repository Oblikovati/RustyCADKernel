"""Real compiler fixtures distinguish generic/bool Vec and map cursor bodies."""

from __future__ import annotations

import copy
import gzip
import json
from pathlib import Path
import unittest

from api_crosswalk import row_rust_candidates, rust_crates, resolve_status, rust_summary
from api_inventory.inputs import JSONObject
from api_rust_declaration_selectors import declaration_selectors

FIXTURE = Path(__file__).parent / "fixtures/rust-specialization-selectors.json.gz"
VECTOR_DOT = "rustycad_tkernel::ncollection::Vec2::dot"
CURSOR_NEW = "rustycad_tkernel::ncollection::BaseMapBasicIterator::new"
CURSOR_KEY = "rustycad_tkernel::ncollection::BaseMapBasicIterator::key"


def explicit_body_selector(body: JSONObject) -> JSONObject:
    """Copy exact compiler identity; `explicit_body_selector(body)`."""
    return {
        key: copy.deepcopy(body[key])
        for key in ("path", "declaration_sha256", "impl_context")
    }


class RustDeclarationSelectorTests(unittest.TestCase):
    def setUp(self) -> None:
        with gzip.open(FIXTURE, "rt") as stream:
            self.crates = rust_crates(json.load(stream))
        self.vector = self.crates["rustycad_tkernel"][VECTOR_DOT]
        self.row: JSONObject = {
            "rust_item": VECTOR_DOT,
            "status": "partial",
            "rust_declaration_selectors": [explicit_body_selector(self.vector[0])],
        }

    def test_no_selector_keeps_ambiguity_for_all_fixture_paths(self) -> None:
        for path in (VECTOR_DOT, CURSOR_NEW, CURSOR_KEY):
            status, bodies = row_rust_candidates({"rust_item": path}, self.crates)
            self.assertEqual(status, "ambiguous")
            self.assertGreater(len(bodies), 1)

    def test_generic_and_bool_vector_bodies_are_selected_separately(self) -> None:
        for body in self.vector:
            self.row["rust_declaration_selectors"] = [explicit_body_selector(body)]
            status, bodies = row_rust_candidates(self.row, self.crates)
            self.assertEqual((status, bodies), ("unique", [body]))
        self.assertNotEqual(
            self.vector[0]["impl_context"], self.vector[1]["impl_context"]
        )

    def test_mutable_and_const_cursor_constructors_remain_distinct(self) -> None:
        for body in self.crates["rustycad_tkernel"][CURSOR_NEW]:
            row = {
                "rust_item": CURSOR_NEW,
                "rust_declaration_selectors": [explicit_body_selector(body)],
            }
            self.assertEqual(row_rust_candidates(row, self.crates), ("unique", [body]))

    def test_all_three_key_dispatch_bodies_can_be_explicitly_bound(self) -> None:
        bodies = self.crates["rustycad_tkernel"][CURSOR_KEY]
        row = {
            "rust_item": CURSOR_KEY,
            "rust_declaration_selectors": [
                explicit_body_selector(body) for body in bodies
            ],
        }
        self.assertEqual(len(bodies), 3)
        self.assertEqual(row_rust_candidates(row, self.crates), ("unique", bodies))

    def test_nonexistent_digest_does_not_choose_another_specialization(self) -> None:
        self.row["rust_declaration_selectors"][0]["declaration_sha256"] = "0" * 64
        self.assertEqual(
            row_rust_candidates(self.row, self.crates), ("missing_rust_selection", [])
        )

    def test_digest_and_impl_context_must_name_the_same_body(self) -> None:
        self.row["rust_declaration_selectors"][0]["impl_context"] = self.vector[1][
            "impl_context"
        ]
        self.assertEqual(
            row_rust_candidates(self.row, self.crates), ("missing_rust_selection", [])
        )

    def test_duplicate_inventory_bodies_remain_ambiguous(self) -> None:
        self.crates["rustycad_tkernel"][VECTOR_DOT].append(
            copy.deepcopy(self.vector[0])
        )
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "ambiguous_rust_selection"
        )

    def test_repeated_selector_identity_is_invalid(self) -> None:
        self.row["rust_declaration_selectors"] *= 2
        self.assertIsNone(declaration_selectors(self.row, [VECTOR_DOT]))
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
        )

    def test_malformed_selector_lists_are_rejected(self) -> None:
        for invalid in (None, [], "bad", [None], [{}]):
            self.row["rust_declaration_selectors"] = invalid
            self.assertEqual(
                row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
            )

    def test_incomplete_or_unknown_selector_fields_are_rejected(self) -> None:
        original = explicit_body_selector(self.vector[0])
        for field in original:
            selector = dict(original)
            del selector[field]
            self.row["rust_declaration_selectors"] = [selector]
            self.assertEqual(
                row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
            )
        self.row["rust_declaration_selectors"] = [dict(original, unexpected=True)]
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
        )

    def test_invalid_digest_and_context_shapes_are_rejected(self) -> None:
        for field, value in (
            ("path", ""),
            ("declaration_sha256", "bad"),
            ("impl_context", []),
        ):
            selector = explicit_body_selector(self.vector[0])
            selector[field] = value
            self.row["rust_declaration_selectors"] = [selector]
            self.assertEqual(
                row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
            )

    def test_selected_paths_must_match_declared_paths(self) -> None:
        self.row["rust_item"] = CURSOR_KEY
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
        )
        self.row["rust_items"] = [CURSOR_KEY, VECTOR_DOT]
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
        )

    def test_paired_paths_require_every_selector_group_in_declared_order(self) -> None:
        key = self.crates["rustycad_tkernel"][CURSOR_KEY][0]
        self.row["rust_items"] = [VECTOR_DOT, CURSOR_KEY]
        self.row["rust_declaration_selectors"].append(explicit_body_selector(key))
        self.assertEqual(
            row_rust_candidates(self.row, self.crates),
            ("unique", [self.vector[0], key]),
        )
        self.row["rust_declaration_selectors"].reverse()
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "invalid_rust_selectors"
        )

    def test_source_pinning_is_not_bypassed_by_a_selector(self) -> None:
        self.vector[0]["source_sha256"] = "invalid"
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "unverifiable_rust_selection"
        )

    def test_kind_filter_and_selector_must_agree(self) -> None:
        self.row["rust_kind"] = "struct_field"
        self.assertEqual(
            row_rust_candidates(self.row, self.crates)[0], "missing_rust_selection"
        )
        self.row["rust_kind"] = "function"
        self.assertEqual(row_rust_candidates(self.row, self.crates)[0], "unique")

    def test_selection_failures_are_not_hidden_by_partial_status(self) -> None:
        for failure in (
            "invalid_rust_selectors",
            "missing_rust_selection",
            "ambiguous_rust_selection",
            "unverifiable_rust_selection",
        ):
            self.assertEqual(resolve_status("partial", "unique", failure), failure)
        self.assertEqual(resolve_status("partial", "unique", "unique"), "partial")

    def test_report_summary_keeps_selected_compiler_identity(self) -> None:
        summary = rust_summary(self.vector[0])
        self.assertEqual(summary["impl_context"], self.vector[0]["impl_context"])
        self.assertEqual(
            summary["declaration_sha256"], self.vector[0]["declaration_sha256"]
        )


if __name__ == "__main__":
    unittest.main()
