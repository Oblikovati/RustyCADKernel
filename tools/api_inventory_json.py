"""Typed JSON utilities shared by API inventory tools."""

from __future__ import annotations

import hashlib
import json
import pathlib
from collections.abc import Mapping, Sequence
from typing import TypeAlias

JSONScalar: TypeAlias = None | bool | int | float | str
JSONValue: TypeAlias = JSONScalar | Sequence["JSONValue"] | Mapping[str, "JSONValue"]


def object_value(value: JSONValue, field: str) -> dict[str, JSONValue]:
    if not isinstance(value, dict):
        raise ValueError(f"{field} must be a JSON object")
    return dict(value)


def array_value(value: JSONValue | None, field: str) -> list[JSONValue]:
    if not isinstance(value, list):
        raise ValueError(f"{field} must be a JSON array")
    return list(value)


def string_value(value: JSONValue | None, field: str) -> str:
    if not isinstance(value, str):
        raise ValueError(f"{field} must be a JSON string")
    return value


def integer_value(value: JSONValue | None, field: str) -> int:
    if type(value) is not int:
        raise ValueError(f"{field} must be a JSON integer")
    return value


def read_json(path: pathlib.Path) -> JSONValue:
    """Decode JSON with its recursive type shape and no unchecked parser values."""
    decoded = json.loads(path.read_text(encoding="utf-8"))
    return json_value(decoded)


def json_value(value: object) -> JSONValue:
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if isinstance(value, list):
        return [json_value(item) for item in value]
    if isinstance(value, dict) and all(isinstance(key, str) for key in value):
        return {key: json_value(item) for key, item in value.items()}
    raise ValueError("decoded JSON contains a value outside the JSON data model")


def sha256_bytes(content: bytes) -> str:
    """Hash content with SHA-256 for provenance records."""
    digest = hashlib.sha256(content)
    return digest.hexdigest()


def canonical_json(value: JSONValue) -> str:
    """Encode stable JSON for declaration identity fingerprints."""
    # Compact sorted keys keep identity independent of source object ordering.
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


def declaration_digest(entry: Mapping[str, JSONValue]) -> str:
    """Fingerprint a public path, normalized signature, and source identity."""
    stable_fields = {key: value for key, value in entry.items() if key != "declaration_sha256"}
    return sha256_bytes(canonical_json(stable_fields).encode())
