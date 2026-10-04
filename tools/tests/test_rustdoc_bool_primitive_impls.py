"""Exercise primitive-Self conversion inventory from captured rustdoc JSON."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
import unittest

from api_crosswalk import rust_candidates, rust_crates
from api_inventory_json import JSONValue
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_foreign_impls import resolve_foreign_impl_candidates

FIXTURE = Path(__file__).parent / "fixtures/rustdoc-bool-primitive-self"
TARGET = "<bool as core::convert::From<&bool_fixture::api::ResultRecord>>::from"


class RustdocBoolPrimitiveImplTests(unittest.TestCase):
    def test_unique_public_anchor_emits_pinned_exact_ufcs_target(self) -> None:
        crate = normalized_fixture()
        rows = resolve_fixture([crate])
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["path"], TARGET)
        self.assertEqual(rows[0]["foreign_impl_anchor_path"], "bool_fixture::api::ResultRecord")
        self.assertEqual(rows[0]["foreign_impl_api_owner"], "bool_fixture")
        self.assertEqual(rows[0]["foreign_impl_owner"], "bool")
        self.assert_source_hashes(rows[0])
        self.assertEqual(rust_candidates(TARGET, rust_crates({"crates": [crate]}))[0], "unique")

    def test_ambiguous_public_aliases_withhold_primitive_impl(self) -> None:
        crate = normalized_fixture()
        duplicate_anchor_alias(crate)
        rows = resolve_fixture([crate])
        self.assertEqual(rows, [])
        self.assertTrue(any("foreign-impl-unreachable-owner" in gap for gap in crate["unresolved_links"]))

    def test_missing_public_anchor_withholds_primitive_impl(self) -> None:
        crate = normalized_fixture()
        remove_anchor_declaration(crate)
        self.assertEqual(resolve_fixture([crate]), [])
        self.assertTrue(any("foreign-impl-unreachable-owner" in gap for gap in crate["unresolved_links"]))

    def test_generic_bool_conversion_remains_unresolved(self) -> None:
        crate = normalized_fixture(add_generic_parameter(read_fixture()))
        self.assertEqual(resolve_fixture([crate]), [])
        self.assert_bool_gap(crate)

    def test_non_nominal_argument_is_not_published(self) -> None:
        crate = normalized_fixture(replace_reference_with_tuple(read_fixture()))
        self.assertEqual(resolve_fixture([crate]), [])
        self.assert_bool_gap(crate)

    def test_synthetic_and_blanket_bool_impls_are_ignored(self) -> None:
        for mutation in (mark_synthetic, mark_blanket):
            with self.subTest(mutation=mutation.__name__):
                crate = normalized_fixture(mutation(read_fixture()))
                self.assertEqual(resolve_fixture([crate]), [])

    def test_negative_bool_impl_is_not_published(self) -> None:
        crate = normalized_fixture(mark_negative(read_fixture()))
        self.assertEqual(resolve_fixture([crate]), [])

    def test_rustdoc_item_renumbering_does_not_change_row_identity(self) -> None:
        original = resolve_fixture([normalized_fixture()])
        renumbered = resolve_fixture([normalized_fixture(renumber_ids(read_fixture()))])
        self.assertEqual(stable_foreign_rows(original), stable_foreign_rows(renumbered))

    def test_bool_crosswalk_requires_exact_target_and_origin_metadata(self) -> None:
        crate = normalized_fixture()
        row = resolve_fixture([crate])[0]
        indexes = rust_crates({"crates": [crate]})
        self.assertEqual(rust_candidates(TARGET + "_typo", indexes)[0], "missing")
        row["foreign_impl_origin_crate"] = "another_crate"
        self.assertEqual(rust_candidates(TARGET, indexes)[0], "missing")
        row["foreign_impl_origin_crate"] = "bool_fixture"
        row.pop("foreign_impl_self_primitive")
        self.assertEqual(rust_candidates(TARGET, indexes)[0], "missing")

    def test_bool_crosswalk_requires_pinned_anchor_source(self) -> None:
        crate = normalized_fixture()
        row = resolve_fixture([crate])[0]
        row["foreign_impl_anchor_source_sha256"] = "not-a-hash"
        self.assertEqual(rust_candidates(TARGET, rust_crates({"crates": [crate]}))[0], "unverifiable_source")

    def test_bool_crosswalk_rejects_duplicate_exact_candidates(self) -> None:
        crate = normalized_fixture()
        row = dict(resolve_fixture([crate])[0])
        crate["declarations"].append(row)
        self.assertEqual(rust_candidates(TARGET, rust_crates({"crates": [crate]}))[0], "ambiguous")

    def assert_source_hashes(self, row: dict[str, JSONValue]) -> None:
        expected = hashlib.sha256((FIXTURE / "src/lib.rs").read_bytes()).hexdigest()
        self.assertEqual(row.get("source_sha256"), expected)
        self.assertEqual(row.get("foreign_impl_anchor_source_sha256"), expected)

    def assert_bool_gap(self, crate: dict[str, JSONValue]) -> None:
        self.assertTrue(any("unsupported-bool-anchor:" in gap for gap in crate["unresolved_links"]))


def read_fixture() -> dict[str, JSONValue]:
    document = json.loads((FIXTURE / "rustdoc-61.json").read_text(encoding="utf-8"))
    if document.get("format_version") != 61:
        raise AssertionError("primitive bool fixture must use rustdoc JSON version 61")
    return document


def normalized_fixture(document: dict[str, JSONValue] | None = None) -> dict[str, JSONValue]:
    source_root = FIXTURE.resolve()
    raw = document if document is not None else read_fixture()
    return make_reachable_inventory(raw, source_root, "bool_fixture")


def resolve_fixture(crates: list[dict[str, JSONValue]]) -> list[dict[str, JSONValue]]:
    resolve_foreign_impl_candidates(crates)
    return [
        row for crate in crates for row in crate["declarations"]
        if isinstance(row, dict) and row.get("foreign_impl_self_primitive") == "bool"
    ]


def bool_impl(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    index = document.get("index")
    if not isinstance(index, dict):
        raise AssertionError("rustdoc index is missing")
    for item in index.values():
        body = item.get("inner", {}).get("impl") if isinstance(item, dict) else None
        if isinstance(body, dict) and body.get("for") == {"primitive": "bool"}:
            return body
    raise AssertionError("fixture must contain a primitive bool impl")


def duplicate_anchor_alias(crate: dict[str, JSONValue]) -> None:
    declarations = crate.get("declarations")
    if not isinstance(declarations, list):
        raise AssertionError("normalized declarations are missing")
    anchor = next(row for row in declarations if isinstance(row, dict) and row.get("path") == "bool_fixture::api::ResultRecord")
    duplicate = dict(anchor)
    duplicate["path"] = "bool_fixture::api::RecordAlias"
    declarations.append(duplicate)


def remove_anchor_declaration(crate: dict[str, JSONValue]) -> None:
    declarations = crate.get("declarations")
    if not isinstance(declarations, list):
        raise AssertionError("normalized declarations are missing")
    crate["declarations"] = [
        row for row in declarations
        if not (isinstance(row, dict) and row.get("path") == "bool_fixture::api::ResultRecord")
    ]


def add_generic_parameter(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    impl = bool_impl(document)
    generics = impl.get("generics")
    if not isinstance(generics, dict):
        raise AssertionError("fixture impl generics are missing")
    generics["params"] = [{"name": "T", "kind": {"type": {}}}]
    return document


def replace_reference_with_tuple(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    impl = bool_impl(document)
    trait = impl.get("trait")
    arguments = trait.get("args", {}).get("angle_bracketed", {}).get("args")
    if not isinstance(arguments, list) or len(arguments) != 1:
        raise AssertionError("fixture From argument is missing")
    arguments[0] = {"type": {"tuple": []}}
    return document


def mark_synthetic(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    bool_impl(document)["is_synthetic"] = True
    return document


def mark_blanket(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    bool_impl(document)["blanket_impl"] = {"generic": "T"}
    return document


def mark_negative(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    bool_impl(document)["is_negative"] = True
    return document


def renumber_ids(document: dict[str, JSONValue]) -> dict[str, JSONValue]:
    raw = copy.deepcopy(document)
    index, paths = raw.get("index"), raw.get("paths")
    if not isinstance(index, dict) or not isinstance(paths, dict):
        raise AssertionError("rustdoc index and paths must be objects")
    mapping = {identifier: str(int(identifier) + 100_000) for identifier in set(index) | set(paths)}
    raw["index"] = {mapping[key]: remap_ids(item, mapping) for key, item in index.items()}
    raw["paths"] = {mapping[key]: remap_ids(item, mapping) for key, item in paths.items()}
    raw["root"] = int(mapping[str(raw["root"])])
    return raw


def remap_ids(value: JSONValue, mapping: dict[str, str]) -> JSONValue:
    if isinstance(value, list):
        return [remap_field("items", item, mapping) for item in value]
    if not isinstance(value, dict):
        return value
    return {key: remap_field(key, item, mapping) for key, item in value.items()}


def remap_field(key: str, value: JSONValue, mapping: dict[str, str]) -> JSONValue:
    if key == "id" and isinstance(value, int) and str(value) in mapping:
        return int(mapping[str(value)])
    if key in {"items", "impls", "fields", "variants"} and isinstance(value, list):
        return [remap_reference(item, mapping) for item in value]
    return remap_ids(value, mapping)


def remap_reference(value: JSONValue, mapping: dict[str, str]) -> JSONValue:
    if isinstance(value, int) and str(value) in mapping:
        return int(mapping[str(value)])
    return remap_ids(value, mapping)


def stable_foreign_rows(rows: list[dict[str, JSONValue]]) -> list[tuple[JSONValue, JSONValue]]:
    return sorted(
        (row.get("path"), row.get("declaration_sha256")) for row in rows
    )
