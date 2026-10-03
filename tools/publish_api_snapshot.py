#!/usr/bin/env python3
"""Publish reproducible compressed inventory evidence and a small readable summary."""

from __future__ import annotations

import argparse
import gzip
import json
import pathlib
import shutil

from api_inventory.inputs import JSONObject, digest, read_object, object_rows
from api_tool_hashes import implementation_hashes_match

ROOT = pathlib.Path(__file__).resolve().parents[1]
DETAIL_TABLES = {
    "crosswalk_rows",
    "unresolved_rows",
    "unclaimed_cpp_declarations",
    "excluded_rows",
    "partial_rows",
    "upstream_unimplemented_rows",
    "duplicate_cpp_claims",
}


def compress_snapshot(source: pathlib.Path, target: pathlib.Path) -> JSONObject:
    """Bind deterministic compressed bytes to the original; e.g. `compress_snapshot(raw, archived)`."""
    with source.open("rb") as incoming, target.open("wb") as outgoing:
        with gzip.GzipFile(
            filename="", mode="wb", fileobj=outgoing, mtime=0
        ) as compressed:
            shutil.copyfileobj(incoming, compressed)
    return {
        "path": target.name,
        "sha256": digest(target),
        "uncompressed_sha256": digest(source),
    }


def snapshot_summary(
    document: JSONObject, rust: pathlib.Path, artifacts: JSONObject
) -> JSONObject:
    """Keep source provenance beside the small counts; e.g. `snapshot_summary(report, rust, artifacts)`."""
    public = read_object(rust)
    crates = object_rows(public.get("crates", []), "Rust crates")
    summary = {
        key: value for key, value in document.items() if key not in DETAIL_TABLES
    }
    summary.update(
        {
            "artifacts": artifacts,
            "rust_capture_revision": public.get("capture_workspace_revision"),
            "rust_capture_inputs": public.get("capture_inputs"),
            "rust_crates": len(crates),
            "rust_public_paths": sum(
                len(object_rows(crate.get("declarations", []), "Rust declarations"))
                for crate in crates
            ),
        }
    )
    return summary


def publish_snapshot(
    rust: pathlib.Path,
    report: pathlib.Path,
    output: pathlib.Path,
    project_root: pathlib.Path = ROOT,
) -> None:
    """Persist the exact crosswalk inputs; e.g. `publish_snapshot(rust, report, output)`."""
    document = read_object(report)
    fingerprints = document.get("input_sha256")
    if not isinstance(fingerprints, dict) or fingerprints.get("rust") != digest(rust):
        raise ValueError(
            f"Rust snapshot {rust}: expected the exact crosswalk input hash, got {fingerprints!r}"
        )
    tool_hashes = document.get("tool_implementation_sha256")
    if not implementation_hashes_match(project_root, tool_hashes):
        raise ValueError("API report implementation hashes are missing or stale")
    rust_tools = read_object(rust).get("tool_implementation_sha256")
    expected_rust_tools = (
        {"rust_normalizer": tool_hashes["rust_normalizer"]}
        if isinstance(tool_hashes, dict)
        else None
    )
    if rust_tools != expected_rust_tools:
        raise ValueError("Rust snapshot normalizer hashes differ from the API report")
    output.mkdir(parents=True, exist_ok=True)
    artifacts = {
        label: compress_snapshot(path, output / f"{label}.json.gz")
        for label, path in (("rust-api", rust), ("api-crosswalk", report))
    }
    summary = snapshot_summary(document, rust, artifacts)
    (output / "completeness.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n"
    )


def main() -> None:
    """Publish generated reports; e.g. `publish_api_snapshot.py --help`."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--rust",
        type=pathlib.Path,
        default=ROOT / "target/api-proof/rust-api-inventory.json",
    )
    parser.add_argument(
        "--report",
        type=pathlib.Path,
        default=ROOT / "target/api-proof/api-coverage-report.json",
    )
    parser.add_argument(
        "--output", type=pathlib.Path, default=ROOT / "porting/api-inventory"
    )
    args = parser.parse_args()
    publish_snapshot(args.rust, args.report, args.output)


if __name__ == "__main__":
    main()
