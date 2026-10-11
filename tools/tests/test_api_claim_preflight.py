"""Preflight regressions use the same declaration matcher as full coverage."""

from __future__ import annotations

import unittest
from unittest.mock import patch

from api_claim_preflight import ClaimFragmentPreflight
from api_crosswalk import cpp_indexes, rust_crates
from api_inventory.inputs import JSONObject
from verify_api_claims import claim_source_digest

HEADER = "src/Visualization/TKV3d/Select3D/Select3D_SensitiveGroup.hxx"
RUST_FILE = "crates/tkv3d/src/select3d/group_entity.rs"
SOURCE_HASH = "a" * 64


class FakeCapturedSourceDigests:
    def __init__(self, actual: str = SOURCE_HASH) -> None:
        """Retain named fake hashes without filesystem reads."""
        self.actual = actual
        self.requested: list[str] = []

    def __call__(self, filename: str) -> str:
        self.requested.append(filename)
        if filename != RUST_FILE:
            raise ValueError(f"source={filename!r}; expected {RUST_FILE!r}")
        return self.actual


class FakeClaimSourcePath:
    def __init__(self, exists: bool) -> None:
        """Record source-boundary calls without creating files."""
        self.exists = exists
        self.requested: list[str] = []

    def __truediv__(self, filename: str) -> FakeClaimSourcePath:
        """Keep the requested relative path for boundary assertions."""
        self.requested.append(filename)
        return self

    def is_file(self) -> bool:
        """Expose the injected existence policy.
        This fake never touches a host checkout.
        """
        return self.exists


class FakeClaimFileDigest:
    def __call__(self, source: FakeClaimSourcePath) -> str:
        """Hash only the named fake source boundary.
        An unexpected source type would indicate real I/O escaped the test.
        """
        assert isinstance(source, FakeClaimSourcePath)
        return SOURCE_HASH


def clear_claim_fixture() -> JSONObject:
    return {
        "upstream_id": "Select3D_SensitiveGroup::Clear",
        "toolkit": "TKV3d",
        "source": HEADER,
        "status": "implemented",
        "cpp_declaration_id": "group-clear",
        "rust_item": "rustycad_tkv3d::select3d::Group::clear",
    }


def clear_native_fixture() -> JSONObject:
    return {
        "id": "group-clear",
        "qualified_name": "Select3D_SensitiveGroup::Clear",
        "kind": "CXXMethod",
        "signature": "void ()",
        "source": HEADER,
        "api_visible": True,
        "parameter_types": [],
    }


def clear_rust_fixture() -> JSONObject:
    return {
        "path": "rustycad_tkv3d::select3d::Group::clear",
        "kind": "function",
        "source_file": RUST_FILE,
        "source_sha256": SOURCE_HASH,
        "source_status": None,
        "declaration_sha256": "b" * 64,
    }


def clear_preflight_fixture(actual: str = SOURCE_HASH) -> ClaimFragmentPreflight:
    names, _, _ = cpp_indexes([clear_native_fixture()], {HEADER: {"TKV3d"}})
    rust = rust_crates(
        {
            "crates": [
                {"crate": "rustycad_tkv3d", "declarations": [clear_rust_fixture()]}
            ]
        }
    )
    return ClaimFragmentPreflight(
        names, rust, {HEADER}, FakeCapturedSourceDigests(actual)
    )


