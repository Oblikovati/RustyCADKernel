"""Sort OCCT declarations into categories so parity metrics separate real API from noise.

The Clang inventory counts every declaration a header produces. About a third of them are
expansions of OCCT boilerplate macros (allocation operators, RTTI accessors, exception
constructors) or explicit special members. Counting them as "missing API" made the
published gap look three times larger than the work that remains. This module is the one
place that decides the category; reports and ledger generators import it.
"""

from __future__ import annotations

from collections.abc import Mapping

ALLOC_MACRO = "alloc-macro"
RTTI_MACRO = "rtti-macro"
EXCEPTION_MACRO = "exception-macro"
SPECIAL_MEMBER = "special-member"
PROTECTED_MEMBER = "protected-member"
ENUM_CONSTANT = "enum-constant"
DATA_FIELD = "field"
TYPE_ALIAS = "alias"
RECORD = "record"
MEMBER = "member"

# Categories that a faithful Rust port carries by construction (allocator, `Drop`, `Clone`,
# runtime-type traits, error types); they need a justified ledger row, not new code.
BOILERPLATE_CATEGORIES = frozenset({ALLOC_MACRO, RTTI_MACRO, EXCEPTION_MACRO, SPECIAL_MEMBER})
CATEGORY_ORDER = (
    MEMBER, RECORD, PROTECTED_MEMBER, ENUM_CONSTANT, DATA_FIELD, TYPE_ALIAS,
    SPECIAL_MEMBER, EXCEPTION_MACRO, RTTI_MACRO, ALLOC_MACRO,
)

_ALLOC_HEADERS = frozenset({"Standard_DefineAlloc.hxx", "NCollection_DefineAlloc.hxx"})
_ALLOC_OPERATORS = ("operator new", "operator delete")
_ALIAS_KINDS = frozenset({
    "TypeAlias", "Typedef", "TypeAliasTemplate", "UsingDirective", "Using",
    "Namespace", "UnresolvedUsingValue",
})
_RECORD_KINDS = frozenset({
    "CXXRecord", "ClassTemplate", "ClassTemplateSpecialization",
    "ClassTemplatePartialSpecialization", "Enum",
})


def declaration_category(declaration: Mapping[str, object]) -> str:
    """Return the category of one `occt-declarations` record.

    Example: `declaration_category({"kind": "CXXDestructor", ...}) == "special-member"`.
    """
    macro = _macro_category(declaration)
    if macro:
        return macro
    if _is_special_member(declaration):
        return SPECIAL_MEMBER
    return _plain_category(declaration)


def _macro_category(declaration: Mapping[str, object]) -> str:
    """Name the boilerplate macro family, or return "" for ordinary code."""
    if not declaration.get("macro_generated"):
        return ""
    header = _spelling_header(declaration)
    if header in _ALLOC_HEADERS or _member_name(declaration).startswith(_ALLOC_OPERATORS):
        return ALLOC_MACRO
    if header == "Standard_Type.hxx":
        return RTTI_MACRO
    if header == "Standard_DefineException.hxx" or _is_exception_constructor(declaration):
        return EXCEPTION_MACRO
    return ""


def _is_exception_constructor(declaration: Mapping[str, object]) -> bool:
    """DEFINE_STANDARD_EXCEPTION spells its constructors with the class-name token of the header."""
    owner = _owner_name(declaration)
    return (
        declaration.get("kind") == "CXXConstructor"
        and _spelling_header(declaration) == owner + ".hxx"
        and all(kind == "const char *" for kind in _parameter_types(declaration))
    )


def _is_special_member(declaration: Mapping[str, object]) -> bool:
    """Destructors, copy/move constructors and assignment operators."""
    kind = declaration.get("kind")
    if kind == "CXXDestructor" or _member_name(declaration) == "operator=":
        return True
    if kind != "CXXConstructor":
        return False
    parameters = _parameter_types(declaration)
    return len(parameters) == 1 and _names_own_class(parameters[0], _owner_name(declaration))


def _names_own_class(parameter: str, owner: str) -> bool:
    """True for `const Owner &`, `Owner &&` and their template spellings."""
    stripped = parameter.replace("const ", "").replace("class ", "").replace("struct ", "")
    stripped = stripped.rstrip("& ").split("<", 1)[0].strip()
    return bool(owner) and stripped.split("::")[-1] == owner and parameter.rstrip().endswith("&")


def _plain_category(declaration: Mapping[str, object]) -> str:
    """Category of hand-written code by declaration kind and access."""
    kind = declaration.get("kind")
    if kind == "EnumConstant":
        return ENUM_CONSTANT
    if kind == "Field":
        return DATA_FIELD
    if kind in _ALIAS_KINDS:
        return TYPE_ALIAS
    if kind in _RECORD_KINDS:
        return RECORD
    if declaration.get("access") == "protected":
        return PROTECTED_MEMBER
    return MEMBER


def _qualified(declaration: Mapping[str, object]) -> str:
    return str(declaration.get("qualified_name", ""))


def _member_name(declaration: Mapping[str, object]) -> str:
    return _qualified(declaration).rsplit("::", 1)[-1]


def _owner_name(declaration: Mapping[str, object]) -> str:
    qualified = _qualified(declaration)
    return qualified.rsplit("::", 2)[-2] if "::" in qualified else ""


def _spelling_header(declaration: Mapping[str, object]) -> str:
    location = declaration.get("spelling_location")
    if not isinstance(location, Mapping):
        return ""
    return str(location.get("file", "")).rsplit("/", 1)[-1]


def _parameter_types(declaration: Mapping[str, object]) -> list[str]:
    parameters = declaration.get("parameter_types")
    return [str(value) for value in parameters] if isinstance(parameters, list) else []
