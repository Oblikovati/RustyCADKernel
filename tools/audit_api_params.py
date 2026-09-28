#!/usr/bin/env python3
"""C++ parameter-list normalization shared by the api-row audit (`audit_api_rows.py`).

A parameter is reduced to its type as the ledger spells it: names, default values and
by-value top-level `const` are dropped, OCCT scalar aliases become builtins, arrays decay
like C++ adjusts them, and a C variadic ellipsis stays `...`.
"""
from __future__ import annotations

import re

PARAM_TOKEN = re.compile(r"[A-Za-z_]\w*|[&*]+|\[[^\]]*\]")
TYPE_ALIASES = {"Standard_Real": "double", "Standard_Integer": "int", "Standard_Boolean": "bool"}


def matching_close(text: str, start: int) -> int:
    """Index of the bracket closing the one at `start` (`(`, `{`, `[` or `<`).

    >>> matching_close('(a(b)c)', 0)
    6
    """
    pairs = {"(": ")", "{": "}", "[": "]", "<": ">"}
    opening, depth = text[start], 0
    for index in range(start, len(text)):
        depth += (text[index] == opening) - (text[index] == pairs[opening])
        if depth == 0:
            return index
    raise ValueError(f"unbalanced {opening!r} at offset {start}; expected a matching {pairs[opening]!r}")


def split_top_level(text: str) -> list[str]:
    """Split on commas that are not nested in (), <>, {} or []."""
    parts, depth, start = [], 0, 0
    for index, char in enumerate(text):
        depth += (char in "(<{[") - (char in ")>}]")
        if char == "," and depth == 0:
            parts.append(text[start:index])
            start = index + 1
    parts.append(text[start:])
    return [part for part in parts if part.strip()]


def normalize_param(param: str) -> str:
    """Drop the default value, the parameter name and by-value top-level `const`.

    >>> normalize_param('const gp_Vec& theV = gp_Vec()')
    'const gp_Vec&'
    >>> normalize_param('const double theX')
    'double'
    """
    param = split_default(param).strip()
    if param == "...":
        return param  # a C variadic tail (`Standard_Dump::DumpRealValues`) is part of the signature
    if (array := array_param(param)) is not None:
        return array
    tokens = [token for token in type_tokens(param) if token != "typename"]
    words = [token for token in tokens if re.match(r"\w", token)]
    type_words = [word for word in words if word not in {"const", "volatile", "unsigned", "signed"}]
    if len(type_words) >= 2 and tokens[-1] == words[-1] and not is_builtin_pair(type_words):
        tokens = tokens[:-1]
    tokens = [TYPE_ALIASES.get(token, token) for token in tokens]
    if tokens and tokens[0] == "const" and not any(token[0] in "&*" for token in tokens):
        tokens = tokens[1:]
    return re.sub(r" ([&*])", r"\1", " ".join(tokens))


def is_builtin_pair(type_words: list[str]) -> bool:
    """True when the last two words spell one builtin type (`long double`), not `Type name`.

    >>> is_builtin_pair(["long", "double"])
    True
    """
    return type_words[-2] in {"long", "short"} and type_words[-1] in {"long", "int", "double"}


def array_param(param: str) -> str | None:
    """Canonical spelling of an array or reference-to-array parameter, else None.

    Arrays keep their `const` (it is not top-level); a by-value array decays to a pointer
    as in C++, while a reference or pointer to an array keeps its extents.

    >>> array_param("const double theJ[3][3]")
    'const double(*)[3]'
    >>> array_param("const double (&theF)[N]")
    'const double(&)[N]'
    >>> array_param("const double[9]")
    'const double*'
    """
    match = re.fullmatch(r"(.*?)(\(\s*[&*]+\s*\w*\s*\)|\b\w+)?\s*((?:\[[^\]]*\]\s*)+)", param, re.S)
    if match is None:
        return None
    base, declarator = match.group(1), match.group(2) or ""
    if not set(base.split()) - {"const", "volatile", "unsigned", "signed"} and not declarator.startswith("("):
        base, declarator = f"{base} {declarator}", ""  # nameless `const double[9]`: the word is the type
    base = " ".join(TYPE_ALIASES.get(word, word) for word in base.split())
    extents = re.findall(r"\[[^\]]*\]", match.group(3))
    declarator = re.sub(r"\w+|\s+", "", declarator)
    if declarator:  # a reference or pointer to an array keeps every extent
        return base + declarator + "".join(re.sub(r"\s+", "", extent) for extent in extents)
    # C++ adjusts an array parameter to a pointer: `T p[8]` is `T*`, `T p[2][3]` is `T(*)[3]`.
    rest = "".join(re.sub(r"\s+", "", extent) for extent in extents[1:])
    return base + ("(*)" + rest if rest else "*")


def type_tokens(param: str) -> list[str]:
    """Split a parameter into qualified names, `&`/`*` runs and array extents.

    Template argument lists are balanced, so `handle<B<T, N>>` and the dependent
    `typename B<T, N>::V` each stay one token.

    >>> type_tokens('const typename BVH_Box<T, N>::BVH_VecNt& theV')
    ['const', 'typename', 'BVH_Box<T,N>::BVH_VecNt', '&', 'theV']
    """
    tokens: list[str] = []
    index = 0
    while (match := PARAM_TOKEN.search(param, index)) is not None:
        token, index = match.group(0), match.end()
        if token[0] not in "&*[":
            token, index = extend_qualified(param, token, index)
        tokens.append(squeeze(token))
    return tokens


def extend_qualified(param: str, token: str, index: int) -> tuple[str, int]:
    """Append balanced `<...>` argument lists and `::Name` parts that follow `token`."""
    while (step := re.compile(r"\s*(<|::\s*~?[A-Za-z_]\w*)").match(param, index)) is not None:
        if step.group(1) != "<":
            token, index = token + step.group(1), step.end()
            continue
        close = matching_close(param, step.end() - 1)
        token, index = token + param[step.end() - 1:close + 1], close + 1
    return token, index


def squeeze(text: str) -> str:
    """Drop whitespace except the single space separating two words.

    >>> squeeze('NCollection_Vec2 < unsigned  int >')
    'NCollection_Vec2<unsigned int>'
    """
    return re.sub(r" ?([^\w ]) ?", r"\1", re.sub(r"\s+", " ", text.strip()))


def split_default(param: str) -> str:
    """Return the parameter text before a top-level `=` default value."""
    depth = 0
    for index, char in enumerate(param):
        depth += (char in "(<{[") - (char in ")>}]")
        if char == "=" and depth == 0:
            return param[:index]
    return param


def normalize_params(params: str) -> str:
    """Canonical comma-joined parameter list (`void` means empty)."""
    parts = [normalize_param(part) for part in split_top_level(params)]
    return "" if parts == ["void"] else ",".join(parts)
