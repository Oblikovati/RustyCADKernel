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
Inside a named namespace (the modern `MathUtils`/`MathPoly`-style packages, which have
no primary class) the namespace's structs, classes, `using` aliases and `constexpr`
constants are public API too, so each becomes one bare `Namespace::Name` row.

Row identities follow the ledger: `Class(params)` for constructors,
`Class::Name` when the name is not overloaded, `Class::Name(params)[ const]`
when it is. Parameter lists are compared after dropping parameter names,
default values, top-level `const` on by-value parameters and whitespace.

Known false-positive sources and how they are handled:
- Out-of-class definitions (`inline gp_Vec::gp_Vec(...)`) are recognised by
  the qualified name and ignored; only in-class declarations count.
- `= delete` members, `friend` declarations, typedefs, data members (also those whose
  default initializer calls a function, such as `= std::numeric_limits<double>::quiet_NaN()`)
  and protected/private members are never expected; `using` aliases only in a namespace.
- A row with status `excluded` still covers its member but must carry a `notes` reason;
  rows that do not are reported as `excluded without notes`.
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


@dataclass(frozen=True)
class Scope:
    """Where a statement is declared: qualified name, the header's primary class, namespace-ness."""

    name: str
    primary: str
    is_namespace: bool = False


def scan_scope(body: str, scope: Scope, public_default: bool) -> list[Member]:
    """Collect expected members declared in one namespace or class body."""
    members: list[Member] = []
    public = public_default
    for head, braced in split_statements(body):
        label, head = strip_access(head)
        public = public if label is None else label == "public"
        if public:
            members.extend(statement_members(head.strip(), braced, scope))
    return members


def statement_members(head: str, braced: str | None, scope: Scope) -> list[Member]:
    """Expected members produced by one statement of a scope."""
    scope_match = SCOPE_HEAD.match(head)
    if scope_match is not None and "(" not in head:
        return nested_scope_members(scope_match, braced, scope)
    if scope.is_namespace and (entity := namespace_entity(head)) is not None:
        return [Member(f"{scope.name}::{entity}", None, "")]
    if not head or re.match(r"(friend|typedef|using|static_assert)\b", head) or "(" not in head:
        return []
    return function_members(head, scope)


def namespace_entity(head: str) -> str | None:
    """Name of a namespace-scope `using X = ...` alias or `constexpr` constant, else None.

    >>> namespace_entity("inline constexpr int THE_MAX = 20")
    'THE_MAX'
    >>> namespace_entity("using LongCoefficients = std::array<long double, 21>")
    'LongCoefficients'
    """
    alias = re.match(r"using\s+(\w+)\s*=", head)
    if alias is not None:
        return alias.group(1)
    constant = re.match(r"(?:(?:inline|static)\s+)*constexpr\b[^(=]*?\b(\w+)\s*=", head)
    return None if constant is None else constant.group(1)


def nested_scope_members(match: re.Match[str], braced: str | None, scope: Scope) -> list[Member]:
    """Members of a nested class/namespace, or the enum itself as one member.

    A struct/class defined directly in a named namespace is itself public API (there is no
    primary class to hang it on), so it is expected as one bare type row besides its members.
    """
    kind, name = match.group(1), re.sub(r"\s+", "", match.group(2))
    qualified = f"{scope.name}::{name}" if scope.name else name
    if braced is None:
        return []
    if kind.startswith("enum"):
        return [Member(qualified, None, "")]
    if kind == "namespace":
        # `namespace std` only holds hash/equal_to specializations of this library's types.
        return scan_scope(braced, Scope(qualified, scope.primary, name != "std"), True)
    # A specialization (`VecComp<T, 2>`) refines the primary template's row, not a new type.
    own = [Member(qualified, None, "")] if scope.is_namespace and "<" not in name else []
    return own + scan_scope(braced, Scope(qualified, scope.primary), kind != "class")


