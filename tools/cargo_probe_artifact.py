"""Select the executable Cargo actually built, including custom target directories."""

from __future__ import annotations

import json
from pathlib import Path

from api_inventory.inputs import JSONValue


def cargo_message_rows(output: str) -> list[dict[str, JSONValue]]:
    """Reject malformed build evidence; `cargo_message_rows(stdout)` returns objects."""
    rows: list[dict[str, JSONValue]] = []
    for line in output.splitlines():
        if not line.strip():
            continue
        value: JSONValue = json.loads(line)
        if not isinstance(value, dict):
            raise ValueError(f"Cargo message {value!r}; expected a JSON object")
        rows.append(value)
    return rows


def matches_probe_artifact(row: dict[str, JSONValue], example: str) -> bool:
    """Select only the requested example; `matches_probe_artifact(row, 'probe')`."""
    target = row.get("target")
    if row.get("reason") != "compiler-artifact" or not isinstance(target, dict):
        return False
    return target.get("name") == example and target.get("kind") == ["example"]


def require_successful_build(rows: list[dict[str, JSONValue]]) -> None:
    """Require a completed build; `require_successful_build(cargo_message_rows(stdout))`."""
    finished = [
        row.get("success") for row in rows if row.get("reason") == "build-finished"
    ]
    if len(finished) != 1 or finished[0] is not True:
        raise ValueError(
            f"Cargo build results {finished!r}; expected one successful build"
        )


def cargo_probe_executable(output: str, example: str) -> Path:
    """Find one successful executable; `cargo_probe_executable(stdout, 'probe')`."""
    rows = cargo_message_rows(output)
    require_successful_build(rows)
    artifacts = [
        row.get("executable") for row in rows if matches_probe_artifact(row, example)
    ]
    if len(artifacts) != 1 or not isinstance(artifacts[0], str):
        raise ValueError(
            f"Cargo executables for {example!r}: {artifacts!r}; expected one path"
        )
    executable = Path(artifacts[0])
    if not executable.is_absolute():
        raise ValueError(
            f"Cargo executable {str(executable)!r}; expected an absolute path"
        )
    return executable
