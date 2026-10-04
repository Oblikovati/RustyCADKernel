"""Normalize foreign-Self impls without inventing inaccessible owner paths."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from api_crosswalk import rust_candidates, rust_crates
from api_inventory_json import JSONValue
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_foreign_impls import resolve_foreign_impl_candidates
from rustdoc_trait_paths import resolve_public_trait_paths

FIXTURES = Path(__file__).parent / "fixtures/rustdoc-foreign-self-impls"


class RustdocForeignSelfImplTests(unittest.TestCase):
    def test_foreign_self_impls_attach_to_reachable_owner_with_distinct_args(
        self: RustdocForeignSelfImplTests,
    ) -> None:
        crates = normalized_fixture_crates()
        duplicate_foreign_candidate(crates[0], 1)
        resolve_foreign_impl_candidates(crates)
        resolve_public_trait_paths(crates)
        rows = declaration_paths(crates[1])
        reference_path, value_path, ellipse_path = conversion_paths()
        assert_distinct_traits(self, rows, reference_path, value_path, ellipse_path)
        assert_local_trait_alias(self, rows)
        assert_public_alias_metadata(self, crates)
        assert_crosswalk_match(self, crates, reference_path)
        self.assertEqual(declaration_count(crates[1], reference_path), 1)

    def test_foreign_impl_owner_must_be_publicly_reachable(
        self: RustdocForeignSelfImplTests,
    ) -> None:
        crates = normalized_fixture_crates()
        declarations = crates[1]["declarations"]
        assert isinstance(declarations, list)
        crates[1]["declarations"] = [
            row for row in declarations
            if not (isinstance(row, dict) and row.get("path") == "rustycad_tkmath::gp::Circle3")
        ]
        resolve_foreign_impl_candidates(crates)
        rows = declaration_paths(crates[1])
        self.assertFalse(any("TryFrom<" in path for path in rows))
        self.assertTrue(
            any(
                "foreign-impl-unreachable-owner" in gap
                for gap in crates[0]["unresolved_links"]
            )
        )

    def test_unsupported_foreign_trait_arguments_remain_an_explicit_gap(
        self: RustdocForeignSelfImplTests,
    ) -> None:
        base = read_fixture("base-rustdoc-61.json")
        replace_first_reference_argument(base)
        crates = normalized_fixture_crates(base)
        resolve_foreign_impl_candidates(crates)
        self.assertTrue(any(
            "unsupported-foreign-owner-or-trait" in gap
            for gap in crates[0]["unresolved_links"]
        ))

    def test_parameterized_foreign_self_is_explicitly_withheld(
        self: RustdocForeignSelfImplTests,
    ) -> None:
        crates = normalized_fixture_crates()
        resolve_foreign_impl_candidates(crates)
        rows = declaration_paths(crates[1])
        self.assertFalse(any("GenericCircle" in path for path in rows if path.startswith("<")))
        self.assertTrue(any(
            "unsupported-foreign-owner-or-trait" in gap
            for gap in crates[0]["unresolved_links"]
        ))

    def test_foreign_impl_digest_ignores_rustdoc_item_numbers(
        self: RustdocForeignSelfImplTests,
    ) -> None:
        original = normalized_fixture_crates()
        renumbered_base = read_fixture("base-rustdoc-61.json")
        renumber_rustdoc_ids(renumbered_base)
        renumbered = normalized_fixture_crates(renumbered_base)
        for crates in (original, renumbered):
            resolve_foreign_impl_candidates(crates)
            resolve_public_trait_paths(crates)
        self.assertEqual(foreign_rows(original), foreign_rows(renumbered))
        assert_foreign_ids_discarded(self, original)

    def test_private_local_trait_argument_is_not_published_as_ufcs_path(
        self: RustdocForeignSelfImplTests,
    ) -> None:
        crates = normalized_fixture_crates()
        remove_public_make_circ_type(crates[0])
        resolve_foreign_impl_candidates(crates)
        rows = declaration_paths(crates[1])
        self.assertFalse(any("MakeCirc" in path for path in rows if path.startswith("<")))
        self.assertTrue(any(
            "foreign-impl-unreachable-local-type" in gap
            for gap in crates[0]["unresolved_links"]
        ))


def conversion_paths() -> tuple[str, str, str]:
    owner = "rustycad_tkmath::gp::Circle3 as core::convert::TryFrom<"
    maker = "rustycad_tkgeombase::gce::MakeCirc"
    ellipse = "rustycad_tkgeombase::gce::MakeEllipse"
    return (
        f"<{owner}&{maker}>>::try_from",
        f"<{owner}{maker}>>::try_from",
        f"<{owner}&{ellipse}>>::try_from",
    )


def normalized_fixture_crates(
    base: dict[str, JSONValue] | None = None,
) -> list[dict[str, JSONValue]]:
    base = base or read_fixture("base-rustdoc-61.json")
    math = read_fixture("math-rustdoc-61.json")
    return [
        make_reachable_inventory(base, FIXTURES / "base", "rustycad_tkgeombase"),
        make_reachable_inventory(math, FIXTURES / "math", "rustycad_tkmath"),
    ]


def read_fixture(name: str) -> dict[str, JSONValue]:
    value = json.loads((FIXTURES / name).read_text(encoding="utf-8"))
    if value.get("format_version") != 61:
        raise AssertionError(f"{name} must use rustdoc JSON format 61")
    return value


def replace_first_reference_argument(document: dict[str, JSONValue]) -> None:
    index = document.get("index")
    if not isinstance(index, dict):
        raise AssertionError("rustdoc index must be an object")
    for item in index.values():
        impl = item.get("inner", {}).get("impl") if isinstance(item, dict) else None
        if not isinstance(impl, dict) or not foreign_circle_owner(impl):
            continue
        args = impl.get("trait", {}).get("args", {}).get("angle_bracketed", {}).get("args")
        if isinstance(args, list) and args and args[0].get("type", {}).get("borrowed_ref"):
            args[0] = {"type": {"tuple": [{"primitive": "u8"}]}}
            return
    raise AssertionError("foreign TryFrom<&MakeCirc> fixture impl is absent")


def foreign_circle_owner(impl: dict[str, JSONValue]) -> bool:
    self_type = impl.get("for")
    if not isinstance(self_type, dict):
        return False
    path = self_type.get("resolved_path")
    return isinstance(path, dict) and path.get("path") == "rustycad_tkmath::gp::Circle3"


def remove_public_make_circ_type(crate: dict[str, JSONValue]) -> None:
    declarations = crate.get("declarations")
    if not isinstance(declarations, list):
        raise AssertionError("normalized declarations must be an array")
    crate["declarations"] = [
        row for row in declarations
        if not (isinstance(row, dict) and row.get("path") == "rustycad_tkgeombase::gce::MakeCirc")
    ]


def declaration_paths(crate: dict[str, JSONValue]) -> set[str]:
    declarations = crate.get("declarations")
    if not isinstance(declarations, list):
        raise AssertionError("normalized declarations must be an array")
    return {
        row["path"] for row in declarations
        if isinstance(row, dict) and isinstance(row.get("path"), str)
    }


def declaration_count(crate: dict[str, JSONValue], path: str) -> int:
    declarations = crate.get("declarations")
    if not isinstance(declarations, list):
        return 0
    return sum(isinstance(row, dict) and row.get("path") == path for row in declarations)


def duplicate_foreign_candidate(crate: dict[str, JSONValue], index: int) -> None:
    candidates = crate.get("foreign_impl_candidates")
    if not isinstance(candidates, list) or index >= len(candidates):
        raise AssertionError("foreign impl fixture candidate is absent")
    candidate = candidates[index]
    if not isinstance(candidate, dict):
        raise AssertionError("foreign impl fixture candidate must be an object")
    candidates.append(dict(candidate))


def assert_public_alias_metadata(
    test: unittest.TestCase, crates: list[dict[str, JSONValue]]
) -> None:
    math = declaration_for_path(crates[1], "rustycad_tkmath::gp::Circle3")
    maker = declaration_for_path(crates[0], "rustycad_tkgeombase::gce::MakeCirc")
    test.assertEqual(math.get("canonical_type_path"), "rustycad_tkmath::gp::circle::Circle3")
    test.assertEqual(maker.get("canonical_type_path"), "rustycad_tkgeombase::gce::circ::MakeCirc")


def declaration_for_path(crate: dict[str, JSONValue], path: str) -> dict[str, JSONValue]:
    declarations = crate.get("declarations")
    if isinstance(declarations, list):
        for row in declarations:
            if isinstance(row, dict) and row.get("path") == path:
                return row
    raise AssertionError(f"public declaration {path!r} is absent")


def assert_crosswalk_match(
    test: unittest.TestCase, crates: list[dict[str, JSONValue]], target: str
) -> None:
    status, candidates = rust_candidates(target, rust_crates({"crates": crates}))
    test.assertEqual(status, "unique")
    test.assertEqual(candidates[0]["foreign_impl_origin_crate"], "rustycad_tkgeombase")


def assert_distinct_traits(
    test: unittest.TestCase, paths: set[str], *expected: str
) -> None:
    test.assertTrue(set(expected).issubset(paths))
    test.assertEqual(len(set(expected)), len(expected))


def assert_local_trait_alias(
    test: unittest.TestCase, paths: set[str]
) -> None:
    expected = (
        "<rustycad_tkmath::gp::Circle3 as rustycad_tkgeombase::PublicBuild<"
        "&rustycad_tkgeombase::gce::MakeCirc>>::build_from"
    )
    test.assertIn(expected, paths)


def foreign_rows(crates: list[dict[str, JSONValue]]) -> list[tuple[str, str]]:
    return sorted(
        (str(row.get("path")), str(row.get("declaration_sha256")))
        for crate in crates
        for row in crate.get("declarations", [])
        if isinstance(row, dict) and "foreign_impl_origin_crate" in row
    )


def assert_foreign_ids_discarded(
    test: unittest.TestCase, crates: list[dict[str, JSONValue]]
) -> None:
    rows = [
        row for crate in crates for row in crate.get("declarations", [])
        if isinstance(row, dict) and "foreign_impl_origin_crate" in row
    ]
    test.assertTrue(all("foreign_impl_id" not in row for row in rows))


def renumber_rustdoc_ids(document: dict[str, JSONValue]) -> None:
    index, paths = document.get("index"), document.get("paths")
    if not isinstance(index, dict) or not isinstance(paths, dict):
        raise AssertionError("rustdoc index and paths must be objects")
    identifiers = set(index) | set(paths)
    mapping = {identifier: str(int(identifier) + 100_000) for identifier in identifiers}
    document["index"] = {mapping[key]: remap_ids(item, mapping) for key, item in index.items()}
    document["paths"] = {mapping[key]: remap_ids(item, mapping) for key, item in paths.items()}
    if isinstance(document.get("root"), int):
        document["root"] = int(mapping[str(document["root"])])


def remap_ids(value: JSONValue, mapping: dict[str, str]) -> JSONValue:
    if isinstance(value, list):
        return [remap_ids(item, mapping) for item in value]
    if not isinstance(value, dict):
        return value
    return {key: remap_field(key, item, mapping) for key, item in value.items()}


def remap_field(key: str, value: JSONValue, mapping: dict[str, str]) -> JSONValue:
    if key == "id" and isinstance(value, int) and str(value) in mapping:
        return int(mapping[str(value)])
    if key in {"items", "impls", "fields", "variants"} and isinstance(value, list):
        return [remapped_reference(item, mapping) for item in value]
    return remap_ids(value, mapping)


def remapped_reference(value: JSONValue, mapping: dict[str, str]) -> JSONValue:
    if isinstance(value, int) and str(value) in mapping:
        return int(mapping[str(value)])
    return remap_ids(value, mapping)
