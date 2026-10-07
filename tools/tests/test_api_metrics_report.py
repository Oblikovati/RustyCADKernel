"""Per-toolkit metrics: one status per declaration, scope decisions and ledger defects."""

from __future__ import annotations

import gzip
import json
import tempfile
import unittest
from pathlib import Path

from api_metrics_markdown import metrics_markdown
from api_metrics_report import build_metrics, toolkit_of

SOURCE = "src/ModelingData/TKG2d/Geom2d/Geom2d_Line.hxx"
VIEW_SOURCE = "src/Visualization/TKV3d/V3d/V3d_View.hxx"


def cpp(cpp_id: str, name: str, kind: str = "CXXMethod", source: str = SOURCE, **extra: object) -> dict:
    record = {"id": cpp_id, "qualified_name": name, "kind": kind, "access": "public", "source": source}
    record.update(extra)
    return record


def row(resolution: str, cpp_ids: list[str], source: str = SOURCE) -> dict:
    return {"resolution": resolution, "cpp_ids": cpp_ids, "source": source}


def fixture_files(folder: Path) -> dict[str, Path]:
    """Write a small crosswalk, declaration table, legacy table and scope file."""
    declarations = [
        cpp("a", "Geom2d_Line::Value"), cpp("b", "Geom2d_Line::Reverse"),
        cpp("c", "Geom2d_Line::Lin2d"), cpp("d", "Geom2d_Line::Dummy"),
        cpp("e", "Geom2d_Line::operator new", macro_generated=True,
            spelling_location={"file": "Standard_DefineAlloc.hxx"}),
        cpp("f", "V3d_View::Redraw", source=VIEW_SOURCE),
    ]
    crosswalk = {
        "input_sha256": {"cpp": "x"},
        "crosswalk_rows": [
            row("implemented", ["a"]), row("missing_rust_target", ["b"]),
            row("partial", ["c"]), row("missing_cpp", []), row("implemented", ["a"]),
        ],
        "unclaimed_cpp_declarations": [{"id": "d"}, {"id": "e"}, {"id": "f"}],
    }
    legacy = [
        {"kind": "gtest_declaration", "toolkit": "TKG2d", "mapping_status": "mapped"},
        {"kind": "gtest_declaration", "toolkit": "TKV3d", "mapping_status": "unmapped"},
        {"kind": "draw_case_candidate", "toolkit": None},
    ]
    scope = {"decisions": [{"scope": "replaced", "reason": "Vulkan renderer", "toolkits": ["TKV3d"]}]}
    paths = {name: folder / file for name, file in (
        ("crosswalk", "crosswalk.json.gz"), ("declarations", "decl.jsonl.gz"),
        ("legacy", "legacy.jsonl.gz"), ("scope", "scope.json"))}
    paths["crosswalk"].write_bytes(gzip.compress(json.dumps(crosswalk).encode()))
    paths["declarations"].write_bytes(gzip.compress("\n".join(map(json.dumps, declarations)).encode()))
    paths["legacy"].write_bytes(gzip.compress("\n".join(map(json.dumps, legacy)).encode()))
    paths["scope"].write_text(json.dumps(scope))
    return paths


class MetricsReportTest(unittest.TestCase):
    def setUp(self) -> None:
        self.folder = tempfile.TemporaryDirectory()
        self.metrics = build_metrics(fixture_files(Path(self.folder.name)))

    def tearDown(self) -> None:
        self.folder.cleanup()

    def test_each_declaration_has_one_status(self) -> None:
        api = self.metrics["toolkits"]["TKG2d"]["api"]
        self.assertEqual(api["declarations"], 4)
        self.assertEqual(api["by_status"]["verified"], 1)
        self.assertEqual(api["by_status"]["claimed_unverified"], 1)
        self.assertEqual(api["by_status"]["partial"], 1)
        self.assertEqual(api["done_percent"], 25.0)

    def test_boilerplate_is_counted_apart(self) -> None:
        toolkit = self.metrics["toolkits"]["TKG2d"]
        self.assertEqual(toolkit["boilerplate"]["declarations"], 1)
        self.assertEqual(toolkit["open_boilerplate_by_category"], {"alloc-macro": 1})
        self.assertEqual(toolkit["open_gap_by_category"], {"member": 2})

    def test_ledger_defects(self) -> None:
        defects = self.metrics["toolkits"]["TKG2d"]["ledger_defects"]
        self.assertEqual(defects, {"rows_with_unresolved_rust_target": 1, "rows_without_cpp_match": 1})

    def test_scope_and_tests(self) -> None:
        view = self.metrics["toolkits"]["TKV3d"]
        self.assertEqual(view["scope"], "replaced")
        self.assertEqual(view["gtests"], {"mapped": 0, "unmapped": 1})
        self.assertEqual(self.metrics["totals_by_scope"]["port"]["gtests_mapped"], 1)

    def test_markdown_lists_each_scope(self) -> None:
        text = metrics_markdown(self.metrics)
        self.assertIn("## Scope: port", text)
        self.assertIn("## Scope: replaced", text)
        self.assertIn("| TKG2d | 4 | 1 | 25.0 |", text)


class ToolkitOfTest(unittest.TestCase):
    def test_paths(self) -> None:
        self.assertEqual(toolkit_of(SOURCE), "TKG2d")
        self.assertEqual(toolkit_of("tools/x.py"), "unknown")


if __name__ == "__main__":
    unittest.main()
