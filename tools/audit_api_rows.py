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

Class template specializations keep their argument list in the scope name
(`BVH_Tree<T,N,BVH_BinaryTree>::Child`); primary templates use the bare name
(`NCollection_Lerp::Init`), as the ledger already spells them.

Beside the expected members the scanner records *known* identities: public or
protected typedefs, `using` aliases, data members, class/struct names,
enumerators, and protected member functions. A row naming a known identity is
not stale, but a known identity never needs a row. Rows whose `status` is
`excluded` cover members that have no Rust counterpart by design (destructors,
the RTTI macro); they must carry a `notes` reason.

Known false-positive sources and how they are handled:
- Out-of-class definitions (`inline gp_Vec::gp_Vec(...)`,
  `BVH_Box<T, N>::Area() const`) are recognised by the qualified name and
  ignored; only in-class declarations count.
- `= delete` members, body-less `friend` declarations, friend classes and
  private members are never expected or known. An in-class friend *definition*
  (a hidden friend such as `friend Vec2 operator+(...) {...}`) is expected as a
  member of the enclosing class.
- Function-like macros defined in the header itself (the NCollection_Vec
  swizzle generators) are expanded before scanning; the `DEFINE_STANDARD_*`
  macros are not, since they keep their dedicated identities.
- `DEFINE_STANDARD_RTTIEXT(Class, Base)` expands to `public:` plus the RTTI
  members, so it is rewritten to that label and a statement of its own: it can
  no longer swallow the next declaration, members after it are public as in
  C++, and it is expected as `Class::DEFINE_STANDARD_RTTIEXT(Class,Base)`,
  mapped by an excluded row. `DEFINE_STANDARD_HANDLE` only declares a handle
  typedef and is dropped.
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
from dataclasses import dataclass, field, replace
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IGNORED_MACROS = r"\b(?:DEFINE_STANDARD_ALLOC|DEFINE_NCOLLECTION_ALLOC|Standard_EXPORT|Standard_NODISCARD)\b"
TOKEN_NOISE = re.compile(r'//[^\n]*|/\*.*?\*/|"(?:\\.|[^"\\\n])*"|\'(?:\\.|[^\'\\\n])*\'', re.S)
ACCESS_LABEL = re.compile(r"^\s*(public|protected|private)\s*:(?!:)")
SCOPE_HEAD = re.compile(r"^(class|struct|union|namespace|enum(?:\s+class)?)\s+([\w:]+)\s*(<)?")
RTTI_MACRO = re.compile(r"\b(DEFINE_STANDARD_RTTIEXT|DEFINE_STANDARD_RTTI_INLINE)\s*\(([^)]*)\)")
PARAM_TOKEN = re.compile(r"[A-Za-z_]\w*|[&*]+|\[[^\]]*\]")
HANDLE_MACRO = re.compile(r"\bDEFINE_STANDARD_HANDLE\s*\([^)]*\)")
TYPE_ALIASES = {"Standard_Real": "double", "Standard_Integer": "int", "Standard_Boolean": "bool"}


@dataclass(frozen=True)
class Member:
    """One expected ledger identity: `prefix` is `Class` or `Class::Name`."""

    prefix: str
    params: str | None
    qualifier: str
    required: bool = field(default=True, compare=False, repr=False)


def clean_source(text: str, included_macros: dict[str, tuple[list[str], str]] | None = None) -> str:
    """Remove comments, literals, preprocessor lines, attributes and ignored macros.

    >>> clean_source('int a; // x\\n#define B 1\\n[[nodiscard]] int b();')
    'int a;  \\n\\n int b();'
    """
    text = TOKEN_NOISE.sub(lambda m: '""' if m.group(0)[0] in "\"'" else " ", text)
    macros = {**(included_macros or {}), **header_macros(text)}
    text = re.sub(r"^\s*#(?:[^\n]*\\\n)*[^\n]*$", "", text, flags=re.M)
    text = expand_header_macros(text, macros)
    text = re.sub(r"\[\[[^\]]*\]\]|__attribute__\s*\(\([^)]*\)\)", "", text)
    # The message may be several adjacent literals (`"a " "b"`), each already reduced to `""`.
    text = re.sub(r"\bStandard_DEPRECATED\s*\((?:\s*\"*)*\s*\)", "", text)
    text = RTTI_MACRO.sub(r"public: \1(\2);", HANDLE_MACRO.sub(";", text))  # the macro opens with `public:`
    return re.sub(IGNORED_MACROS, "", text)


