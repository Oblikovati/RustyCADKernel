"""Impls with raw-pointer or projection trait arguments stay visible with exact identities."""

from __future__ import annotations

import unittest
from pathlib import Path

from api_inventory_reexports import structural_gaps
from api_rust_candidates import rust_candidates
from rustdoc_generic_arguments import display_type, trait_arguments
from rustdoc_impl_ownership import render_linked_impl
from rustdoc_pointer_impls import pointer_anchor_status, pointer_impl_entries, pointer_self_text

PATHS = {
    "1": {"crate_id": 0, "path": ["krate", "standard", "handle", "Handle"], "kind": "struct"},
    "2": {"crate_id": 1, "path": ["core", "cmp", "PartialEq"], "kind": "trait"},
    "3": {"crate_id": 0, "path": ["krate", "bvh", "BvhVector"], "kind": "trait"},
    "9": {"crate_id": 4, "path": ["krate", "standard", "handle", "Handle"], "kind": "struct"},
}
GENERIC_T = {"generic": "T"}
HANDLE_T = {"resolved_path": {"path": "Handle", "id": 1, "args": {
    "angle_bracketed": {"args": [{"type": GENERIC_T}], "constraints": []}}}}
POINTER_T = {"raw_pointer": {"is_mutable": False, "type": GENERIC_T}}


def angle(*types: dict) -> dict:
    return {"angle_bracketed": {"args": [{"type": value} for value in types], "constraints": []}}


def pointer_impl(anchor: dict = HANDLE_T) -> dict:
    """`impl<T> PartialEq<Handle<T>> for *const T` as rustdoc records it."""
    return {"for": POINTER_T, "trait": {"path": "PartialEq", "id": 2, "args": angle(anchor)},
            "generics": {"params": [], "where_predicates": []}, "items": [7]}


class FakeImplState:
    """Minimal walk state; records gaps and owned impl ids like the normalizer's state."""

    def __init__(self, paths: dict) -> None:
        self.items: dict = {}
        self.paths = paths
        self.source_root = Path(".")
        self.unresolved: list[str] = []
        self.foreign_impl_links: dict = {}
        self.foreign_impl_candidates: list = []
        self.owned_impl_ids: set[str] = set()


class FakeImplRenderer:
    """Stands in for visible_impl_items: one member rendered with the owner-style path."""

    def __init__(self) -> None:
        self.calls = 0

    def __call__(self, impl, path, items, paths, source_root, unresolved) -> list[dict]:
        self.calls += 1
        return [{"path": f"{path}::<core::cmp::PartialEq<x>>::eq", "kind": "function",
                 "source_file": "crates/k/src/h.rs", "source_sha256": "0" * 64}]


class DisplayTest(unittest.TestCase):
    def test_raw_pointers_render_like_rust(self) -> None:
        self.assertEqual(display_type(POINTER_T, PATHS), "*const T")
        mutable = {"raw_pointer": {"is_mutable": True, "type": HANDLE_T}}
        self.assertEqual(display_type(mutable, PATHS), "*mut krate::standard::handle::Handle<T>")
        self.assertIsNone(display_type({"raw_pointer": {"type": GENERIC_T}}, PATHS))

    def test_projection_spells_its_trait(self) -> None:
        projection = {"qualified_path": {"name": "Scalar", "args": None, "self_type": {"generic": "V"},
                                         "trait": {"path": "BvhVector", "id": 3, "args": None}}}
        self.assertEqual(display_type(projection, PATHS), "<V as krate::bvh::BvhVector>::Scalar")
        inherent = {"qualified_path": {"name": "Scalar", "args": None, "self_type": GENERIC_T, "trait": None}}
        self.assertIsNone(display_type(inherent, PATHS))

    def test_lifetime_arguments_are_kept(self) -> None:
        arguments = {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": GENERIC_T}], "constraints": []}}
        self.assertEqual(trait_arguments(arguments, PATHS), "<'_, T>")
        bad = {"angle_bracketed": {"args": [{"lifetime": "x"}], "constraints": []}}
        self.assertIsNone(trait_arguments(bad, PATHS))


class PointerSelfImplTest(unittest.TestCase):
    def test_anchor_status(self) -> None:
        self.assertEqual(pointer_self_text(pointer_impl(), PATHS), "*const T")
        self.assertEqual(pointer_anchor_status(pointer_impl(), "1", PATHS), "emit")
        self.assertEqual(pointer_anchor_status(pointer_impl(), "9", PATHS),
                         "pointer-self-anchor-not-in-trait-arguments")
        facade = pointer_impl({"resolved_path": {"path": "Handle", "id": 9, "args": None}})
        self.assertEqual(pointer_anchor_status(facade, "9", PATHS), "skip")

    def test_members_are_renamed_to_ufcs(self) -> None:
        rows = pointer_impl_entries([{"path": "krate::Handle::<x>::eq"}], "*const T", "core::cmp::PartialEq<H>")
        self.assertEqual(rows[0]["path"], "<*const T as core::cmp::PartialEq<H>>::eq")
        self.assertEqual(rows[0]["pointer_self_type"], "*const T")
        self.assertIn("declaration_sha256", rows[0])

    def test_defining_crate_emits_the_impl_once(self) -> None:
        state, renderer = FakeImplState(PATHS), FakeImplRenderer()
        entries = render_linked_impl(5, "1", "krate::standard::Handle", pointer_impl(), state, renderer)
        expected = "<*const T as core::cmp::PartialEq<krate::standard::handle::Handle<T>>>::eq"
        self.assertEqual([row["path"] for row in entries], [expected])
        self.assertEqual((state.unresolved, state.owned_impl_ids), ([], {"5"}))

    def test_unanchored_pointer_impl_is_a_named_gap(self) -> None:
        state = FakeImplState(PATHS)
        entries = render_linked_impl(5, "9", "krate::Other", pointer_impl(), state, FakeImplRenderer())
        self.assertEqual(entries, [])
        self.assertEqual(state.unresolved, ["impl:5:pointer-self-anchor-not-in-trait-arguments"])

    def test_crosswalk_resolves_the_ufcs_path_across_crates(self) -> None:
        path = "<*const T as core::cmp::PartialEq<krate::standard::handle::Handle<T>>>::eq"
        item = {"path": path, "pointer_self_type": "*const T", "source_file": "a.rs",
                "source_sha256": "a" * 64}
        crates = {"krate": {path: [item]}, "facade": {}}
        self.assertEqual(rust_candidates(path, crates), ("unique", [item]))
        self.assertEqual(rust_candidates(path, {"krate": {}})[0], "missing")


class StructuralGapTest(unittest.TestCase):
    def test_named_gaps_survive_and_item_ids_do_not(self) -> None:
        values = ["trait-impl:core::cmp::PartialEq:unsupported-generic-arguments", "42",
                  "rustycad_x::Y", "impl:5:unknown-self-type"]
        self.assertEqual(structural_gaps(values), [values[0], values[3]])
        self.assertEqual(structural_gaps(None), [])


if __name__ == "__main__":
    unittest.main()
