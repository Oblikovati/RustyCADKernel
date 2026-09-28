"""The api-row audit parses OCCT headers into ledger identities and matches rows to them."""
import tempfile
import unittest
from pathlib import Path

import time

from audit_api_rows import (Member, audit_header, clean_source, header_members, is_brace_initializer,
                            matching_close, normalize_param, normalize_params, parse_row_id,
                            split_statements)

HEADER = """
#define X_Raise_if(C, M) \\
  if (C) throw X(M);
class gp_Demo
{
public:
  DEFINE_STANDARD_ALLOC
  enum class D { X, Y };
  constexpr gp_Demo() noexcept = default;
  gp_Demo(const gp_Demo&) = delete;
  constexpr gp_Demo(const double theX, const D theD = D::X) : myX{theX}, myD(theD) {}
  void Mirror(const gp_Pnt& theP) noexcept;
  [[nodiscard]] gp_Demo Mirrored(const gp_Pnt& theP) const noexcept { return *this; }
  void Mirror(const gp_Ax1& theA1);
  double Value(int theI) const { return myX; } // "{ not a body }"
  double& Value(int theI) { return myX; }
  double operator()(int theI) const { return myX; }
  Standard_EXPORT void DumpJson(Standard_OStream& theOStream, int theDepth = -1) const;
  friend class gp_Other;
protected:
  void Hidden();
private:
  double myX;
};
inline void gp_Demo::Mirror(const gp_Pnt& theP) noexcept {}
inline gp_Demo operator*(const double theS, const gp_Demo& theD) { return theD; }
DEFINE_STANDARD_EXCEPTION(gp_DemoError, Standard_DomainError)
"""


def members_of(text: str) -> list[str]:
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "gp_Demo.hxx"
        path.write_text(text)
        return [f"{m.prefix}|{m.params}|{m.qualifier}" for m in header_members(path)]


class HeaderParsingTests(unittest.TestCase):
    def test_public_members_become_ledger_identities(self):
        self.assertEqual(members_of(HEADER), [
            "gp_DemoError|None|", "gp_Demo::D|None|", "gp_Demo||", "gp_Demo|double,D|",
            "gp_Demo::Mirror|const gp_Pnt&|", "gp_Demo::Mirrored|const gp_Pnt&|const",
            "gp_Demo::Mirror|const gp_Ax1&|", "gp_Demo::Value|int|const", "gp_Demo::Value|int|",
            "gp_Demo::operator()|int|const", "gp_Demo::DumpJson|Standard_OStream&,int|const",
            "gp_Demo::operator*|double,const gp_Demo&|"])

    def test_rtti_macro_opens_a_public_section(self):
        # Regression: DEFINE_STANDARD_RTTIEXT right after `{` swallowed the `public:` label
        # that follows it, hiding every member of Adaptor2d_Curve2d; at the end of a class
        # it still yields the ledger's RTTI row.
        header = """
class Adaptor2d_Demo : public Standard_Transient
{
  DEFINE_STANDARD_RTTIEXT(Adaptor2d_Demo, Standard_Transient)
public:
  double FirstParameter() const;
private:
  double myFirst;
};
class Geom2d_Demo : public Standard_Transient
{
public:
  void Reverse();
  DEFINE_STANDARD_RTTIEXT(Geom2d_Demo, Standard_Transient)
private:
  double myU;
};
"""
        self.assertEqual(members_of(header), [
            "Adaptor2d_Demo::DEFINE_STANDARD_RTTIEXT|Adaptor2d_Demo,Standard_Transient|",
            "Adaptor2d_Demo::FirstParameter||const", "Geom2d_Demo::Reverse||",
            "Geom2d_Demo::DEFINE_STANDARD_RTTIEXT|Geom2d_Demo,Standard_Transient|"])

    def test_cleaning_drops_comments_literals_attributes_and_continued_macros(self):
        cleaned = clean_source('#define A \\\n  b\n[[nodiscard]] int f(); // x\nchar* s = "{";')
        self.assertNotIn("define", cleaned)
        self.assertNotIn("nodiscard", cleaned)
        self.assertNotIn("{", cleaned)

    def test_brace_initializers_do_not_end_statements(self):
        heads = [head.strip() for head, _ in split_statements("A() : m{{1}, {2}} {} int b;")]
        self.assertEqual(heads, ["A() : m{{1}, {2}}", "int b"])

    def test_long_heads_do_not_backtrack(self):
        # Regression: a nested `(\w+\s*)*` quantifier hung on long NCollection template heads.
        head = "template <typename T> " + "Type " * 60 + "f() const noexcept"
        started = time.perf_counter()
        self.assertFalse(is_brace_initializer(head + " x"))
        self.assertTrue(is_brace_initializer("A() noexcept : myMat"))
        self.assertLess(time.perf_counter() - started, 0.5)

    def test_parameter_normalization(self):
        self.assertEqual(normalize_param("const gp_Vec& theV = gp_Vec()"), "const gp_Vec&")
        self.assertEqual(normalize_param("const double theX"), "double")
        self.assertEqual(normalize_param("Standard_Real"), "double")
        self.assertEqual(normalize_param("const NCollection_Mat4<T>& theMat"), "const NCollection_Mat4<T>&")
        self.assertEqual(normalize_params("void"), "")

    def test_unbalanced_brackets_name_the_offset(self):
        with self.assertRaisesRegex(ValueError, "offset 0"):
            matching_close("(a", 0)


class RowMatchingTests(unittest.TestCase):
    def test_row_ids_split_prefix_params_and_qualifier(self):
        self.assertEqual(parse_row_id("gp_GTrsf::operator()(int,int) const"),
                         Member("gp_GTrsf::operator()", "int,int", "const"))
        self.assertEqual(parse_row_id("gp_Ax1::Rotate"), Member("gp_Ax1::Rotate", None, ""))
        self.assertEqual(parse_row_id("gp_Dir(const gp_Dir::D)"), Member("gp_Dir", "gp_Dir::D", ""))
        self.assertEqual(parse_row_id("NCollection_Vec2::NCollection_Vec2(int)"), Member("NCollection_Vec2", "int", ""))

    def test_overloads_need_signatures_and_cv_twins_need_qualifiers(self):
        with tempfile.TemporaryDirectory() as folder:
            header = Path(folder) / "gp_Demo.hxx"
            header.write_text(HEADER)
            rows = [{"upstream_id": upstream_id, "source": "gp/gp_Demo.hxx"} for upstream_id in [
                "gp_Demo::Mirror", "gp_Demo::Mirrored", "gp_Demo::Value(int) const",
                "gp_Demo(double,const gp_Demo::D)", "gp_Demo::Unknown"]]
            missing, stale = audit_header(header, "gp/gp_Demo.hxx", {"api": rows})
        rendered = {f"{m.prefix}({m.params}){m.qualifier}" for m in missing}
        self.assertIn("gp_Demo::Mirror(const gp_Pnt&)", rendered)
        self.assertIn("gp_Demo::Value(int)", rendered)
        self.assertNotIn("gp_Demo::Value(int)const", rendered)
        self.assertNotIn("gp_Demo::Mirrored(const gp_Pnt&)const", rendered)
        self.assertNotIn("gp_Demo(double,D)", rendered)
        self.assertEqual(stale, ["gp_Demo::Mirror", "gp_Demo::Unknown"])


if __name__ == "__main__":
    unittest.main()
