"""Keep base exception assignment bound to its native declaration."""

import gzip
import json
import unittest
from pathlib import Path

from api_crosswalk import cpp_candidates, cpp_indexes, header_owners

ROOT = Path(__file__).resolve().parents[2]
SELECTOR = "Standard_Failure::operator=(const Standard_Failure&)"
COMPILER_ID = "b94cc1f21b3fd499940427c345ffce659a5a091901b8a6c02d53c6508582151a"


BYTE_BINDINGS = (
    (
        "Standard_OutOfMemory::Standard_OutOfMemory(const char* const)",
        "fa68bfd8d5e7473ba997452a2bdc683175e4d320fbe21734c9e6d08c1410ef59",
        "rustycad_tkernel::standard::exceptions::out_of_memory_bytes",
        "partial",
    ),
    (
        "Standard_OutOfMemory::SetMessageString(const char* const)",
        "a8a461b6141372aeab77344619d3c685fb1a12d56317b4bef706a88be6c8f527",
        "rustycad_tkernel::standard::Failure::set_message_bytes",
        "partial",
    ),
    (
        "Standard_Failure::Standard_Failure(const char*)",
        "b1fe29956a4bac2d98fc7223b486fc2be85e2f49c5666ae4855d35f4cdd6803c",
        "rustycad_tkernel::standard::Failure::from_message_bytes",
        "implemented",
    ),
    (
        "Standard_Failure::Standard_Failure(const char* const,const char* const)",
        "63a3ce0ae9241c03704ccb8caeb48cc3eb41a6b1d94368c75071f52a7065f91a",
        "rustycad_tkernel::standard::Failure::with_type_bytes",
        "implemented",
    ),
    (
        "Standard_Failure::GetStackString() const",
        "b0c1b4dd328e5a608942c749b205a2832183404699f942d228134d6a1a92b078",
        "rustycad_tkernel::standard::Failure::stack_bytes",
        "implemented",
    ),
    (
        "Standard_Failure::Print(Standard_OStream&) const",
        "5674c27025f511e5def0959207dfe252931588a1f190941d53e0228fc1b3fb13",
        "rustycad_tkernel::standard::Failure::print_to",
        "partial",
    ),
    (
        "operator<<(Standard_OStream&,const Standard_Failure&)",
        "42056e82762cb2000cab50f68c286f3e36062dc56cefcd92448a25c787e8ce78",
        "rustycad_tkernel::standard::Failure::insert_into",
        "partial",
    ),
)


class FailureAssignmentBindingTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        with gzip.open(
            ROOT / "porting/api-inventory/occt-declarations.jsonl.gz", "rt"
        ) as stream:
            declarations = [json.loads(line) for line in stream]
        manifest = json.loads((ROOT / "porting/api-header-manifest.json").read_text())
        cls.by_name, _, _ = cpp_indexes(declarations, header_owners(manifest))

    def test_base_assignment_uses_explicit_rust_operation(self) -> None:
        status, candidates = cpp_candidates(SELECTOR, "TKernel", self.by_name)
        self.assertEqual(status, "unique")
        self.assertEqual([row["id"] for row in candidates], [COMPILER_ID])
        rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        matches = [row for row in rows if row["upstream_id"] == SELECTOR]
        self.assertEqual(len(matches), 1)
        self.assertEqual(
            matches[0]["rust_item"], "rustycad_tkernel::standard::Failure::assign_base"
        )
        self.assertEqual(matches[0]["status"], "implemented")
        self.assertEqual(
            matches[0]["rust_items"],
            [
                "rustycad_tkernel::standard::Failure::assign_base",
                "rustycad_tkernel::standard::Failure::assign_base_self",
            ],
        )

    def test_base_copy_slices_derived_identity(self) -> None:
        selector = "Standard_Failure(const Standard_Failure&)"
        compiler_id = "c72444b328de71bc61f432cb717bba784d9c25ac7ea8ee0965162d3a0f96c909"
        status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
        self.assertEqual(status, "unique")
        self.assertEqual([row["id"] for row in candidates], [compiler_id])
        rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        matches = [row for row in rows if row["upstream_id"] == selector]
        self.assertEqual(len(matches), 1)
        self.assertEqual(
            matches[0]["rust_item"], "rustycad_tkernel::standard::Failure::copy_base"
        )
        self.assertEqual(matches[0]["status"], "implemented")

    def test_byte_adapters_use_exact_native_declarations(self) -> None:
        rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        by_selector = {row["upstream_id"]: row for row in rows}
        for selector, compiler_id, method, parity_status in BYTE_BINDINGS:
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [compiler_id])
            self.assertEqual(
                by_selector[selector]["rust_item"],
                method,
            )
            self.assertEqual(by_selector[selector]["status"], parity_status)

    def _assert_generated_exception_bindings(self, audit_name: str) -> None:
        receipt = json.loads((ROOT / "porting/audits" / audit_name).read_text())
        rows = json.loads((ROOT / "porting/parity.json").read_text())["api"]
        by_selector = {row["upstream_id"]: row for row in rows}
        self.assertEqual(len(receipt["bindings"]), 38)
        for binding in receipt["bindings"]:
            selector = binding["selector"]
            status, candidates = cpp_candidates(selector, "TKernel", self.by_name)
            self.assertEqual(status, "unique", selector)
            self.assertEqual([row["id"] for row in candidates], [binding["cpp_id"]])
            self.assertEqual(by_selector[selector]["rust_item"], binding["rust_item"])
            self.assertEqual(by_selector[selector]["status"], "implemented")

    def test_generated_byte_constructors_bind_exact_declarations(self) -> None:
        self._assert_generated_exception_bindings("exception-byte-constructors.json")

    def test_generated_stack_constructors_bind_exact_declarations(self) -> None:
        self._assert_generated_exception_bindings("exception-stack-constructors.json")

    def test_generated_exception_types_bind_exact_declarations(self) -> None:
        self._assert_generated_exception_bindings("exception-type-bindings.json")
