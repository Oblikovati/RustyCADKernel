"""Resolve declared cross-language mappings without guessing overload identity."""

from __future__ import annotations

from collections import defaultdict
import re
from api_inventory.inputs import JSONValue
from api_identity import leaf_name, norm_cpp, parse_upstream_id
from api_rust_kind import select_rust_kind
from api_scoped_alias_selectors import qualify_member_aliases
from api_template_selectors import (
    constructor_candidates,
    declaration_template_owner,
    prefer_nonconst_overload,
    suffix_matches,
    template_parameter_match,
)


def row_list(document: dict[str, JSONValue], field: str) -> list[dict[str, JSONValue]]:
    rows = document.get(field)
    if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
        raise ValueError(f"JSON field {field!r} must be an array of objects")
    return rows


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
    """Recognize exact array references, e.g. `const char(&)[N]`, without decay."""
    element = r"[A-Za-z_]\w*(?:::[A-Za-z_]\w*)*(?: [A-Za-z_]\w*)*"
    bounds = r"(?:\[(?:[A-Za-z_]\w*|[0-9]+)\])+"
    return re.fullmatch(rf"{element}\(&&?\){bounds}", text) is not None


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
    identity: str, toolkit: str, by_name: dict[str, list[dict[str, JSONValue]]]
) -> tuple[str, list[dict[str, JSONValue]]]:
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


def candidate_status(candidates: list[dict[str, JSONValue]]) -> str:
    if not candidates:
        return "missing"
    return "unique" if len(candidates) == 1 else "ambiguous"


def rust_crates(
    document: dict[str, JSONValue],
) -> dict[str, dict[str, list[dict[str, JSONValue]]]]:
    result = {}
    for crate in row_list(document, "crates"):
        crate_name = crate.get("crate")
        if not isinstance(crate_name, str):
            continue
        paths: dict[str, list[dict[str, JSONValue]]] = defaultdict(list)
        for item in crate.get("declarations", []):
            if isinstance(item, dict) and isinstance(item.get("path"), str):
                paths[item["path"]].append(item)
        result[crate_name.replace("-", "_")] = dict(paths)
    return result


def rust_candidates(
    rust_item: JSONValue, crates: dict[str, dict[str, list[dict[str, JSONValue]]]]
) -> tuple[str, list[dict[str, JSONValue]]]:
    if not isinstance(rust_item, str) or not rust_item:
        return "no_rust_target_declared", []
    if bool_primitive_ufcs_target(rust_item):
        return bool_primitive_candidates(rust_item, crates)
    prefix, separator = rust_target_prefix(rust_item)
    if not separator or prefix not in crates:
        return "unknown_rust_crate", []
    candidates = crates[prefix].get(rust_item, [])
    status = candidate_status(candidates)
    if status == "unique" and not rust_source_is_pinned(candidates[0]):
        return "unverifiable_source", candidates
    return status, candidates


def bool_primitive_ufcs_target(target: str) -> bool:
    return target.startswith("<bool as ") and ">::" in target


def bool_primitive_candidates(
    target: str, crates: dict[str, dict[str, list[dict[str, JSONValue]]]]
) -> tuple[str, list[dict[str, JSONValue]]]:
    matches = [
        item
        for crate_name, paths in crates.items()
        for item in paths.get(target, [])
        if item.get("foreign_impl_self_primitive") == "bool"
        and normalized_origin(item) == crate_name
    ]
    status = candidate_status(matches)
    if status == "unique" and not bool_source_is_pinned(matches[0]):
        return "unverifiable_source", matches
    return status, matches


def bool_source_is_pinned(item: dict[str, JSONValue]) -> bool:
    anchor_file = item.get("foreign_impl_anchor_source_file")
    anchor_hash = item.get("foreign_impl_anchor_source_sha256")
    return (
        rust_source_is_pinned(item)
        and isinstance(anchor_file, str)
        and bool(anchor_file)
        and isinstance(anchor_hash, str)
        and re.fullmatch(r"[0-9a-f]{64}", anchor_hash) is not None
    )


