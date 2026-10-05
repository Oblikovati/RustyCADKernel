"""Bind LinearVector rows to unique compiler declarations."""

import gzip
import json
import unittest
from pathlib import Path
from api_crosswalk import cpp_candidates, cpp_indexes
from api_inventory.inputs import JSONValue

ROOT = Path(__file__).resolve().parents[2]
SOURCE = "src/FoundationClasses/TKernel/NCollection/NCollection_LinearVector.hxx"
OLD_SELECTORS = (
    "NCollection_LinearVector(const NCollection_LinearVector&)",
    "NCollection_LinearVector(NCollection_LinearVector&&)",
    "NCollection_LinearVector::operator=(const NCollection_LinearVector&)",
    "NCollection_LinearVector::operator=(NCollection_LinearVector&&)",
    "NCollection_LinearVector::EmplaceAppend(Args&&)",
)
EXPECTED = {
    "NCollection_LinearVector(const NCollection_LinearVector<TheItemType>&)": (
        "78f3a1d2dae89b2d22901d45b252ac9890e5a49c9f85c3bb11bcc31e4e0b0c6c",
        "void (const NCollection_LinearVector<TheItemType> &)",
        "rustycad_tkernel::ncollection::LinearVector::<core::clone::Clone>::clone",
        "implemented",
    ),
    "NCollection_LinearVector(NCollection_LinearVector<TheItemType>&&)": (
        "98acd105a8505fe1b4a20474d6f3651f8aaa3bf826e455ede63479a0fb019bb9",
        "void (NCollection_LinearVector<TheItemType> &&) noexcept",
        "rustycad_tkernel::ncollection::LinearVector::move_from",
        "implemented",
    ),
    (
        "NCollection_LinearVector::operator=(const "
        "NCollection_LinearVector<TheItemType>&)"
    ): (
        "89f01bbf71284e3279ec7ce1f8257d279384cebe1332b41ddd7fe344e3b1ee27",
        (
            "NCollection_LinearVector<TheItemType> &(const "
            "NCollection_LinearVector<TheItemType> &)"
        ),
        "rustycad_tkernel::ncollection::LinearVector::assign",
        "partial",
    ),
    "NCollection_LinearVector::operator=(NCollection_LinearVector<TheItemType>&&)": (
        "c801644c4c70988b579335d314c2fee74348414c441c0d9d562af7801c301d99",
        (
            "NCollection_LinearVector<TheItemType> &("
            "NCollection_LinearVector<TheItemType> &&) noexcept"
        ),
        "rustycad_tkernel::ncollection::LinearVector::move_from",
        "implemented",
    ),
    "NCollection_LinearVector::EmplaceAppend(Args&&...)": (
        "ae297df4aa7a28d7067bd7ea4cc3906b3f33dcc614e57363c7d6f9ad97381b06",
        "type-parameter-0-0 &(type-parameter-1-0 &&...)",
        "rustycad_tkernel::ncollection::LinearVector::emplace_append",
        "partial",
    ),
}


def load_declarations() -> list[dict[str, JSONValue]]:
    path = ROOT / "porting/api-inventory/occt-declarations.jsonl.gz"
    names = {
        "NCollection_LinearVector",
        "NCollection_LinearVector::NCollection_LinearVector<TheItemType>",
        "NCollection_LinearVector::operator=",
        "NCollection_LinearVector::EmplaceAppend",
    }
    with gzip.open(path, "rt") as stream:
        return [
            row
            for line in stream
            if (row := json.loads(line)).get("qualified_name") in names
        ]


def declaration_index() -> dict[str, list[dict[str, JSONValue]]]:
    declarations = load_declarations()
    manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
    owners = {
        header["path"]: {owner["toolkit"] for owner in header.get("owners", [])}
        for header in manifest["headers"]
    }
    by_name, _, _ = cpp_indexes(declarations, owners)
    return by_name


def load_ledger() -> dict[str, dict[str, JSONValue]]:
    rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
    return {row["upstream_id"]: row for row in rows}


class LinearVectorSelectorTest(unittest.TestCase):
    def test_five_corrected_selectors_match_unique_compiler_rows(self) -> None:
        by_name = declaration_index()
        ledger = load_ledger()
        for selector, values in EXPECTED.items():
            compiler_id, signature, rust_path, expected_status = values
            with self.subTest(selector=selector):
                status, candidates = cpp_candidates(selector, "TKernel", by_name)
                self.assertEqual(status, "unique")
                candidate_ids = [candidate["id"] for candidate in candidates]
                self.assertEqual(candidate_ids, [compiler_id])
                self.assertEqual(candidates[0]["signature"], signature)
                self.assertEqual(candidates[0]["source"], SOURCE)
                self.assertEqual(ledger[selector]["rust_item"], rust_path)
                self.assertEqual(ledger[selector]["status"], expected_status)

    def test_ledger_keeps_the_five_exact_selectors_partial_or_mapped(self) -> None:
        ledger = load_ledger()
        self.assertEqual(set(EXPECTED), set(ledger) & set(EXPECTED))

    def test_old_uninstantiated_template_selectors_stay_unresolved(self) -> None:
        by_name = declaration_index()
        for selector in OLD_SELECTORS:
            with self.subTest(selector=selector):
                status, candidates = cpp_candidates(selector, "TKernel", by_name)
                self.assertEqual((status, candidates), ("missing", []))


if __name__ == "__main__":
    unittest.main()
