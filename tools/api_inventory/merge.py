"""Normalize compiler records while preserving unsupported and incomplete evidence."""

from __future__ import annotations

import gzip
import hashlib
import json
import pathlib
from collections import Counter
from dataclasses import dataclass, field
from functools import lru_cache
from typing import Iterator, cast

from .inputs import JSONObject, JSONValue, digest, object_rows, text_field
from .template_identity import normalize_template_signature

CALLABLE_DECLARATION_KINDS = frozenset(
    {
        "Function",
        "FunctionTemplate",
        "CXXMethod",
        "CXXConstructor",
        "CXXDestructor",
        "CXXConversion",
    }
)


@dataclass
class InventoryRecords:
    source_aliases: dict[str, str] = field(default_factory=dict)
    site_keys: dict[str, set[str]] = field(default_factory=dict)
    declarations: dict[str, JSONObject] = field(default_factory=dict)
    macros: dict[str, JSONObject] = field(default_factory=dict)
    includes: set[str] = field(default_factory=set)
    skipped_ranges: list[JSONObject] = field(default_factory=list)
    diagnostics: list[JSONObject] = field(default_factory=list)
    unsupported: list[JSONObject] = field(default_factory=list)
    summaries: list[JSONObject] = field(default_factory=list)
    context_retries: list[JSONObject] = field(default_factory=list)


def json_lines(path: pathlib.Path) -> Iterator[JSONObject]:
    """Read compiler events without loading the entire file; e.g. `json_lines(raw)`."""
    with path.open() as stream:
        for number, line in enumerate(stream, 1):
            value: JSONValue = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError(
                    f"{path}:{number}: expected JSON object, got {value!r}"
                )
            yield value


@lru_cache(maxsize=65536)
def source_identity(filename: str, source: pathlib.Path) -> str:
    """Keep source-relative identities and explicit external locations."""
    path = pathlib.Path(filename).resolve()
    if path.is_relative_to(source):
        return path.relative_to(source).as_posix()
    return str(path)


def declaration_identity(row: JSONObject) -> str:
    """Use signatures for callable overloads while retaining USR identity elsewhere."""
    usr = row.get("usr")
    signature = row.get("identity_signature", row.get("signature"))
    components: list[JSONValue] = [usr, row.get("kind")]
    if row.get("kind") in CALLABLE_DECLARATION_KINDS:
        template = row.get("template_signature")
        normalized = (
            normalize_template_signature(template).normalized
            if isinstance(template, str)
            else template
        )
        components.extend([signature, normalized])
    if not usr:
        components = [
            row.get("source"),
            row.get("expansion_location"),
            row.get("spelling_location"),
            row.get("parent_usr"),
            signature,
            row.get("qualified_name"),
            row.get("kind"),
        ]
    return hashlib.sha256(json.dumps(components, sort_keys=True).encode()).hexdigest()


def declaration_site(row: JSONObject, header: str) -> JSONObject:
    """Retain macro expansion and declaration locations; e.g. `declaration_site(row, header)`."""
    keys = (
        "source",
        "line",
        "column",
        "offset",
        "spelling_location",
        "expansion_location",
    )
    site = {key: row[key] for key in keys if key in row}
    site["observed_in"] = header
    return site


def compare_declarations(
    records: InventoryRecords, first: JSONObject, other: JSONObject, header: str
) -> None:
    """Retain mismatches as blockers instead of choosing a convenient declaration."""
    callable_kind = first.get("kind") in CALLABLE_DECLARATION_KINDS
    signature_keys = (
        ["identity_signature"]
        if callable_kind and "identity_signature" in first
        else ["signature", "identity_signature"]
    )
    for key in (
        *signature_keys,
        "api_visible",
        "exclusion_reason",
        "api_resolution",
        "unstable_identity",
    ):
        if first.get(key) != other.get(key):
            first["inventory_conflict"] = True
            conflict = {
                "reason": "conflicting_declaration",
                "id": first["id"],
                "field": key,
                "first": first.get(key),
                "other": other.get(key),
                "header": header,
            }
            records.unsupported.append(conflict)


def merge_default_arguments(existing: JSONObject, row: JSONObject) -> None:
    """Preserve default-argument AST observations without using them as identity."""
    defaults = row.get("default_arguments")
    if not isinstance(defaults, list):
        return
    observations = cast(list[JSONValue], existing["default_argument_observations"])
    if defaults not in observations:
        observations.append(defaults)
    existing["default_argument_capture"] = "complete"


