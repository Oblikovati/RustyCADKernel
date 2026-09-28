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
class gp_DemoError;
DEFINE_STANDARD_HANDLE(gp_DemoError, Standard_DomainError)
#if !defined No_Exception
#endif
DEFINE_STANDARD_EXCEPTION(gp_DemoError, Standard_DomainError)
"""


def members_of(text: str) -> list[str]:
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "gp_Demo.hxx"
        path.write_text(text)
        return [f"{m.prefix}|{m.params}|{m.qualifier}" for m in header_members(path) if m.required]


def known_of(text: str, stem: str = "gp_Demo") -> list[str]:
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / f"{stem}.hxx"
        path.write_text(text)
        return [f"{m.prefix}|{m.params}|{m.qualifier}" for m in header_members(path) if not m.required]


TEMPLATE_HEADER = """
template <class T, int N> class BVH_Tree;
template <class T, int N>
class BVH_Tree<T, N, BVH_BinaryTree> : public BVH_TreeBase<T, N>
{
public:
  typedef typename BVH_TreeBase<T, N>::BVH_VecNt BVH_VecNt;
  BVH_Tree() {}
  int EstimateSAH() const;
  void Build(const opencascade::handle<BVH_Builder<T, N>>& theB);
  static T Area(const typename BVH_Box<T, N>::BVH_VecNt& theSize);
  BVH_VecNt Origin;
  ~BVH_Tree() override = default;
protected:
  struct BVH_Range
  {
    BVH_Range(int theS) {}
  };
  int myDepth;
};
template <class T, int N>
int BVH_Tree<T, N, BVH_BinaryTree>::EstimateSAH() const { return 0; }
template <class T, int N, template <class, int> class TheBox>
class BVH_BaseBox {};
enum
{
  BVH_Constants_MaxTreeDepth = 32,
};
namespace BVH
{
constexpr double THE_NODE_MIN_SIZE = std::numeric_limits<double>::epsilon();
}
class BVH_Transient : public Standard_Transient
{
  DEFINE_STANDARD_RTTIEXT(BVH_Transient, Standard_Transient)
  int Depth() const;
};
"""


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

    def test_multi_literal_deprecation_messages_are_dropped(self):
        # NCollection_Map splits its deprecation message over two adjacent literals.
        cleaned = clean_source('Standard_DEPRECATED("Use methods from "\n  "MapAlgo.") bool IsEqual();')
        self.assertEqual(cleaned.strip(), "bool IsEqual();")

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

    def test_array_parameters_decay_to_pointers(self):
        # Bnd_OBB::GetVertex(gp_Pnt theP[8]) is the function GetVertex(gp_Pnt*) in C++.
        self.assertEqual(normalize_param("gp_Pnt theP [8]"), "gp_Pnt*")
        self.assertEqual(normalize_param("const double theM[2][3]"), "const double(*)[3]")
        self.assertEqual(normalize_params("gp_Pnt theP[8]"), normalize_params("gp_Pnt* theP"))

    def test_unbalanced_brackets_name_the_offset(self):
        with self.assertRaisesRegex(ValueError, "offset 0"):
            matching_close("(a", 0)


class TemplateAndKnownIdentityTests(unittest.TestCase):
    def test_specializations_keep_arguments_and_out_of_class_definitions_are_ignored(self):
        self.assertEqual(members_of(TEMPLATE_HEADER), [
            "BVH_Tree<T,N,BVH_BinaryTree>||", "BVH_Tree<T,N,BVH_BinaryTree>::EstimateSAH||const",
            "BVH_Tree<T,N,BVH_BinaryTree>::Build|const opencascade::handle<BVH_Builder<T,N>>&|",
            "BVH_Tree<T,N,BVH_BinaryTree>::Area|const BVH_Box<T,N>::BVH_VecNt&|",
            "BVH_Tree<T,N,BVH_BinaryTree>::~BVH_Tree||",
            "BVH_Transient::DEFINE_STANDARD_RTTIEXT|BVH_Transient,Standard_Transient|",
            "BVH_Transient::Depth||const"])

    def test_types_data_enumerators_and_protected_members_are_known_only(self):
        self.assertEqual(known_of(TEMPLATE_HEADER), [
            "BVH_Tree<T,N,BVH_BinaryTree>|None|", "BVH_Tree<T,N,BVH_BinaryTree>::BVH_VecNt|None|",
            "BVH_Tree<T,N,BVH_BinaryTree>::Origin|None|", "BVH_Tree<T,N,BVH_BinaryTree>::BVH_Range|None|",
            "BVH_Tree<T,N,BVH_BinaryTree>::BVH_Range|int|", "BVH_Tree<T,N,BVH_BinaryTree>::myDepth|None|",
            "BVH_BaseBox|None|", "BVH_Constants_MaxTreeDepth|None|", "BVH::THE_NODE_MIN_SIZE|None|",
            "BVH_Transient|None|"])

    def test_known_rows_are_not_stale_and_excluded_rows_need_notes(self):
        with tempfile.TemporaryDirectory() as folder:
            header = Path(folder) / "BVH_Tree.hxx"
            header.write_text(TEMPLATE_HEADER)
            rows = [{"upstream_id": upstream_id, "source": "BVH/BVH_Tree.hxx"} for upstream_id in [
                "BVH_Tree<T,N,BVH_BinaryTree>::Origin", "BVH_Constants_MaxTreeDepth", "BVH_Tree<T,N,BVH_BinaryTree>",
                "BVH_Tree<T,N,BVH_BinaryTree>::BVH_Range::BVH_Range(int)", "BVH_Tree<T,N,BVH_BinaryTree>::Private"]]
            rows.append({"upstream_id": "BVH_Tree<T,N,BVH_BinaryTree>::~BVH_Tree()", "source": "BVH/BVH_Tree.hxx",
                         "status": "excluded"})
            missing, stale = audit_header(header, "BVH/BVH_Tree.hxx", {"api": rows})
        self.assertNotIn("BVH_Tree<T,N,BVH_BinaryTree>::~BVH_Tree", [m.prefix for m in missing])
        self.assertEqual(stale, ["BVH_Tree<T,N,BVH_BinaryTree>::Private",
                                 "BVH_Tree<T,N,BVH_BinaryTree>::~BVH_Tree() (excluded without notes)"])

    def test_nested_templates_and_dependent_types_normalize(self):
        self.assertEqual(normalize_param("const occ::handle<NCollection_HArray1<Bnd_Box>>& theSet"),
                         "const occ::handle<NCollection_HArray1<Bnd_Box>>&")
        self.assertEqual(normalize_param("std::array<long double, 4>& theRoots"), "std::array<long double,4>&")
        self.assertEqual(normalize_param("unsigned int theX"), "unsigned int")


NAMESPACE_HEADER = """
namespace MathDemo
{
struct Result
{
  explicit operator bool() const { return true; }
};
namespace Utils
{
inline long double ToDouble(long double theValue) { return theValue; }
inline bool Solve3x3(const double theJ[3][3], double theDelta[3]) { return true; }
template <size_t N>
inline double SafeNormN(const double (&theValues)[N]) { return 0.0; }
}
}
"""


class ConversionAndArrayTests(unittest.TestCase):
    """Regressions found on the modern Math* namespaces (MathPoly, MathSys)."""

    def test_conversion_operators_and_array_parameters_render_exactly(self):
        self.assertEqual(members_of(NAMESPACE_HEADER), [
            "MathDemo::Result::operator bool||const", "MathDemo::Utils::ToDouble|long double|",
            "MathDemo::Utils::Solve3x3|const double(*)[3],double*|",
            "MathDemo::Utils::SafeNormN|const double(&)[N]|"])

    def test_nameless_multiword_builtins_and_arrays_normalize_like_named_ones(self):
        self.assertEqual(normalize_param("long double"), normalize_param("long double theValue"))
        # By-value arrays decay as C++ adjusts them; references to arrays keep extents.
        self.assertEqual(normalize_param("const double theJ[3][3]"), "const double(*)[3]")
        self.assertEqual(normalize_param("const double (&theF)[N]"), "const double(&)[N]")
        self.assertEqual(normalize_param("const double[9]"), "const double*")
        self.assertEqual(normalize_param("Standard_Real theP[2]"), "double*")

    def test_rows_spelled_like_the_header_are_not_stale(self):
        with tempfile.TemporaryDirectory() as folder:
            header = Path(folder) / "MathDemo_Types.hxx"
            header.write_text(NAMESPACE_HEADER)
            rows = [{"upstream_id": upstream_id, "source": "x.hxx"} for upstream_id in [
                "MathDemo::Result::operator bool() const", "MathDemo::Utils::ToDouble(long double)",
                "MathDemo::Utils::Solve3x3(const double[3][3],double[3])",
                "MathDemo::Utils::SafeNormN(const double(&)[N])"]]
            missing, stale = audit_header(header, "x.hxx", {"api": rows})
        self.assertEqual(([m.prefix for m in missing], stale), ([], []))


SWIZZLE_HEADER = """
#define VEC_COMPONENTS_2D(theX, theY) \\
  constexpr NCollection_Vec2<Element_t> theX##theY() const noexcept \\
  { return NCollection_Vec2<Element_t>(theX(), theY()); }