class ClaimPreflightTests(unittest.TestCase):
    def test_array_and_api_object_resolve_same_real_declarations(self) -> None:
        verifier = clear_preflight_fixture()
        row = clear_claim_fixture()
        self.assertEqual(
            verifier.verify_fragment([row]), verifier.verify_fragment({"api": [row]})
        )
        self.assertEqual(verifier.verify_row(row)["resolution"], "implemented")

    def test_return_type_in_api_name_fails_despite_valid_cpp_id(self) -> None:
        row = clear_claim_fixture()
        row["upstream_id"] = "Select3D_SensitiveGroup::Clear void ()"
        with self.assertRaisesRegex(ValueError, "cpp='missing'.*expected unique"):
            clear_preflight_fixture().verify_row(row)

    def test_partial_status_remains_partial(self) -> None:
        row = clear_claim_fixture()
        row["status"] = "partial"
        self.assertEqual(
            clear_preflight_fixture().verify_row(row)["resolution"], "partial"
        )

    def test_stale_rust_selector_cannot_grant_credit(self) -> None:
        row = clear_claim_fixture()
        row["rust_declaration_selectors"] = [
            {
                "path": row["rust_item"],
                "declaration_sha256": "c" * 64,
                "impl_context": None,
            }
        ]
        with self.assertRaisesRegex(ValueError, "missing_rust_selection"):
            clear_preflight_fixture().verify_row(row)

    def test_source_changed_after_capture_fails(self) -> None:
        verifier = clear_preflight_fixture("d" * 64)
        with self.assertRaisesRegex(ValueError, "expected captured hash"):
            verifier.verify_row(clear_claim_fixture())

    def test_invisible_cpp_declaration_cannot_grant_credit(self) -> None:
        verifier = clear_preflight_fixture()
        verifier.native_by_name["Select3D_SensitiveGroup::Clear"][0][
            "api_visible"
        ] = False
        with self.assertRaisesRegex(ValueError, "cpp='missing'"):
            verifier.verify_row(clear_claim_fixture())

    def test_ambiguous_rust_target_cannot_grant_credit(self) -> None:
        verifier = clear_preflight_fixture()
        targets = verifier.rust_by_crate["rustycad_tkv3d"]
        targets["rustycad_tkv3d::select3d::Group::clear"].append(clear_rust_fixture())
        with self.assertRaisesRegex(ValueError, "rust='ambiguous'"):
            verifier.verify_row(clear_claim_fixture())

    def test_duplicate_cpp_claims_fail(self) -> None:
        row = clear_claim_fixture()
        with self.assertRaisesRegex(ValueError, "expected one claim per declaration"):
            clear_preflight_fixture().verify_fragment([row, row])

    def test_bad_fragment_and_unclaimed_status_fail(self) -> None:
        verifier = clear_preflight_fixture()
        with self.assertRaisesRegex(ValueError, "expected array of objects"):
            verifier.verify_fragment({"claims": []})
        row = clear_claim_fixture()
        row["status"] = "excluded"
        with self.assertRaisesRegex(ValueError, "expected implemented or partial"):
            verifier.verify_row(row)

    def test_unknown_native_header_cannot_grant_credit(self) -> None:
        verifier = clear_preflight_fixture()
        verifier.known_sources.clear()
        with self.assertRaisesRegex(ValueError, "expected a captured native header"):
            verifier.verify_row(clear_claim_fixture())

    def test_claim_header_must_equal_pinned_declaration_header(self) -> None:
        verifier = clear_preflight_fixture()
        other = "src/Visualization/TKV3d/Other.hxx"
        verifier.known_sources.add(other)
        row = clear_claim_fixture()
        row["source"] = other
        with self.assertRaisesRegex(ValueError, "expected claim header"):
            verifier.verify_row(row)

    def test_source_boundary_rejects_missing_file(self) -> None:
        root = FakeClaimSourcePath(False)
        with self.assertRaisesRegex(
            ValueError, "expected an existing captured source file"
        ):
            claim_source_digest(root, RUST_FILE)

    def test_source_boundary_hashes_checked_file(self) -> None:
        root = FakeClaimSourcePath(True)
        with patch("verify_api_claims.digest", new=FakeClaimFileDigest()):
            self.assertEqual(claim_source_digest(root, RUST_FILE), SOURCE_HASH)
        self.assertEqual(root.requested, [RUST_FILE])


if __name__ == "__main__":
    unittest.main()
