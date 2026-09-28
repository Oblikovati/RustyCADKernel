#!/usr/bin/env python3
"""List public OCCT header members that lack an `api` row in porting/parity.json.

Usage:
    python3 tools/audit_api_rows.py src/FoundationClasses/TKMath/gp \
        --source /path/to/OCCT [--ledger porting/parity.json] [--members]

The parser is a heuristic C++ declaration scanner, not a compiler front end.
It strips comments, string literals, attributes and the OCCT allocation/export
macros, then walks namespace, class and struct scopes tracking access labels.
Every public member function, constructor, operator and nested enum becomes an
expected row; namespace-scope free functions (such as `operator*(double, gp_Vec)`)
are attributed to the header's primary class, which is the ledger convention.

Row identities follow the ledger: `Class(params)` for constructors,
`Class::Name` when the name is not overloaded, `Class::Name(params)[ const]`
when it is. Parameter lists are compared after dropping parameter names,
default values, top-level `const` on by-value parameters and whitespace.

Known false-positive sources and how they are handled:
- Out-of-class definitions (`inline gp_Vec::gp_Vec(...)`) are recognised by
  the qualified name and ignored; only in-class declarations count.
- `= delete` members, `friend` declarations, typedefs, `using` aliases, data
  members and protected/private members are never expected.
- A bare `Class::Name` row does not cover an overloaded name; overloads that
  differ only in cv-qualification must carry the trailing ` const`.
- Macros that expand to members are not expanded; an unknown macro that ends
  without a semicolon merges into the next declaration and may hide it. Add
  such macros to IGNORED_MACROS when a new package needs them.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IGNORED_MACROS = r"\b(?:DEFINE_STANDARD_ALLOC|DEFINE_NCOLLECTION_ALLOC|Standard_EXPORT|Standard_NODISCARD)\b"
TOKEN_NOISE = re.compile(r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\\n])*"|\'(?:\\.|[^\'\\\n])*\'', re.S)
ACCESS_LABEL = re.compile(r"^\s*(public|protected|private)\s*:(?!:)")
SCOPE_HEAD = re.compile(r"^(?:template\s*<[^{]*?>\s*)?(class|struct|union|namespace|enum(?:\s+class)?)\s+([\w:]+(?:\s*<[^{]*>)?)")
TYPE_ALIASES = {"Standard_Real": "double", "Standard_Integer": "int", "Standard_Boolean": "bool"}


@dataclass(frozen=True)
class Member:
    """One expected ledger identity: `prefix` is `Class` or `Class::Name`."""

    prefix: str
    params: str | None
    qualifier: str


def clean_source(text: str) -> str:
    """Remove comments, literals, preprocessor lines, attributes and ignored macros.

    >>> clean_source('int a; // x\\n#define B 1\\n[[nodiscard]] int b();')
    'int a;  \\n\\n int b();'
    """
    text = TOKEN_NOISE.sub(lambda m: '""' if m.group(0)[0] in "\"'" else " ", text)
    text = re.sub(r"^\s*#(?:[^\n]*\\\n)*[^\n]*$", "", text, flags=re.M)
    text = re.sub(r"\[\[[^\]]*\]\]|__attribute__\s*\(\([^)]*\)\)", "", text)
    text = re.sub(r"\bStandard_DEPRECATED\s*\(\s*\"*\s*\)", "", text)
    # DEFINE_STANDARD_RTTIEXT/_INLINE expand to `public:` plus the RTTI members. Without a
    # semicolon the macro swallowed the next access label, which hid every public member
    # of a class that opens with it (Adaptor2d_Curve2d). The expansion keeps one public
    # `Class::DEFINE_STANDARD_RTTIEXT` row (the ledger's RTTI convention) and ends public.
    text = re.sub(r"\b(DEFINE_STANDARD_RTTI(?:EXT|_INLINE))\s*(\([^)]*\))", r"public: \1\2; public:", text)
    return re.sub(IGNORED_MACROS, "", text)


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


def is_brace_initializer(head: str) -> bool:
    """True when a `{` opened after `head` initializes a value instead of opening a body."""
    stripped = head.rstrip()
    if stripped.endswith("=") or stripped.endswith(","):
        return True
    if re.search(r"\w$", stripped) is None:
        return False
    # Qualifiers are an alternation of fixed words, not `(\w+\s*)*`: the nested quantifier
    # backtracked exponentially on long NCollection template heads.
    qualifiers = r"(?:\s|\b(?:noexcept|const|override|final|volatile|constexpr)\b)*"
    return re.search(r"\)" + qualifiers + r":(?!:)", stripped) is not None


def split_statements(body: str) -> list[tuple[str, str | None]]:
    """Split a scope into `(head, braced_body)` statements at `;` or closing braces."""
    statements: list[tuple[str, str | None]] = []
    start, index = 0, 0
    while index < len(body):
        char = body[index]
        if char == ";":
            statements.append((body[start:index], None))
            start = index + 1
        elif char == "{":
            index, start = consume_brace(body, start, index, statements)
        index += 1
    return statements


def consume_brace(body: str, start: int, index: int, out: list[tuple[str, str | None]]) -> tuple[int, int]:
    """Handle a `{` inside `split_statements`; returns the new scan index and statement start."""
    close = matching_close(body, index)
    head = body[start:index]
    if is_brace_initializer(head):
        return close, start
    out.append((head, body[index + 1:close]))
    bare = strip_access(head)[1].strip()
    if SCOPE_HEAD.match(bare) and not bare.startswith("namespace"):
        end = body.find(";", close)  # class/enum definitions end at their trailing semicolon
        return end, end + 1
    return close, close + 1


def strip_access(head: str) -> tuple[str | None, str]:
    """Pop leading access labels; returns the last label seen and the remaining text."""
    label = None
    while (match := ACCESS_LABEL.match(head)) is not None:
        label, head = match.group(1), head[match.end():]
    return label, head


def scan_scope(body: str, scope: str, public_default: bool, primary: str) -> list[Member]:
    """Collect expected members declared in one namespace or class body."""
    members: list[Member] = []
    public = public_default
    for head, braced in split_statements(body):
        label, head = strip_access(head)
        public = public if label is None else label == "public"
        if public:
            members.extend(statement_members(head.strip(), braced, scope, primary))
    return members


def statement_members(head: str, braced: str | None, scope: str, primary: str) -> list[Member]:
    """Expected members produced by one statement of a scope."""
    scope_match = SCOPE_HEAD.match(head)
    if scope_match is not None and "(" not in head:
        return nested_scope_members(scope_match, braced, scope, primary)
    if not head or re.match(r"(friend|typedef|using|static_assert)\b", head) or "(" not in head:
        return []
    return function_members(head, scope, primary)


def nested_scope_members(match: re.Match[str], braced: str | None, scope: str, primary: str) -> list[Member]:
    """Members of a nested class/namespace, or the enum itself as one member."""
    kind, name = match.group(1), re.sub(r"\s+", "", match.group(2))
    qualified = f"{scope}::{name}" if scope else name
    if braced is None:
        return []
    if kind.startswith("enum"):
        return [Member(qualified, None, "")]
    if kind == "namespace":
        return scan_scope(braced, qualified, True, primary)
    return scan_scope(braced, qualified, kind != "class", primary)


def function_members(head: str, scope: str, primary: str) -> list[Member]:
    """The member declared by a function statement, or nothing for definitions/deletions."""
    if re.search(r"=\s*delete\s*$", head):
        return []
    name_match = re.search(r"(operator\s*(?:\(\)|\[\]|[^\s(\w]+)|~?[A-Za-z_]\w*(?:\s*::\s*~?\w+)*)\s*\(", head)
    if name_match is None:
        return []
    name = re.sub(r"\s+", "", name_match.group(1))
    if "::" in name and not name.startswith("operator"):
        return []  # out-of-class definition of an already declared member
    params_open = name_match.end() - 1
    params_close = matching_close(head, params_open)
    params = normalize_params(head[params_open + 1:params_close])
    qualifier = "const" if re.match(r"\s*const\b", head[params_close + 1:]) else ""
    owner = scope or primary
    prefix = owner if name == owner.split("<")[0].split("::")[-1] else f"{owner}::{name}"
    return [Member(prefix, params, qualifier)]


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
    tokens = re.findall(r"[A-Za-z_]\w*(?:\s*::\s*\w+)*(?:\s*<[^>]*>)?|[&*]+|\[[^\]]*\]", param)
    words = [token for token in tokens if re.match(r"\w", token)]
    type_words = [word for word in words if word not in {"const", "volatile", "unsigned", "signed"}]
    if len(type_words) >= 2 and tokens[-1] == words[-1]:
        tokens = tokens[:-1]
    tokens = [TYPE_ALIASES.get(token, re.sub(r"\s+", "", token)) for token in tokens]
    if tokens and tokens[0] == "const" and not any(token[0] in "&*" for token in tokens):
        tokens = tokens[1:]
    return re.sub(r" ([&*])", r"\1", " ".join(tokens))


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


def header_members(path: Path) -> list[Member]:
    """Every expected public member of one OCCT header.

    `DEFINE_STANDARD_EXCEPTION(Name, Base)` declares a whole exception class through a
    macro; it is expected as one `Name` row (the Rust `Failure` type-name mapping).
    """
    text = clean_source(path.read_text(errors="replace"))
    exception = re.compile(r"\bDEFINE_STANDARD_EXCEPTION\s*\(\s*(\w+)\s*,[^)]*\)")
    exceptions = [Member(name, None, "") for name in exception.findall(text)]
    return exceptions + scan_scope(exception.sub(";", text), "", True, path.stem)


def parse_row_id(upstream_id: str) -> Member:
    """Split a ledger `upstream_id` into prefix, parameter list and qualifier.

    >>> parse_row_id('gp_GTrsf::operator()(int,int) const')
    Member(prefix='gp_GTrsf::operator()', params='int,int', qualifier='const')
    """
    text, qualifier = upstream_id.strip(), ""
    if re.search(r"\)\s*const$", text):
        text, qualifier = re.sub(r"\s*const$", "", text), "const"
    if not text.endswith(")") or text.endswith("operator()"):
        return Member(text, None, qualifier)
    open_index = len(text) - 1 - matching_close(text[::-1].translate(str.maketrans("()", ")(")), 0)
    return Member(constructor_prefix(text[:open_index]), normalize_params(text[open_index + 1:-1]), qualifier)


def constructor_prefix(prefix: str) -> str:
    """Reduce the qualified constructor spelling `X::X` to the ledger's `X`.

    >>> constructor_prefix('NCollection_Vec4::NCollection_Vec4')
    'NCollection_Vec4'
    """
    owner, _, name = prefix.rpartition("::")
    return owner if owner and owner.split("<")[0].split("::")[-1] == name else prefix


def owner_of(member: Member) -> str:
    """Class part of a member prefix (`gp_Dir` for `gp_Dir::X` and for the ctor `gp_Dir`)."""
    return member.prefix if member.params is not None and "::" not in member.prefix.split("<")[0] \
        else member.prefix.rsplit("::", 1)[0]


def same_params(row: Member, member: Member) -> bool:
    """Compare parameter lists, treating `Class::Nested` and in-class `Nested` as equal."""
    if row.params is None or member.params is None:
        return False
    owner = owner_of(member)
    return row.params.replace(f"{owner}::", "") == member.params.replace(f"{owner}::", "")


def covers(row: Member, member: Member, members: list[Member]) -> bool:
    """True when `row` identifies `member` unambiguously among the header's `members`."""
    if row.prefix != member.prefix:
        return False
    same_prefix = [other for other in members if other.prefix == member.prefix]
    if row.params is None:
        return len(same_prefix) == 1
    twins = [other for other in same_prefix if same_params(row, other)]
    return same_params(row, member) and (len(twins) == 1 or row.qualifier == member.qualifier)


