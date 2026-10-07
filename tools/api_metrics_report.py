#!/usr/bin/env python3
"""Build per-toolkit parity metrics from the published crosswalk.

The crosswalk summary (`completeness.json`) counts every Clang declaration, counts a row as
implemented even when its Rust path does not resolve, and counts a shared package once per
toolkit. These metrics fix the three problems:
- each declaration belongs to exactly one toolkit (the `src/<Area>/<Toolkit>/` of its header);
- each declaration has one status, and a claim whose Rust target does not resolve is
  `claimed_unverified`, not implemented;
- boilerplate (macro expansions, special members) is reported apart from the API that needs
  Rust code (see `api_gap_categories`), and user scope decisions come from `porting/scope.json`.

Example: `python3 tools/api_metrics_report.py` writes `porting/api-inventory/metrics.json`
and `porting/api-inventory/METRICS.md`.
"""

from __future__ import annotations

import argparse
import gzip
import json
import pathlib
from collections import Counter
from collections.abc import Iterator, Mapping

from api_gap_categories import BOILERPLATE_CATEGORIES, CATEGORY_ORDER, declaration_category
from api_metrics_markdown import metrics_markdown

ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "porting/api-inventory"
STATUS_ORDER = (
    "verified", "partial", "excluded", "upstream_unimplemented", "claimed_unverified", "unclaimed",
)
DONE_STATUSES = frozenset({"verified", "excluded", "upstream_unimplemented"})
_RESOLUTION_STATUS = {
    "implemented": "verified",
    "partial": "partial",
    "excluded": "excluded",
    "upstream_unimplemented": "upstream_unimplemented",
}
Record = Mapping[str, object]


def read_document(path: pathlib.Path) -> dict[str, object]:
    """Read JSON or gzip JSON by suffix; e.g. `read_document(Path("api-crosswalk.json.gz"))`."""
    opener = gzip.open if path.suffix == ".gz" else open
    with opener(path, "rt", encoding="utf-8") as stream:
        return json.load(stream)


def read_lines(path: pathlib.Path) -> Iterator[dict[str, object]]:
    """Stream a gzip JSON-lines table; e.g. `next(read_lines(Path("legacy-tests.jsonl.gz")))`."""
    with gzip.open(path, "rt", encoding="utf-8") as stream:
        for line in stream:
            yield json.loads(line)


def toolkit_of(source: str) -> str:
    """Owning toolkit of an OCCT path; e.g. `toolkit_of("src/ModelingData/TKG2d/Geom2d/x.hxx") == "TKG2d"`."""
    parts = source.split("/")
    return parts[2] if len(parts) > 3 and parts[0] == "src" else "unknown"


def row_status(row: Record) -> str:
    """Status that a ledger row earns for its C++ declarations after crosswalk resolution."""
    return _RESOLUTION_STATUS.get(str(row.get("resolution")), "claimed_unverified")


def declaration_statuses(crosswalk: Record) -> dict[str, str]:
    """One status per observed declaration id; the strongest claim wins over weaker ones."""
    rank = {status: index for index, status in enumerate(STATUS_ORDER)}
    statuses: dict[str, str] = {}
    for row in crosswalk["crosswalk_rows"]:
        status = row_status(row)
        for cpp_id in row["cpp_ids"]:
            if rank[status] < rank[statuses.get(cpp_id, "unclaimed")]:
                statuses[cpp_id] = status
    for declaration in crosswalk["unclaimed_cpp_declarations"]:
        statuses.setdefault(declaration["id"], "unclaimed")
    return statuses


def declaration_records(path: pathlib.Path, wanted: set[str]) -> dict[str, Record]:
    """Index the declaration table rows whose ids the crosswalk observed."""
    return {row["id"]: row for row in read_lines(path) if row["id"] in wanted}


def tally_declarations(statuses: Mapping[str, str], records: Mapping[str, Record]) -> dict[str, Counter]:
    """Count (category, status) pairs per toolkit."""
    tallies: dict[str, Counter] = {}
    for cpp_id, status in statuses.items():
        record = records.get(cpp_id)
        if record is None:
            raise ValueError(f"crosswalk declaration {cpp_id} is missing from the declaration table")
        toolkit = toolkit_of(str(record.get("source", "")))
        tallies.setdefault(toolkit, Counter())[(declaration_category(record), status)] += 1
    return tallies


def ledger_defects(crosswalk: Record) -> dict[str, Counter]:
    """Ledger rows that prove nothing: no C++ match, or a C++ match with a bad Rust target."""
    defects: dict[str, Counter] = {}
    for row in crosswalk["crosswalk_rows"]:
        if row["cpp_ids"] and row_status(row) != "claimed_unverified":
            continue
        kind = "rows_with_unresolved_rust_target" if row["cpp_ids"] else "rows_without_cpp_match"
        defects.setdefault(toolkit_of(str(row.get("source", ""))), Counter())[kind] += 1
    return defects


def gtest_counts(path: pathlib.Path) -> dict[str, Counter]:
    """Mapped and unmapped GoogleTest declarations per toolkit."""
    counts: dict[str, Counter] = {}
    for row in read_lines(path):
        if row.get("kind") == "gtest_declaration":
            counts.setdefault(str(row.get("toolkit")), Counter())[str(row.get("mapping_status"))] += 1
    return counts


def unmapped_gtests(tests: Counter) -> int:
    """Every declaration without a Rust translation: unmapped, pending_dependency (waiting for an
    unported class) and duplicate mappings alike; unmapped_gtests(Counter(mapped=2, unmapped=1)) == 1."""
    return sum(number for status, number in tests.items() if status != "mapped")


