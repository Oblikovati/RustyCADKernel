"""Recognize macro-generated and special C++ members and spell their ledger identities.

The OCCT compiler inventory records only user-declared members: implicit copy, move and
destructor members never appear, so nothing here has to match compiler-implicit records.
Macro members are recognized by `macro_generated` plus the header that spells them, never
by name alone, so a hand-written `DynamicType` override stays a toolkit member.
"""

from __future__ import annotations

import re

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
EXCEPTION_HEADER = "Standard_DefineException.hxx"
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


def special_member_kind(declaration: JSONObject) -> str | None:
    """Classify one declaration; the single category rule of this generator.

    Example: `special_member_kind(geom2d_line_get_type_name) == 'rtti_name'`.
    """
    if declaration.get("macro_generated") is True:
        return macro_member_kind(declaration)
    if declaration.get("kind") == "CXXDestructor":
        return "destructor"
    return copy_move_kind(declaration)


def macro_member_kind(declaration: JSONObject) -> str | None:
    header = spelling_header(declaration)
    leaf = leaf_name(str(declaration.get("qualified_name", "")))
    if header in ALLOCATION_HEADERS and leaf.startswith("operator "):
        return "allocation"
    if header == RTTI_HEADER and declaration.get("kind") == "CXXMethod":
        return RTTI_KINDS.get(leaf)
    if header == EXCEPTION_HEADER and leaf == "ExceptionType":
        return "exception_type"
    if declaration.get("kind") == "CXXConstructor" and exception_constructor(declaration):
        return "exception_constructor"
    return None


def exception_constructor(declaration: JSONObject) -> bool:
    """`DEFINE_STANDARD_EXCEPTION` constructors take one or two `const char *`."""
    parameters = declaration.get("parameter_types")
    return parameters in (["const char *"], ["const char *", "const char *"])


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
    if declaration.get("kind") == "CXXRecord":
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
