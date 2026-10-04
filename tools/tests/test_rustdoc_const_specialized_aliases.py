"""Resolve generic inherent methods through exact const-specialized type aliases."""

from __future__ import annotations

import copy
import hashlib
import json
import unittest
from pathlib import Path

from api_inventory_json import JSONValue
from normalize_rustdoc_inventory import make_reachable_inventory

FIXTURE_ROOT = Path(__file__).parent / "fixtures/rustdoc-const-specialized-alias"
METHODS = ("compare_dist", "compare_dist_d")


class ConstSpecializedAliasTests(unittest.TestCase):
    def test_compiler_capture_exposes_methods_under_exact_alias_paths(self) -> None:
        inventory = normalize_fixture()
        paths = declaration_paths(inventory)
        expected = alias_method_paths("rustdoc_const_specialized_alias")
        for path in expected:
            self.assertIn(path, paths)
            self.assertEqual(declaration_count(inventory, path), 1)
            row = declaration_by_path(inventory, path)
            self.assert_alias_specialization(row, path)
        assert_no_private_paths(paths)
        self.assertEqual(
            inventory["unresolved_links"],
            ["const-specialized-alias:rustdoc_const_specialized_alias::BoundedBox2:unsupported-impl-where-predicates"],
        )

    def assert_alias_specialization(
        self, row: dict[str, JSONValue], path: str
    ) -> None:
        specialization = row.get("alias_specialization")
        self.assertEqual(row.get("source_file"), "src/lib.rs")
        self.assertEqual(row.get("source_sha256"), fixture_source_hash())
        self.assertIsInstance(specialization, dict)
        expected_const = 2 if "CompactBox2" in path else 3
        self.assertEqual(specialization.get("const_value"), expected_const)
        self.assertIsNone(specialization.get("impl_const_parameter"))
        self.assertEqual(specialization.get("alias_path"), path.rsplit("::", 1)[0])
        self.assertEqual(specialization.get("alias_type_parameter"), "T")
        self.assertEqual(specialization.get("impl_type_parameter"), "T")
        self.assertEqual(
            specialization.get("target_path"),
            "rustdoc_const_specialized_alias::compact::storage::CompactBounds",
        )

    def test_nonliteral_const_alias_is_reported_and_not_expanded(self) -> None:
        document = fixture_document()
        replace_alias_const(document, "CompactBox2", "1 + 1", False)
        inventory = normalize_document(document)
        paths = declaration_paths(inventory)
        box2_paths = alias_method_paths("rustdoc_const_specialized_alias")[: len(METHODS)]
        self.assertTrue(all(path not in paths for path in box2_paths))
        box3_paths = alias_method_paths("rustdoc_const_specialized_alias")[-len(METHODS) :]
        self.assertTrue(all(path in paths for path in box3_paths))
        self.assertIn(
            "const-specialized-alias:rustdoc_const_specialized_alias::CompactBox2:unsupported-const-expression",
            inventory["unresolved_links"],
        )

    def test_const_dependent_where_clause_is_unresolved(self) -> None:
        inventory = normalize_fixture()
        bounded_path = "rustdoc_const_specialized_alias::BoundedBox2::constrained"
        self.assertNotIn(bounded_path, declaration_paths(inventory))
        self.assertIn(
            "const-specialized-alias:rustdoc_const_specialized_alias::BoundedBox2:unsupported-impl-where-predicates",
            inventory["unresolved_links"],
        )

    def test_default_capture_reports_missing_private_impl_index(self) -> None:
        inventory = normalize_document(fixture_document(document_private=False))
        expected = (
            "const-specialized-alias:rustdoc_const_specialized_alias::CompactBox2:"
            "target-impl-index-missing"
        )
        self.assertIn(expected, inventory["unresolved_links"])
        self.assertNotIn(alias_method_paths("rustdoc_const_specialized_alias")[0], declaration_paths(inventory))

    def test_private_capture_does_not_publish_private_target_paths(self) -> None:
        paths = declaration_paths(normalize_fixture())
        self.assertFalse(any(path.endswith("CompactBounds") for path in paths))
        assert_no_private_paths(paths)

    def test_generic_impl_row_records_const_binding(self) -> None:
        inventory = normalize_fixture()
        path = "rustdoc_const_specialized_alias::GenericBox2::compare_generic_dist"
        row = declaration_by_path(inventory, path)
        specialization = row.get("alias_specialization")
        self.assertIsInstance(specialization, dict)
        self.assertEqual(specialization.get("const_value"), 2)
        self.assertEqual(specialization.get("impl_const_parameter"), "N")

    def test_methods_for_other_const_impl_do_not_leak_into_aliases(self) -> None:
        document = fixture_document()
        replace_impl_const(document, 4)
        inventory = normalize_document(document)
        paths = declaration_paths(inventory)
        aliases = alias_method_paths("rustdoc_const_specialized_alias")
        self.assertTrue(all(path not in paths for path in aliases[: len(METHODS)]))
        self.assertTrue(all(path in paths for path in aliases[-len(METHODS) :]))


