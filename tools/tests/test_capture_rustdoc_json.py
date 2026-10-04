"""Fail-closed validation for workspace rustdoc JSON captures."""

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from capture_rustdoc_json import (
    LibraryCrate,
    capture_command,
    require_unchanged_inputs,
    validate_capture,
    digest_paths,
    validate_artifact,
    validate_rustdoc_json,
    workspace_library_crates,
)


class WorkspaceLibraryTests(unittest.TestCase):
    def test_metadata_enumerates_each_library_and_features(self):
        package = {
            "id": "demo-id",
            "name": "rustycad-demo",
            "manifest_path": "/workspace/crates/demo/Cargo.toml",
            "targets": [{"name": "rustycad_demo", "kind": ["lib"]}],
            "features": {"extra-api": []},
        }
        crates = workspace_library_crates({"workspace_members": ["demo-id"], "packages": [package]})
        self.assertEqual([(crate.package_name, crate.target_name, crate.features) for crate in crates],
                         [("rustycad-demo", "rustycad_demo", ("extra-api",))])

    def test_workspace_package_without_library_fails(self):
        package = {
            "id": "demo-id",
            "name": "rustycad-demo",
            "manifest_path": "/workspace/crates/demo/Cargo.toml",
            "targets": [{"name": "demo", "kind": ["bin"]}],
            "features": {},
        }
        with self.assertRaisesRegex(ValueError, "0 library targets"):
            workspace_library_crates({"workspace_members": ["demo-id"], "packages": [package]})


class RustdocArtifactTests(unittest.TestCase):
    def test_capture_command_requests_private_items(self):
        crate = LibraryCrate("demo-id", "rustycad-demo", Path("Cargo.toml"), "demo", Path("."), ())
        command = capture_command("nightly-2026-09-25", crate, Path("target"))
        separator = command.index("--")
        self.assertIn("--document-private-items", command[separator + 1 :])

    def test_artifact_requires_a_matching_root_and_index(self):
        document = {"root": 0, "index": {"0": {"name": "rustycad_demo"}},
                    "paths": {}, "format_version": 61}
        self.assertEqual(validate_rustdoc_json(document, "rustycad_demo", 61), (61, 1))
        with self.assertRaisesRegex(ValueError, "lacks fields"):
            validate_rustdoc_json({"root": 0}, "rustycad_demo", 61)
        with self.assertRaisesRegex(ValueError, "does not match crate"):
            validate_rustdoc_json({**document, "index": {"0": {"name": "other"}}}, "rustycad_demo", 61)

    def test_artifact_rejects_a_format_version_other_than_the_pin(self):
        document = {"root": 0, "index": {"0": {"name": "rustycad_demo"}},
                    "paths": {}, "format_version": 60}
        with self.assertRaisesRegex(ValueError, "unsupported rustdoc JSON format version 60.*expected 61"):
            validate_rustdoc_json(document, "rustycad_demo", 61)

    def test_artifact_digest_and_schema_are_checked(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            artifact = root / "crates/rustycad_demo.json"
            artifact.parent.mkdir()
            artifact.write_text(json.dumps({"root": 0, "index": {"0": {"name": "rustycad_demo"}},
                                            "paths": {}, "format_version": 61}))
            record = {"artifact": "crates/rustycad_demo.json",
                      "artifact_sha256": hashlib.sha256(artifact.read_bytes()).hexdigest(),
                      "artifact_bytes": artifact.stat().st_size,
                      "library_target": "rustycad_demo", "rustdoc_format_version": 61,
                      "indexed_items": 1}
            validate_artifact(root, record, 61)
            record["artifact_sha256"] = "stale"
            with self.assertRaisesRegex(ValueError, "missing or changed"):
                validate_artifact(root, record, 61)

    def test_capture_rejects_a_missing_workspace_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            context = {"source": "pinned"}
            manifest = {"raw_capture_complete": True, "normalized_public_inventory": False,
                        "rustdoc_json_format_version": 61,
                        "rustdoc_item_policy": "document-private-items-v1",
                        "cargo_arguments": ["-- --document-private-items --output-format json -Z unstable-options"],
                        "inputs": context,
                        "rustdoc_json": [{"library_target": "rustycad_demo",
                                          "artifact": "crates/rustycad_demo.json",
                                          "artifact_sha256": "missing", "artifact_bytes": 10,
                                          "rustdoc_format_version": 61, "indexed_items": 1}]}
            (root / "capture-manifest.json").write_text(json.dumps(manifest))
            crate = LibraryCrate("demo-id", "rustycad-demo", root / "Cargo.toml",
                                 "rustycad_demo", root, ())
            with self.assertRaisesRegex(ValueError, "missing or changed"):
                validate_capture(root, [crate], context, 61)

    def test_capture_rejects_missing_private_item_policy(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "capture-manifest.json").write_text(json.dumps({
                "raw_capture_complete": True,
                "normalized_public_inventory": False,
                "rustdoc_json_format_version": 61,
            }))
            with self.assertRaisesRegex(ValueError, "document-private-items"):
                validate_capture(root, [], {}, 61)

    def test_file_fingerprint_changes_when_an_input_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "lib.rs"
            source.write_text("pub struct Shape;\n")
            original = digest_paths([source], root)
            source.write_text("pub struct OtherShape;\n")
            self.assertNotEqual(original, digest_paths([source], root))

    def test_capture_rejects_inputs_that_change_during_the_run(self):
        original = {"workspace_inputs_sha256": "before"}
        changed = {"workspace_inputs_sha256": "after"}
        with self.assertRaisesRegex(ValueError, "changed during rustdoc capture"):
            require_unchanged_inputs(original, changed)


if __name__ == "__main__":
    unittest.main()
