"""Scoped Block binding replay rejects stale, ambiguous and non-body proof rows."""

from __future__ import annotations

import copy
import unittest

from api_inventory.inputs import JSONObject
from api_inventory.merge import declaration_identity
from verify_sparse_block_surface import verify_block_records


class FakeBlockBinding:
    """Keep native/Rust snapshots and source hashes independent of the host checkout."""

    def __init__(self) -> None:
        self.native: JSONObject = {
            "kind": "CXXMethod",
            "usr": "c:@Block@Size",
            "signature": "unsigned long (unsigned long, unsigned long)",
            "qualified_name": "NCollection_SparseArrayBase::Block::Size",
            "api_visible": False,
        }
        self.native["id"] = declaration_identity(self.native)
        self.rust = self.compiler_declaration()
        self.record: JSONObject = {
            "native_declaration": self.native,
            "rust_declaration": self.rust,
            "status": "partial",
            "implemented_credit": False,
        }

    def compiler_declaration(self) -> JSONObject:
        return {
            "kind": "function",
            "path": "rustycad_tkernel::ncollection::SparseArrayBlock::size",
            "source_file": "block.rs",
            "source_sha256": "fresh",
            "signature": {"has_body": True},
            "impl_context": self.block_context(),
        }

    def block_context(self) -> JSONObject:
        return {
            "trait": None,
            "generics": {"params": [], "where_predicates": []},
            "for": {
                "resolved_path": {
                    "resolved_item_path": "rustycad_tkernel::ncollection::sparse_array_base::block::SparseArrayBlock"
                }
            },
        }

    def source_hash(self, filename: str) -> str:
        if filename != "block.rs":
            raise ValueError(f"filename {filename!r}: expected block.rs")
        return "fresh"

    def replay(self) -> int:
        return verify_block_records(
            [self.record], [self.native], [self.rust], self.source_hash
        )


class SparseBlockBindingTests(unittest.TestCase):
    def test_accept_one_exact_current_inherent_body(self) -> None:
        self.assertEqual(FakeBlockBinding().replay(), 1)

    def test_reject_duplicate_native_binding(self) -> None:
        f = FakeBlockBinding()
        with self.assertRaisesRegex(ValueError, "one scoped"):
            verify_block_records(
                [f.record, f.record], [f.native], [f.rust], f.source_hash
            )

    def test_reject_ambiguous_rust_path(self) -> None:
        f = FakeBlockBinding()
        with self.assertRaisesRegex(ValueError, "unique exact Rust"):
            verify_block_records(
                [f.record], [f.native], [f.rust, f.rust], f.source_hash
            )

    def test_reject_native_signature_change(self) -> None:
        f = FakeBlockBinding()
        changed = {**f.native, "signature": "wrong overload"}
        with self.assertRaisesRegex(ValueError, "native declaration"):
            verify_block_records([f.record], [changed], [f.rust], f.source_hash)

    def test_reject_stale_source_and_absent_body(self) -> None:
        for field, value in [
            ("source_sha256", "old"),
            ("signature", {"has_body": False}),
        ]:
            f = FakeBlockBinding()
            f.rust[field] = value
            with self.assertRaises(ValueError):
                f.replay()

    def test_reject_wrong_impl_owner_and_specialization(self) -> None:
        for field, value in [
            ("trait", {"path": "Clone"}),
            ("generics", {"params": ["T"], "where_predicates": []}),
            ("for", {"resolved_path": {"resolved_item_path": "Wrong"}}),
        ]:
            f = FakeBlockBinding()
            f.rust["impl_context"][field] = value
            with self.assertRaises(ValueError):
                f.replay()

    def test_reject_visibility_and_implementation_promotion(self) -> None:
        f = FakeBlockBinding()
        f.native["api_visible"] = True
        with self.assertRaisesRegex(ValueError, "primary visibility"):
            f.replay()
        f = FakeBlockBinding()
        f.record["implemented_credit"] = True
        with self.assertRaisesRegex(ValueError, "implemented credit"):
            f.replay()

    def test_type_alias_is_verified_without_invented_function_body(self) -> None:
        f = FakeBlockBinding()
        f.native.update(
            kind="Typedef", qualified_name="NCollection_SparseArrayBase::Block::Cell"
        )
        f.native["id"] = declaration_identity(f.native)
        f.rust.update(kind="type_alias", signature={"type": {"primitive": "u8"}})
        self.assertEqual(f.replay(), 1)
        f.record = copy.deepcopy(f.record)
        f.record["rust_declaration"]["kind"] = "struct_field"
        with self.assertRaises(ValueError):
            f.replay()


if __name__ == "__main__":
    unittest.main()
