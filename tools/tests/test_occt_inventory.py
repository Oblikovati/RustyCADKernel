"""Inventory regressions: lost overloads and partial compiler output must stay visible."""

from __future__ import annotations

import json
import pathlib
import tempfile
import unittest
from typing import TextIO, cast
from unittest.mock import patch

from api_inventory.capture import (
    CaptureContext,
    capture_header,
    extractor_command,
    job_fingerprint,
)
from api_inventory.inputs import (
    HeaderJob,
    JSONObject,
    compiler_flags,
    digest,
    object_rows,
    read_object,
    select_header_jobs,
    text_field,
)
from api_inventory.merge import (
    InventoryRecords,
    consume_capture,
    consume_event,
    coverage_rows,
    declaration_identity,
    inventory_summary,
    write_jsonl_gzip,
)


class FakeCompiler:
    """A deterministic compiler process replacement, independent of installed Clang."""

    returncode = 0
    stderr = ""

    def run(self, command: list[str], **options: object) -> FakeCompiler:
        stream = cast(TextIO, options["stdout"])
        stream.write('{"event":"summary","complete_for_emit_scope":true}\n')
        return self


class InventoryFixture(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = pathlib.Path(self.directory.name)
        self.context = CaptureContext(
            self.root, self.root / "clang", self.root / "out", "pinned", 1
        )
        self.job = HeaderJob("src/a.hxx", "TKernel", str(self.root), ("-DREAL=1",))

    def declaration(
        self, usr: str = "c:@F@f#I#", signature: str = "void (int)"
    ) -> JSONObject:
        return {
            "event": "declaration",
            "usr": usr,
            "kind": "Function",
            "signature": signature,
            "source": str(self.root / "a.hxx"),
            "qualified_name": "f",
            "api_visible": True,
            "expansion_location": {
                "file": str(self.root / "a.hxx"),
                "line": 1,
                "column": 2,
            },
        }


class CaptureTests(InventoryFixture):
    def test_real_flags_preserved(self) -> None:
        self.assertEqual(
            compiler_flags(
                ["clang++", "-DAPI=1", "-isystem", "/sdk", "-c", "a.cxx", "-o", "a.o"]
            ),
            ["-DAPI=1", "-isystem", "/sdk"],
        )
        self.assertIn("-DREAL=1", extractor_command(self.job, self.context))
        other = HeaderJob(self.job.path, "TKernel", str(self.root), ("-DREAL=2",))
        self.assertNotEqual(
            job_fingerprint(other, self.context),
            job_fingerprint(self.job, self.context),
        )

    def test_cache_requires_matching_bytes(self) -> None:
        compiler = FakeCompiler()
        with patch(
            "api_inventory.capture.subprocess.run", side_effect=compiler.run
        ) as process:
            first = capture_header(self.job, self.context)
            self.assertEqual(capture_header(self.job, self.context), first)
            self.assertEqual(process.call_count, 1)
            (self.context.output / str(first["artifact"])).write_text("truncated")
            capture_header(self.job, self.context)
            self.assertEqual(process.call_count, 2)

    def test_missing_configuration_and_fragments_preserved(self) -> None:
        headers = [
            {"path": path, "owners": [{"toolkit": toolkit}]}
            for path, toolkit in [
                ("a.hxx", "TKernel"),
                ("a.lxx", "TKernel"),
                ("b.hxx", "TKIVtk"),
            ]
        ]
        jobs, coverage = select_header_jobs(
            {"headers": headers}, {"TKernel": ("/tmp", ["-DREAL"])}
        )
        self.assertEqual(len(jobs), 1)
        self.assertEqual(
            [row["status"] for row in coverage],
            ["pending", "fragment_pending", "missing_compile_configuration"],
        )
        rows = coverage_rows(
            {"header_candidates": coverage, "results": []},
            InventoryRecords(includes={"a.lxx"}),
        )
        self.assertEqual(rows[1]["status"], "included_fragment")

    def test_invalid_schema_rejected(self) -> None:
        for value in (None, "x", [1]):
            with self.assertRaises(ValueError):
                object_rows(value, "fixture")
        with self.assertRaises(ValueError):
            text_field({"path": 2}, "path")
        path = self.root / "bad.json"
        path.write_text("[]")
        with self.assertRaises(ValueError):
            read_object(path)


class MergeTests(InventoryFixture):
    def test_noncallable_signature_variance_conflicts_under_one_usr(self) -> None:
        records = InventoryRecords()
        first = self.declaration("c:@T@math_Vector")
        first["kind"] = "TypedefName"
        first["signature"] = "class math_Vector"
        first["identity_signature"] = "math_Vector"
        second = dict(first)
        second["signature"] = "math_Vector"
        second["identity_signature"] = "math_Vector"
        consume_event(records, first, "a.hxx", self.root)
        consume_event(records, second, "b.hxx", self.root)
        merged = next(iter(records.declarations.values()))
        self.assertEqual(len(records.declarations), 1)
        self.assertTrue(merged["inventory_conflict"])
        self.assertEqual(len(records.unsupported), 1)

    def test_template_defaults_merge_identity_but_preserve_both_spellings(self) -> None:
        records = InventoryRecords()
        for template_signature in (
            "template <typename __template_param_0 = int>",
            "template <typename __template_param_0>",
        ):
            row = self.declaration("c:@FT@defaulted#t0.0#")
            row["kind"] = "FunctionTemplate"
            row["identity_signature"] = "void (type-parameter-0-0)"
            row["template_signature"] = template_signature
            consume_event(records, row, "a.hxx", self.root)
        merged = next(iter(records.declarations.values()))
        self.assertEqual(len(records.declarations), 1)
        self.assertEqual(len(merged["template_signature_variants"]), 2)
        self.assertFalse(records.unsupported)

    def test_default_arguments_merge_as_observations_not_identity(self) -> None:
        records = InventoryRecords()
        for source, default in (("a.hxx", "7"), ("a.cxx", "")):
            row = self.declaration()
            row["default_arguments"] = [{"state": "explicit", "expression": default}]
            consume_event(records, row, source, self.root)
        merged = next(iter(records.declarations.values()))
        self.assertEqual(len(records.declarations), 1)
        self.assertEqual(len(merged["default_argument_observations"]), 2)
        self.assertEqual(merged["default_argument_capture"], "complete")

    def test_overloads_remain_distinct_redeclarations_merge(self) -> None:
        records = InventoryRecords()
        consume_event(records, self.declaration(), "a.hxx", self.root)
        consume_event(records, self.declaration(), "b.hxx", self.root)
        consume_event(
            records, self.declaration("c:@F@f#d#", "void (double)"), "a.hxx", self.root
        )
        self.assertEqual(len(records.declarations), 2)
        self.assertEqual(len(next(iter(records.declarations.values()))["sites"]), 2)
        self.assertFalse(records.unsupported)

    def test_location_fallback_and_macro_redefinitions_distinct(self) -> None:
        first = self.declaration(usr="")
        second = self.declaration(usr="")
        cast(JSONObject, second["expansion_location"])["line"] = 2
        self.assertNotEqual(declaration_identity(first), declaration_identity(second))
        records = InventoryRecords()
        for line in (1, 2):
            consume_event(
                records,
                {
                    "event": "macro",
                    "source": "a.hxx",
                    "name": "PI",
                    "location": {"line": line},
                },
                "a.hxx",
                self.root,
            )
        self.assertEqual(len(records.macros), 2)

    def test_conflicts_and_unknown_events_block_proof(self) -> None:
        records = InventoryRecords()
        consume_event(records, self.declaration(), "a.hxx", self.root)
        hidden = self.declaration()
        hidden["api_visible"] = False
        consume_event(records, hidden, "b.hxx", self.root)
        consume_event(records, {"event": "new_clang_event"}, "a.hxx", self.root)
        self.assertEqual(len(records.unsupported), 2)
        summary = inventory_summary({"inputs": {}, "fingerprint": "abc"}, records, [])
        self.assertFalse(summary["coverage_proof"])
        self.assertFalse(summary["declaration_inventory_complete"])

    def test_failed_and_truncated_capture_never_proves_coverage(self) -> None:
        artifact = self.root / "raw.jsonl"
        artifact.write_text(json.dumps(self.declaration()) + "\n")
        result = {
            "path": "a.hxx",
            "artifact": "raw.jsonl",
            "output_sha256": digest(artifact),
            "status": "parse_failed",
        }
        records = InventoryRecords()
        consume_capture(records, result, self.root, self.root)
        self.assertFalse(records.declarations)
        result["status"] = "parsed"
        consume_capture(records, result, self.root, self.root)
        self.assertEqual(
            records.unsupported[0]["reason"], "incomplete_compiler_summary"
        )
        artifact.write_text("changed")
        with self.assertRaises(ValueError):
            consume_capture(records, result, self.root, self.root)

    def test_generated_locations_and_deterministic_gzip(self) -> None:
        generated = str(self.root / "a.hxx")
        records = InventoryRecords(source_aliases={generated: "@generated/a.hxx"})
        consume_event(records, self.declaration(), "@generated/a.hxx", self.root)
        row = next(iter(records.declarations.values()))
        self.assertEqual(row["source"], "@generated/a.hxx")
        self.assertEqual(row["expansion_location"]["file"], "@generated/a.hxx")
        first = write_jsonl_gzip(self.root / "a.gz", [row])
        second = write_jsonl_gzip(self.root / "b.gz", [row])
        self.assertEqual(first["sha256"], second["sha256"])


if __name__ == "__main__":
    unittest.main()
