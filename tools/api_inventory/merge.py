"""Normalize compiler records while preserving unsupported and incomplete evidence."""

from __future__ import annotations

import gzip
import hashlib
import json
import pathlib
from collections import Counter
from dataclasses import dataclass, field
from typing import Iterator

from .inputs import JSONObject, JSONValue, digest, object_rows, text_field


@dataclass
class InventoryRecords:
    source_aliases: dict[str, str] = field(default_factory=dict)
    declarations: dict[str, JSONObject] = field(default_factory=dict)
    macros: dict[str, JSONObject] = field(default_factory=dict)
    includes: set[str] = field(default_factory=set)
    skipped_ranges: list[JSONObject] = field(default_factory=list)
    diagnostics: list[JSONObject] = field(default_factory=list)
    unsupported: list[JSONObject] = field(default_factory=list)
    summaries: list[JSONObject] = field(default_factory=list)


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


def source_identity(filename: str, source: pathlib.Path) -> str:
    """Keep source-relative identities and explicit external locations."""
    path = pathlib.Path(filename).resolve()
    if path.is_relative_to(source):
        return path.relative_to(source).as_posix()
    return str(path)


def declaration_identity(row: JSONObject) -> str:
    """Use compiler USRs; location fallback remains explicitly unverified."""
    usr = row.get("usr")
    components = [usr, row.get("kind")]
    if not usr:
        components = [
            row.get("source"),
            row.get("expansion_location"),
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
    for key in ("signature", "api_visible", "exclusion_reason"):
        if first.get(key) != other.get(key):
            conflict = {
                "reason": "conflicting_declaration",
                "id": first["id"],
                "field": key,
                "first": first.get(key),
                "other": other.get(key),
                "header": header,
            }
            records.unsupported.append(conflict)


def merge_declaration(records: InventoryRecords, row: JSONObject, header: str) -> None:
    """Merge redeclarations without losing conflicting signatures or visibility."""
    identifier = declaration_identity(row)
    site = declaration_site(row, header)
    if identifier not in records.declarations:
        row.update(
            {"id": identifier, "sites": [site], "stable_identity": bool(row.get("usr"))}
        )
        records.declarations[identifier] = row
        return
    existing = records.declarations[identifier]
    sites = object_rows(existing["sites"], "declaration sites")
    if site not in sites:
        sites.append(site)
    compare_declarations(records, existing, row, header)


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
        capture_diagnostic(records, result, "parse_failed")
        return
    summary_count = len(records.summaries)
    for event in json_lines(artifact):
        consume_event(records, event, text_field(result, "path"), source)
    verify_capture_summary(records, summary_count, text_field(result, "path"))
    if result.get("stderr"):
        capture_diagnostic(records, result, "compiler_stderr")


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
        "macro_candidates": len(records.macros),
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
