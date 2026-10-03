"""Tests for conservative OCCT capture-gap reporting."""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import tempfile
import unittest

from tools.api_inventory.gap_audit import (
    build_report,
    classify_failed,
    classify_unconfigured,
    context_gap,
    load_json,
    merged_capture_evidence,
    sha256,
    source_revision,
)


class GapAuditTests(unittest.TestCase):
    def init_source(self, path: pathlib.Path) -> None:
        subprocess.run(["git", "init", "-q", str(path)], check=True)
        subprocess.run(
            [
                "git",
                "-C",
                str(path),
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "fixture",
            ],
            check=True,
        )

    def test_classification_preserves_specific_blockers(self) -> None:
        self.assertEqual(
            classify_failed("src/FoundationClasses/TKMath/BVH/BVH_BoxSet.hxx")[0],
            "extractor_unhandled_decl_kind",
        )
        self.assertEqual(
            classify_failed(
                "src/DataExchange/TKDEGLTF/RWGltf/RWGltf_GltfJsonParser.hxx"
            )[0],
            "external_dependency_parse_error",
        )
        self.assertEqual(
            classify_unconfigured("TKIVtk")[0], "optional_dependency_unconfigured"
        )
        self.assertEqual(
            context_gap("src/FoundationClasses/TKernel/NCollection/NCollection_Haft.h")[
                0
            ],
            "language_mode_unavailable",
        )
        self.assertEqual(
            context_gap("src/Visualization/TKOpenGl/OpenGl/OpenGl_GLESExtensions.hxx")[
                0
            ],
            "optional_platform_feature_unconfigured",
        )

    def test_report_counts_supplemental_successes_separately(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            source = root / "source"
            source.mkdir()
            self.init_source(source)
            header = source / "src/B.hxx"
            compile_source = source / "src/B.cxx"
            header.parent.mkdir(parents=True)
            header.write_text("// header")
            compile_source.write_text('#include "B.hxx"\n')
            capture = {
                "fingerprint": "capture-fp",
                "inputs": {
                    "extractor_sha256": "extractor",
                    "database_sha256": "database",
                    "cmake_cache_sha256": "cache",
                },
                "header_candidates": [
                    {
                        "path": "src/A.hxx",
                        "status": "missing_compile_configuration",
                        "owners": [{"toolkit": "TKIVtk"}],
                    },
                    {
                        "path": "src/B.hxx",
                        "status": "pending",
                        "owners": [{"toolkit": "TKBase"}],
                    },
                    {
                        "path": "src/C.hxx",
                        "status": "pending",
                        "owners": [{"toolkit": "TKMath"}],
                    },
                ],
                "results": [],
            }
            capture_path = root / "capture.json"
            capture_path.write_text(json.dumps(capture))
            success = {
                "path": "src/B.hxx",
                "status": "parsed_with_context",
                "target_header_seen": True,
                "header_sha256": sha256(header),
                "compile_source": str(compile_source),
                "compile_source_sha256": sha256(compile_source),
                "context_evidence": "compile_commands source include resolves to this OCCT file",
            }
            failed = {
                "path": "src/C.hxx",
                "status": "parse_failed",
                "toolkit": "TKMath",
                "output_sha256": "artifact",
                "stderr": "UnresolvedUsingValue",
            }
            retry = {
                "original_capture_sha256": sha256(capture_path),
                "original_fingerprint": "capture-fp",
                "extractor_sha256": "extractor",
                "database_sha256": "database",
                "retry_results": [success, failed],
            }
            retry_path = root / "retry.json"
            retry_path.write_text(json.dumps(retry))
            report = build_report(capture_path, retry_path, source)
            self.assertEqual(
                report["verified_context_retries"]["direct_source_context"], 1
            )
            self.assertEqual(report["unresolved_gap_count"], 2)
            self.assertFalse(report["coverage_proof"])
            self.assertEqual(
                report["unresolved_gaps"][0]["category"],
                "optional_dependency_unconfigured",
            )
            self.assertEqual(
                report["unresolved_gaps"][1]["category"], "parse_failure_unclassified"
            )

    def test_input_helpers_reject_wrong_json_shape_and_bind_git_revision(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            document = root / "object.json"
            document.write_text('{"ok": true}')
            self.assertEqual(load_json(document), {"ok": True})
            self.assertEqual(
                sha256(document), hashlib.sha256(document.read_bytes()).hexdigest()
            )
            self.init_source(root / "repo")
            self.assertEqual(len(source_revision(root / "repo")), 40)
            document.write_text("[]")
            with self.assertRaisesRegex(ValueError, "expected JSON object"):
                load_json(document)

    def test_merged_manifest_must_reference_the_same_retry_file(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            retry_path = root / "retry.json"
            retry_path.write_text(
                '{"retry_results": [{"status": "parsed_with_context"}]}'
            )
            manifest_path = root / "manifest.json"
            manifest_path.write_text(
                json.dumps(
                    {
                        "context_retry_sha256": sha256(retry_path),
                        "headers_by_status": {
                            "parsed_with_context": 1,
                            "parse_failed": 0,
                        },
                        "coverage_proof": False,
                    }
                )
            )
            result = merged_capture_evidence(manifest_path, retry_path)
            self.assertEqual(result["headers_by_status"]["parsed_with_context"], 1)
            self.assertFalse(result["coverage_proof"])
            retry_path.write_text('{"changed": true}')
            with self.assertRaisesRegex(ValueError, "expected this context retry"):
                merged_capture_evidence(manifest_path, retry_path)


if __name__ == "__main__":
    unittest.main()
