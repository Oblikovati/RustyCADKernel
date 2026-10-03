"""Context retries remain supplementary and reject unrelated or changed source evidence."""

from pathlib import Path
import tempfile
import unittest

from api_inventory.context_retries import (
    reconcile_retry_row,
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
            self.root / "raw.jsonl",
        )
        header.write_text("int a;")
        source.write_text('#include "a.hxx"')
        artifact.write_text('{"event":"summary","complete_for_emit_scope":true}\n')
        row = {
            "path": "a.hxx",
            "status": "parsed_with_context",
            "compile_source": str(source),
            "header_sha256": digest(header),
            "compile_source_sha256": digest(source),
            "target_header_seen": True,
            "artifact": str(artifact),
            "output_sha256": digest(artifact),
        }
        originals = {"a.hxx": {"status": "parse_failed"}}
        reconcile_retry_row(InventoryRecords(), row, originals, self.root)
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
            )


if __name__ == "__main__":
    unittest.main()