def function_members(head: str, scope: Scope) -> list[Member]:
    """The member declared by a function statement, or nothing for definitions/deletions."""
    if re.search(r"=\s*delete\s*$", head):
        return []
    name_match = re.search(r"(operator\s*(?:\(\)|\[\]|[^\s(\w]+)|operator\s+[A-Za-z_][\w:]*\s*[&*]*"
                           r"|~?[A-Za-z_]\w*(?:\s*::\s*~?\w+)*)\s*\(", head)
    if name_match is None or is_data_initializer(head[:name_match.start()]):
        return []
    name = conversion_spelling(name_match.group(1))
    if "::" in name and not name.startswith("operator"):
        return []  # out-of-class definition of an already declared member
    params_open = name_match.end() - 1
    params_close = matching_close(head, params_open)
    params = normalize_params(head[params_open + 1:params_close])
    qualifier = "const" if re.match(r"\s*const\b", head[params_close + 1:]) else ""
    owner = scope.name or scope.primary
    prefix = owner if name == owner.split("<")[0].split("::")[-1] else f"{owner}::{name}"
    return [Member(prefix, params, qualifier)]


def is_data_initializer(before_name: str) -> bool:
    """True when the call found in a statement sits in a data member's default initializer.

    `double Fa = std::numeric_limits<double>::quiet_NaN();` declares data, not `quiet_NaN`.

    >>> is_data_initializer("double Fa = std::numeric_limits<double>::")
    True
    >>> is_data_initializer("bool ")
    False
    """
    if (template := re.match(r"\s*template\s*<", before_name)) is not None:
        before_name = before_name[matching_close(before_name, template.end() - 1) + 1:]  # `<class T = double>`
    return re.search(r"(?<![=!<>])=(?!=)", before_name) is not None


def conversion_spelling(raw: str) -> str:
    """Compact a member name, keeping the one space of a conversion operator (`operator bool`).

    >>> conversion_spelling("operator bool")
    'operator bool'
    >>> conversion_spelling("operator ()")
    'operator()'
    """
    match = re.fullmatch(r"operator\s+([A-Za-z_].*)", raw.strip())
    compact = re.sub(r"\s+", "", raw)
    return compact if match is None else "operator " + re.sub(r"\s+", "", match.group(1))


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
    if (array := array_param(param)) is not None:
        return array
    tokens = re.findall(r"[A-Za-z_]\w*(?:\s*::\s*\w+)*(?:\s*<[^>]*>)?|[&*]+|\[[^\]]*\]", param)
    words = [token for token in tokens if re.match(r"\w", token)]
    type_words = [word for word in words if word not in {"const", "volatile", "unsigned", "signed"}]
    if len(type_words) >= 2 and tokens[-1] == words[-1] and not is_builtin_pair(type_words):
        tokens = tokens[:-1]
    tokens = [TYPE_ALIASES.get(token, re.sub(r"\s+", "", token)) for token in tokens]
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

    Arrays keep their `const` (it is not top-level) and extents; only the name goes.

    >>> array_param("const double theJ[3][3]")
    'const double[3][3]'
    >>> array_param("const double (&theF)[N]")
    'const double(&)[N]'
    """
    match = re.fullmatch(r"(.*?)(\(\s*[&*]+\s*\w*\s*\)|\b\w+)?\s*((?:\[[^\]]*\]\s*)+)", param, re.S)
    if match is None:
        return None
    base, declarator = match.group(1), match.group(2) or ""
    if not set(base.split()) - {"const", "volatile", "unsigned", "signed"} and not declarator.startswith("("):
        base, declarator = f"{base} {declarator}", ""  # nameless `const double[9]`: the word is the type
    base = " ".join(TYPE_ALIASES.get(word, word) for word in base.split())
    declarator = re.sub(r"\w+|\s+", "", declarator)
    return base + declarator + re.sub(r"\s+", "", match.group(3))


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
    return exceptions + scan_scope(exception.sub(";", text), Scope("", path.stem), True)


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
        # A bare row names a type/constant exactly, or a function only when it is not overloaded.
        return member.params is None or len(same_prefix) == 1
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


def excluded_without_reason(ledger: dict[str, list[dict[str, str]]], relative: str) -> list[str]:
    """Ids of a package's `excluded` rows that carry no `notes` justification."""
    return [row["upstream_id"] for row in ledger["api"]
            if row["source"].startswith(f"{relative}/") and row.get("status") == "excluded"
            and not row.get("notes", "").strip()]


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
    unjustified = excluded_without_reason(ledger, args.package.rstrip("/"))
    print("\n".join(gaps + [f"stale row {row}" for row in stale]
                    + [f"excluded without notes {row}" for row in unjustified]))
    print(f"{len(gaps)} public members without api rows; {len(stale)} rows matching no member; "
          f"{len(unjustified)} excluded rows without notes", file=sys.stderr)
    return 1 if gaps or stale or unjustified else 0

if __name__ == "__main__":
    sys.exit(main())
