"""Compiler-backed selector rules for C++ class templates and member qualifiers."""

from __future__ import annotations

import re

from api_identity import leaf_name, norm_cpp, parse_upstream_id
from api_inventory.inputs import JSONValue

QUALIFIERS = {"const", "volatile", "&", "&&", "noexcept"}
CVREF = {"const", "volatile", "&", "&&"}


def constructor_candidates(
    name: str,
    candidates: list[dict[str, JSONValue]],
    by_name: dict[str, list[dict[str, JSONValue]]],
) -> list[dict[str, JSONValue]]:
    if exact_candidates(name, "CXXRecord", candidates):
        return constructors_for_record(name, candidates, by_name)
    templates = exact_candidates(name, "ClassTemplate", candidates)
    if templates:
        return constructors_for_template(templates, by_name)
    owner = name.rsplit("::", 1)[0] if "::" in name else name
    if leaf_name(name) != leaf_name(owner):
        return candidates
    owner_templates = exact_candidates(owner, "ClassTemplate", by_name.get(owner, []))
    if owner_templates:
        return constructors_for_template(owner_templates, by_name)
    return constructors_for_record(name, candidates, by_name)


def exact_candidates(
    name: str, kind: str, candidates: list[dict[str, JSONValue]]
) -> list[dict[str, JSONValue]]:
    return [
        row
        for row in candidates
        if row.get("kind") == kind and row.get("qualified_name") == name
    ]


def constructors_for_template(
    templates: list[dict[str, JSONValue]],
    by_name: dict[str, list[dict[str, JSONValue]]],
) -> list[dict[str, JSONValue]]:
    owners = {
        row.get("usr"): row.get("qualified_name")
        for row in templates
        if isinstance(row.get("usr"), str) and isinstance(row.get("qualified_name"), str)
    }
    return [
        row
        for declarations in by_name.values()
        for row in declarations
        if template_constructor(row, owners)
    ]


def template_constructor(
    row: dict[str, JSONValue], owners: dict[str, JSONValue]
) -> bool:
    parent = row.get("parent_usr")
    owner = owners.get(parent) if isinstance(parent, str) else None
    name = row.get("qualified_name")
    if not isinstance(owner, str) or not isinstance(name, str):
        return False
    ctor_name = f"{owner}::{leaf_name(owner)}"
    return row.get("kind") in {"CXXConstructor", "FunctionTemplate"} and (
        name == ctor_name or name.startswith(f"{ctor_name}<")
    )


def constructors_for_record(
    name: str,
    candidates: list[dict[str, JSONValue]],
    by_name: dict[str, list[dict[str, JSONValue]]],
) -> list[dict[str, JSONValue]]:
    records = [row for row in candidates if row.get("kind") == "CXXRecord"]
    if not any(row.get("qualified_name") == name for row in records):
        return candidates
    constructors = by_name.get(f"{name}::{leaf_name(name)}", [])
    return [row for row in candidates if row not in records] + [
        row for row in constructors if row.get("kind") == "CXXConstructor"
    ]


def declaration_template_owner(
    declaration: dict[str, JSONValue],
    by_name: dict[str, list[dict[str, JSONValue]]],
) -> str | None:
    name = declaration.get("qualified_name")
    parent = declaration.get("parent_usr")
    if not isinstance(name, str) or not isinstance(parent, str):
        return None
    owner = name.rsplit("::", 1)[0]
    return owner if template_usr(parent, by_name.get(owner, [])) else None


def template_usr(parent: str, declarations: list[dict[str, JSONValue]]) -> bool:
    return any(
        row.get("kind") == "ClassTemplate" and row.get("usr") == parent
        for row in declarations
    )


def injected_template_parameter(
    expected: str | None, actual: str | None, owner: str
) -> bool:
    if expected is None or actual is None:
        return False
    self_type = f"{owner}<type-parameter-0-0>"
    if self_type not in actual:
        return False
    pattern = rf"(?<![\w:]){re.escape(owner)}(?![\w:<>])"
    expanded = re.sub(pattern, self_type, expected)
    return expanded == actual


def suffix_matches(identity_suffix: str, declaration: dict[str, JSONValue]) -> bool:
    if not identity_suffix:
        return True
    signature = declaration.get("signature")
    if not isinstance(signature, str):
        return False
    wanted = qualifier_tokens(identity_suffix)
    actual = signature_qualifiers(signature)
    if wanted is None or actual is None:
        return norm_cpp(signature).endswith(identity_suffix)
    return qualifier_match(wanted, actual)


def qualifier_tokens(suffix: str) -> set[str] | None:
    tokens = set(suffix.split())
    return tokens if tokens and tokens <= QUALIFIERS else None


def signature_qualifiers(signature: str) -> set[str] | None:
    parsed = parse_upstream_id(signature)
    return None if parsed is None else qualifier_tokens(parsed[2])


def qualifier_match(wanted: set[str], actual: set[str]) -> bool:
    if (wanted & CVREF) != (actual & CVREF):
        return False
    noexcept_matches = "noexcept" not in wanted or "noexcept" in actual
    return noexcept_matches


def prefer_nonconst_overload(
    candidates: list[dict[str, JSONValue]],
) -> list[dict[str, JSONValue]]:
    if len(candidates) != 2 or any(row.get("kind") != "CXXMethod" for row in candidates):
        return candidates
    if not same_overload_shape(candidates):
        return candidates
    qualifiers = [signature_qualifiers(row.get("signature", "")) for row in candidates]
    if not qualifier_pair(qualifiers):
        return candidates
    return [row for row, values in zip(candidates, qualifiers, strict=True) if "const" not in values]


def same_overload_shape(candidates: list[dict[str, JSONValue]]) -> bool:
    first, second = candidates
    return all(
        first.get(field) == second.get(field)
        for field in ("qualified_name", "parent_usr", "parameter_types", "written_parameter_types")
    )


def qualifier_pair(qualifiers: list[set[str] | None]) -> bool:
    if len(qualifiers) != 2 or any(value is None for value in qualifiers):
        return False
    first, second = qualifiers
    same_except_const = first != second and (first ^ second) == {"const"}
    return same_except_const


def template_parameter_match(
    expected: list[str | None], actual: list[str | None], owner: str
) -> bool:
    if len(expected) != len(actual):
        return False
    return all(
        left == right or injected_template_parameter(left, right, owner)
        for left, right in zip(expected, actual, strict=True)
    )
