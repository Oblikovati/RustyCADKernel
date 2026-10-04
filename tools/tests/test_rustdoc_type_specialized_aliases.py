"""Resolve exact concrete type arguments through public aliases."""

from __future__ import annotations

import copy
import hashlib
import json
import unittest
from pathlib import Path

from api_inventory_json import JSONValue
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_trait_paths import resolve_public_trait_paths

FIXTURE_ROOT = Path(__file__).parent / "fixtures/rustdoc-type-specialized-alias"
CRATE = "rustdoc_type_specialized_alias"


class TypeSpecializedAliasTests(unittest.TestCase):
    def test_alias_members_preserve_concrete_and_trait_identity(self) -> None:
        inventory = normalized_fixture()
        paths = declaration_paths(inventory)
        assert_alias_members(self, inventory, paths)

    def test_unsatisfied_extra_bound_is_withheld(self) -> None:
        inventory = normalized_fixture()
        paths = declaration_paths(inventory)
        self.assertNotIn(f"{CRATE}::CmpFile::fast_only", paths)
        impl_id = impl_id_named_member(fixture_document(), "fast_only")
        self.assertIn(
            f"generic-type-alias:{CRATE}::CmpFile:impl:{impl_id}:concrete-type-bound-unproven",
            inventory["unresolved_links"],
        )

    def test_unsupported_where_predicate_is_explicit(self) -> None:
        inventory = normalized_fixture()
        self.assertNotIn(f"{CRATE}::File::where_bound", declaration_paths(inventory))
        impl_id = impl_id_named_member(fixture_document(), "where_bound")
        self.assertIn(
            f"generic-type-alias:{CRATE}::File:impl:{impl_id}:unsupported-where-predicates",
            inventory["unresolved_links"],
        )

    def test_member_where_predicate_is_withheld_for_every_alias(self) -> None:
        inventory = normalized_fixture()
        paths = declaration_paths(inventory)
        for alias in ("File", "CmpFile"):
            self.assertNotIn(f"{CRATE}::{alias}::conditional", paths)
        member_id = item_id_named(fixture_document(), "conditional")
        for alias in ("File", "CmpFile"):
            expected = (
                f"generic-type-alias:{CRATE}::{alias}:member:{member_id}:"
                "unsupported-member-where-predicates"
            )
            self.assertIn(expected, inventory["unresolved_links"])

    def test_trait_impl_where_predicate_is_checked_despite_default_visibility(self) -> None:
        document = fixture_document()
        impl_id = impl_id_named_member(document, "sized")
        member_id = impl_member_id(document, impl_id, "sized")
        inventory = normalized_fixture()
        rows = inventory["declarations"]
        specialized = [row for row in rows if row.get("alias_specialization")]
        self.assertFalse(any(row["path"].endswith("::sized") for row in specialized))
        for alias in ("File", "CmpFile"):
            expected = (
                f"generic-type-alias:{CRATE}::{alias}:member:{member_id}:"
                "unsupported-member-where-predicates"
            )
            self.assertIn(expected, inventory["unresolved_links"])

    def test_alias_without_resolved_concrete_argument_is_not_expanded(self) -> None:
        document = fixture_document()
        alias = item_named(document, "File")
        alias["inner"]["type_alias"]["type"]["resolved_path"]["args"] = None
        inventory = normalize_document(document)
        self.assertNotIn(f"{CRATE}::File::common", declaration_paths(inventory))

    def test_generic_self_owner_must_match_target_type(self) -> None:
        document = fixture_document()
        common_impl = impl_named_member(document, "common")
        common_impl["for"]["resolved_path"]["id"] = "0"
        inventory = normalize_document(document)
        paths = declaration_paths(inventory)
        self.assertNotIn(f"{CRATE}::File::common", paths)
        self.assertNotIn(f"{CRATE}::CmpFile::common", paths)

    def test_negative_trait_impl_does_not_prove_alias_bound(self) -> None:
        document = fixture_document()
        concrete_trait_impl(document, "Text", "Fast")["is_negative"] = True
        self.assert_alias_fast_bound_is_unresolved(document)

    def test_trait_impl_with_arguments_does_not_prove_zero_arg_bound(self) -> None:
        document = fixture_document()
        trait_impl = concrete_trait_impl(document, "Text", "Fast")
        trait_impl["trait"]["args"] = {
            "angle_bracketed": {"args": [{"type": {"primitive": "u8"}}], "constraints": []}
        }
        self.assert_alias_fast_bound_is_unresolved(document)

    def assert_alias_fast_bound_is_unresolved(self, document: dict[str, JSONValue]) -> None:
        inventory = normalize_document(document)
        paths = declaration_paths(inventory)
        self.assertNotIn(f"{CRATE}::File::fast_only", paths)


