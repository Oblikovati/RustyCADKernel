"""Fixture tests for rustdoc reachability and stable declaration identities."""

from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from api_inventory_json import JSONValue
from api_inventory_capture import index_records
from api_inventory_reexports import resolve_cross_crate_reexports
from normalize_rustdoc_inventory import make_reachable_inventory, verify_artifact


def item(item_id: int, name: str | None, visibility: str, kind: str,
         body: dict[str, JSONValue]) -> dict[str, JSONValue]:
    """Build one fixture node with the rustdoc JSON envelope."""
    return {"id": item_id, "name": name, "visibility": visibility, "span": None,
            "inner": {kind: body}}


def fixture_document() -> dict[str, JSONValue]:
    root = item(0, "demo", "public", "module", {"is_crate": True, "items": [1]})
    module = item(1, "api", "public", "module", {"items": [2, 8, 12, 14]})
    items = [root, module, *fixture_surface_items()]
    index = {str(entry["id"]): entry for entry in items}
    return {"root": 0, "index": index, "paths": fixture_paths()}


def fixture_surface_items() -> list[dict[str, JSONValue]]:
    reexport = item(2, None, "public", "use", {"source": "types::Thing", "name": "Alias",
                                                  "id": 3, "is_glob": False})
    reexport["span"] = {"filename": "src/lib.rs", "begin": [2, 1], "end": [2, 2]}
    record = item(3, "Thing", "public", "struct", {"kind": {"plain": {"fields": [4]}},
                  "generics": {"params": [], "where_predicates": []}, "impls": [6, 16]})
    record["span"] = {"filename": "src/lib.rs", "begin": [1, 1], "end": [1, 2]}
    record["attrs"] = [{"other": "#[doc(hidden)]"}]
    return [reexport, record, item(4, "value", "public", "struct_field", {"primitive": "i32"}),
            *fixture_implementation(), *fixture_trait_implementation(),
            item(8, "Color", "public", "enum", {"variants": [9, 19], "impls": []}),
            item(9, "Red", "default", "variant", {"kind": "plain", "discriminant": None}),
            item(19, "Rgb", "default", "variant", {"kind": {"tuple": [20]}, "discriminant": None}),
            item(20, None, "default", "struct_field", {"primitive": "u8"}),
            item(12, "Read", "public", "trait", {"items": [13], "generics": {"params": []}}),
            item(13, "read", "default", "function", {"sig": {"inputs": [], "output": {"primitive": "i32"}}}),
            item(14, "Hidden", "default", "struct", {"kind": {"plain": {"fields": []}}})]


def fixture_implementation() -> list[dict[str, JSONValue]]:
    implementation = item(6, None, "default", "impl", {
        "trait": None, "for": {"resolved_path": {"path": "Thing", "id": 3, "args": None}},
        "generics": {"params": [], "where_predicates": []}, "items": [7]})
    return [implementation, item(7, "get", "public", "function", {
        "sig": {"inputs": [["self", {"generic": "Self"}]], "output": {"primitive": "i32"}}})]


def fixture_trait_implementation() -> list[dict[str, JSONValue]]:
    implementation = item(16, None, "default", "impl", {
        "trait": {"path": "Read", "id": 12, "args": None},
        "for": {"resolved_path": {"path": "Thing", "id": 3, "args": None}},
        "generics": {"params": []}, "items": [17], "is_synthetic": False})
    method = item(17, "read", "default", "function", {
        "sig": {"inputs": [["self", {"generic": "Self"}]], "output": {"primitive": "i32"}}})
    return [implementation, method]


def fixture_paths() -> dict[str, JSONValue]:
    paths = ((3, "demo::types::Thing"), (4, "demo::types::Thing::value"),
             (7, "demo::types::Thing::get"), (12, "demo::api::Read"))
    return {str(item_id): {"crate_id": 0, "path": path.split("::"), "kind": "struct"}
            for item_id, path in paths}


def rustdoc_date_period_fixture() -> dict[str, JSONValue]:
    fixture_path = Path(__file__).parent / "fixtures/rustdoc-date-period-foreign-trait-impls.json"
    return json.loads(fixture_path.read_text(encoding="utf-8"))