def normalize_fixture() -> dict[str, JSONValue]:
    return normalize_document(fixture_document())


def normalize_document(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    return make_reachable_inventory(document, FIXTURE_ROOT, "rustdoc_const_specialized_alias")


def fixture_document(document_private: bool = True) -> dict[str, JSONValue]:
    artifact = "rustdoc-61.json" if document_private else "rustdoc-61-default.json"
    path = FIXTURE_ROOT / artifact
    document = json.loads(path.read_text(encoding="utf-8"))
    if document.get("format_version") != 61:
        raise AssertionError("const alias fixture must use rustdoc JSON format 61")
    return copy.deepcopy(document)


def declaration_paths(inventory: dict[str, JSONValue]) -> set[str]:
    rows = inventory.get("declarations")
    if not isinstance(rows, list):
        raise AssertionError("normalized declarations must be an array")
    return {row["path"] for row in rows if isinstance(row, dict) and isinstance(row.get("path"), str)}


def assert_no_private_paths(paths: set[str]) -> None:
    private_segments = ("::compact", "::storage", "::box2", "::box3")
    for path in paths:
        if any(segment in path for segment in private_segments):
            raise AssertionError(f"private rustdoc path leaked into public inventory: {path}")


def declaration_count(inventory: dict[str, JSONValue], path: str) -> int:
    rows = inventory.get("declarations")
    if not isinstance(rows, list):
        return 0
    return sum(isinstance(row, dict) and row.get("path") == path for row in rows)


def declaration_by_path(inventory: dict[str, JSONValue], path: str) -> dict[str, JSONValue]:
    rows = inventory.get("declarations")
    if isinstance(rows, list):
        for row in rows:
            if isinstance(row, dict) and row.get("path") == path:
                return row
    raise AssertionError(f"normalized declaration {path!r} is absent")


def fixture_source_hash() -> str:
    source = (FIXTURE_ROOT / "src/lib.rs").read_bytes()
    return hashlib.sha256(source).hexdigest()


def alias_method_paths(crate: str) -> tuple[str, ...]:
    return tuple(f"{crate}::{alias}::{method}" for alias in ("CompactBox2", "CompactBox3") for method in METHODS)


def replace_alias_const(document: dict[str, JSONValue], name: str, expr: str, literal: bool) -> None:
    alias = item_by_name(document, name)
    target = alias.get("inner", {}).get("type_alias", {}).get("type", {}).get("resolved_path", {})
    args = target.get("args", {}).get("angle_bracketed", {}).get("args", [])
    args[1]["const"]["expr"] = expr
    args[1]["const"]["is_literal"] = literal


def replace_impl_const(document: dict[str, JSONValue], value: int) -> None:
    impl = item_by_name(document, "CompactBounds")["inner"]["struct"]["impls"][0]
    body = document["index"][str(impl)]["inner"]["impl"]
    body["for"]["resolved_path"]["args"]["angle_bracketed"]["args"][1]["const"] = {
        "expr": str(value), "value": None, "is_literal": True
    }


def item_by_name(document: dict[str, JSONValue], name: str) -> dict[str, JSONValue]:
    index = document.get("index")
    if not isinstance(index, dict):
        raise AssertionError("rustdoc index must be an object")
    for item in index.values():
        if isinstance(item, dict) and item.get("name") == name:
            return item
    raise AssertionError(f"rustdoc item {name!r} is absent")


if __name__ == "__main__":
    unittest.main()