template <typename Element_t>
class NCollection_Vec2
{
public:
  VEC_COMPONENTS_2D(x, y)
  constexpr Element_t& x() noexcept { return v[0]; }
  friend constexpr NCollection_Vec2 operator+(const NCollection_Vec2& theL,
                                              const NCollection_Vec2& theR) noexcept
  { return theL; }
  friend class NCollection_Mat3;
  friend bool operator==(const NCollection_Vec2& theL, const NCollection_Vec2& theR);
private:
  Element_t v[2];
};
"""


class MacroAndFriendTests(unittest.TestCase):
    def test_header_local_macros_expand_into_members(self):
        self.assertIn("NCollection_Vec2::xy||const", members_of(SWIZZLE_HEADER))
        self.assertIn("NCollection_Vec2::x||", members_of(SWIZZLE_HEADER))

    def test_in_class_friend_definitions_are_public_members(self):
        members = members_of(SWIZZLE_HEADER)
        self.assertIn("NCollection_Vec2::operator+|const NCollection_Vec2&,const NCollection_Vec2&|", members)
        # Friend classes and body-less friend declarations are not API of this class.
        self.assertFalse([m for m in members if "operator==" in m or "Mat3" in m])

    def test_macro_arity_mismatch_names_the_arguments(self):
        with self.assertRaisesRegex(ValueError, r"\['x'\]; expected 2 arguments"):
            members_of(SWIZZLE_HEADER.replace("VEC_COMPONENTS_2D(x, y)", "VEC_COMPONENTS_2D(x)"))

    def test_deprecation_markers_are_not_members(self):
        header = ('Standard_HEADER_DEPRECATED("a is deprecated. " "Use b.");\n'
                  'template <class T>\nusing NCollection_Old = NCollection_New<T> Standard_DEPRECATED("x");\n')
        self.assertEqual(members_of(header), [])
        self.assertEqual(known_of(header), ["NCollection_Old|None|"])
    def test_deprecation_with_concatenated_literals_keeps_the_member(self):
        header = ('class gp_Demo\n{\npublic:\n  Standard_DEPRECATED("Use the "\n'
                  '                      "algo instead.")\n  bool Unite(const gp_Demo& theOther);\n};\n')
        self.assertEqual(members_of(header), ["gp_Demo::Unite|const gp_Demo&|"])

    def test_standard_rtti_macros_are_never_expanded(self):
        header = "#define DEFINE_STANDARD_RTTIEXT(C, B) public: int Bad();\n" + TEMPLATE_HEADER
        self.assertNotIn("BVH_Transient::Bad||", members_of(header))

    def test_macros_of_transitively_included_siblings_expand(self):
        # NCollection_Vec4 reaches NCOLLECTION_VEC_COMPONENTS_2D only through NCollection_Vec3.
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "A.hxx").write_text("#define PAIR(a, b) int a##b() const;\n#include <C.hxx>\n")
            (root / "B.hxx").write_text("#include <A.hxx>\n")
            (root / "C.hxx").write_text("#include <B.hxx>\nclass C\n{\npublic:\n  PAIR(x, y)\n};\n")
            members = [f"{m.prefix}|{m.params}|{m.qualifier}" for m in header_members(root / "C.hxx")]
        self.assertIn("C::xy||const", members)


TRAITS_HEADER = """
namespace Detail
{
template <typename T, typename = void>
struct HasValue : std::false_type {};
template <typename T>
struct HasValue<T, std::void_t<decltype(std::declval<const T&>().Value())>> : std::true_type
{
  static int Get(const T& theIter) { return 0; }
};
template <class K>
struct KeyRef
{
  template <std::size_t I>
  decltype(auto) get() const noexcept { return 0; }
};
}
template <typename H>
class Range
{
public:
  explicit Range(H&& theHost) {}
};
template <typename T>
Range(T&&) -> Range<std::remove_reference_t<T>>;
"""


class TemplateMachineryTests(unittest.TestCase):
    def test_specialization_arguments_with_calls_open_a_scope(self):
        members = members_of(TRAITS_HEADER)
        self.assertIn("Detail::HasValue<T,std::void_t<decltype(std::declval<const T&>().Value())>>::Get|const T&|",
                      members)
        self.assertFalse([m for m in members if "decltype(std" in m.split("|")[0].split("<")[0]])

    def test_decltype_return_types_are_not_names(self):
        self.assertIn("Detail::KeyRef::get||const", members_of(TRAITS_HEADER))

    def test_deduction_guides_are_not_constructors(self):
        self.assertEqual([m for m in members_of(TRAITS_HEADER) if m.startswith("Range")], ["Range|H&&|"])


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

    def test_rtti_macros_end_their_statement(self):
        # Regression: DEFINE_STANDARD_RTTIEXT has no semicolon, so the private members
        # after it were once parsed as public (Geom2dEval_TBezierCurve::evalBasisDeriv).
        text = """class G_T : public G_B
{
public:
  void Visible();
  DEFINE_STANDARD_RTTIEXT(G_T, G_B)
private:
  void hiddenFirst(double theT);
  void hiddenSecond(int theN);
};
class G_U : public G_B
{
public:
  DEFINE_STANDARD_RTTI_INLINE(G_U, G_B)
};"""
        self.assertEqual(members_of(text), [
            "G_T::Visible||", "G_T::DEFINE_STANDARD_RTTIEXT|G_T,G_B|",
            "G_U::DEFINE_STANDARD_RTTI_INLINE|G_U,G_B|"])

    def test_public_field_rows_are_accepted_but_not_required(self):
        text = """namespace Desc
{
class Mapped
{
public:
  std::size_t MaxDerivOrder = 3;
  std::optional<Domain1d> Domain;
  int Kind() const;
private:
  double myHidden;
};
}"""
        with tempfile.TemporaryDirectory() as folder:
            header = Path(folder) / "Desc.hxx"
            header.write_text(text)
            rows = [{"upstream_id": upstream_id, "source": "p/Desc.hxx"} for upstream_id in [
                "Desc::Mapped::Domain", "Desc::Mapped::Kind", "Desc::Mapped::myHidden"]]
            missing, stale = audit_header(header, "p/Desc.hxx", {"api": rows})
        self.assertEqual(missing, [])
        self.assertEqual(stale, ["Desc::Mapped::myHidden"])


STANDARD_HEADER = """
Standard_DEPRECATED("This function duplicates std::acos and will be removed in future releases. "
                    "Use std::acos instead.")
