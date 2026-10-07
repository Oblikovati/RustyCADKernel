"""Exact parsing helpers for the hand-authored C++ API ID format."""

from __future__ import annotations

import re


def norm_cpp(text: str) -> str:
    compact = re.sub(r"\s+", " ", text.strip())
    return re.sub(r"\s*([*&,:<>\[\]()])\s*", r"\1", compact)


def split_parameters(text: str) -> list[str]:
    parts, depths, start = [], {"<": 0, "(": 0, "[": 0, "{": 0}, 0
    closing = {">": "<", ")": "(", "]": "[", "}": "{"}
    for index, char in enumerate(text):
        if char in depths:
            depths[char] += 1
        elif char in closing and depths[closing[char]]:
            depths[closing[char]] -= 1
        elif char == "," and not any(depths.values()):
            parts.append(norm_cpp(text[start:index]))
            start = index + 1
    final = norm_cpp(text[start:])
    return [*parts, final] if final else parts


def operator_angle_span(text: str) -> tuple[int, int] | None:
    offset = text.rfind("operator")
    if offset < 0:
        return None
    start = offset + len("operator")
    suffix = text[start:]
    # `->` and `->*` contain `>` but are operator symbols, not template brackets.
    match = re.match(r"\s*(->\*|->|<=>|<<=|>>=|<=|>=|<<|>>|<|>)", suffix)
    if match is None:
        return None
    symbol_start = start + match.start(1)
    return symbol_start, symbol_start + len(match.group(1))


# Clang spells scopes without a name as parenthesized words, e.g.
# `(anonymous namespace)::Helper` or `(unnamed enum at src/X/X.hxx:17:1)`.
UNNAMED_SCOPE = re.compile(
    r"\((?:anonymous namespace|anonymous|unnamed (?:enum|struct|union|class)(?: at [^()]*)?)\)"
)


def unnamed_scope_end(text: str, index: int) -> int | None:
    """End offset of an unnamed-scope token that starts a name segment at `index`.

    Example: `unnamed_scope_end("(anonymous namespace)::F(int)", 0) == 21`.
    """
    if index > 0 and not text[:index].endswith("::"):
        return None
    match = UNNAMED_SCOPE.match(text, index)
    if match is None:
        return None
    rest = text[match.end() :]
    return match.end() if rest == "" or rest.startswith("::") else None


def signature_open(text: str) -> int:
    offset = text.find("operator()")
    if offset >= 0:
        return text.find("(", offset + len("operator()"))
    symbol_span = operator_angle_span(text)
    angle_depth, index = 0, 0
    while index < len(text):
        char = text[index]
        skipped = unnamed_scope_end(text, index) if char == "(" else None
        if skipped is not None:
            index = skipped
            continue
        if not (symbol_span and symbol_span[0] <= index < symbol_span[1]):
            angle_depth += (char == "<") - (char == ">")
            if char == "(" and angle_depth == 0:
                return index
        index += 1
    return -1


def matching_paren(text: str, opening: int) -> int:
    depth = 0
    for index in range(opening, len(text)):
        depth += (text[index] == "(") - (text[index] == ")")
        if depth == 0:
            return index
    return -1


def parse_upstream_id(identity: str) -> tuple[str, list[str] | None, str] | None:
    text = identity.removesuffix(" (definition)").strip()
    opening = signature_open(text)
    if opening < 0:
        return text, None, ""
    closing = matching_paren(text, opening)
    if closing < 0:
        return None
    return (
        text[:opening].strip(),
        split_parameters(text[opening + 1 : closing]),
        norm_cpp(text[closing + 1 :]),
    )


def leaf_name(qualified_name: str) -> str:
    return qualified_name.rsplit("::", 1)[-1]


UNNAMED_LOCATION = re.compile(r"\((unnamed (?:enum|struct|union|class) at )([^():]*)(:[0-9]+:[0-9]+)\)")


def portable_qualified_name(name: str, source: object) -> str:
    """Spell an unnamed declaration's location relative to the OCCT tree.

    Clang names an unnamed enum by the absolute path of the checkout that captured it, so
    a ledger row could never name it on another machine. Example:
    `portable_qualified_name("(unnamed enum at /o/src/A/B.hxx:3:1)", "src/A/B.hxx")`
    returns `"(unnamed enum at src/A/B.hxx:3:1)"`.
    """
    if not isinstance(source, str) or not source:
        return name

    def relative(match: re.Match[str]) -> str:
        path = match.group(2)
        portable = source if path.endswith("/" + source) else path
        return f"({match.group(1)}{portable}{match.group(3)})"

    return UNNAMED_LOCATION.sub(relative, name)
