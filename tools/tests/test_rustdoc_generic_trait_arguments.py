"""Verify generic trait identity survives rustdoc path normalization."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from api_crosswalk import rust_candidates, rust_crates
from api_inventory_json import JSONValue, array_value, object_value
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_trait_paths import resolve_public_trait_paths

FIXTURE_ROOT = Path(__file__).parent / "fixtures/rustdoc-generic-trait-args"
CRATE = "rustdoc_generic_trait_args"


class GenericTraitArgumentTests(unittest.TestCase):
    def test_concrete_trait_args_keep_impls_distinct(self) -> None:
        inventory = normalized_fixture()
        paths = declaration_paths(inventory)
        float_path = f"{CRATE}::Xy::<core::ops::arith::Mul<f64>>::mul"
        matrix_path = f"{CRATE}::Xy::<core::ops::arith::Mul<{CRATE}::Matrix2>>::mul"
        self.assertIn(float_path, paths)
        self.assertIn(matrix_path, paths)
        self.assertEqual(sum(row["path"] == float_path for row in declarations(inventory)), 1)
        self.assertEqual(sum(row["path"] == matrix_path for row in declarations(inventory)), 1)
        status, matches = rust_candidates(float_path, rust_crates({"crates": [inventory]}))
        self.assertEqual(status, "unique")
        self.assertEqual(matches[0]["path"], float_path)
        self.assertEqual(
            inventory["unresolved_links"],
            ["trait-impl:rustdoc_generic_trait_args::ConstOperation:unsupported-generic-arguments"],
        )

    def test_default_and_explicit_self_follow_rustdoc_application(self) -> None:
        inventory = normalized_fixture()
        paths = declaration_paths(inventory)
        default_path = f"{CRATE}::DefaultSelf::<core::ops::arith::Mul>::mul"
        explicit_path = f"{CRATE}::ExplicitSelf::<core::ops::arith::Mul>::mul"
        self.assertIn(default_path, paths)
        self.assertIn(explicit_path, paths)
        self.assertNotIn(f"{CRATE}::DefaultSelf::<core::ops::arith::Mul<{CRATE}::DefaultSelf>>::mul", paths)
        self.assertIsNone(declaration(inventory, default_path)["impl_context"]["trait"]["args"])
        self.assertIsNone(declaration(inventory, explicit_path)["impl_context"]["trait"]["args"])

    def test_reference_and_collection_arguments_keep_compiler_paths(self) -> None:
        inventory = normalized_fixture()
        paths = declaration_paths(inventory)
        reference = (
            f"{CRATE}::ReferenceArgument::<core::ops::arith::Add<"
            f"&'static {CRATE}::Matrix2>>::add"
        )
        collection = (
            f"{CRATE}::CollectionArgument::<core::convert::From<"
            f"alloc::vec::Vec<{CRATE}::Matrix2>>>::from"
        )
        self.assertIn(reference, paths)
        self.assertIn(collection, paths)

    def test_trait_alias_renaming_preserves_generic_suffix(self) -> None:
        inventory = normalized_fixture()
        resolve_public_trait_paths([inventory])
        paths = declaration_paths(inventory)
        self.assertIn(f"{CRATE}::Value::<{CRATE}::PublicOperation<f64>>::apply", paths)

    def test_unsupported_const_argument_does_not_emit_erased_path(self) -> None:
        inventory = normalized_fixture()
        paths = declaration_paths(inventory)
        self.assertFalse(any(path.startswith(f"{CRATE}::Flag::") for path in paths))
        self.assertIn(
            "trait-impl:rustdoc_generic_trait_args::ConstOperation:unsupported-generic-arguments",
            inventory["unresolved_links"],
        )


def normalized_fixture() -> dict[str, JSONValue]:
    document = json.loads((FIXTURE_ROOT / "rustdoc-61.json").read_text())
    if document.get("format_version") != 61:
        raise AssertionError("generic-trait fixture requires rustdoc JSON 61")
    return make_reachable_inventory(document, Path(__file__).resolve().parents[2], CRATE)


def declarations(inventory: dict[str, JSONValue]) -> list[dict[str, JSONValue]]:
    return [
        object_value(row, "declaration")
        for row in array_value(inventory.get("declarations"), "declarations")
    ]


def declaration_paths(inventory: dict[str, JSONValue]) -> set[str]:
    return {object_path(row) for row in declarations(inventory)}


def declaration(inventory: dict[str, JSONValue], path: str) -> dict[str, JSONValue]:
    matches = [row for row in declarations(inventory) if row.get("path") == path]
    if len(matches) != 1:
        raise AssertionError(f"expected one compiler path {path!r}, found {len(matches)}")
    return matches[0]


def object_path(value: JSONValue) -> str:
    row = object_value(value, "declaration")
    path = row.get("path")
    if not isinstance(path, str):
        raise AssertionError("declaration path should be a string")
    return path
