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
    match = re.match(r"\s*(<=>|<<=|>>=|<=|>=|<<|>>|<|>)", suffix)
    if match is None:
        return None
    symbol_start = start + match.start(1)
    return symbol_start, symbol_start + len(match.group(1))


def signature_open(text: str) -> int:
    offset = text.find("operator()")
    if offset >= 0:
        return text.find("(", offset + len("operator()"))
    symbol_span = operator_angle_span(text)
    angle_depth = 0
    for index, char in enumerate(text):
        if symbol_span and symbol_span[0] <= index < symbol_span[1]:
            continue
        angle_depth += (char == "<") - (char == ">")
        if char == "(" and angle_depth == 0:
            return index
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
