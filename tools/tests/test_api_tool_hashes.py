"""Project tool edits invalidate implementation metadata even when inputs stay fixed."""

import json
from pathlib import Path
import tempfile
import unittest

from api_inventory_json import JSONValue, sha256_bytes
from api_inventory_capture import capture_files
from api_tool_hashes import (
    IMPLEMENTATION_FILES,
    implementation_hashes,
    implementation_hashes_match,
    stage_hashes,
)
from api_coverage_report import report_tool_hashes
from normalize_rustdoc_inventory import ROOT, normalized_manifest, package_digest


class ToolHashFixture:
    """Create a minimal repository tree matching the declared tool inventory."""

    def __init__(self, root: Path) -> None:
        self.root = root
        for relative in all_paths():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f"fixture:{relative}")


def all_paths() -> set[str]:
    selected: set[str] = set()
    for paths in IMPLEMENTATION_FILES.values():
        selected.update(paths)
    return selected


class ToolImplementationHashTests(unittest.TestCase):
    def test_crosswalk_hashes_template_selector_rules(self) -> None:
        self.assertIn(
            "tools/api_template_selectors.py",
            IMPLEMENTATION_FILES["api_crosswalk"],
        )

    def test_normalizer_rejects_capture_without_private_item_policy(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            capture = Path(directory)
            (capture / "capture-manifest.json").write_text(json.dumps({
                "raw_capture_complete": True,
                "normalized_public_inventory": False,
            }))
            with self.assertRaisesRegex(ValueError, "document-private-items"):
                capture_files(capture)

    def test_changed_tool_invalidates_recorded_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = ToolHashFixture(Path(directory))
            recorded = implementation_hashes(fixture.root)
            self.assertTrue(implementation_hashes_match(fixture.root, recorded))
            target = fixture.root / "tools/rustdoc_declarations.py"
            target.write_text("changed normalizer helper")
            self.assertFalse(implementation_hashes_match(fixture.root, recorded))

    def test_deleted_tool_invalidates_recorded_hashes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = ToolHashFixture(Path(directory))
            recorded = implementation_hashes(fixture.root)
            (fixture.root / "tools/api_crosswalk.py").unlink()
            self.assertFalse(implementation_hashes_match(fixture.root, recorded))

    def test_missing_or_incomplete_metadata_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = ToolHashFixture(Path(directory))
            self.assertFalse(implementation_hashes_match(fixture.root, None))
            self.assertFalse(implementation_hashes_match(fixture.root, {}))
            incomplete = implementation_hashes(fixture.root)
            del incomplete["rust_normalizer"]["tools/rustdoc_declarations.py"]
            self.assertFalse(implementation_hashes_match(fixture.root, incomplete))

    def test_report_requires_fresh_normalizer_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = ToolHashFixture(Path(directory))
            rust = {
                "tool_implementation_sha256": {
                    "rust_normalizer": stage_hashes(fixture.root, "rust_normalizer")
                }
            }
            self.assertEqual(
                report_tool_hashes(fixture.root, rust),
                implementation_hashes(fixture.root),
            )
            rust.pop("tool_implementation_sha256")
            with self.assertRaisesRegex(ValueError, "missing or stale"):
                report_tool_hashes(fixture.root, rust)

    def test_normalized_inventory_records_normalizer_implementation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fixture = ToolHashFixture(Path(directory))
            capture = rustdoc_capture(fixture.root)
            result = normalized_manifest(capture, fixture.root)
            self.assertEqual(
                result["tool_implementation_sha256"],
                {"rust_normalizer": stage_hashes(ROOT, "rust_normalizer")},
            )
            self.assertEqual(
                result["visibility_policy"],
                "public-reachable-from-private-item-capture-v1",
            )
            self.assertEqual(result["rustdoc_item_policy"], "document-private-items-v1")
            self.assertNotEqual(
                result["tool_implementation_sha256"]["rust_normalizer"],
                stage_hashes(fixture.root, "rust_normalizer"),
            )


def rustdoc_capture(root: Path) -> Path:
    capture = root / "capture"
    package = root / "package"
    capture.mkdir()
    package.mkdir()
    (package / "Cargo.toml").write_text("[package]\nname='demo'\nversion='0.1.0'\n")
    artifact = capture / "demo.json"
    artifact.write_text(json.dumps(minimal_rustdoc_document()))
    record = rustdoc_record(root, capture, artifact)
    manifest = capture_manifest(record)
    (capture / "capture-manifest.json").write_text(json.dumps(manifest))
    return capture


def minimal_rustdoc_document() -> dict[str, JSONValue]:
    return {
        "format_version": 61,
        "root": 0,
        "index": {
            "0": {
                "id": 0,
                "name": "demo",
                "visibility": "public",
                "inner": {"module": {"is_crate": True, "items": []}},
            }
        },
        "paths": {},
    }


def rustdoc_record(root: Path, capture: Path, artifact: Path) -> dict[str, JSONValue]:
    content = artifact.read_bytes()
    return {
        "package": "demo",
        "package_id": "demo 0.1.0",
        "library_target": "demo",
        "manifest_path": "package/Cargo.toml",
        "artifact": artifact.relative_to(capture).as_posix(),
        "artifact_sha256": sha256_bytes(content),
        "artifact_bytes": len(content),
        "indexed_items": 1,
        "package_inputs_sha256": package_digest(root, "package/Cargo.toml"),
        "feature_selection": "all-features",
        "available_features": [],
    }


def capture_manifest(record: dict[str, JSONValue]) -> dict[str, JSONValue]:
    return {
        "raw_capture_complete": True,
        "normalized_public_inventory": False,
        "rustdoc_item_policy": "document-private-items-v1",
        "rustdoc_json_format_version": 61,
        "workspace_library_count": 1,
        "rustdoc_json": [record],
        "inputs": {"workspace_revision": "fixture-revision"},
        "feature_policy": {"mode": "all-features"},
        "cargo_arguments": [
            "-- --document-private-items --output-format json -Z unstable-options"
        ],
    }


if __name__ == "__main__":
    unittest.main()
