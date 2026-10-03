"""Regression fixtures for rustdoc's public glob-reexport paths."""

from __future__ import annotations

import json
import unittest
from pathlib import Path

from api_inventory_reexports import resolve_cross_crate_reexports
from api_inventory_json import JSONValue
from normalize_rustdoc_inventory import make_reachable_inventory


FIXTURE_ROOT = Path(__file__).parent / "fixtures/rustdoc-glob-reexports"


class RustdocGlobReexportTests(unittest.TestCase):
    def test_nightly_61_capture_exposes_glob_members_at_public_paths(
        self: RustdocGlobReexportTests,
    ) -> None:
        document = json.loads((FIXTURE_ROOT / "rustdoc-61.json").read_text())
        self.assertEqual(document["format_version"], 61)
        result = make_reachable_inventory(document, FIXTURE_ROOT, "rustdoc_glob_reexports")
        paths = {entry["path"] for entry in result["declarations"]}
        assert_glob_paths(self, paths)

    def test_cross_crate_glob_reexport_drops_private_module_segment(
        self: RustdocGlobReexportTests,
    ) -> None:
        crates = cross_crate_glob_fixture()
        resolve_cross_crate_reexports(crates)
        paths = {entry["path"] for entry in crates[0]["declarations"]}
        self.assertIn("rustycad_shell::api::Value", paths)
        self.assertIn("rustycad_shell::api::nested::CONST", paths)
        self.assertNotIn("rustycad_shell::api::hidden::Value", paths)
        self.assertEqual(crates[0]["unresolved_links"], [])


def assert_glob_paths(test: unittest.TestCase, paths: set[str]) -> None:
    expected = {
        "rustdoc_glob_reexports::api::Value",
        "rustdoc_glob_reexports::api::Value::field",
        "rustdoc_glob_reexports::api::Value::method",
        "rustdoc_glob_reexports::api::function",
        "rustdoc_glob_reexports::api::nested::CONST",
        "rustdoc_glob_reexports::Value",
        "rustdoc_glob_reexports::Value::field",
        "rustdoc_glob_reexports::Value::method",
        "rustdoc_glob_reexports::function",
        "rustdoc_glob_reexports::nested::CONST",
    }
    for path in expected:
        test.assertIn(path, paths)
    forbidden = {path for path in paths if "::hidden" in path}
    test.assertEqual(forbidden, set())


def cross_crate_glob_fixture() -> list[dict[str, JSONValue]]:
    shell = fixture_crate("rustycad_shell", shell_surface())
    core = fixture_crate("rustycad_core", core_surface())
    return [shell, core]


def shell_surface() -> list[dict[str, JSONValue]]:
    return [
        fixture_declaration("rustycad_shell", "module"),
        fixture_declaration("rustycad_shell::api", "module"),
        {
            **fixture_declaration("rustycad_shell::api", "reexport_module"),
            "target_path": "rustycad_core::hidden",
            "reexport": {"source": "rustycad_core::hidden", "is_glob": True},
        },
    ]


def core_surface() -> list[dict[str, JSONValue]]:
    paths = (
        ("rustycad_core", "module"),
        ("rustycad_core::hidden", "module"),
        ("rustycad_core::hidden::Value", "struct"),
        ("rustycad_core::hidden::Value::field", "struct_field"),
        ("rustycad_core::hidden::Value::method", "function"),
        ("rustycad_core::hidden::function", "function"),
        ("rustycad_core::hidden::nested", "module"),
        ("rustycad_core::hidden::nested::CONST", "constant"),
    )
    return [fixture_declaration(path, kind) for path, kind in paths]


def fixture_crate(name: str, declarations: list[dict[str, JSONValue]]) -> dict[str, JSONValue]:
    return {"crate": name, "declarations": declarations, "source_files": [], "unresolved_links": []}


def fixture_declaration(path: str, kind: str) -> dict[str, JSONValue]:
    return {"path": path, "kind": kind, "declaration_sha256": f"fixture:{path}:{kind}"}


if __name__ == "__main__":
    unittest.main()