def header_macros(text: str) -> dict[str, tuple[list[str], str]]:
    """Function-like macros `#define`d in the header itself, as `{name: (params, body)}`.

    NCollection_Vec2/3 declare their GLSL-style swizzles (`xy()`, `zyx()`) through
    `NCOLLECTION_VEC_COMPONENTS_2D/3D`, defined and invoked in the same header; expanding
    them is the only way those members become visible to the scanner.

    >>> header_macros('#define M(a, b) \\\\\\n  int a##b();')
    {'M': (['a', 'b'], 'int a##b();')}
    """
    pattern = re.compile(r"^\s*#\s*define\s+(\w+)\(([^)]*)\)((?:[^\n]*\\\n)*[^\n]*)$", re.M)
    # The RTTI/handle/exception macros keep their dedicated ledger identities (see
    # `clean_source`), even inside Standard_Type.hxx, which defines them.
    return {match.group(1): ([param.strip() for param in match.group(2).split(",") if param.strip()],
                             re.sub(r"\\\n", " ", match.group(3)).strip())
            for match in pattern.finditer(text) if not match.group(1).startswith("DEFINE_STANDARD_")}


def expand_header_macros(text: str, macros: dict[str, tuple[list[str], str]]) -> str:
    """Replace each invocation of a header-local function-like macro by its body.

    >>> expand_header_macros('struct S { M(x, y) };', {'M': (['a', 'b'], 'int a##b();')})
    'struct S { int xy(); };'
    """
    for name, (params, body) in macros.items():
        while (match := re.search(rf"\b{name}\s*\(", text)) is not None:
            close = matching_close(text, match.end() - 1)
            arguments = macro_arguments(text[match.end():close])
            if len(arguments) != len(params):
                raise ValueError(f"macro {name} invoked with {arguments!r}; expected {len(params)} arguments")
            text = text[:match.start()] + substitute_macro(body, params, arguments) + text[close + 1:]
    return text


def macro_arguments(text: str) -> list[str]:
    """Split macro arguments as the preprocessor does: only parentheses nest, not `<>`.

    >>> macro_arguments('a <= b || c, "x"')
    ['a <= b || c', '"x"']
    """
    parts, depth, start = [], 0, 0
    for index, char in enumerate(text):
        depth += (char == "(") - (char == ")")
        if char == "," and depth == 0:
            parts.append(text[start:index].strip())
            start = index + 1
    return parts + [text[start:].strip()] if text.strip() else parts


def substitute_macro(body: str, params: list[str], arguments: list[str]) -> str:
    """Macro body with parameters replaced, `#param` stringized to `""` and `##` pasted.

    >>> substitute_macro('T a##b() { return #a; }', ['a', 'b'], ['x', 'y'])
    'T xy() { return ""; }'
    """
    values = dict(zip(params, arguments))
    body = re.sub(r"(?<!#)#\s*(\w+)", lambda m: '""' if m.group(1) in values else m.group(0), body)
    body = re.sub(r"\b\w+\b", lambda m: values.get(m.group(0), m.group(0)), body)
    return re.sub(r"\s*##\s*", "", body)


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


def scan_scope(body: str, scope: str, public_default: bool, primary: str, exposed: bool = True) -> list[Member]:
    """Collect expected and known members declared in one namespace or class body.

    `exposed` is False inside a protected nested scope, where nothing is required.
    """
    members: list[Member] = []
    access = "public" if public_default else "private"
    for head, braced in split_statements(body):
        label, head = strip_access(head)
        access = label or access
        if access != "private":
            members.extend(statement_members(head.strip(), braced, scope, primary, exposed and access == "public"))
    return members


def statement_members(head: str, braced: str | None, scope: str, primary: str, exposed: bool) -> list[Member]:
    """Members produced by one statement of a scope; non-exposed ones are only known."""
    head = strip_template_heads(head)
    scope_match = SCOPE_HEAD.match(head)
    if scope_match is not None and "(" not in head:
        return nested_scope_members(head, scope_match, braced, scope, primary, exposed)
    if re.fullmatch(r"enum", head) and braced is not None:
        return enumerator_members(braced, scope)  # anonymous enum: its constants join the scope
    if braced is not None and (friend := re.match(r"friend\s+(?!class\b|struct\b)", head)) is not None:
        # An in-class friend definition (`friend Vec2 operator+(...) {...}`) is a hidden friend:
        # public API reachable only through argument-dependent lookup, attributed to the class.
        return exposure(function_members(head[friend.end():], scope, primary), exposed)
    if not head or re.match(r"(friend|static_assert)\b", head):
        return []
    if re.match(r"(typedef|using)\b", head):
        return alias_members(head, scope)
    if is_data_member(head):
        return data_members(head, scope)
    return exposure(function_members(head, scope, primary), exposed)