def scope_index(scope: Record) -> dict[str, dict[str, str]]:
    """Toolkit -> {scope, reason} from `porting/scope.json`."""
    index: dict[str, dict[str, str]] = {}
    for decision in scope["decisions"]:
        for toolkit in decision["toolkits"]:
            index[toolkit] = {"scope": decision["scope"], "reason": decision["reason"]}
    return index


def status_counts(tally: Counter, boilerplate: bool) -> dict[str, int]:
    """Status counts over the boilerplate or the non-boilerplate categories."""
    counts = Counter()
    for (category, status), number in tally.items():
        if (category in BOILERPLATE_CATEGORIES) == boilerplate:
            counts[status] += number
    return {status: counts[status] for status in STATUS_ORDER}


def surface_metrics(counts: Mapping[str, int]) -> dict[str, object]:
    """Totals and the done fraction of one surface (API or boilerplate)."""
    total = sum(counts.values())
    done = sum(counts[status] for status in DONE_STATUSES)
    return {
        "declarations": total,
        "done": done,
        "done_percent": round(100.0 * done / total, 1) if total else 100.0,
        "by_status": dict(counts),
    }


def open_gap(tally: Counter, boilerplate: bool) -> dict[str, int]:
    """Unclaimed or unverified declarations by category, over one surface."""
    gap = Counter()
    for (category, status), number in tally.items():
        open_status = status in ("unclaimed", "claimed_unverified")
        if open_status and (category in BOILERPLATE_CATEGORIES) == boilerplate:
            gap[category] += number
    return {category: gap[category] for category in CATEGORY_ORDER if gap[category]}


def toolkit_metrics(toolkit: str, inputs: Mapping[str, Mapping]) -> dict[str, object]:
    """Assemble the metrics of one toolkit from the prepared tallies."""
    tally = inputs["tallies"].get(toolkit, Counter())
    tests = inputs["gtests"].get(toolkit, Counter())
    scope = inputs["scope"].get(toolkit, {"scope": "port", "reason": ""})
    return {
        "scope": scope["scope"],
        "scope_reason": scope["reason"],
        "api": surface_metrics(status_counts(tally, boilerplate=False)),
        "boilerplate": surface_metrics(status_counts(tally, boilerplate=True)),
        "open_gap_by_category": open_gap(tally, boilerplate=False),
        "open_boilerplate_by_category": open_gap(tally, boilerplate=True),
        "ledger_defects": dict(inputs["defects"].get(toolkit, Counter())),
        "gtests": {"mapped": tests["mapped"], "unmapped": unmapped_gtests(tests)},
    }


def scope_totals(toolkits: Mapping[str, Mapping]) -> dict[str, dict[str, int]]:
    """Sum the API surface, gap and tests of every toolkit in each scope."""
    totals: dict[str, Counter] = {}
    for metrics in toolkits.values():
        total = totals.setdefault(str(metrics["scope"]), Counter())
        total["toolkits"] += 1
        total["api_declarations"] += metrics["api"]["declarations"]
        total["api_done"] += metrics["api"]["done"]
        total["api_partial"] += metrics["api"]["by_status"]["partial"]
        total["api_open"] += sum(metrics["open_gap_by_category"].values())
        total["gtests_mapped"] += metrics["gtests"]["mapped"]
        total["gtests_unmapped"] += metrics["gtests"]["unmapped"]
    return {scope: dict(total) for scope, total in sorted(totals.items())}


def build_metrics(paths: Mapping[str, pathlib.Path]) -> dict[str, object]:
    """Join the crosswalk, declaration table, legacy tests and scope into one document."""
    crosswalk = read_document(paths["crosswalk"])
    statuses = declaration_statuses(crosswalk)
    records = declaration_records(paths["declarations"], set(statuses))
    inputs = {
        "tallies": tally_declarations(statuses, records),
        "defects": ledger_defects(crosswalk),
        "gtests": gtest_counts(paths["legacy"]),
        "scope": scope_index(read_document(paths["scope"])),
    }
    names = sorted(set(inputs["tallies"]) | set(inputs["gtests"]) - {"None"})
    toolkits = {name: toolkit_metrics(name, inputs) for name in names}
    return {
        "schema_version": 1,
        "crosswalk_input_sha256": crosswalk.get("input_sha256"),
        "totals_by_scope": scope_totals(toolkits),
        "toolkits": toolkits,
    }


def argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--crosswalk", type=pathlib.Path, default=INVENTORY / "api-crosswalk.json.gz")
    parser.add_argument("--declarations", type=pathlib.Path, default=INVENTORY / "occt-declarations.jsonl.gz")
    parser.add_argument("--legacy", type=pathlib.Path, default=INVENTORY / "legacy-tests.jsonl.gz")
    parser.add_argument("--scope", type=pathlib.Path, default=ROOT / "porting/scope.json")
    parser.add_argument("--json", type=pathlib.Path, default=INVENTORY / "metrics.json")
    parser.add_argument("--markdown", type=pathlib.Path, default=INVENTORY / "METRICS.md")
    return parser


def main() -> int:
    args = argument_parser().parse_args()
    paths = {key: getattr(args, key) for key in ("crosswalk", "declarations", "legacy", "scope")}
    metrics = build_metrics(paths)
    args.json.write_text(json.dumps(metrics, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    args.markdown.write_text(metrics_markdown(metrics), encoding="utf-8")
    print(f"Wrote {args.json} and {args.markdown}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
