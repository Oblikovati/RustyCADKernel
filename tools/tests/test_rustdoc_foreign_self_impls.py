"""Normalize foreign-Self impls without inventing inaccessible owner paths."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from api_crosswalk import rust_candidates, rust_crates
from api_inventory_json import JSONValue
from normalize_rustdoc_inventory import make_reachable_inventory
from rustdoc_foreign_impls import resolve_foreign_impl_candidates

FIXTURES = Path(__file__).parent / "fixtures/rustdoc-foreign-self-impls"


class RustdocForeignSelfImplTests(unittest.TestCase):
    def test_foreign_self_impls_attach_to_reachable_owner_with_distinct_args(
        self: RustdocForeignSelfImplTests,
    ) -> None:
        crates = normalized_fixture_crates()
        resolve_foreign_impl_candidates(crates)
        rows = declaration_paths(crates[1])
        reference_path = (
            "<rustycad_tkmath::gp::Circle3 as core::convert::TryFrom<"
            "&rustycad_tkgeombase::gce::MakeCirc>>::try_from"
        )
        value_path = (
            "<rustycad_tkmath::gp::Circle3 as core::convert::TryFrom<"
            "rustycad_tkgeombase::gce::MakeCirc>>::try_from"
        )
        ellipse_path = (
            "<rustycad_tkmath::gp::Circle3 as core::convert::TryFrom<"
            "&rustycad_tkgeombase::gce::MakeEllipse>>::try_from"
        )
        assert_distinct_traits(self, rows, reference_path, value_path, ellipse_path)
        assert_crosswalk_match(self, crates, reference_path)

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
            "unsupported-foreign-identity" in gap for gap in crates[0]["unresolved_links"]
        ))

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
