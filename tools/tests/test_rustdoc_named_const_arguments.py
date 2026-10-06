"""Keep native iterator const specializations visible in compiler inventories."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from api_crosswalk import rust_candidates, rust_crates
from api_inventory_json import JSONValue, array_value, object_value
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_generic_arguments import (
    display_generic_argument,
    display_named_const_argument,
)

CRATE = "rustdoc_iterator_const_arguments"
FIXTURE = Path(__file__).parent / "fixtures/rustdoc-iterator-const-arguments"


class RustdocNamedConstArgumentsTests(unittest.TestCase):
    def test_named_const_preserves_compiler_spelling(self) -> None:
        for name in ("N", "MUTABLE", "_Count", "INDEX_2"):
            with self.subTest(name=name):
                body: JSONValue = {"expr": name, "value": None, "is_literal": False}
                self.assertEqual(display_named_const_argument(body), name)
                self.assertEqual(display_generic_argument({"const": body}, {}), name)

    def test_expressions_literals_and_unknown_shapes_stay_unresolved(self) -> None:
        for expression in ("", "_", "N + 1", "{ N }", "Owner::N", "3", "-1"):
            with self.subTest(expression=expression):
                body = {"expr": expression, "value": None, "is_literal": False}
                self.assertIsNone(display_named_const_argument(body))
        for body in malformed_const_arguments():
            with self.subTest(body=body):
                self.assertIsNone(display_named_const_argument(body))

    def test_cross_const_iterator_equality_has_one_compiler_body(self) -> None:
        inventory = captured_const_inventory()
        path = f"{CRATE}::Cursor::<core::cmp::PartialEq<{CRATE}::Cursor<T, O>>>::eq"
        status, matches = rust_candidates(path, rust_crates({"crates": [inventory]}))
        self.assertEqual(status, "unique")
        self.assertEqual(len(matches), 1)
        self.assertIsNotNone(matches[0].get("source_sha256"))
        self.assertEqual(inventory["unresolved_links"], [])

    def test_const_trait_parameter_is_not_erased(self) -> None:
        inventory = captured_const_inventory()
        paths = [
            object_value(row, "declaration")["path"]
            for row in array_value(inventory["declarations"], "declarations")
        ]
        self.assertIn(f"{CRATE}::Lookup::<{CRATE}::ConstLookup<N>>::lookup", paths)
        self.assertNotIn(f"{CRATE}::Lookup::<{CRATE}::ConstLookup>::lookup", paths)


def malformed_const_arguments() -> list[JSONValue]:
    """Exercise unsupported compiler shapes; `malformed_const_arguments()`."""
    return [
        None,
        [],
        {},
        {"expr": "N"},
        {"expr": "N", "value": None, "is_literal": True},
        {"expr": "N", "value": "3", "is_literal": False},
        {"expr": 3, "value": None, "is_literal": False},
        {"expr": "N", "value": None, "is_literal": 0},
        {"expr": "N", "value": None, "is_literal": False, "extra": None},
    ]


def captured_const_inventory() -> dict[str, JSONValue]:
    """Normalize the actual rustdoc capture; `captured_const_inventory()`."""
    document = json.loads((FIXTURE / "rustdoc-61.json").read_text())
    if document.get("format_version") != 61:
        raise AssertionError(
            f"expected rustdoc JSON 61, got {document.get('format_version')}"
        )
    return make_reachable_inventory(
        document, Path(__file__).resolve().parents[2], CRATE
    )
