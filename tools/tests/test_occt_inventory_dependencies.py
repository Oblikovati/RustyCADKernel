"""Input fingerprint regressions using small isolated SDK fixtures."""

from __future__ import annotations

import json
import pathlib
import tempfile
import unittest

from api_inventory.dependencies import (
    compiler_library_fingerprints,
    external_include_fingerprints,
    header_tree_fingerprint,
    sdk_fingerprints,
)
from api_inventory.inputs import digest, toolkit_commands


class DependencyFingerprintTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = pathlib.Path(self.directory.name)

    def test_tree_digest_binds_names_and_bytes(self) -> None:
        header = self.root / "a.hxx"
        header.write_text("int a;")
        first = header_tree_fingerprint(self.root)
        self.assertEqual(first["files"], 1)
        self.assertEqual(first, header_tree_fingerprint(self.root))
        header.write_text("int b;")
        self.assertNotEqual(first, header_tree_fingerprint(self.root))
        header.rename(self.root / "b.hxx")
        self.assertNotEqual(first, header_tree_fingerprint(self.root))

    def test_compiler_library_changes_invalidate_cache(self) -> None:
        library = self.root / "lib/libclang-cpp.so"
        library.parent.mkdir()
        library.write_bytes(b"fixture compiler")
        first = compiler_library_fingerprints(self.root)
        self.assertEqual(first[0]["sha256"], digest(library))
        library.write_bytes(b"changed compiler")
        self.assertNotEqual(first, compiler_library_fingerprints(self.root))

    def test_database_uses_toolkit_flags_and_external_sdk(self) -> None:
        external = self.root / "thirdparty"
        external.mkdir()
        (external / "api.hxx").write_text("int external;")
        source = self.root / "occt"
        database = self.root / "build/compile_commands.json"
        database.parent.mkdir()
        command = {
            "file": str(source / "src/FoundationClasses/TKernel/A.cxx"),
            "directory": str(database.parent),
            "arguments": ["clang++", f"-I{external}", "-c", "A.cxx", "-o", "A.o"],
        }
        database.write_text(json.dumps([command]))
        self.assertEqual(
            toolkit_commands(database, source)["TKernel"][1], [f"-I{external}"]
        )
        self.assertEqual(
            external_include_fingerprints(database, source),
            [header_tree_fingerprint(external)],
        )


if __name__ == "__main__":
    unittest.main()