def normalized_origin(item: dict[str, JSONValue]) -> str | None:
    origin = item.get("foreign_impl_origin_crate")
    return origin.replace("-", "_") if isinstance(origin, str) else None


def rust_target_prefix(target: str) -> tuple[str, bool]:
    if target.startswith("<"):
        owner, separator, _rest = target[1:].partition(" as ")
        prefix, path_separator, _tail = owner.partition("::")
        return prefix, bool(separator and path_separator)
    prefix, separator, _path = target.partition("::")
    return prefix, bool(separator)


def row_rust_candidates(
    row: dict[str, JSONValue],
    crates: dict[str, dict[str, list[dict[str, JSONValue]]]],
) -> tuple[str, list[dict[str, JSONValue]]]:
    """Require all accessor targets; e.g. `row_rust_candidates(row, crates)`."""
    if "rust_kind" in row:
        selected = select_rust_kind(crates, row["rust_kind"])
        if selected is None:
            return "invalid_rust_kind", []
        crates = selected
    if "rust_items" not in row:
        return rust_candidates(row.get("rust_item"), crates)
    targets = paired_rust_targets(row)
    if targets is None:
        return "invalid_rust_targets", []
    matches = [rust_candidates(target, crates) for target in targets]
    status = next((status for status, _ in matches if status != "unique"), "unique")
    return status, [item for _, candidates in matches for item in candidates]


def paired_rust_targets(row: dict[str, JSONValue]) -> list[str] | None:
    """Validate an ordered target set; e.g. `paired_rust_targets(row)`."""
    targets = row.get("rust_items")
    if not isinstance(targets, list) or not targets:
        return None
    if any(not isinstance(target, str) or not target for target in targets):
        return None
    if len(set(targets)) != len(targets):
        return None
    primary = row.get("rust_item")
    if "rust_item" in row and primary != targets[0]:
        return None
    return targets


def rust_source_is_pinned(item: dict[str, JSONValue]) -> bool:
    source_file = item.get("source_file")
    source_hash = item.get("source_sha256")
    return (
        isinstance(source_file, str)
        and bool(source_file)
        and item.get("source_status") is None
        and isinstance(source_hash, str)
        and re.fullmatch(r"[0-9a-f]{64}", source_hash) is not None
    )


def match_api_row(
    row: dict[str, JSONValue],
    by_name: dict[str, list[dict[str, JSONValue]]],
    rust: dict[str, dict[str, list[dict[str, JSONValue]]]],
    known_sources: set[str],
) -> dict[str, JSONValue]:
    cpp_status, cpp = cpp_candidates(
        row.get("upstream_id", ""), row.get("toolkit", ""), by_name
    )
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
        "declared_rust_paths": row.get("rust_items", [row.get("rust_item")]),
        "rust_paths": [item.get("path") for item in rust_rows],
        "rust_matches": [rust_summary(item) for item in rust_rows],
        "source": row.get("source"),
    }


def resolve_status(parity: str, cpp: str, rust: str, source_known: bool = True) -> str:
    if parity == "excluded" and cpp == "missing":
        return "unmatched_exclusion" if source_known else "stale_exclusion"
    if cpp == "conflicting_inventory":
        return "conflicting_cpp_inventory"
    if cpp == "unresolved_member_target":
        return "unresolved_cpp_member_target"
    if cpp == "unstable_cpp_identity":
        return "unstable_cpp_identity"
    if cpp == "ambiguous":
        return "ambiguous_cpp"
    if cpp != "unique":
        return "missing_cpp" if cpp == "missing" else "invalid_cpp_identity"
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
    if parity == "implemented" and rust == "unique":
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
            "source_file",
            "source_status",
            "source_sha256",
        )
    }
