"""Separate spelling-only inventory conflicts from conflicts that change a declaration.

Clang prints a typedef's target type once per translation unit; where a class template
has default arguments, one unit prints `math_VectorBase<>` and another prints
`math_VectorBase<double>`. Both name the same type, so the declaration is stable. A
conflict is spelling-only when expanding every empty argument list `T<>` with the
defaults that T's ClassTemplate declaration records turns one spelling into the other.
Every other conflict still withholds the declaration from the crosswalk.
"""

from __future__ import annotations

import re

from api_inventory.inputs import JSONValue

SIGNATURE_FIELDS = frozenset({"signature", "identity_signature"})
EMPTY_ARGUMENTS = re.compile(r"(?<![\w:])((?:[A-Za-z_]\w*::)*[A-Za-z_]\w*)<>")


def template_defaults(
    declarations: list[dict[str, JSONValue]],
) -> dict[str, str]:
    """Default argument list per class template whose parameters all have defaults.

    Example: a `math_VectorBase` with `template <typename P = double> ` maps to `double`.
    """
    defaults: dict[str, str] = {}
    for row in declarations:
        name, header = row.get("qualified_name"), row.get("template_signature")
        if row.get("kind") != "ClassTemplate" or not isinstance(name, str):
            continue
        values = all_default_arguments(header) if isinstance(header, str) else None
        if values is not None:
            defaults[name] = ", ".join(values)
    return defaults


def all_default_arguments(header: str) -> list[str] | None:
    """Defaults of `template <typename A = int, int N = 2>`, or None if one is missing."""
    match = re.fullmatch(r"\s*template\s*<(.*)>\s*", header)
    if match is None or not match.group(1).strip():
        return None
    values = []
    for parameter in top_level_split(match.group(1)):
        _name, separator, value = parameter.partition("=")
        if not separator or not value.strip():
            return None
        values.append(value.strip())
    return values


def top_level_split(text: str) -> list[str]:
    """Split on commas outside angle brackets and parentheses."""
    parts, depth, start = [], 0, 0
    for index, char in enumerate(text):
        depth += (char in "<(") - (char in ">)")
        if char == "," and depth == 0:
            parts.append(text[start:index])
            start = index + 1
    return [*parts, text[start:]]


def expand_default_arguments(text: str, defaults: dict[str, str]) -> str:
    """Spell `T<>` as `T<defaults>`; e.g. `class math_VectorBase<>` -> `...<double>`."""

    def expanded(match: re.Match[str]) -> str:
        name = match.group(1)
        return f"{name}<{defaults[name]}>" if name in defaults else match.group(0)

    return EMPTY_ARGUMENTS.sub(expanded, text)


def spelling_only(conflict: dict[str, JSONValue], defaults: dict[str, str]) -> bool:
    """True when a signature conflict differs only by defaulted template arguments."""
    first, other = conflict.get("first"), conflict.get("other")
    if conflict.get("field") not in SIGNATURE_FIELDS:
        return False
    if not isinstance(first, str) or not isinstance(other, str) or first == other:
        return False
    return expand_default_arguments(first, defaults) == expand_default_arguments(other, defaults)


def material_conflicts(
    conflicts: list[dict[str, JSONValue]], declarations: list[dict[str, JSONValue]]
) -> list[dict[str, JSONValue]]:
    """Conflicts that withhold a declaration: an id stays conflicted if any record is material.

    Example: `mark_conflicted_declarations(cpp, material_conflicts(conflicts, cpp))`.
    """
    defaults = template_defaults(declarations)
    material_ids = {
        row.get("id") for row in conflicts if not spelling_only(row, defaults)
    }
    return [row for row in conflicts if row.get("id") in material_ids]
