"""Reading `= delete`, `= default` and bodies of special members from OCCT sources."""

import unittest

from api_macro_source import DEFAULTED, DELETED, EMPTY_BODY, UNKNOWN, USER_BODY, definition_form
from macro_fakes import FakeOcctSources, declaration

HEADER = "src/P/TK/Pkg/Pkg_A.hxx"
TEXT = """class Pkg_A
{
public:
  Pkg_A(const Pkg_A&) = delete;
  Pkg_A(Pkg_A&&) = default;
  ~Pkg_A() {} // nothing
  Pkg_A& operator=(const Pkg_A& theOther) { Assign(theOther); return *this; }
  Standard_EXPORT virtual ~Pkg_B();
};
"""


def at(line: int, name: str, kind: str = "CXXConstructor") -> dict:
    column = TEXT.split("\n")[line - 1].index(name.rsplit("::", 1)[-1].split("(")[0]) + 1
    return declaration("1", name, kind, header=HEADER, spelled=(HEADER, line, column))


class DefinitionFormTests(unittest.TestCase):
    def setUp(self) -> None:
        self.reader = FakeOcctSources({HEADER: TEXT, "src/P/TK/Pkg/Pkg_B.cxx": "Pkg_B::~Pkg_B()\n{\n  Clear();\n}\n"})

    def test_deleted_and_defaulted(self) -> None:
        self.assertEqual(definition_form(self.reader, at(4, "Pkg_A::Pkg_A")), DELETED)
        self.assertEqual(definition_form(self.reader, at(5, "Pkg_A::Pkg_A")), DEFAULTED)

    def test_inline_bodies(self) -> None:
        self.assertEqual(definition_form(self.reader, at(6, "Pkg_A::~Pkg_A", "CXXDestructor")), EMPTY_BODY)
        self.assertEqual(definition_form(self.reader, at(7, "Pkg_A::operator=", "CXXMethod")), USER_BODY)

    def test_out_of_line_body_is_read_from_package_sources(self) -> None:
        row = at(8, "Pkg_B::~Pkg_B", "CXXDestructor")
        self.assertEqual(definition_form(self.reader, row), USER_BODY)

    def test_missing_file_is_unknown(self) -> None:
        row = declaration("1", "X::~X", "CXXDestructor", spelled=("src/none.hxx", 1, 1))
        self.assertEqual(definition_form(self.reader, row), UNKNOWN)


if __name__ == "__main__":
    unittest.main()
