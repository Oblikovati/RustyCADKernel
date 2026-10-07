"""Resolve declared cross-language mappings without guessing overload identity."""

from __future__ import annotations

from collections import defaultdict
import re
from api_inventory.inputs import JSONValue
from api_cpp_configuration import (
    CONFIGURATION_FAILURES,
    ConfigurationEvidence,
    INACTIVE as INACTIVE_CONFIGURATION,
    configured_cpp_status,
)
from api_cpp_selectors import CppPin, INVALID_PIN, pinned_candidates, row_cpp_pin
from api_identity import leaf_name, norm_cpp, parse_upstream_id, portable_qualified_name
from api_rust_candidates import (
    candidate_status,
    row_list,
    row_rust_candidates,
    rust_candidates,
    rust_crates,
)
from api_rust_declaration_selectors import SELECTION_FAILURES
from api_scoped_alias_selectors import qualify_member_aliases
from api_template_selectors import (
    constructor_candidates,
    declaration_template_owner,
    prefer_nonconst_overload,
    suffix_matches,
    template_parameter_match,
)


def header_owners(manifest: dict[str, JSONValue]) -> dict[str, set[str]]:
    owners: dict[str, set[str]] = defaultdict(set)
    for header in row_list(manifest, "headers"):
        path = header.get("path")
        if not isinstance(path, str):
            continue
        for owner in header.get("owners", []):
            if isinstance(owner, dict) and isinstance(owner.get("toolkit"), str):
                owners[path].add(owner["toolkit"])
    return dict(owners)


def observed_headers(declaration: dict[str, JSONValue]) -> set[str]:
    main_header = declaration.get("main_header")
    result = {main_header} if isinstance(main_header, str) else set()
    for site in declaration.get("sites", []):
        if not isinstance(site, dict):
            continue
        observed = site.get("observed_in")
        if isinstance(observed, str):
            result.add(observed)
        elif isinstance(observed, list):
            result.update(path for path in observed if isinstance(path, str))
    return result


def declaration_toolkits(
    declaration: dict[str, JSONValue], owners: dict[str, set[str]]
) -> set[str]:
    source = declaration.get("source")
    direct = owners.get(source, set()) if isinstance(source, str) else set()
    if direct:
        return direct
    paths = observed_headers(declaration)
    return set().union(*(owners.get(path, set()) for path in paths)) if paths else set()


def cpp_indexes(
    declarations: list[dict[str, JSONValue]], owners: dict[str, set[str]]
) -> tuple[dict[str, list[dict[str, JSONValue]]], list[str], int]:
    by_name: dict[str, list[dict[str, JSONValue]]] = defaultdict(list)
    by_id = {}
    duplicates = set()
    malformed = 0
    for declaration in declarations:
        identity = declaration.get("id")
        name = declaration.get("qualified_name")
        if not isinstance(identity, str) or not isinstance(name, str):
            malformed += 1
            continue
        declaration["_toolkits"] = declaration_toolkits(declaration, owners)
        name = portable_qualified_name(name, declaration.get("source"))
        declaration["qualified_name"] = name
        by_name[name].append(declaration)
        if identity in by_id:
            duplicates.add(identity)
        by_id[identity] = declaration
    return dict(by_name), sorted(duplicates), malformed


def parameter_match(
    params: list[str],
    declaration: dict[str, JSONValue],
    template_owner: str | None = None,
) -> bool:
    fixed = fixed_selector_parameters(params, declaration)
    if fixed is None:
        return False
    expected = [canonical_parameter_type(item) for item in fixed]
    if any(item is None for item in expected):
        return False
    choices = []
    for field in ("written_parameter_types", "parameter_types"):
        value = declaration.get(field)
        if isinstance(value, list) and all(isinstance(item, str) for item in value):
            choice = [canonical_parameter_type(item) for item in value]
            if all(item is not None for item in choice):
                choices.append(choice)
                if field == "parameter_types" and template_owner:
                    if template_parameter_match(expected, choice, template_owner):
                        return True
    return expected in choices


def fixed_selector_parameters(
    params: list[str], declaration: dict[str, JSONValue]
) -> list[str] | None:
    """Require an exact variadic contract; e.g. `['const char*', '...']` for Printf."""
    variadic = bool(params) and params[-1].strip() == "..."
    if declaration.get("variadic", False) is not variadic:
        return None
    fixed = params[:-1] if variadic else params
    return None if any(item.strip() == "..." for item in fixed) else fixed


def scoped_parameter_match(
    parameters: list[str],
    declaration: dict[str, JSONValue],
    by_name: dict[str, list[dict[str, JSONValue]]],
) -> bool:
    """Match exact types or proven local aliases; e.g. `Buffer::Init(value_type&)`."""
    owner = declaration_template_owner(declaration, by_name)
    if parameter_match(parameters, declaration, owner):
        return True
    qualified = qualify_member_aliases(parameters, declaration, by_name)
    return qualified != parameters and parameter_match(qualified, declaration, owner)


