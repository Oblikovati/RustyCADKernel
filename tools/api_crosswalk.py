"""Resolve declared cross-language mappings without guessing overload identity."""

from __future__ import annotations

from collections import defaultdict
import re
from api_inventory.inputs import JSONValue
from api_identity import leaf_name, norm_cpp, parse_upstream_id


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


def suffix_matches(identity_suffix: str, declaration: dict[str, JSONValue]) -> bool:
    if not identity_suffix:
        return True
    signature = declaration.get("signature")
    if not isinstance(signature, str):
        return False
    return norm_cpp(signature).endswith(identity_suffix)


def parameter_match(params: list[str], declaration: dict[str, JSONValue]) -> bool:
    expected = [canonical_parameter_type(item) for item in params]
    if any(item is None for item in expected):
        return False
    choices = []
    for field in ("written_parameter_types", "parameter_types"):
        value = declaration.get(field)
        if isinstance(value, list) and all(isinstance(item, str) for item in value):
            choice = [canonical_parameter_type(item) for item in value]
            if all(item is not None for item in choice):
                choices.append(choice)
    return expected in choices


def canonical_parameter_type(text: str) -> str | None:
    # C++ function types discard only top-level cv on by-value parameters.
    normalized = norm_cpp(text)
    tokens = top_level_type_tokens(normalized)
    if tokens is None:
        return None
    if any(token in {"&", "&&"} for token, _, _ in tokens):
        return normalized
    # Parenthesized declarators need a full declarator parser before matching.
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
    candidates = [
        row
        for row in candidates
        if row.get("api_visible") is True and toolkit in row.get("_toolkits", set())
    ]
    if params is not None:
        candidates = [row for row in candidates if parameter_match(params, row)]
    candidates = [row for row in candidates if suffix_matches(suffix, row)]
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
    prefix, separator, _path = rust_item.partition("::")
    if not separator or prefix not in crates:
        return "unknown_rust_crate", []
    candidates = crates[prefix].get(rust_item, [])
    status = candidate_status(candidates)
    if status == "unique" and not rust_source_is_pinned(candidates[0]):
        return "unverifiable_source", candidates
    return status, candidates


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
    rust_status, rust_rows = rust_candidates(row.get("rust_item"), rust)
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
        "rust_paths": [item.get("path") for item in rust_rows],
        "rust_matches": [rust_summary(item) for item in rust_rows],
        "source": row.get("source"),
    }


def resolve_status(parity: str, cpp: str, rust: str, source_known: bool = True) -> str:
    if parity == "excluded" and cpp == "missing":
        return "unmatched_exclusion" if source_known else "stale_exclusion"
    if cpp == "conflicting_inventory":
        return "conflicting_cpp_inventory"
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
