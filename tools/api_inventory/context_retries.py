"""Reconcile authentic translation-unit retries without hiding failed standalone probes."""

from __future__ import annotations

import pathlib
from .inputs import JSONObject, digest, object_rows, read_object, text_field
from .merge import InventoryRecords, consume_capture


def verify_retry_parent(
    retry: JSONObject, capture: JSONObject, path: pathlib.Path
) -> None:
    """Require the original capture and compiler identity; e.g. `verify_retry_parent(retry, capture, path)`."""
    if retry.get("original_capture_sha256") != digest(path) or retry.get(
        "original_fingerprint"
    ) != capture.get("fingerprint"):
        raise ValueError(
            f"retry parent {retry.get('original_capture')!r}: expected exact capture {path}"
        )
    inputs = capture.get("inputs")
    for key in ("database_sha256", "extractor_sha256"):
        if not isinstance(inputs, dict) or retry.get(key) != inputs.get(key):
            raise ValueError(
                f"retry {key} {retry.get(key)!r}: expected original compiler configuration"
            )


def verify_retry_source(row: JSONObject, source: pathlib.Path) -> None:
    """Bind each successful context to its pinned source and target header."""
    paths = (
        (source / text_field(row, "path"), "header_sha256"),
        (pathlib.Path(text_field(row, "compile_source")), "compile_source_sha256"),
    )
    for path, key in paths:
        if not path.resolve().is_relative_to(source) or digest(path) != row.get(key):
            raise ValueError(
                f"retry source {path}: expected pinned source hash {row.get(key)!r}"
            )
    if row.get("target_header_seen") is not True:
        raise ValueError(
            f"retry header {row.get('path')!r}: expected observed target header"
        )


def reconcile_retry_row(
    records: InventoryRecords,
    row: JSONObject,
    originals: dict[str, JSONObject],
    source: pathlib.Path,
) -> None:
    """Keep context success distinct from a standalone parse; e.g. `reconcile_retry_row(records, row, originals, source)`."""
    path = text_field(row, "path")
    original = originals.get(path)
    if original is None or original.get("status") != "parse_failed":
        raise ValueError(
            f"retry header {path}: expected an original failed standalone probe"
        )
    if row.get("status") != "parsed_with_context":
        return
    verify_retry_source(row, source)
    consume_capture(records, {**row, "status": "parsed"}, pathlib.Path("/"), source)
    original["original_status"] = original["status"]
    original["status"] = "parsed_with_context"


def merge_context_retries(
    records: InventoryRecords,
    capture: JSONObject,
    capture_path: pathlib.Path,
    retry_path: pathlib.Path,
    source: pathlib.Path,
) -> None:
    """Load supplemental evidence while retaining every initial failure."""
    retry = read_object(retry_path)
    verify_retry_parent(retry, capture, capture_path)
    originals = {
        text_field(row, "path"): row
        for row in object_rows(capture["results"], "capture results")
    }
    for row in object_rows(retry.get("retry_results"), "retry results"):
        reconcile_retry_row(records, row, originals, source)
    records.context_retries = object_rows(retry.get("retry_results"), "retry results")
    capture["context_retry_sha256"] = digest(retry_path)