def is_simple_array_reference(text: str) -> bool:
    """Recognize exact array references and pointers, e.g. `const char(&)[N]`.

    Pointer-to-array parameters such as MathSys::Utils::Solve2x2's `const double(*)[2]`
    keep their declarator: the element const is not top-level, so nothing is removed.
    """
    element = r"[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*(?: [A-Za-z_]\w*)*"
    bounds = r"(?:\[(?:[A-Za-z_]\w*|[0-9]+)\])+"
    return re.fullmatch(rf"{element}\((?:\*|&&?)\){bounds}", text) is not None


def canonical_parameter_type(text: str) -> str | None:
    # C++ function types discard only top-level cv on by-value parameters.
    normalized = norm_cpp(text)
    if is_simple_array_reference(normalized):
        return normalized
    tokens = top_level_type_tokens(normalized)
    if tokens is None:
        return None
    if any(token in {"&", "&&"} for token, _, _ in tokens):
        return normalized
    # Other parenthesized declarators still need a full parser before matching.
    if any(token in {"(", ")", "[", "]"} for token, _, _ in tokens):
        return None
    stars = [end for token, _, end in tokens if token == "*"]
    removable = top_cv_tokens(tokens, stars[-1] if stars else 0)
    return remove_token_spans(normalized, removable)


def top_level_type_tokens(text: str) -> list[tuple[str, int, int]] | None:
    depths = {"<": 0, "(": 0, "[": 0}
    closing = {">": "<", ")": "(", "]": "["}
    pattern = re.compile(r"[A-Za-z_]\w*|&&|[<>()\[\]*&]")
    tokens = []
    for match in pattern.finditer(text):
        token = match.group()
        if token in closing:
            key = closing[token]
            depths[key] -= 1
            if depths[key] < 0:
                return None
            continue
        if not any(depths.values()):
            tokens.append((token, match.start(), match.end()))
        if token in depths:
            depths[token] += 1
    return tokens if not any(depths.values()) else None


def top_cv_tokens(
    tokens: list[tuple[str, int, int]], minimum_start: int
) -> list[tuple[str, int, int]]:
    return [
        item
        for item in tokens
        if item[0] in {"const", "volatile"} and item[1] >= minimum_start
    ]


def remove_token_spans(text: str, tokens: list[tuple[str, int, int]]) -> str:
    for _, start, end in reversed(tokens):
        text = text[:start] + text[end:]
    return norm_cpp(text)


def cpp_candidates(
    identity: str,
    toolkit: str,
    by_name: dict[str, list[dict[str, JSONValue]]],
    pin: CppPin | None = None,
) -> tuple[str, list[dict[str, JSONValue]]]:
    """Match an upstream id, narrowed by an explicit row pin when one is declared."""
    parsed = parse_upstream_id(identity)
    if parsed is None:
        return "invalid_upstream_id", []
    name, params, suffix = parsed
    exact = by_name.get(name, [])
    if exact or "::" in name:
        candidates = exact
    else:
        candidates = [
            row
            for key, rows in by_name.items()
            if leaf_name(key) == name
            for row in rows
        ]
    if params is not None:
        candidates = constructor_candidates(name, candidates, by_name)
    candidates = [
        row
        for row in candidates
        if row.get("api_visible") is True and toolkit in row.get("_toolkits", set())
    ]
    # Pin before the unresolved-using check: a pinned method must not inherit the
    # unresolved status of a using-declaration that only shares its name.
    candidates = pinned_candidates(pin, candidates)
    unresolved = [
        row
        for row in candidates
        if row.get("api_resolution")
        in {
            "unresolved_using_value_target",
            "unsupported_template_signature_normalization",
        }
    ]
    if unresolved:
        resolution = unresolved[0].get("api_resolution")
        if resolution == "unsupported_template_signature_normalization":
            return "unstable_cpp_identity", unresolved
        return "unresolved_member_target", unresolved
    if params is not None:
        candidates = [
            row for row in candidates if scoped_parameter_match(params, row, by_name)
        ]
    candidates = [row for row in candidates if suffix_matches(suffix, row)]
    if params is not None and not suffix:
        candidates = prefer_nonconst_overload(candidates)
    status = (
        "conflicting_inventory"
        if any(row.get("inventory_conflict") for row in candidates)
        else candidate_status(candidates)
    )
    return status, candidates


