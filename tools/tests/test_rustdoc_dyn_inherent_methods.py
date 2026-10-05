"""Keep dyn inherent method paths distinct from trait methods in rustdoc proof."""

from __future__ import annotations

import gzip
import json
import unittest
from pathlib import Path

from api_inventory_json import JSONValue, array_value, object_value
from normalize_rustdoc_inventory import make_reachable_inventory

FIXTURE_ROOT = Path(__file__).parent / "fixtures/rustdoc-dyn-inherent"
WORKSPACE_ROOT = Path(__file__).resolve().parents[2]


class DynInherentRustdocTests(unittest.TestCase):
    def test_compiler_capture_exposes_omission_without_aliasing_trait_items(self) -> None:
        raw = captured_rustdoc()
        assert_callable_example_present(raw)
        assert_no_dyn_impl_items(raw)
        inventory = make_reachable_inventory(raw, WORKSPACE_ROOT, "rustdoc_dyn_inherent")
        paths = declaration_paths(inventory)
        self.assertIn("rustdoc_dyn_inherent::StorageDriver::put_integer", paths)
        self.assertIn("rustdoc_dyn_inherent::DriverValue::write", paths)
        self.assertNotIn("rustdoc_dyn_inherent::StorageDriver::put", paths)
        self.assertNotIn("rustdoc_dyn_inherent::StorageDriver::get", paths)
        self.assertFalse(any(path.endswith("::put") for path in paths))
        self.assertFalse(any(path.endswith("::get") for path in paths))


def captured_rustdoc() -> dict[str, JSONValue]:
    with gzip.open(FIXTURE_ROOT / "rustdoc-61.json.gz", "rt", encoding="utf-8") as capture:
        document = json.load(capture)
    if document.get("format_version") != 61:
        raise AssertionError("dyn inherent fixture requires rustdoc JSON format 61")
    return document


def assert_callable_example_present(document: dict[str, JSONValue]) -> None:
    items = object_value(document.get("index"), "rustdoc index")
    example = [
        item for item in items.values()
        if isinstance(item, dict) and item.get("name") == "exercise_dyn_methods"
    ]
    if len(example) != 1:
        raise AssertionError(f"expected one callable example, found {len(example)}")


def assert_no_dyn_impl_items(document: dict[str, JSONValue]) -> None:
    items = object_value(document.get("index"), "rustdoc index")
    dyn_impls = [
        item for item in items.values()
        if isinstance(item, dict) and is_dyn_impl(item)
    ]
    if dyn_impls:
        raise AssertionError(f"expected rustdoc to omit dyn inherent impl, found {len(dyn_impls)}")


def is_dyn_impl(item: dict[str, JSONValue]) -> bool:
    inner = item.get("inner")
    impl = inner.get("impl") if isinstance(inner, dict) else None
    implemented_type = impl.get("for") if isinstance(impl, dict) else None
    return isinstance(implemented_type, dict) and "dyn_trait" in implemented_type


def declaration_paths(inventory: dict[str, JSONValue]) -> set[str]:
    rows = array_value(inventory.get("declarations"), "normalized declarations")
    return {
        path
        for row in rows
        if isinstance(row, dict)
        for path in [object_value(row, "declaration").get("path")]
        if isinstance(path, str)
    }
