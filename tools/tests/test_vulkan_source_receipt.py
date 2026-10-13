"""Named isolated input workspace proves toolchain/configuration belong to the build fingerprint."""

import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import vulkan_source_receipt as receipt


class NamedReceiptWorkspace:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.contents = {
            "Cargo.toml": b"owner",
            "Cargo.lock": b"lock",
            "rust-toolchain.toml": b"compiler",
            ".cargo/config.toml": b"aliases",
            "crates/vulkan/src/lib.rs": b"renderer",
        }
        for name, contents in self.contents.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(contents)


class SourceReceiptTests(unittest.TestCase):
    def test_compiler_configuration_and_renderer_have_exact_fingerprints(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            fake = NamedReceiptWorkspace(Path(folder))
            with patch.object(receipt, "ROOT", fake.root), patch.object(
                receipt, "RECEIPT_ORACLE_PATHS", ()
            ):
                actual = receipt.source_hashes()
            expected = {
                name: hashlib.sha256(contents).hexdigest()
                for name, contents in fake.contents.items()
            }
            self.assertEqual(actual, expected)

    def test_toolchain_mutation_changes_the_full_build_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as folder:
            fake = NamedReceiptWorkspace(Path(folder))
            with patch.object(receipt, "ROOT", fake.root), patch.object(
                receipt, "RECEIPT_ORACLE_PATHS", ()
            ):
                before = receipt.source_hashes()
                (fake.root / "rust-toolchain.toml").write_bytes(b"changed-compiler")
                after = receipt.source_hashes()
            self.assertNotEqual(
                before["rust-toolchain.toml"], after["rust-toolchain.toml"]
            )
            self.assertEqual(
                before["crates/vulkan/src/lib.rs"], after["crates/vulkan/src/lib.rs"]
            )


if __name__ == "__main__":
    unittest.main()