def match_api_row(
    row: dict[str, JSONValue],
    by_name: dict[str, list[dict[str, JSONValue]]],
    rust: dict[str, dict[str, list[dict[str, JSONValue]]]],
    known_sources: set[str],
    configuration: ConfigurationEvidence | None = None,
) -> dict[str, JSONValue]:
    cpp_status, cpp = row_cpp_candidates(row, by_name)
    cpp_status = configured_cpp_status(row, cpp_status, configuration)
    rust_status, rust_rows = row_rust_candidates(row, rust)
    source_status = row.get("status", "missing_status")
    source = row.get("source")
    source_known = isinstance(source, str) and source in known_sources
    resolution = resolve_status(source_status, cpp_status, rust_status, source_known)
    return {
        "upstream_id": row.get("upstream_id"),
        "toolkit": row.get("toolkit"),
        "parity_status": source_status,
        "resolution": resolution,
        "cpp_status": cpp_status,
        "cpp_ids": [item.get("id") for item in cpp],
        "cpp_matches": [declaration_summary(item) for item in cpp],
        "rust_status": rust_status,
        "declared_rust_kind": row.get("rust_kind"),
        "declared_rust_selectors": row.get("rust_declaration_selectors"),
        "declared_rust_paths": row.get("rust_items", [row.get("rust_item")]),
        "rust_paths": [item.get("path") for item in rust_rows],
        "rust_matches": [rust_summary(item) for item in rust_rows],
        "source": row.get("source"),
    }


def row_cpp_candidates(
    row: dict[str, JSONValue], by_name: dict[str, list[dict[str, JSONValue]]]
) -> tuple[str, list[dict[str, JSONValue]]]:
    """Resolve a row's C++ side with its pin; e.g. `row_cpp_candidates(row, by_name)`."""
    pin, error = row_cpp_pin(row)
    if error is not None:
        return INVALID_PIN, []
    identity, toolkit = row.get("upstream_id", ""), row.get("toolkit", "")
    status, candidates = cpp_candidates(identity, toolkit, by_name, pin)
    return narrow_by_declaring_header(status, candidates, row.get("source"))


def narrow_by_declaring_header(
    status: str, candidates: list[dict[str, JSONValue]], source: JSONValue
) -> tuple[str, list[dict[str, JSONValue]]]:
    """Split same-name declarations by the row's header; e.g. one std::hash per header."""
    if status != "ambiguous" or not isinstance(source, str):
        return status, candidates
    local = [row for row in candidates if row.get("source") == source]
    return ("unique", local) if len(local) == 1 else (status, candidates)


CPP_FAILURES = {
    "conflicting_inventory": "conflicting_cpp_inventory",
    "unresolved_member_target": "unresolved_cpp_member_target",
    "unstable_cpp_identity": "unstable_cpp_identity",
    "ambiguous": "ambiguous_cpp",
    "missing": "missing_cpp",
}


def cpp_resolution(parity: str, cpp: str, source_known: bool) -> str | None:
    """Resolution decided by the C++ side alone, or None when the Rust side decides."""
    if cpp == INVALID_PIN or cpp in CONFIGURATION_FAILURES:
        return cpp
    if parity == "excluded" and cpp == "missing":
        return "unmatched_exclusion" if source_known else "stale_exclusion"
    # A proven inactive-configuration declaration resolves like a unique match.
    if cpp in {"unique", INACTIVE_CONFIGURATION}:
        return None
    return CPP_FAILURES.get(cpp, "invalid_cpp_identity")


def resolve_status(parity: str, cpp: str, rust: str, source_known: bool = True) -> str:
    if rust in SELECTION_FAILURES:
        return rust
    decided = cpp_resolution(parity, cpp, source_known)
    if decided is not None:
        return decided
    if parity == "upstream_unimplemented":
        return "upstream_unimplemented"
    if parity == "partial":
        return "partial"
    if parity == "excluded" and rust == "no_rust_target_declared":
        return "excluded"
    if rust == "invalid_rust_targets":
        return "invalid_rust_targets"
    if rust == "ambiguous":
        return "ambiguous_rust"
    if rust == "unverifiable_source":
        return "unverifiable_rust_source"
    if rust != "unique":
        return "missing_rust_target"
    if parity == "excluded":
        return "excluded"
    if parity == "implemented":
        return "implemented"
    return "unhandled_parity_status"


def declaration_summary(row: dict[str, JSONValue]) -> dict[str, JSONValue]:
    return {
        key: row.get(key)
        for key in ("id", "usr", "qualified_name", "kind", "signature", "source")
    }


def rust_summary(row: dict[str, JSONValue]) -> dict[str, JSONValue]:
    return {
        key: row.get(key)
        for key in (
            "path",
            "kind",
            "signature",
            "impl_context",
            "declaration_sha256",
            "source_file",
            "source_status",
            "source_sha256",
        )
    }