def audit_header(header: Path, source: str, ledger: dict[str, list[dict[str, str]]]) -> tuple[list[Member], list[str]]:
    """Members of one header without rows, and row ids of that header matching no member."""
    ids = [row["upstream_id"] for row in ledger["api"] if row["source"] == source]
    rows = [parse_row_id(upstream_id) for upstream_id in ids]
    members = header_members(header)
    missing = [m for m in members if not any(covers(row, m, members) for row in rows)]
    stale = [i for i, row in zip(ids, rows) if not any(covers(row, m, members) for m in members)]
    return missing, stale


def audit_package(package: Path, ledger: dict[str, list[dict[str, str]]], relative: str) -> tuple[list[str], list[str]]:
    """Rendered gaps and stale row ids, each prefixed by its header name."""
    gaps: list[str] = []
    stale: list[str] = []
    for header in sorted(package.glob("*.hxx")):
        missing, orphans = audit_header(header, f"{relative}/{header.name}", ledger)
        gaps += [f"{header.name}: {render(member)}" for member in missing]
        stale += [f"{header.name}: {upstream_id}" for upstream_id in orphans]
    return gaps, stale


def render(member: Member) -> str:
    """Human-readable identity of a member in ledger form."""
    params = "" if member.params is None else f"({member.params})"
    return f"{member.prefix}{params}{' const' if member.qualifier else ''}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("package", help="package directory relative to the OCCT root")
    parser.add_argument("--source", type=Path, required=True, help="OCCT source root")
    parser.add_argument("--ledger", type=Path, default=ROOT / "porting/parity.json")
    parser.add_argument("--members", action="store_true", help="list every parsed member instead of gaps")
    args = parser.parse_args()
    package = args.source / args.package
    if not package.is_dir():
        print(f"error: {package} is not a directory; expected an OCCT package path", file=sys.stderr)
        return 2
    if args.members:
        for header in sorted(package.glob("*.hxx")):
            print("\n".join(f"{header.name}: {render(m)}" for m in header_members(header)))
        return 0
    ledger = json.loads(args.ledger.read_text())
    gaps, stale = audit_package(package, ledger, args.package.rstrip("/"))
    print("\n".join(gaps + [f"stale row {row}" for row in stale]))
    print(f"{len(gaps)} public members without api rows; {len(stale)} rows matching no member", file=sys.stderr)
    return 1 if gaps or stale else 0

if __name__ == "__main__":
    sys.exit(main())