def add_alias_self_type(document: dict[str, JSONValue], target: dict[str, JSONValue]) -> None:
    alias = item(500, "DateAlias", "public", "type_alias", {"type": target})
    document["index"]["500"] = alias
    document["paths"]["500"] = {
        "crate_id": 0,
        "path": ["rustycad_tkernel", "quantity", "DateAlias"],
        "kind": "type_alias",
    }
    implementation = document["index"]["176"]["inner"]["impl"]
    implementation["for"] = {
        "resolved_path": {"path": "DateAlias", "id": 500, "args": None}
    }


class RustdocReachabilityTests(unittest.TestCase):
    def test_trait_impls_follow_the_rustdoc_self_type(self: RustdocReachabilityTests) -> None:
        document = rustdoc_date_period_fixture()
        with tempfile.TemporaryDirectory() as directory:
            result = make_reachable_inventory(document, Path(directory), "rustycad_tkernel")
        paths = {entry["path"] for entry in result["declarations"]}
        self.assertIn(
            "rustycad_tkernel::quantity::Date::<core::ops::arith::Add>::add", paths
        )
        self.assertNotIn(
            "rustycad_tkernel::quantity::Period::<core::ops::arith::Add>::add", paths
        )
        self.assertEqual(result["unresolved_links"], [])

    def test_unknown_trait_impl_self_type_is_reported_not_attached(
        self: RustdocReachabilityTests,
    ) -> None:
        document = rustdoc_date_period_fixture()
        implementation = document["index"]["176"]
        implementation["inner"]["impl"]["for"] = {
            "borrowed_ref": {
                "lifetime": None,
                "type": {"resolved_path": {"path": "Date", "id": 174, "args": None}},
            }
        }
        with tempfile.TemporaryDirectory() as directory:
            result = make_reachable_inventory(document, Path(directory), "rustycad_tkernel")
        paths = {entry["path"] for entry in result["declarations"]}
        self.assertFalse(any(path.endswith("arith::Add>::add") for path in paths))
        self.assertIn("impl:176:unknown-self-type", result["unresolved_links"])

    def test_trait_impl_self_alias_resolves_to_its_nominal_owner(
        self: RustdocReachabilityTests,
    ) -> None:
        document = rustdoc_date_period_fixture()
        add_alias_self_type(document, {"resolved_path": {"path": "Date", "id": 174}})
        with tempfile.TemporaryDirectory() as directory:
            result = make_reachable_inventory(document, Path(directory), "rustycad_tkernel")
        paths = {entry["path"] for entry in result["declarations"]}
        self.assertIn("rustycad_tkernel::quantity::Date::<core::ops::arith::Add>::add", paths)
        self.assertNotIn("rustycad_tkernel::quantity::Period::<core::ops::arith::Add>::add", paths)
        self.assertEqual(result["unresolved_links"], [])

    def test_unresolved_alias_impl_target_is_reported_explicitly(
        self: RustdocReachabilityTests,
    ) -> None:
        document = rustdoc_date_period_fixture()
        add_alias_self_type(document, {"generic": "T"})
        with tempfile.TemporaryDirectory() as directory:
            result = make_reachable_inventory(document, Path(directory), "rustycad_tkernel")
        self.assertIn("impl:176:unknown-self-alias", result["unresolved_links"])

    def test_external_impl_self_owner_is_reported_when_not_reachable(
        self: RustdocReachabilityTests,
    ) -> None:
        document = rustdoc_date_period_fixture()
        document["index"]["176"]["inner"]["impl"]["for"] = {
            "resolved_path": {"path": "Foreign", "id": 999, "args": None}
        }
        with tempfile.TemporaryDirectory() as directory:
            result = make_reachable_inventory(document, Path(directory), "rustycad_tkernel")
        self.assertIn(
            "impl:176:unreachable-self-owner:999", result["unresolved_links"]
        )

    def test_public_reexports_members_and_signatures_are_normalized(
            self: RustdocReachabilityTests) -> None:
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            (workspace / "src").mkdir()
            source = workspace / "src/lib.rs"
            source.write_text("pub struct Thing { pub value: i32 }\n", encoding="utf-8")
            result = make_reachable_inventory(fixture_document(), workspace, "demo")
            entries = {entry["path"]: entry for entry in result["declarations"]}
            assert_expected_surface(self, entries, source)

    def test_artifact_digest_size_and_format_are_required(self: RustdocReachabilityTests) -> None:
        with tempfile.TemporaryDirectory() as directory:
            capture = Path(directory)
            artifact = capture / "crates/demo.json"
            artifact.parent.mkdir()
            raw = json.dumps({"format_version": 61, "index": {}}).encode()
            artifact.write_bytes(raw)
            record = {"artifact": "crates/demo.json", "artifact_sha256": hashlib.sha256(raw).hexdigest(),
                      "artifact_bytes": len(raw), "indexed_items": 0}
            self.assertEqual(verify_artifact(capture, record, 61)["format_version"], 61)
            with self.assertRaisesRegex(ValueError, "unsupported rustdoc format 61.*expected 60"):
                verify_artifact(capture, record, 60)

    def test_capture_manifest_requires_every_counted_crate(self: RustdocReachabilityTests) -> None:
        """Reject a raw capture whose inventory count exceeds its artifact rows."""
        with self.assertRaisesRegex(ValueError, "lists 0 libraries; expected 1"):
            index_records({"rustdoc_json": [], "workspace_library_count": 1})

    def test_cross_crate_module_reexport_expands_stable_paths(self: RustdocReachabilityTests) -> None:
        crates = cross_crate_fixture()
        resolve_cross_crate_reexports(crates)
        declarations = crates[0]["declarations"]
        paths = {entry["path"] for entry in declarations}
        self.assertIn("rustycad_kernel::foundation::math::Point", paths)
        self.assertIn("rustycad_kernel::foundation::math::Point::origin", paths)
        self.assertEqual(crates[0]["unresolved_links"], [])

    def test_missing_cross_crate_reexport_stays_in_unresolved_list(
            self: RustdocReachabilityTests) -> None:
        crates = missing_cross_crate_fixture()
        resolve_cross_crate_reexports(crates)
        self.assertEqual(crates[0]["unresolved_links"], ["unknown-target"])