inline double ACos(const double theValue) { return std::acos(theValue); }
namespace MurmurHash
{
template <typename T1, typename T = size_t>
typename std::enable_if<sizeof(T) == 8, uint64_t>::type hash_combine(const T1& theValue,
                                                                     const int theLen = sizeof(T1));
}
class Standard_Dump
{
public:
  static void DumpRealValues(Standard_OStream& theOStream, int theCount, ...);
};
namespace std
{
template <class TheTransientType>
struct hash<Handle(TheTransientType)>
{
  size_t operator()(const Handle(TheTransientType)& theHandle) const noexcept { return 0; }
};
}
#ifdef __cplusplus
extern "C"
{
#endif
  Standard_EXPORT double Atof(const char* theStr);
#ifdef __cplusplus
}
#endif
"""


class StandardPackageTests(unittest.TestCase):
    """Regressions found on TKernel/Standard, whose headers mix macros and free templates."""

    def test_deprecated_messages_split_over_literals_do_not_hide_the_function(self):
        self.assertIn("gp_Demo::ACos|double|", members_of(STANDARD_HEADER))

    def test_sizeof_in_a_return_type_is_not_the_function_name(self):
        self.assertIn("MurmurHash::hash_combine|const T1&,int|", members_of(STANDARD_HEADER))

    def test_variadic_ellipsis_is_kept(self):
        self.assertEqual(normalize_params("Standard_OStream& theOStream, int theCount, ..."),
                         "Standard_OStream&,int,...")
        self.assertIn("Standard_Dump::DumpRealValues|Standard_OStream&,int,...|", members_of(STANDARD_HEADER))

    def test_extern_c_blocks_declare_members_of_the_enclosing_scope(self):
        # Standard_CString.hxx declares Atof, Strtod and the Printf family in such a block.
        self.assertIn("gp_Demo::Atof|const char*|", members_of(STANDARD_HEADER))

    def test_specializations_whose_arguments_hold_macro_calls_are_scopes(self):
        # The `Handle(T)` macro is expanded as the preprocessor does.
        self.assertIn("std::hash<opencascade::handle<TheTransientType>>::operator()"
                      "|const opencascade::handle<TheTransientType>&|const",
                      members_of(STANDARD_HEADER))


if __name__ == "__main__":
    unittest.main()
