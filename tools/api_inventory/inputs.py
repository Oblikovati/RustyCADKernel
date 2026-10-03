"""Pinned source inputs and compilation flags for OCCT declaration capture."""

from __future__ import annotations

import hashlib
import json
import pathlib
import shlex
import subprocess
from dataclasses import dataclass
from typing import TypeAlias, cast

JSONValue: TypeAlias = (
    None | bool | int | float | str | list["JSONValue"] | dict[str, "JSONValue"]
)
JSONObject: TypeAlias = dict[str, JSONValue]


def read_object(path: pathlib.Path) -> JSONObject:
    """Require a JSON object; e.g. `read_object(Path('manifest.json'))`."""
    value: JSONValue = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected JSON object, got {type(value).__name__}")
    return value


def digest(path: pathlib.Path) -> str:
    """Hash file contents; e.g. `digest(Path('Cargo.lock'))`."""
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def object_rows(value: JSONValue, label: str) -> list[JSONObject]:
    """Check a table before consuming rows; e.g. `object_rows([], 'headers')`."""
    if not isinstance(value, list) or not all(isinstance(row, dict) for row in value):
        raise ValueError(
            f"{label}: expected array of objects, got {type(value).__name__}"
        )
    return cast(list[JSONObject], value)


def text_field(row: JSONObject, key: str) -> str:
    """Require a string field; e.g. `text_field({'path': 'a'}, 'path')`."""
    value = row.get(key)
    if not isinstance(value, str):
        raise ValueError(f"{key}: expected string, got {value!r}")
    return value


def compiler_flags(arguments: list[str]) -> list[str]:
    """Remove only driver input/output flags; e.g. `compiler_flags(['clang++','-c','a.cxx'])`."""
    flags: list[str] = []
    skip = False
    for argument in arguments[1:]:
        if skip:
            skip = False
            continue
        if argument in {"-o", "-MF", "-MT", "-MQ"}:
            skip = True
        elif argument not in {"-c", "-MD", "-MMD", "-MP"} and not argument.endswith(
            (".cxx", ".cpp", ".cc", ".c")
        ):
            flags.append(argument)
    return flags


def toolkit_commands(
    path: pathlib.Path, source: pathlib.Path
) -> dict[str, tuple[str, list[str]]]:
    """Index real CMake commands by toolkit; e.g. `toolkit_commands(database, source)`."""
    rows = object_rows(json.loads(path.read_text()), str(path))
    commands: dict[str, tuple[str, list[str]]] = {}
    for row in rows:
        relative = pathlib.Path(text_field(row, "file")).relative_to(source)
        toolkit = relative.parts[2]
        arguments = row.get("arguments")
        argv = (
            cast(list[str], arguments)
            if isinstance(arguments, list)
            else shlex.split(text_field(row, "command"))
        )
        commands.setdefault(
            toolkit, (text_field(row, "directory"), compiler_flags(argv))
        )
    return commands


@dataclass(frozen=True)
class HeaderJob:
    path: str
    toolkit: str
    directory: str
    flags: tuple[str, ...]
    input_path: str = ""


def owning_toolkit(row: JSONObject, commands: dict[str, tuple[str, list[str]]]) -> str:
    """Select an available owner while retaining all owners in coverage evidence."""
    owners = object_rows(row.get("owners"), "header owners")
    available = [
        text_field(owner, "toolkit")
        for owner in owners
        if text_field(owner, "toolkit") in commands
    ]
    return available[0] if available else ""


def select_header_jobs(
    manifest: JSONObject, commands: dict[str, tuple[str, list[str]]]
) -> tuple[list[HeaderJob], list[JSONObject]]:
    """Account every source header as a job, fragment, or configuration gap."""
    jobs: list[HeaderJob] = []
    coverage: list[JSONObject] = []
    for row in object_rows(manifest.get("headers"), "headers"):
        path = text_field(row, "path")
        toolkit = owning_toolkit(row, commands)
        status = "fragment_pending" if path.endswith(".lxx") else "pending"
        if not toolkit:
            status = "missing_compile_configuration"
        coverage.append({"path": path, "status": status, "owners": row["owners"]})
        if toolkit and status == "pending":
            directory, flags = commands[toolkit]
            jobs.append(HeaderJob(path, toolkit, directory, tuple(flags)))
    return jobs, coverage


def verify_checkout(source: pathlib.Path, revision: JSONValue) -> None:
    """Reject any tracked source edits or a different baseline."""
    actual = subprocess.check_output(
        ["git", "-C", str(source), "rev-parse", "HEAD"], text=True
    ).strip()
    if actual != revision:
        raise ValueError(f"OCCT revision {actual}: expected {revision}")
    dirty = subprocess.check_output(
        ["git", "-C", str(source), "status", "--porcelain", "--untracked-files=no"],
        text=True,
    )
    if dirty:
        raise ValueError(
            f"OCCT checkout has tracked changes: expected pinned clean source, got {dirty}"
        )


def verify_sources(source: pathlib.Path, manifest: JSONObject) -> None:
    """Reject stale source inputs before capture; e.g. `verify_sources(source, manifest)`."""
    verify_checkout(source, manifest.get("revision"))
    for row in object_rows(manifest.get("headers"), "headers"):
        path = source / text_field(row, "path")
        if digest(path) != row.get("sha256"):
            raise ValueError(
                f'changed OCCT header {path}: expected hash {row.get("sha256")}'
            )
