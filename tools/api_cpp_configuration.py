"""Account for ledger rows whose C++ declaration the captured configuration does not compile.

The Clang capture builds one Linux configuration. Code under `#ifdef _WIN32`, `_DEBUG`,
`OCCT_HANDLE_NOCAST` or a compiler fallback branch, and headers that need Windows SDK
headers (OSD_WNT.hxx), produce no declaration, so a row that describes them can never
match. Such a row declares

    "cpp_configuration": {"condition": "defined(_WIN32)", "line": 90}

where `condition` names the preprocessor condition that enables the declaration and
`line` is the declaration's line in the row's `source` header. The crosswalk accepts the
row only when the capture proves the declaration is absent from the configuration:
`line` lies inside a recorded skipped preprocessor range of `source`, or the whole
header was not parsed (status `parse_failed` or `missing_compile_configuration`), and no
declaration matches the row's `upstream_id`. The row then resolves with C++ status
`inactive_configuration` and claims no compiled declaration.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from api_inventory.inputs import JSONValue

INACTIVE = "inactive_configuration"
CONFIGURATION_FAILURES = frozenset(
    {"invalid_cpp_configuration", "unverified_configuration", "configuration_conflict"}
)
UNPARSED_HEADER_STATUSES = frozenset({"parse_failed", "missing_compile_configuration"})


@dataclass(frozen=True)
class ConfigurationEvidence:
    """Skipped line ranges per header and the headers the capture could not parse."""

    skipped_ranges: dict[str, list[tuple[int, int]]] = field(default_factory=dict)
    unparsed_headers: frozenset[str] = frozenset()

    def proves_inactive(self, source: str, line: int) -> bool:
        """True when `source:line` is compiled out; e.g. `evidence.proves_inactive(h, 90)`."""
        if source in self.unparsed_headers:
            return True
        ranges = self.skipped_ranges.get(source, [])
        return any(start <= line <= end for start, end in ranges)


def configuration_evidence(
    skipped_rows: list[dict[str, JSONValue]], header_rows: list[dict[str, JSONValue]]
) -> ConfigurationEvidence:
    """Index the capture's skipped ranges and header statuses.

    Example: `configuration_evidence(read_jsonl(skipped), read_jsonl(headers))`.
    """
    ranges: dict[str, list[tuple[int, int]]] = {}
    for row in skipped_rows:
        span = skipped_span(row)
        if span is not None:
            ranges.setdefault(span[0], []).append((span[1], span[2]))
    unparsed = frozenset(
        str(row.get("path"))
        for row in header_rows
        if row.get("status") in UNPARSED_HEADER_STATUSES and isinstance(row.get("path"), str)
    )
    return ConfigurationEvidence(ranges, unparsed)


def skipped_span(row: dict[str, JSONValue]) -> tuple[str, int, int] | None:
    """(file, first line, last line) of one skipped-range record, or None if malformed."""
    start, end = row.get("start"), row.get("end")
    if not isinstance(start, dict) or not isinstance(end, dict):
        return None
    path, first, last = start.get("file"), start.get("line"), end.get("line")
    if not isinstance(path, str) or path != end.get("file"):
        return None
    if not isinstance(first, int) or not isinstance(last, int):
        return None
    return path, first, last


def configuration_claim(row: dict[str, JSONValue]) -> tuple[int | None, str | None]:
    """Validate `cpp_configuration`; return (line, None) or (None, error message)."""
    value = row.get("cpp_configuration")
    expected = 'an object {"condition": non-empty string, "line": positive integer}'
    if not isinstance(value, dict) or set(value) != {"condition", "line"}:
        return None, f"cpp_configuration {value!r}: expected {expected}"
    condition, line = value.get("condition"), value.get("line")
    if not isinstance(condition, str) or not condition.strip():
        return None, f"cpp_configuration {value!r}: expected {expected}"
    if not isinstance(line, int) or isinstance(line, bool) or line < 1:
        return None, f"cpp_configuration {value!r}: expected {expected}"
    return line, None


def configured_cpp_status(
    row: dict[str, JSONValue], cpp_status: str, evidence: ConfigurationEvidence | None
) -> str:
    """C++ status of a row after its configuration claim; unchanged for unclaimed rows.

    Example: `configured_cpp_status(row, "missing", evidence) == "inactive_configuration"`.
    """
    if "cpp_configuration" not in row:
        return cpp_status
    line, error = configuration_claim(row)
    if line is None or error is not None:
        return "invalid_cpp_configuration"
    if cpp_status != "missing":
        # A compiled declaration matches the row: the configuration claim is wrong.
        return "configuration_conflict"
    source = row.get("source")
    known = evidence is not None and isinstance(source, str)
    if known and evidence.proves_inactive(source, line):
        return INACTIVE
    return "unverified_configuration"
