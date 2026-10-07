"""Recognize macro-generated and special C++ members and spell their ledger identities.

The OCCT compiler inventory records only user-declared members: implicit copy, move and
destructor members never appear, so nothing here has to match compiler-implicit records.
Categories come from the shared `api_gap_categories` classifier (macro members by
`macro_generated` plus spelling header, never by name alone).
"""

from __future__ import annotations

import re

from api_gap_categories import (
    ALLOC_MACRO,
    EXCEPTION_MACRO,
    RTTI_MACRO,
    SPECIAL_MEMBER,
    declaration_category,
)
from api_identity import norm_cpp, parse_upstream_id, leaf_name
from api_inventory.inputs import JSONObject

ALLOCATION_HEADERS = {
    "Standard_DefineAlloc.hxx": "DEFINE_STANDARD_ALLOC",
    "NCollection_DefineAlloc.hxx": "DEFINE_NCOLLECTION_ALLOC",
    "IMeshData_Types.hxx": "DEFINE_INC_ALLOC",
}
RTTI_KINDS = {
    "get_type_name": "rtti_name",
    "get_type_descriptor": "rtti_descriptor",
    "DynamicType": "rtti_dynamic",
}
RTTI_HEADER = "Standard_Type.hxx"


def declaration_owner(declaration: JSONObject) -> str:
    """The enclosing C++ class; `declaration_owner({'qualified_name': 'A::B'}) == 'A'`."""
    name = str(declaration.get("qualified_name", ""))
    return name.rsplit("::", 1)[0] if "::" in name else ""


def spelling_header(declaration: JSONObject) -> str:
    """The file name that spells the declaration, e.g. `Standard_Type.hxx` for RTTI."""
    location = declaration.get("spelling_location")
    path = location.get("file") if isinstance(location, dict) else None
    return path.rsplit("/", 1)[-1] if isinstance(path, str) else ""


def allocation_macro(declaration: JSONObject) -> str:
    """The allocation macro that spells an operator, e.g. `DEFINE_STANDARD_ALLOC`."""
    return ALLOCATION_HEADERS.get(spelling_header(declaration), "an OCCT allocation macro")


def special_member_kind(declaration: JSONObject) -> str | None:
    """Refine the shared gap category into this generator's row kind.

    `api_gap_categories.declaration_category` decides the category (one classifier for
    reports and generators); this only names the member inside it, or None when the
    generator has no rule for it (e.g. a converting `operator=`).
    Example: `special_member_kind(geom2d_line_get_type_name) == 'rtti_name'`.
    """
    category = declaration_category(declaration)
    leaf = leaf_name(str(declaration.get("qualified_name", "")))
    if category == ALLOC_MACRO:
        return "allocation"
    if category == RTTI_MACRO:
        return RTTI_KINDS.get(leaf, "rtti_base" if leaf == "base_type" else None)
    if category == EXCEPTION_MACRO:
        return "exception_type" if leaf == "ExceptionType" else exception_constructor_kind(declaration)
    if category != SPECIAL_MEMBER:
        return None
    return "destructor" if declaration.get("kind") == "CXXDestructor" else copy_move_kind(declaration)


def exception_constructor_kind(declaration: JSONObject) -> str | None:
    """`DEFINE_STANDARD_EXCEPTION` constructors take one or two `const char *`."""
    parameters = declaration.get("parameter_types")
    exception = parameters in (["const char *"], ["const char *", "const char *"])
    return "exception_constructor" if exception else None


def copy_move_kind(declaration: JSONObject) -> str | None:
    """Name copy/move constructors and assignments of the owner class, else None."""
    kind = declaration.get("kind")
    leaf = leaf_name(str(declaration.get("qualified_name", "")))
    if kind not in {"CXXConstructor", "CXXMethod"}:
        return None
    if kind == "CXXMethod" and leaf != "operator=":
        return None
    reference = self_reference(declaration)
    if reference is None:
        return None
    prefix = "move" if reference == "&&" else "copy"
    return f"{prefix}_{'constructor' if kind == 'CXXConstructor' else 'assignment'}"


def self_reference(declaration: JSONObject) -> str | None:
    """`&` or `&&` when the only parameter is a reference to the owner class itself."""
    parameters = declaration.get("parameter_types")
    if not isinstance(parameters, list) or len(parameters) != 1:
        return None
    owner = leaf_name(declaration_owner(declaration))
    pattern = rf"(?:const )?(?:class |struct )?(?:[\w:]+::)?{re.escape(owner)}(?:<.*>)? (&&?)"
    match = re.fullmatch(pattern, str(parameters[0]))
    return match.group(1) if match else None


def upstream_identity(declaration: JSONObject) -> str:
    """Spell the ledger id of a member, e.g. `Geom2d_Line::DynamicType() const`."""
    name = str(declaration.get("qualified_name", ""))
    if declaration.get("kind") in {"CXXRecord", "Typedef", "TypeAlias"}:
        return name
    parameters = declaration.get("written_parameter_types")
    if not isinstance(parameters, list):
        parameters = declaration.get("parameter_types") or []
    qualifiers = signature_suffix(str(declaration.get("signature", "")))
    text = f"{name}({','.join(norm_cpp(str(item)) for item in parameters)})"
    return f"{text} {qualifiers}" if qualifiers else text


def signature_suffix(signature: str) -> str:
    """Keep the cv/ref/noexcept qualifiers of a compiler signature; `'() const' -> 'const'`."""
    parsed = parse_upstream_id(signature)
    if parsed is None:
        return ""
    tokens = parsed[2].split()
    kept = [token for token in tokens if token in {"const", "volatile", "&", "&&"}]
    if "noexcept" in tokens:
        kept.append("noexcept")
    return " ".join(kept)
