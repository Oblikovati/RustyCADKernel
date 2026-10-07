"""Recognition of macro-generated and special members, and their ledger spellings."""

import unittest

from api_macro_cpp import special_member_kind, upstream_identity
from macro_fakes import declaration

ALLOC = ("src/FoundationClasses/TKernel/Standard/Standard_DefineAlloc.hxx", 55, 9)
RTTI = ("src/FoundationClasses/TKernel/Standard/Standard_Type.hxx", 78, 61)
EXCEPTION = ("src/FoundationClasses/TKernel/Standard/Standard_DefineException.hxx", 48, 17)


class SpecialMemberKindTests(unittest.TestCase):
    def test_allocation_operator_from_define_standard_alloc(self) -> None:
        row = declaration("1", "A::operator new", "CXXMethod", "void *(unsigned long)", ["size_t"], spelled=ALLOC, macro=True)
        self.assertEqual(special_member_kind(row), "allocation")

    def test_rtti_members_need_the_macro_header(self) -> None:
        macro = declaration("1", "A::DynamicType", "CXXMethod", "const T &() const", spelled=RTTI, macro=True)
        hand = declaration("2", "A::DynamicType", "CXXMethod", "const T &() const")
        self.assertEqual(special_member_kind(macro), "rtti_dynamic")
        self.assertIsNone(special_member_kind(hand))

    def test_exception_members(self) -> None:
        kind = declaration("1", "E::ExceptionType", "CXXMethod", "const char *() const noexcept", spelled=EXCEPTION, macro=True)
        ctor = declaration("2", "E::E", "CXXConstructor", "void (const char *)", ["const char *"], macro=True)
        self.assertEqual(special_member_kind(kind), "exception_type")
        self.assertEqual(special_member_kind(ctor), "exception_constructor")

    def test_copy_and_move_members_of_the_owner_only(self) -> None:
        copy = declaration("1", "A::A", "CXXConstructor", "void (const class A &)", ["const class A &"])
        move = declaration("2", "A::operator=", "CXXMethod", "class A &(class A &&)", ["class A &&"])
        other = declaration("3", "A::A", "CXXConstructor", "void (const class B &)", ["const class B &"])
        self.assertEqual(special_member_kind(copy), "copy_constructor")
        self.assertEqual(special_member_kind(move), "move_assignment")
        self.assertIsNone(special_member_kind(other))

    def test_template_owner_reference(self) -> None:
        copy = declaration("1", "NCollection_List::NCollection_List<T>", "CXXConstructor", "", ["const NCollection_List<T> &"])
        copy["qualified_name"] = "NCollection_List::NCollection_List"
        self.assertEqual(special_member_kind(copy), "copy_constructor")

    def test_destructor(self) -> None:
        self.assertEqual(special_member_kind(declaration("1", "A::~A", "CXXDestructor", "void () noexcept")), "destructor")


class UpstreamIdentityTests(unittest.TestCase):
    def test_qualifiers_and_written_parameters(self) -> None:
        row = declaration("1", "A::DynamicType", "CXXMethod", "const T &() const", spelled=RTTI, macro=True)
        self.assertEqual(upstream_identity(row), "A::DynamicType() const")
        delete = declaration("2", "A::operator delete", "CXXMethod", "void (void *) noexcept", ["void *"])
        self.assertEqual(upstream_identity(delete), "A::operator delete(void*) noexcept")

    def test_conditional_noexcept_is_not_spelled(self) -> None:
        row = declaration("1", "M::M", "CXXConstructor", "void (M &&) noexcept(X<H>::value)", ["M &&"])
        self.assertEqual(upstream_identity(row), "M::M(M&&)")

    def test_record_identity_is_the_class_name(self) -> None:
        self.assertEqual(upstream_identity(declaration("1", "A", "CXXRecord")), "A")


if __name__ == "__main__":
    unittest.main()
