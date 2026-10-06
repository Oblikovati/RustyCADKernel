#!/usr/bin/env python3
"""Bind historical native source-context hints to a new capture without reusing parse credit."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from api_inventory.context_replay import (
    CompileContext,
    compile_contexts,
    context_flags_hash,
    git_revision,
)
from api_inventory.context_replay_plan import capture_inputs, verify_capture_source
from api_inventory.inputs import (
    JSONObject,
    digest,
    object_rows,
    read_object,
    text_field,
)


def rebind_row(
    row: JSONObject, source: Path, contexts: dict[str, CompileContext]
) -> JSONObject:
    """Require unchanged native sources and a current real compile command before replay."""
    name = text_field(row, "compile_source")
    context = contexts.get(name)
    if context is None or context.toolkit != row.get("compile_toolkit"):
        raise ValueError(
            f"compile source {name!r}: expected current command for toolkit {row.get('compile_toolkit')!r}"
        )
    verify_hint_sources(row, source, name)
    return {
        **row,
        "previous_compile_flags_sha256": row.get("compile_flags_sha256"),
        "compile_flags_sha256": context_flags_hash(context.flags),
        "context_basis": "historical_source_hint_requires_fresh_success",
    }


def verify_hint_sources(row: JSONObject, source: Path, name: str) -> None:
    """Require exact historical source hashes before reusing a context hint."""
    for key, filename in [
        ("header_sha256", text_field(row, "path")),
        ("compile_source_sha256", name),
    ]:
        path = (source / filename).resolve()
        if not path.is_relative_to(source) or digest(path) != row.get(key):
            raise ValueError(
                f"native source {path}: expected historical source SHA256 {row.get(key)!r}"
            )


def selected_rows(prior: JSONObject, capture: JSONObject) -> list[JSONObject]:
    """Use only historical contexts for failed headers in this explicit capture scope."""
    failures = {
        text_field(row, "path")
        for row in object_rows(capture["results"], "capture results")
        if row.get("status") == "parse_failed"
    }
    return [
        row
        for row in object_rows(prior["rows"], "prior plan rows")
        if row.get("path") in failures
    ]


def rebound_plan(
    prior_path: Path, capture_path: Path, source: Path, database: Path, extractor: Path
) -> JSONObject:
    """Create new provenance; e.g. `rebound_plan(prior, capture, source, database, extractor)`."""
    prior, capture = read_object(prior_path), read_object(capture_path)
    if prior.get("occt_revision") != git_revision(source):
        raise ValueError(
            f"OCCT revision: expected historical revision {prior.get('occt_revision')!r}"
        )
    capture_inputs(capture, database, extractor)
    verify_capture_source(capture, source)
    contexts = compile_contexts(database, source)
    rows = [rebind_row(row, source, contexts) for row in selected_rows(prior, capture)]
    return {
        **rebound_identity(prior_path, capture_path, database, extractor),
        "occt_revision": prior["occt_revision"],
        "original_fingerprint": capture["fingerprint"],
        "rows": rows,
    }


def rebound_identity(
    prior_path: Path, capture_path: Path, database: Path, extractor: Path
) -> JSONObject:
    """Bind new replay inputs without inheriting previous parse credit."""
    return {
        "schema": "occt-header-context-replay-plan-v1",
        "historical_hint_plan_sha256": digest(prior_path),
        "database_sha256": digest(database),
        "original_extractor_sha256": digest(extractor),
        "original_capture_sha256": digest(capture_path),
        "historical_parse_credit": False,
        "coverage_proof": False,
    }


def main() -> None:
    """Write a fresh plan; follow with replay_occt_header_context.py run."""
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ["prior-plan", "capture", "source", "database", "extractor", "output"]:
        parser.add_argument(f"--{name}", type=Path, required=True)
    args = parser.parse_args()
    result = rebound_plan(
        args.prior_plan,
        args.capture,
        args.source.resolve(),
        args.database,
        args.extractor,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("x") as stream:
        stream.write(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(
        f"Bound {len(result['rows'])} historical hints; every result requires fresh successful parsing"
    )


if __name__ == "__main__":
    main()
