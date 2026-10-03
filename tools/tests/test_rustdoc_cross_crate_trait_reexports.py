"""Exercise cross-crate trait aliases from pinned rustdoc captures."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from api_inventory_json import (
    JSONValue,
    array_value,
    declaration_digest,
    object_value,
    string_value,
)
from api_inventory_reexports import resolve_cross_crate_reexports
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_trait_paths import resolve_public_trait_paths

FIXTURE_ROOT = Path(__file__).parent / "fixtures/rustdoc-cross-crate-trait"


class RustdocCrossCrateTraitReexportTests(unittest.TestCase):
    def test_compiler_alias_reaches_trait_members_across_crate_boundary(self) -> None:
        crates = captured_crates()
        consumer_rows = array_value(crates[1].get("declarations"), "declarations")
        self.assertTrue(
            any(row["kind"] == "unresolved_reexport" for row in consumer_rows)
        )
        resolve_cross_crate_reexports(crates)
        resolve_public_trait_paths(crates)
        assert_public_alias_paths(self, crates)

    def test_alias_keeps_canonical_trait_identity_and_unique_rows(self) -> None:
        crates = captured_crates()
        resolve_cross_crate_reexports(crates)
        resolve_public_trait_paths(crates)
        assert_alias_identity(self, crates)


def captured_crates() -> list[dict[str, JSONValue]]:
    provider = read_capture("provider-rustdoc-61.json")
    consumer = read_capture("consumer-rustdoc-61.json")
    return [
        make_reachable_inventory(
            provider, FIXTURE_ROOT, "trait_provider"
        ),
        make_reachable_inventory(
            consumer, FIXTURE_ROOT, "trait_consumer"
        ),
    ]


def read_capture(relative_path: str) -> dict[str, JSONValue]:
    document = json.loads((FIXTURE_ROOT / relative_path).read_text())
    if document.get("format_version") != 61:
        raise AssertionError(f"{relative_path} must use rustdoc JSON format 61")
    return document


def assert_public_alias_paths(
    test: unittest.TestCase, crates: list[dict[str, JSONValue]]
) -> None:
    provider_paths = declaration_paths(crates[0])
    consumer_paths = declaration_paths(crates[1])
    test.assertIn("trait_consumer::PublicOperation", consumer_paths)
    test.assertIn("trait_consumer::PublicOperation::apply", consumer_paths)
    test.assertIn("trait_consumer::apply", consumer_paths)
    provider_alias_method = (
        "trait_provider::Value::<trait_consumer::PublicOperation>::apply"
    )
    consumer_alias_method = (
        "trait_consumer::Value::<trait_consumer::PublicOperation>::apply"
    )
    test.assertIn(provider_alias_method, provider_paths)
    test.assertIn(consumer_alias_method, consumer_paths)
    test.assertFalse(
        any("::hidden::" in path for path in provider_paths | consumer_paths)
    )
    test.assertEqual(crates[1]["unresolved_links"], [])


def assert_alias_identity(
    test: unittest.TestCase, crates: list[dict[str, JSONValue]]
) -> None:
    consumer_entries = array_value(crates[1].get("declarations"), "declarations")
    aliases = entries_for_path(consumer_entries, "trait_consumer::PublicOperation")
    test.assertEqual(len(aliases), 1)
    test.assertEqual(aliases[0]["canonical_trait_path"], "trait_provider::Operation")
    method_path = "trait_consumer::Value::<trait_consumer::PublicOperation>::apply"
    methods = entries_for_path(consumer_entries, method_path)
    test.assertEqual(len(methods), 1)
    canonical_trait = methods[0]["impl_context"]["trait"]["resolved_item_path"]
    test.assertEqual(canonical_trait, "trait_provider::Operation")
    test.assertIsNone(aliases[0]["source_status"])
    test.assertIsNone(methods[0]["source_status"])
    assert_unique_valid_digests(test, crates)


def entries_for_path(
    entries: list[JSONValue], path: str
) -> list[dict[str, JSONValue]]:
    return [
        object_value(value, "declaration")
        for value in entries
        if isinstance(value, dict) and value.get("path") == path
    ]


def assert_unique_valid_digests(
    test: unittest.TestCase, crates: list[dict[str, JSONValue]]
) -> None:
    for crate in crates:
        entries = array_value(crate.get("declarations"), "declarations")
        digests = [entry["declaration_sha256"] for entry in entries]
        test.assertEqual(len(digests), len(set(digests)))
        for entry in entries:
            test.assertEqual(entry["declaration_sha256"], declaration_digest(entry))


def declaration_paths(crate: dict[str, JSONValue]) -> set[str]:
    entries = array_value(crate.get("declarations"), "declarations")
    return {
        string_value(object_value(entry, "declaration").get("path"), "path")
        for entry in entries
    }
