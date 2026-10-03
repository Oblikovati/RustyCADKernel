#!/usr/bin/env python3
"""Build a deduplicated compiler inventory with explicit header and configuration gaps."""

from __future__ import annotations

import argparse
import json
import pathlib

from api_inventory.inputs import (
    JSONObject,
    digest,
    object_rows,
    read_object,
    text_field,
    verify_sources,
)
from api_inventory.dependencies import header_tree_fingerprint
from api_inventory.context_retries import merge_context_retries
from api_inventory.merge import (
    InventoryRecords,
    consume_capture,
    coverage_rows,
    inventory_summary,
    write_jsonl_gzip,
)

ROOT = pathlib.Path(__file__).resolve().parents[1]


def included_file_hashes(
    records: InventoryRecords, source: pathlib.Path
) -> list[JSONObject]:
    """Bind parsed SDK and source headers to their observed file contents."""
    rows: list[JSONObject] = []
    for name in sorted(records.includes):
        originals = {
            alias: original for original, alias in records.source_aliases.items()
        }
        path = pathlib.Path(originals.get(name, str(source / name)))
        rows.append(
            {"source": name, "sha256": digest(path) if path.is_file() else None}
        )
    return rows


def inventory_tables(
    records: InventoryRecords, headers: list[JSONObject], source: pathlib.Path
) -> dict[str, list[JSONObject]]:
    """Export declarations, explicit gaps and all contributing file identities."""
    return {
        "declarations": sorted(
            records.declarations.values(), key=lambda row: str(row["id"])
        ),
        "macros": sorted(records.macros.values(), key=lambda row: str(row["id"])),
        "headers": headers,
        "diagnostics": records.diagnostics,
        "unsupported": records.unsupported,
        "skipped-ranges": records.skipped_ranges,
        "compiler-summaries": records.summaries,
        "context-retries": records.context_retries,
        "included-files": included_file_hashes(records, source),
    }


def export_inventory(
    capture: JSONObject,
    records: InventoryRecords,
    source: pathlib.Path,
    output: pathlib.Path,
) -> None:
    """Write inventory and failure evidence; e.g. `export_inventory(capture, records, source, output)`."""
    output.mkdir(parents=True, exist_ok=True)
    headers = coverage_rows(capture, records)
    tables = inventory_tables(records, headers, source)
    artifacts = {
        key: write_jsonl_gzip(output / f"occt-{key}.jsonl.gz", rows)
        for key, rows in tables.items()
    }
    summary = inventory_summary(capture, records, headers)
    summary["artifacts"] = artifacts
    (output / "occt-manifest.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(
        f'Observed {summary["api_declarations"]} C++ API declarations; header statuses: {summary["headers_by_status"]}'
    )


def verify_recorded_sdks(inputs: JSONObject) -> None:
    """Reject changed native SDK files; e.g. `verify_recorded_sdks(inputs)`."""
    for category in ("sdk_headers", "external_headers"):
        for row in object_rows(inputs.get(category), category):
            actual = header_tree_fingerprint(pathlib.Path(text_field(row, "root")))
            if row != actual:
                raise ValueError(
                    f"{category} {row!r}: expected unchanged SDK tree, got {actual!r}"
                )
    for library in object_rows(inputs.get("compiler_libraries"), "compiler libraries"):
        if digest(pathlib.Path(text_field(library, "path"))) != library.get("sha256"):
            raise ValueError(f"compiler library {library!r}: expected recorded SHA256")


def verify_capture_inputs(inputs: JSONObject) -> pathlib.Path:
    """Reject stale recorded files and source revision; e.g. `verify_capture_inputs(inputs)`."""
    paths = inputs.get("file_inputs")
    if not isinstance(paths, dict):
        raise ValueError(f"file_inputs: expected path object, got {paths!r}")
    for key, value in paths.items():
        if not isinstance(value, str) or digest(pathlib.Path(value)) != inputs.get(
            f"{key}_sha256"
        ):
            raise ValueError(f"{key} input {value!r}: expected recorded SHA256")
    verify_recorded_sdks(inputs)
    source = pathlib.Path(text_field(inputs, "source"))
    verify_sources(source, read_object(pathlib.Path(text_field(paths, "manifest"))))
    return source


def arguments() -> argparse.Namespace:
    """Read merger paths; e.g. `merge_occt_inventory.py --help`."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=pathlib.Path, required=True)
    parser.add_argument("--context-retries", type=pathlib.Path)
    parser.add_argument(
        "--output", type=pathlib.Path, default=ROOT / "porting/api-inventory"
    )
    return parser.parse_args()


def main() -> None:
    """Merge immutable evidence; e.g. `merge_occt_inventory.py --capture target/api-proof/occt-inventory`."""
    args = arguments()
    capture = read_object(args.capture / "capture-manifest.json")
    inputs = capture["inputs"]
    if not isinstance(inputs, dict) or not isinstance(inputs.get("source"), str):
        raise ValueError("capture inputs: expected source path string")
    source = verify_capture_inputs(inputs)
    records = InventoryRecords(
        source_aliases={
            text_field(
                inputs, "generated_version_path"
            ): "@generated/Standard_Version.hxx"
        }
    )
    for result in object_rows(capture["results"], "capture results"):
        consume_capture(records, result, args.capture, source)
    if args.context_retries:
        merge_context_retries(
            records,
            capture,
            args.capture / "capture-manifest.json",
            args.context_retries,
            source,
        )
    export_inventory(capture, records, source, args.output)


if __name__ == "__main__":
    main()
