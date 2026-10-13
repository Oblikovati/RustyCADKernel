"""Named in-memory inputs distinguish normalization from missing shader/fixture bytes."""

import hashlib
from pathlib import Path
import unittest
import renderer_archive_inputs as inputs


class NamedMemoryInputStore:
    def __init__(self, files: dict[Path, bytes]) -> None:
        self.files = files

    def paths(self, root: Path) -> list[Path]:
        return list(self.files)

    def digest(self, path: Path) -> str:
        return hashlib.sha256(self.files[path]).hexdigest()


class ArchiveInputTests(unittest.TestCase):
    def test_all_semantic_inputs_and_original_manifests_are_retained(self) -> None:
        root = Path("/named/source")
        names = [
            "crates/vulkan/src/lib.rs",
            "crates/vulkan/shaders/face.frag.spv",
            "crates/vulkan/tests/fixtures/texture.dds",
            "crates/vulkan/Cargo.toml.orig",
        ]
        store = NamedMemoryInputStore({root / name: name.encode() for name in names})
        result = inputs.renderer_input_hashes(root, store)
        for kind, name in zip(
            ("source", "shader", "fixture", "manifest"), names, strict=True
        ):
            self.assertEqual(list(result[kind]), [name])

    def test_changed_missing_and_extra_bytes_stay_distinct(self) -> None:
        result = inputs.input_differences(
            {"same": "1", "change": "2", "lost": "3"},
            {"same": "1", "change": "4", "added": "5"},
        )
        self.assertEqual(
            result,
            {
                "missing_from_stage": ["lost"],
                "extra_in_stage": ["added"],
                "changed": ["change"],
            },
        )

    def test_input_bookend_rejects_mutation(self) -> None:
        before, after = {"source": {"lib.rs": "old"}}, {"source": {"lib.rs": "new"}}
        inputs.require_input_bookend(before, before)
        with self.assertRaisesRegex(ValueError, "lib.rs.*expected unchanged"):
            inputs.require_input_bookend(before, after)

    def test_unknown_input_kind_and_removed_category_are_typed_failures(self) -> None:
        with self.assertRaisesRegex(ValueError, "kind='unknown'.*expected one of"):
            inputs.matches_input_kind(Path("lib.rs"), "unknown")
        with self.assertRaisesRegex(ValueError, "source.*expected unchanged"):
            inputs.require_input_bookend({"source": {"lib.rs": "sha"}}, {})

    def test_fixture_selector_includes_json_and_texture_files(self) -> None:
        for name in ("trace.json", "image.dds", "rows.txt"):
            self.assertTrue(
                inputs.matches_input_kind(Path("crates/a/fixtures") / name, "fixture")
            )
        self.assertFalse(
            inputs.matches_input_kind(Path("target/cache.json"), "fixture")
        )


if __name__ == "__main__":
    unittest.main()
