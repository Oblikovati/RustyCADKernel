"""Read how an OCCT special member is defined: deleted, defaulted, empty, or a real body.

The compiler inventory has no `= delete` / `= default` flag, so the pinned OCCT source at
the declaration's spelling location is the evidence. Reading goes through an injected
reader so tests use a named fake instead of the file system.
"""

from __future__ import annotations

from pathlib import Path
import re
from typing import Protocol

from api_inventory.inputs import JSONObject
from api_identity import leaf_name

DELETED = "deleted"
DEFAULTED = "defaulted"
EMPTY_BODY = "empty_body"
USER_BODY = "user_body"
UNKNOWN = "unknown"

COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.DOTALL)


class SourceTextReader(Protocol):
    def text(self, relative: str) -> str | None: ...

    def sibling_sources(self, relative: str) -> list[str]: ...


class OcctSourceReader:
    """Read pinned OCCT files; `OcctSourceReader(Path('/src/OCCT')).text('src/a.hxx')`."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.cache: dict[str, str | None] = {}

    def text(self, relative: str) -> str | None:
        if relative not in self.cache:
            path = self.root / relative
            self.cache[relative] = (
                path.read_text(encoding="latin-1") if path.is_file() else None
            )
        return self.cache[relative]

    def sibling_sources(self, relative: str) -> list[str]:
        folder = (self.root / relative).parent
        names = sorted(path.name for path in folder.glob("*.cxx"))
        prefix = relative.rsplit("/", 1)[0]
        return [f"{prefix}/{name}" for name in names]


def declaration_text(reader: SourceTextReader, declaration: JSONObject) -> str | None:
    """Source text from the declaration's spelling point to the end of its file."""
    location = declaration.get("spelling_location")
    if not isinstance(location, dict) or not isinstance(location.get("file"), str):
        return None
    text = reader.text(location["file"])
    line, column = location.get("line"), location.get("column")
    if text is None or not isinstance(line, int) or not isinstance(column, int):
        return None
    lines = text.split("\n")[line - 1 :]
    return COMMENT.sub(" ", "\n".join(lines)[column - 1 :])


def definition_form(reader: SourceTextReader, declaration: JSONObject) -> str:
    """Classify a special member's definition; e.g. `definition_form(r, copy) == 'deleted'`."""
    text = declaration_text(reader, declaration)
    if text is None:
        return UNKNOWN
    close = parameter_close(text)
    if close < 0:
        return UNKNOWN
    rest = text[close + 1 :]
    marker = re.match(r"[^;{=:]*?(=\s*delete|=\s*default|;|\{|:)", rest, re.DOTALL)
    if marker is None:
        return UNKNOWN
    return form_after_marker(reader, declaration, rest, marker)


def form_after_marker(
    reader: SourceTextReader, declaration: JSONObject, rest: str, marker: re.Match[str]
) -> str:
    token = re.sub(r"\s+", "", marker.group(1))
    if token in {"=delete", "=default"}:
        return DELETED if token == "=delete" else DEFAULTED
    if token == ";":
        return out_of_line_form(reader, declaration)
    if token == ":":
        return USER_BODY
    return body_form(rest[marker.end() - 1 :])


def parameter_close(text: str) -> int:
    """Index of the `)` closing the first parameter list, honoring nesting."""
    opening = text.find("(")
    depth = 0
    for index in range(max(opening, 0), len(text)):
        depth += (text[index] == "(") - (text[index] == ")")
        if depth == 0 and opening >= 0:
            return index
    return -1


def body_form(text: str) -> str:
    """`{}` (comments removed) is an empty body; anything else is a user body."""
    inner = braced_body(text)
    if inner is None:
        return UNKNOWN
    return EMPTY_BODY if not inner.strip() else USER_BODY


def braced_body(text: str) -> str | None:
    """Contents of the first balanced `{...}` block of `text`, or None."""
    start = text.find("{")
    depth = 0
    for index in range(max(start, 0), len(text)):
        depth += (text[index] == "{") - (text[index] == "}")
        if depth == 0 and start >= 0:
            return text[start + 1 : index]
    return None


def out_of_line_form(reader: SourceTextReader, declaration: JSONObject) -> str:
    """Find `Class::~Class()` (or the member) in the package's .cxx files."""
    name = str(declaration.get("qualified_name", ""))
    owner, member = name.rsplit("::", 1) if "::" in name else ("", name)
    pattern = re.compile(
        rf"\b{re.escape(leaf_name(owner))}::{re.escape(member)}\s*\(\s*\)\s*(?:noexcept\s*)?(\{{|:)"
    )
    for relative in reader.sibling_sources(source_path(declaration)):
        text = COMMENT.sub(" ", reader.text(relative) or "")
        match = pattern.search(text)
        if match:
            return USER_BODY if match.group(1) == ":" else body_form(text[match.end() - 1 :])
    return UNKNOWN


def source_path(declaration: JSONObject) -> str:
    location = declaration.get("spelling_location")
    path = location.get("file") if isinstance(location, dict) else None
    return path if isinstance(path, str) else str(declaration.get("source", ""))
