"""Published evidence must remain deterministic and bound to the crosswalk inputs."""

import gzip
import json
from pathlib import Path
import tempfile
import unittest

from api_inventory.inputs import digest
from api_tool_hashes import implementation_hashes
from publish_api_snapshot import ROOT, compress_snapshot, publish_snapshot


class SnapshotPublicationTests(unittest.TestCase):
    def test_compression_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "raw.json"
            source.write_text('{"example":true}')
            first = compress_snapshot(source, root / "a.gz")
            second = compress_snapshot(source, root / "b.gz")
            self.assertEqual(first["sha256"], second["sha256"])
            self.assertEqual(
                gzip.decompress((root / "a.gz").read_bytes()), source.read_bytes()
            )

    def test_summary_keeps_gaps_and_rejects_changed_rust(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rust, report = root / "rust.json", root / "report.json"
            tools = implementation_hashes(ROOT)
            rust.write_text(
                json.dumps(
                    {"tool_implementation_sha256": {"rust_normalizer": tools["rust_normalizer"]}}
                )
            )
            report.write_text(
                json.dumps(
                    {
                        "input_sha256": {"rust": digest(rust)},
                        "crosswalk_rows": [],
                        "proof_gaps": ["missing"],
                        "declaration_crosswalk_proof": False,
                        "tool_implementation_sha256": tools,
                    }
                )
            )
            publish_snapshot(rust, report, root / "out")
            summary = json.loads((root / "out/completeness.json").read_text())
            self.assertEqual(summary["proof_gaps"], ["missing"])
            self.assertNotIn("crosswalk_rows", summary)
            self.assertEqual(summary["tool_implementation_sha256"], tools)
            rust.write_text("changed")
            with self.assertRaisesRegex(ValueError, "expected the exact"):
                publish_snapshot(rust, report, root / "out")

    def test_missing_implementation_metadata_cannot_be_published(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            rust, report = root / "rust.json", root / "report.json"
            rust.write_text("{}")
            report.write_text(json.dumps({"input_sha256": {"rust": digest(rust)}}))
            with self.assertRaisesRegex(ValueError, "implementation hashes are missing or stale"):
                publish_snapshot(rust, report, root / "out")
