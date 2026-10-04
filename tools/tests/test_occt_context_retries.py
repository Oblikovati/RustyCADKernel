"""Context retries remain supplementary and reject unrelated or changed source evidence."""

import json
from pathlib import Path
import tempfile
import unittest

from api_inventory.context_retries import (
    merge_context_retries,
    reconcile_retry_row,
    retry_artifact_root,
    resolve_retry_artifact,
    verify_retry_parent,
    verify_retry_source,
)
from api_inventory.inputs import digest
from api_inventory.merge import InventoryRecords


class ContextReconciliationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.capture = self.root / "capture.json"
        self.capture.write_text("{}")

    def test_parent_binding_rejects_stale_fingerprint(self) -> None:
        capture = {
            "fingerprint": "bound",
            "inputs": {"database_sha256": "db", "extractor_sha256": "clang"},
        }
        retry = {
            "original_capture_sha256": digest(self.capture),
            "original_fingerprint": "bound",
            "database_sha256": "db",
            "extractor_sha256": "clang",
        }
        verify_retry_parent(retry, capture, self.capture)
        retry["original_fingerprint"] = "stale"
        with self.assertRaisesRegex(ValueError, "expected exact capture"):
            verify_retry_parent(retry, capture, self.capture)

    def test_success_preserves_original_failure(self) -> None:
        header, source, artifact = (
            self.root / "a.hxx",
            self.root / "a.cxx",
            self.root / "artifacts" / "raw.jsonl",
        )
        header.write_text("int a;")
        source.write_text('#include "a.hxx"')
        artifact.parent.mkdir()
        artifact.write_text(
            '{"event":"summary","complete_for_emit_scope":true,'
            '"alias_reachability_events":0}\n'
        )
        row = {
            "path": "a.hxx",
            "status": "parsed_with_context",
            "compile_source": str(source),
            "header_sha256": digest(header),
            "compile_source_sha256": digest(source),
            "target_header_seen": True,
            "artifact": "artifacts/raw.jsonl",
            "output_sha256": digest(artifact),
        }
        originals = {"a.hxx": {"status": "parse_failed"}}
        records = InventoryRecords()
        reconcile_retry_row(records, row, originals, self.root, self.root)
        self.assertEqual(len(records.summaries), 1)
        self.assertEqual(
            originals["a.hxx"],
            {"status": "parsed_with_context", "original_status": "parse_failed"},
        )
        header.write_text("changed")
        with self.assertRaisesRegex(ValueError, "expected pinned source hash"):
            verify_retry_source(row, self.root)

    def test_unrelated_retry_cannot_replace_successful_capture(self) -> None:
        with self.assertRaisesRegex(ValueError, "expected an original failed"):
            reconcile_retry_row(
                InventoryRecords(),
                {"path": "a.hxx", "status": "parsed_with_context"},
                {"a.hxx": {"status": "parsed"}},
                self.root,
                self.root,
            )

    def test_manifest_relative_and_absolute_artifact_paths_resolve(self) -> None:
        artifact = self.root / "artifacts" / "raw.jsonl"
        artifact.parent.mkdir()
        artifact.write_text("{}\n")
        manifest = self.root / "context-retry-manifest.json"
        self.assertEqual(
            retry_artifact_root({"artifact_path_base": "manifest_directory"}, manifest),
            self.root.resolve(),
        )
        self.assertEqual(
            resolve_retry_artifact({"artifact": "artifacts/raw.jsonl"}, self.root),
            str(artifact.resolve()),
        )
        self.assertEqual(
            resolve_retry_artifact({"artifact": str(artifact)}, Path("/")),
            str(artifact),
        )
        self.assertEqual(
            retry_artifact_root({}, manifest),
            Path.cwd(),
        )

    def test_merge_reads_retry_artifact_relative_to_manifest(self) -> None:
        header, source = self.root / "a.hxx", self.root / "a.cxx"
        header.write_text("int a;")
        source.write_text('#include "a.hxx"')
        capture_path = self.root / "capture.json"
        capture = {
            "fingerprint": "bound",
            "inputs": {"database_sha256": "db", "extractor_sha256": "clang"},
            "results": [{"path": "a.hxx", "status": "parse_failed"}],
        }
        capture_path.write_text(json.dumps(capture))
        retry_directory = self.root / "retries"
        artifact = retry_directory / "artifacts" / "probe.jsonl"
        artifact.parent.mkdir(parents=True)
        artifact.write_text(
            '{"event":"summary","complete_for_emit_scope":true,'
            '"alias_reachability_events":0}\n'
        )
        row = {
            "path": "a.hxx",
            "status": "parsed_with_context",
            "compile_source": str(source),
            "header_sha256": digest(header),
            "compile_source_sha256": digest(source),
            "target_header_seen": True,
            "artifact": "artifacts/probe.jsonl",
            "output_sha256": digest(artifact),
        }
        retry_path = retry_directory / "context-retry-manifest.json"
        retry = {
            "original_capture_sha256": digest(capture_path),
            "original_fingerprint": "bound",
            "database_sha256": "db",
            "extractor_sha256": "clang",
            "artifact_path_base": "manifest_directory",
            "retry_results": [row],
        }
        retry_path.write_text(json.dumps(retry))
        records = InventoryRecords()
        merge_context_retries(records, capture, capture_path, retry_path, self.root)
        self.assertEqual(len(records.summaries), 1)
        self.assertEqual(capture["results"][0]["status"], "parsed_with_context")


if __name__ == "__main__":
    unittest.main()
