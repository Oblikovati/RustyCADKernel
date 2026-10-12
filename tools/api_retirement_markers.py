"""Classify explicit OCCT retirement labels without mistaking obsolete parameters for APIs."""

from __future__ import annotations

import re
from collections.abc import Mapping, Sequence

_API_NOUN = r"(?:class|method|function|type|alias|field|enumerator|constructor|API)"
_STATUS = r"(?P<status>deprecated|obsolete)"
_LABELS = (
    re.compile(r"^@name\s+(?P<status>deprecated|obsolete)\b", re.IGNORECASE),
    re.compile(r"^@name.*\((?P<status>deprecated|obsolete)\)", re.IGNORECASE),
    re.compile(
        r"\b(?:This|The)\s+" + _API_NOUN + r"\s+is\s+" + _STATUS + r"\b", re.IGNORECASE
    ),
    re.compile(
        r'Standard_DEPRECATED\(".*?(?P<status>deprecated|obsolete)', re.IGNORECASE
    ),
    re.compile(r"^(?:@|\\)(?P<status>deprecated)\b", re.IGNORECASE),
    re.compile(r"^" + _STATUS + r"\b", re.IGNORECASE),
    re.compile(
        r"^(?:this|the)\s+is\s+(?:an?\s+)?" + _STATUS + r"\s+" + _API_NOUN,
        re.IGNORECASE,
    ),
    re.compile(
        r"^(?:this|the)\s+" + _API_NOUN + r"\s+(?:is|has been)\s+(?:now\s+)?" + _STATUS,
        re.IGNORECASE,
    ),
    re.compile(r"^an?\s+" + _STATUS + r"\s+" + _API_NOUN, re.IGNORECASE),
)
_ALIAS_GROUP = re.compile(
    r"^\s*//[!/]?\s*(?:@name\s+)?(obsolete|deprecated|active)\s+aliases\b",
    re.IGNORECASE,
)


def enum_alias_group(
    lines: Sequence[str], enum_begin: int, member_line: int
) -> tuple[int, str, str] | None:
    """Read an explicit group within compiler-proven enum bounds; e.g. enum_alias_group(lines, 1, 5)."""
    if not 1 <= enum_begin <= member_line <= len(lines):
        raise ValueError(
            f"enum/member lines={(enum_begin, member_line)!r}; expected 1 <= enum <= member <= {len(lines)}"
        )
    selected: tuple[int, str, str] | None = None
    for number in range(enum_begin, member_line):
        text = lines[number - 1]
        match = _ALIAS_GROUP.match(text)
        if match:
            status = match.group(1).lower()
            selected = None if status == "active" else (number, status, text)
        if "@}" in text:
            selected = None
    return selected


def comment_status_label(text: str, declaration_kind: str) -> str | None:
    """Recognize a label attached to an API; e.g. comment_status_label('//! Obsolete class.', 'CXXRecord')."""
    for line in text.splitlines():
        cleaned = line.strip().lstrip("/*!< ").rstrip("*/ ")
        if declaration_kind not in {"EnumConstant", "Field"} and re.match(
            r"^(?:obsolete|deprecated)\s+(?:argument|parameter|flag|cache|implementation|return)\b",
            cleaned,
            re.IGNORECASE,
        ):
            continue
        label = explicit_line_status(cleaned)
        if label:
            return label
        if declaration_kind in {"EnumConstant", "Field"}:
            match = re.search(
                r"\((deprecated|obsolete)(?:,\s*to be removed)?\)\.?$",
                cleaned,
                re.IGNORECASE,
            )
            if match:
                return match.group(1).lower()
    return None


def explicit_line_status(cleaned: str) -> str | None:
    """Recognize an API status sentence; e.g. explicit_line_status('Obsolete class.')."""
    for pattern in _LABELS:
        match = pattern.search(cleaned)
        if match:
            return match.group("status").lower()
    return None


def upstream_marker_status(
    marker: Mapping[str, object], declaration_kind: str
) -> str | None:
    """Use formal attributes or explicit documentation; e.g. upstream_marker_status({'kind': 'deprecated_attribute'}, 'CXXMethod')."""
    marker_kind = marker.get("kind")
    if marker_kind == "deprecated_attribute":
        return "deprecated"
    if marker_kind != "documentation_candidate":
        return None
    text = marker.get("text")
    if not isinstance(text, str):
        raise ValueError(f"retirement documentation text={text!r}; expected a string")
    return comment_status_label(text, declaration_kind)