def missing_cross_crate_fixture() -> list[dict[str, JSONValue]]:
    source = inventory_crate("rustycad_kernel", "rustycad_kernel::missing", "unresolved_reexport")
    source["declarations"][0]["target_path"] = "missing_crate::item"
    source["declarations"][0]["unresolved_target"] = "unknown-target"
    return [source]


def cross_crate_fixture() -> list[dict[str, JSONValue]]:
    target_root = inventory_crate("rustycad_tkernel", "rustycad_tkernel", "module")
    target_items = [{"path": "rustycad_tkernel::math", "kind": "module",
                     "declaration_sha256": "module"},
                    {"path": "rustycad_tkernel::math::Point", "kind": "struct",
                     "declaration_sha256": "point"},
                    {"path": "rustycad_tkernel::math::Point::origin", "kind": "function",
                     "declaration_sha256": "origin"}]
    target_root["declarations"] = target_items
    source = inventory_crate("rustycad_kernel", "rustycad_kernel::foundation", "reexport_module")
    source["declarations"][0]["target_path"] = "rustycad_tkernel"
    source["declarations"][0]["unresolved_target"] = "1"
    return [source, target_root]


def inventory_crate(crate: str, path: str, kind: str) -> dict[str, JSONValue]:
    marker: dict[str, JSONValue] = {"path": path, "kind": kind, "target_path": "",
                                    "declaration_sha256": "alias"}
    return {"crate": crate, "declarations": [marker], "source_files": [], "unresolved_links": ["1"]}


def assert_expected_surface(test_case: unittest.TestCase, entries: dict[str, JSONValue],
                            source: Path) -> None:
    alias = entries["demo::api::Alias"]
    test_case.assertEqual(alias["kind"], "struct")
    test_case.assertEqual(alias["reexport"]["source"], "types::Thing")
    test_case.assertFalse(alias["reexport"]["is_glob"])
    for public_path in expected_public_paths():
        test_case.assertIn(public_path, entries)
    test_case.assertNotIn("demo::api::Hidden", entries)
    expected_source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
    test_case.assertEqual(alias["source_sha256"], expected_source_hash)
    test_case.assertEqual(alias["reexport"]["source_sha256"], expected_source_hash)
    signature = json.dumps(entries["demo::api::Alias::get"]["signature"])
    test_case.assertNotIn('"id"', signature)


def expected_public_paths() -> tuple[str, ...]:
    return ("demo::api::Alias::value", "demo::api::Alias::get",
            "demo::api::Alias::<demo::api::Read>::read", "demo::api::Color::Red",
            "demo::api::Color::Rgb::tuple_field_0", "demo::api::Read::read")


if __name__ == "__main__":
    unittest.main()
