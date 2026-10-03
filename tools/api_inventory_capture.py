"""Validate raw rustdoc capture manifests and artifacts before normalization."""

from __future__ import annotations

import json
import pathlib
from collections.abc import Mapping

from api_inventory_json import JSONValue, array_value, integer_value, json_value, object_value
from api_inventory_json import read_json, sha256_bytes, string_value


def capture_files(directory: pathlib.Path) -> tuple[dict[str, JSONValue], pathlib.Path]:
    manifest_path = directory / "capture-manifest.json"
    manifest = object_value(read_json(manifest_path), "capture manifest")
    if manifest.get("raw_capture_complete") is not True:
        raise ValueError("capture manifest does not declare a complete raw capture")
    if manifest.get("normalized_public_inventory") is not False:
        raise ValueError("capture manifest must not claim normalized inventory completeness")
    return manifest, manifest_path


def index_records(manifest: Mapping[str, JSONValue]) -> dict[str, dict[str, JSONValue]]:
    values = array_value(manifest.get("rustdoc_json"), "rustdoc_json")
    records = [object_value(value, "rustdoc artifact record") for value in values]
    by_name = {string_value(record.get("library_target"), "library_target"): record for record in records}
    if len(by_name) != len(records):
        raise ValueError("capture manifest contains duplicate library targets")
    expected_count = integer_value(manifest.get("workspace_library_count"), "workspace_library_count")
    if len(records) != expected_count:
        raise ValueError(f"capture lists {len(records)} libraries; expected {expected_count}")
    return by_name


def verify_artifact(capture: pathlib.Path, record: Mapping[str, JSONValue], expected: int) -> dict[str, JSONValue]:
    name = string_value(record.get("artifact"), "artifact")
    path = (capture / name).resolve()
    if not path.is_relative_to(capture.resolve()) or not path.is_file():
        raise ValueError(f"missing or escaping rustdoc artifact: {name}")
    artifact_bytes = path.read_bytes()
    validate_artifact_bytes(name, artifact_bytes, record, expected)
    return object_value(json_value(json.loads(artifact_bytes)), name)


def validate_artifact_bytes(name: str, content: bytes, record: Mapping[str, JSONValue], expected: int) -> None:
    digest = string_value(record.get("artifact_sha256"), "artifact_sha256")
    if sha256_bytes(content) != digest:
        raise ValueError(f"rustdoc artifact digest mismatch: {name}")
    if len(content) != integer_value(record.get("artifact_bytes"), "artifact_bytes"):
        raise ValueError(f"rustdoc artifact size mismatch: {name}")
    document = object_value(json_value(json.loads(content)), name)
    validate_artifact_document(name, document, record, expected)


def validate_artifact_document(name: str, document: Mapping[str, JSONValue],
                               record: Mapping[str, JSONValue], expected: int) -> None:
    version = integer_value(document.get("format_version"), f"{name}.format_version")
    if version != expected:
        raise ValueError(f"unsupported rustdoc format {version} in {name}; expected {expected}")
    count = len(object_value(document.get("index"), f"{name}.index"))
    if count != integer_value(record.get("indexed_items"), "indexed_items"):
        raise ValueError(f"rustdoc item count mismatch: {name}")