def assert_alias_members(
    test: unittest.TestCase, inventory: dict[str, JSONValue], paths: set[str]
) -> None:
    file_common = f"{CRATE}::File::common"
    cmp_common = f"{CRATE}::CmpFile::common"
    file_open = f"{CRATE}::File::<{CRATE}::Open>::open"
    cmp_open = f"{CRATE}::CmpFile::<{CRATE}::Open>::open"
    test.assertTrue({file_common, cmp_common, file_open, cmp_open} <= paths)
    test.assertIn(f"{CRATE}::File::fast_only", paths)
    test.assertNotIn(f"{CRATE}::CmpFile::fast_only", paths)
    assert_specialization(test, inventory, file_common, "private::Text")
    assert_specialization(test, inventory, cmp_common, "private::Compatible")


def assert_specialization(
    test: unittest.TestCase, inventory: dict[str, JSONValue], path: str, concrete: str
) -> None:
    row = declaration_by_path(inventory, path)
    specialization = row.get("alias_specialization")
    context = row.get("impl_context")
    test.assertIsInstance(specialization, dict)
    test.assertEqual(specialization.get("concrete_type_path"), f"{CRATE}::{concrete}")
    test.assertEqual(
        specialization.get("impl_type_parameter"), "DialectImplementationParameter"
    )
    owner = context["for"]["resolved_path"]
    test.assertEqual(owner["resolved_item_path"], f"{CRATE}::private::Generic")
    test.assertEqual(context.get("trait"), None)
    test.assertEqual(row.get("source_file"), "src/lib.rs")
    test.assertEqual(row.get("source_sha256"), source_hash())


def source_hash() -> str:
    data = (FIXTURE_ROOT / "src/lib.rs").read_bytes()
    digest = hashlib.sha256(data)
    return digest.hexdigest()


def normalized_fixture() -> dict[str, JSONValue]:
    inventory = normalize_document(fixture_document())
    resolve_public_trait_paths([inventory])
    return inventory


def normalize_document(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    if document.get("format_version") != 61:
        raise AssertionError("type alias fixture must use rustdoc JSON format 61")
    inventory = make_reachable_inventory(document, FIXTURE_ROOT, CRATE)
    return inventory


def fixture_document() -> dict[str, JSONValue]:
    path = FIXTURE_ROOT / "rustdoc-61.json"
    document = json.loads(path.read_text(encoding="utf-8"))
    if document.get("format_version") != 61:
        raise AssertionError("type alias fixture must use rustdoc JSON format 61")
    return copy.deepcopy(document)


def item_named(document: dict[str, JSONValue], name: str) -> dict[str, JSONValue]:
    items = document["index"]
    for item in items.values():
        if isinstance(item, dict) and item.get("name") == name:
            return item
    raise AssertionError(f"fixture has no item named {name!r}")


def impl_named_member(document: dict[str, JSONValue], name: str) -> dict[str, JSONValue]:
    items = document["index"]
    for item in items.values():
        body = item.get("inner", {}).get("impl") if isinstance(item, dict) else None
        members = body.get("items", []) if isinstance(body, dict) else []
        if any(items[str(member)].get("name") == name for member in members):
            return body
    raise AssertionError(f"no compiler fixture impl contains {name!r}")


def impl_id_named_member(document: dict[str, JSONValue], name: str) -> str:
    items = document["index"]
    for identifier, item in items.items():
        body = item.get("inner", {}).get("impl") if isinstance(item, dict) else None
        members = body.get("items", []) if isinstance(body, dict) else []
        if any(items[str(member)].get("name") == name for member in members):
            return identifier
    raise AssertionError(f"no compiler fixture impl contains {name!r}")


def impl_member_id(document: dict[str, JSONValue], impl_id: str, name: str) -> str:
    impl = document["index"][impl_id]["inner"]["impl"]
    for member_id in impl.get("items", []):
        if document["index"][str(member_id)].get("name") == name:
            return str(member_id)
    raise AssertionError(f"impl {impl_id} has no member named {name!r}")


def item_id_named(document: dict[str, JSONValue], name: str) -> str:
    for identifier, item in document["index"].items():
        if isinstance(item, dict) and item.get("name") == name:
            return identifier
    raise AssertionError(f"fixture has no item named {name!r}")


def concrete_trait_impl(
    document: dict[str, JSONValue], owner: str, trait_name: str
) -> dict[str, JSONValue]:
    items = document["index"]
    nominal = item_named(document, owner)
    impl_ids = nominal.get("inner", {}).get("struct", {}).get("impls", [])
    for impl_id in impl_ids:
        impl_item = items[str(impl_id)]
        impl = impl_item.get("inner", {}).get("impl")
        trait = impl.get("trait") if isinstance(impl, dict) else None
        if isinstance(trait, dict) and trait.get("path") == trait_name:
            return impl
    raise AssertionError(f"{owner} has no direct {trait_name} implementation")


def declaration_paths(inventory: dict[str, JSONValue]) -> set[str]:
    rows = inventory.get("declarations")
    if not isinstance(rows, list):
        return set()
    return {row["path"] for row in rows if isinstance(row, dict) and isinstance(row.get("path"), str)}


def declaration_by_path(
    inventory: dict[str, JSONValue], path: str
) -> dict[str, JSONValue]:
    rows = inventory.get("declarations")
    if isinstance(rows, list):
        for row in rows:
            if isinstance(row, dict) and row.get("path") == path:
                return row
    raise AssertionError(f"declaration {path!r} is absent from normalized fixture")