def merge_declaration(records: InventoryRecords, row: JSONObject, header: str) -> None:
    """Merge redeclarations without losing conflicting signatures or visibility."""
    mark_unsupported_template_identity(records, row, header)
    identifier = declaration_identity(row)
    site = declaration_site(row, header)
    site_key = json.dumps(site, sort_keys=True)
    default_capture = default_argument_capture_status(row)
    if identifier not in records.declarations:
        records.site_keys[identifier] = {site_key}
        row.update(
            {
                "id": identifier,
                "sites": [site],
                "signature_variants": [row.get("signature")],
                "template_signature_variants": [row.get("template_signature")],
                "default_argument_capture": default_capture,
                "default_argument_observations": (
                    [row["default_arguments"]]
                    if isinstance(row.get("default_arguments"), list)
                    else []
                ),
                "stable_identity": bool(row.get("usr"))
                and not bool(row.get("unstable_identity")),
            }
        )
        records.declarations[identifier] = row
        return
    existing = records.declarations[identifier]
    sites = cast(list[JSONObject], existing["sites"])
    if site_key not in records.site_keys[identifier]:
        records.site_keys[identifier].add(site_key)
        sites.append(site)
    signatures = cast(list[JSONValue], existing["signature_variants"])
    if row.get("signature") not in signatures:
        signatures.append(row.get("signature"))
    template_signatures = cast(list[JSONValue], existing["template_signature_variants"])
    if row.get("template_signature") not in template_signatures:
        template_signatures.append(row.get("template_signature"))
    merge_default_arguments(existing, row)
    compare_declarations(records, existing, row, header)


def mark_unsupported_template_identity(
    records: InventoryRecords, row: JSONObject, header: str
) -> None:
    """Withhold declarations whose template signature cannot be safely normalized."""
    signature = row.get("template_signature")
    if not isinstance(signature, str) or not signature.strip():
        return
    result = normalize_template_signature(signature)
    if result.supported:
        return
    row["api_resolution"] = "unsupported_template_signature_normalization"
    row["unstable_identity"] = True
    records.unsupported.append(
        {
            "reason": result.reason,
            "id": row.get("usr"),
            "template_signature": signature,
            "header": header,
        }
    )


def default_argument_capture_status(row: JSONObject) -> str:
    if isinstance(row.get("default_arguments"), list):
        return "complete"
    if isinstance(row.get("parameter_types"), list):
        return "unavailable"
    return "not_applicable"


def merge_macro(records: InventoryRecords, row: JSONObject, header: str) -> None:
    """Record macro candidates separately from C++ declarations."""
    key = json.dumps([row.get("source"), row.get("name"), row.get("location")])
    identifier = hashlib.sha256(key.encode()).hexdigest()
    if identifier not in records.macros:
        row.update({"id": identifier, "observed_in": header})
        records.macros[identifier] = row


def preserve_evidence_event(
    records: InventoryRecords, event: JSONObject, header: str
) -> None:
    """Retain every nondeclaration event; unknown events block completeness."""
    collections = {
        "skipped_range": records.skipped_ranges,
        "diagnostic": records.diagnostics,
        "summary": records.summaries,
        "unsupported": records.unsupported,
    }
    target = collections.get(str(event.get("event")), records.unsupported)
    target.append({**event, "observed_in": header})


def normalize_locations(
    event: JSONObject, source: pathlib.Path, aliases: dict[str, str]
) -> None:
    """Use checkout-independent locations; e.g. `normalize_locations(event, root, {})`."""
    for key, value in event.items():
        if isinstance(value, dict):
            normalize_locations(value, source, aliases)
        elif isinstance(value, str) and key in {"source", "file", "main_header"}:
            event[key] = aliases.get(value, source_identity(value, source))


def consume_event(
    records: InventoryRecords, event: JSONObject, header: str, source: pathlib.Path
) -> None:
    """Dispatch every event or make an unsupported-event gap explicit."""
    kind = event.get("event")
    normalize_locations(event, source, records.source_aliases)
    if kind == "declaration":
        merge_declaration(records, event, header)
    elif kind == "macro":
        merge_macro(records, event, header)
    elif kind == "header_seen":
        records.includes.add(text_field(event, "source"))
    else:
        preserve_evidence_event(records, event, header)


def capture_diagnostic(
    records: InventoryRecords, result: JSONObject, event: str
) -> None:
    """Preserve compiler failures even when no AST could be emitted."""
    records.diagnostics.append(
        {"observed_in": result["path"], "event": event, "text": result.get("stderr")}
    )


def verify_capture_summary(
    records: InventoryRecords, previous: int, header: str
) -> None:
    """Reject silent truncated output; e.g. `verify_capture_summary(records, 0, header)`."""
    summaries = records.summaries[previous:]
    complete = (
        len(summaries) == 1 and summaries[0].get("complete_for_emit_scope") is True
    )
    if not complete:
        records.unsupported.append(
            {"reason": "incomplete_compiler_summary", "header": header}
        )


def preserve_failed_capture(
    records: InventoryRecords,
    artifact: pathlib.Path,
    result: JSONObject,
    source: pathlib.Path,
) -> None:
    """Keep failed-parse diagnostics without granting its partial AST API credit."""
    capture_diagnostic(records, result, "parse_failed")
    for event in json_lines(artifact):
        if event.get("event") in {"diagnostic", "summary"}:
            normalize_locations(event, source, records.source_aliases)
            records.diagnostics.append(
                {**event, "observed_in": result["path"], "failed_parse": True}
            )