def strip_template_heads(head: str) -> str:
    """Drop leading `template <...>` parameter lists, balancing nested template-template ones.

    >>> strip_template_heads('template <class T, template <class> class D> class B')
    'class B'
    """
    while (match := re.match(r"\s*template\s*<", head)) is not None:
        head = head[matching_close(head, match.end() - 1) + 1:].lstrip()
    return head


def exposure(members: list[Member], exposed: bool) -> list[Member]:
    """Keep `members` required when exposed, otherwise demote them to known identities."""
    return members if exposed else [replace(member, required=False) for member in members]


def qualify(scope: str, name: str) -> str:
    """`scope::name`, or `name` at file scope."""
    return f"{scope}::{name}" if scope else name


def nested_scope_members(head: str, match: re.Match[str], braced: str | None, scope: str, primary: str,
                         exposed: bool) -> list[Member]:
    """A nested class/namespace's members, or an enum and its enumerators."""
    kind, name = match.group(1), match.group(2)
    if match.group(3) is not None:  # partial/explicit specialization keeps its argument list
        opening = match.end() - 1
        name += squeeze(head[opening:matching_close(head, opening) + 1])
    qualified = qualify(scope, name)
    if braced is None:
        return []
    if kind.startswith("enum"):
        enumerator_scope = qualified if kind != "enum" else scope
        return exposure([Member(qualified, None, "")], exposed) + enumerator_members(braced, enumerator_scope)
    if kind == "namespace":
        return scan_scope(braced, qualified, True, primary, exposed)
    body = scan_scope(braced, qualified, kind != "class", primary, exposed)
    return [Member(qualified, None, "", required=False)] + body


def enumerator_members(braced: str, scope: str) -> list[Member]:
    """Known identities for the enumerators of an enum body."""
    names = [re.match(r"\s*(\w+)", part) for part in split_top_level(braced)]
    return [Member(qualify(scope, name.group(1)), None, "", required=False) for name in names if name]


def alias_members(head: str, scope: str) -> list[Member]:
    """The known identity declared by a `typedef` or `using X = ...` alias."""
    using = re.match(r"using\s+(\w+)\s*=", head)
    typedef = re.search(r"(\w+)\s*(?:\[[^\]]*\]\s*)*$", head) if head.startswith("typedef") else None
    name = using or typedef
    return [] if name is None else [Member(qualify(scope, name.group(1)), None, "", required=False)]


def is_data_member(head: str) -> bool:
    """True for a variable declaration: no call parentheses, or an `=` initializer before them."""
    if "(" not in head:
        return True
    return "operator" not in head and "=" in head[:head.index("(")]


def data_members(head: str, scope: str) -> list[Member]:
    """The known identity of a data member or namespace-scope constant (`T myX`, `constexpr T X = v`)."""
    declarator = re.sub(r"\[[^\]]*\]", "", split_default(head)).strip()
    words = re.findall(r"[A-Za-z_]\w*", declarator)
    name = re.search(r"([A-Za-z_]\w*)$", declarator)
    if len(words) < 2 or name is None:
        return []
    return [Member(qualify(scope, name.group(1)), None, "", required=False)]


def function_members(head: str, scope: str, primary: str) -> list[Member]:
    """The member declared by a function statement, or nothing for definitions/deletions."""
    if re.search(r"=\s*delete\s*$", head):
        return []
    name_match = re.search(r"(operator\s*(?:\(\)|\[\]|[^\s(\w]+)|operator\s+(?:(?:const|volatile)\s+)*[A-Za-z_][\w:]*\s*[&*]*"
                           r"|~?[A-Za-z_]\w*(?:\s*::\s*~?\w+)*)\s*\(", head)
    if name_match is None:
        return []
    name = conversion_spelling(name_match.group(1))
    if "::" in name and not name.startswith("operator") or head[:name_match.start(1)].rstrip().endswith("::"):
        return []  # out-of-class definition (`X::f`, `X<T, N>::f`) of an already declared member
    params_open = name_match.end() - 1
    params_close = matching_close(head, params_open)
    params = normalize_params(head[params_open + 1:params_close])
    qualifier = "const" if re.match(r"\s*const\b", head[params_close + 1:]) else ""
    owner = scope or primary
    prefix = owner if name == class_name(owner) else f"{owner}::{name}"
    return [Member(prefix, params, qualifier)]


