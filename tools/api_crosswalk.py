"""Resolve declared cross-language mappings without guessing overload identity."""

from __future__ import annotations

from collections import defaultdict
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
    expected = [norm_cpp(item) for item in params]
    choices = []
    for field in ("written_parameter_types", "parameter_types"):
        value = declaration.get(field)
        if isinstance(value, list) and all(isinstance(item, str) for item in value):
            choices.append([norm_cpp(item) for item in value])
    return expected in choices


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
    candidates = [row for row in candidates if toolkit in row.get("_toolkits", set())]
    if params is not None:
        candidates = [row for row in candidates if parameter_match(params, row)]
    candidates = [row for row in candidates if suffix_matches(suffix, row)]
    return candidate_status(candidates), candidates


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
    prefix, separator, path = rust_item.partition("::")
    if not separator or prefix not in crates:
        return "unknown_rust_crate", []
    candidates = crates[prefix].get(f"crate::{path}", [])
    return candidate_status(candidates), candidates


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
        for key in ("path", "kind", "signature", "source_file", "source_status")
    }
