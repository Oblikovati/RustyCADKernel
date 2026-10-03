"""Trait identity paths must not introduce private modules into the public API."""

import json
from pathlib import Path
import unittest

from api_inventory_json import JSONValue, declaration_digest
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_trait_paths import resolve_public_trait_paths, renamed_trait_member

FIXTURE = Path(__file__).parent / "fixtures/rustdoc-trait-reexports"


def captured_trait_crate() -> dict[str, JSONValue]:
    document = json.loads((FIXTURE / "rustdoc-61.json").read_text())
    return make_reachable_inventory(document, FIXTURE, "rustdoc_trait_reexports")


class RustdocTraitPathTests(unittest.TestCase):
    def test_compiler_fixture_uses_public_trait_aliases(self) -> None:
        crate = captured_trait_crate()
        resolve_public_trait_paths([crate])
        paths = {entry["path"] for entry in crate["declarations"]}
        prefix = "rustdoc_trait_reexports::Value::<rustdoc_trait_reexports::"
        self.assertIn(prefix + "Operation>::apply", paths)
        self.assertIn(prefix + "Alias>::apply", paths)
        self.assertFalse(any("::hidden::" in path for path in paths))
        for entry in crate["declarations"]:
            self.assertEqual(entry["declaration_sha256"], declaration_digest(entry))

    def test_unreachable_trait_is_withheld_with_explicit_gap(self) -> None:
        crate = captured_trait_crate()
        crate["declarations"] = [
            entry for entry in crate["declarations"] if entry["kind"] != "trait"
        ]
        resolve_public_trait_paths([crate])
        self.assertFalse(
            any(entry["path"].endswith(">::apply") for entry in crate["declarations"])
        )
        self.assertTrue(
            any(
                link.startswith("unreachable-trait:")
                for link in crate["unresolved_links"]
            )
        )

    def test_cross_crate_trait_reexport_is_retained(self) -> None:
        crate = captured_trait_crate()
        alias = next(
            entry.copy() for entry in crate["declarations"] if entry["kind"] == "trait"
        )
        alias["path"] = "shell::PublicTrait"
        shell = {"crate": "shell", "declarations": [alias], "unresolved_links": []}
        resolve_public_trait_paths([crate, shell])
        paths = {entry["path"] for entry in crate["declarations"]}
        self.assertIn(
            "rustdoc_trait_reexports::Value::<shell::PublicTrait>::apply", paths
        )

    def test_external_core_traits_and_inherent_methods_remain(self) -> None:
        crate = captured_trait_crate()
        before = {
            entry["path"]
            for entry in crate["declarations"]
            if "::<core::" in entry["path"]
        }
        resolve_public_trait_paths([crate])
        after = {entry["path"] for entry in crate["declarations"]}
        self.assertTrue(before)
        self.assertTrue(before <= after)
        self.assertIn("rustdoc_trait_reexports::Value", after)

    def test_repeated_resolution_does_not_duplicate_trait_aliases(self) -> None:
        crate = captured_trait_crate()
        resolve_public_trait_paths([crate])
        before = list(crate["declarations"])
        resolve_public_trait_paths([crate])
        self.assertEqual(crate["declarations"], before)

    def test_invalid_impl_path_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected Owner::<Trait>::member"):
            renamed_trait_member({"path": "wrong"}, "Alias")