def conversion_spelling(raw: str) -> str:
    """Compact a member name, keeping the one space of a conversion operator.

    Without this, `explicit operator bool() const` was read as a member named `bool`.

    >>> conversion_spelling("operator bool")
    'operator bool'
    >>> conversion_spelling("operator ()")
    'operator()'
    >>> conversion_spelling("operator const  Element_t *")
    'operator const Element_t*'
    """
    match = re.fullmatch(r"operator\s+([A-Za-z_].*)", raw.strip())
    compact = re.sub(r"\s+", "", raw)
    return compact if match is None else "operator " + squeeze(match.group(1))


def class_name(owner: str) -> str:
    """Unqualified class name of a scope, without template arguments.

    >>> class_name('BVH_Tree<T,N,BVH_BinaryTree>::BVH_Range')
    'BVH_Range'
    """
    while (stripped := re.sub(r"<[^<>]*>", "", owner)) != owner:
        owner = stripped
    return owner.split("::")[-1]


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


def sibling_macros(path: Path) -> dict[str, tuple[list[str], str]]:
    """Function-like macros of the same-package headers that `path` includes.

    NCollection_Vec3 invokes `NCOLLECTION_VEC_COMPONENTS_2D`, defined in the included
    NCollection_Vec2.hxx; only direct includes that live beside `path` are read.
    """
    text = path.read_text(errors="replace")
    macros: dict[str, tuple[list[str], str]] = {}
    for name in re.findall(r'^\s*#\s*include\s*[<"]([\w.]+)[>"]', text, re.M):
        sibling = path.parent / name
        if sibling.is_file() and sibling != path:
            macros.update(header_macros(TOKEN_NOISE.sub(" ", sibling.read_text(errors="replace"))))
    return macros


def header_members(path: Path) -> list[Member]:
    """Every expected public member of one OCCT header.

    `DEFINE_STANDARD_EXCEPTION(Name, Base)` declares a whole exception class through a
    macro; it is expected as one `Name` row (the Rust `Failure` type-name mapping).
    """
    text = clean_source(path.read_text(errors="replace"), sibling_macros(path))
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
    return owner if owner and class_name(owner) == name else prefix


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
        # A bare row names a function only when it is not overloaded, and a class,
        # alias or data member regardless of the constructors sharing its prefix.
        return len([other for other in same_prefix if (other.params is None) == (member.params is None)]) == 1
    twins = [other for other in same_prefix if same_params(row, other)]
    return same_params(row, member) and (len(twins) == 1 or row.qualifier == member.qualifier)


def audit_header(header: Path, source: str, ledger: dict[str, list[dict[str, str]]]) -> tuple[list[Member], list[str]]:
    """Members of one header without rows, and row ids of that header matching no member."""
    entries = [row for row in ledger["api"] if row["source"] == source]
    ids = [row["upstream_id"] for row in entries]
    rows = [parse_row_id(upstream_id) for upstream_id in ids]
    members = header_members(header)
    missing = [m for m in members if m.required and not any(covers(row, m, members) for row in rows)]
    stale = [i for i, row in zip(ids, rows) if not any(covers(row, m, members) for m in members)]
    unjustified = [f"{row['upstream_id']} (excluded without notes)" for row in entries
                   if row.get("status") == "excluded" and not row.get("notes", "").strip()]
    return missing, stale + unjustified


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
            print("\n".join(f"{header.name}: {render(m)}{'' if m.required else ' (known)'}"
                            for m in header_members(header)))
        return 0
    ledger = json.loads(args.ledger.read_text())
    gaps, stale = audit_package(package, ledger, args.package.rstrip("/"))
    print("\n".join(gaps + [f"stale row {row}" for row in stale]))
    print(f"{len(gaps)} public members without api rows; {len(stale)} rows matching no member", file=sys.stderr)
    return 1 if gaps or stale else 0

if __name__ == "__main__":
    sys.exit(main())