def consume_capture(
    records: InventoryRecords,
    result: JSONObject,
    root: pathlib.Path,
    source: pathlib.Path,
) -> None:
    """Reject changed artifacts and withhold rows from failed parses."""
    artifact = root / text_field(result, "artifact")
    if digest(artifact) != result.get("output_sha256"):
        raise ValueError(
            f'{artifact}: expected captured hash {result.get("output_sha256")}'
        )
    if result.get("status") != "parsed":
        preserve_failed_capture(records, artifact, result, source)
        return
    summary_count = len(records.summaries)
    for event in json_lines(artifact):
        consume_event(records, event, text_field(result, "path"), source)
    verify_capture_summary(records, summary_count, text_field(result, "path"))
    if result.get("stderr"):
        capture_diagnostic(records, result, "compiler_stderr")


def exclude_implementation_only_declarations(records: InventoryRecords) -> int:
    """Withhold source-TU definitions lacking any header declaration site."""
    excluded = 0
    for declaration in records.declarations.values():
        sites = cast(list[JSONObject], declaration["sites"])
        sources = [site.get("source") for site in sites]
        has_header_site = any(
            not is_implementation_source(str(path)) for path in sources
        )
        if not sources or has_header_site:
            continue
        declaration["api_visible"] = False
        declaration["api_resolution"] = "implementation_only_without_header_declaration"
        declaration["exclusion_reason"] = "implementation_only_no_header_site"
        excluded += 1
    return excluded


def is_implementation_source(filename: str) -> bool:
    """Recognize translation-unit source extensions used by OCCT."""
    return pathlib.Path(filename).suffix in {".c", ".cc", ".cpp", ".cxx"}


def exclude_implementation_only_macros(records: InventoryRecords) -> int:
    """Keep source-TU macro evidence while withholding it from API candidates."""
    excluded = 0
    for macro in records.macros.values():
        source = macro.get("source")
        if not isinstance(source, str) or not is_implementation_source(source):
            continue
        if macro.get("api_candidate") is not True:
            continue
        macro["api_candidate"] = False
        macro["api_resolution"] = "implementation_only_macro_without_header_definition"
        macro["exclusion_reason"] = "implementation_only_no_header_site"
        excluded += 1
    return excluded


def coverage_rows(capture: JSONObject, records: InventoryRecords) -> list[JSONObject]:
    """Keep missing configurations and unobserved fragments in the denominator."""
    results = {
        text_field(row, "path"): row
        for row in object_rows(capture["results"], "results")
    }
    rows = object_rows(capture["header_candidates"], "header candidates")
    for row in rows:
        path = text_field(row, "path")
        if path in results:
            row["status"] = results[path]["status"]
            if "original_status" in results[path]:
                row["original_status"] = results[path]["original_status"]
        elif row["status"] == "fragment_pending" and path in records.includes:
            row["status"] = "included_fragment"
    return rows


def write_jsonl_gzip(path: pathlib.Path, rows: list[JSONObject]) -> JSONObject:
    """Write deterministic compressed inventory evidence; e.g. `write_jsonl_gzip(path, rows)`."""
    with path.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as stream:
            for row in rows:
                stream.write(
                    (
                        json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n"
                    ).encode()
                )
    return {"path": path.name, "sha256": digest(path), "rows": len(rows)}


def declaration_counts(records: InventoryRecords) -> JSONObject:
    """Separate API candidates from excluded/internal compiler declarations."""
    public = [
        row for row in records.declarations.values() if row.get("api_visible") is True
    ]
    return {
        "declarations": len(records.declarations),
        "api_declarations": len(public),
        "macro_definitions": len(records.macros),
        "macro_candidates": sum(
            row.get("api_candidate") is True for row in records.macros.values()
        ),
        "declarations_by_kind": dict(Counter(str(row.get("kind")) for row in public)),
    }


def inventory_summary(
    capture: JSONObject, records: InventoryRecords, headers: list[JSONObject]
) -> JSONObject:
    """Report observed declarations without claiming unparsed API coverage."""
    statuses = Counter(text_field(row, "status") for row in headers)
    return {
        "schema": "occt-declaration-inventory-v1",
        "inputs": capture["inputs"],
        "capture_fingerprint": capture["fingerprint"],
        "context_retry_sha256": capture.get("context_retry_sha256"),
        "expected_headers": len(headers),
        "headers_by_status": dict(statuses),
        **declaration_counts(records),
        "unsupported_records": len(records.unsupported),
        "diagnostics": len(records.diagnostics),
        "skipped_ranges": len(records.skipped_ranges),
        "configuration_matrix_complete": False,
        "declaration_inventory_complete": False,
        "coverage_proof": False,
    }
