"""Shared source-context hashes and compiler database decoding."""

from __future__ import annotations
import hashlib
import json
import pathlib
import shlex
import subprocess
from dataclasses import dataclass
from typing import cast
from .inputs import (
    JSONValue,
    JSONObject,
    compiler_flags,
    digest,
    object_rows,
    text_field,
)


@dataclass(frozen=True)
class CompileContext:
    source_file: pathlib.Path
    directory: pathlib.Path
    toolkit: str
    flags: tuple[str, ...]


def context_flags_hash(flags: tuple[str, ...]) -> str:
    """Hash canonical compile flags; e.g. `context_flags_hash(('-std=c++17',))`."""
    return hashlib.sha256(json.dumps(flags).encode()).hexdigest()


def command_argv(row: JSONObject) -> list[str]:
    arguments = row.get("arguments")
    if isinstance(arguments, list) and all(isinstance(item, str) for item in arguments):
        return cast(list[str], arguments)
    command = row.get("command")
    if isinstance(command, str):
        return shlex.split(command)
    raise ValueError(
        f"compile command {row.get('file')!r}: expected string argv or command"
    )


def compile_contexts(
    database: pathlib.Path, source: pathlib.Path
) -> dict[str, CompileContext]:
    rows = object_rows(json.loads(database.read_text()), str(database))
    contexts: dict[str, CompileContext] = {}
    for row in rows:
        context = compile_context(row, source)
        key = context.source_file.relative_to(source).as_posix()
        contexts.setdefault(key, context)
    return contexts


def compile_context(row: JSONObject, source: pathlib.Path) -> CompileContext:
    directory = pathlib.Path(text_field(row, "directory")).resolve()
    source_file = pathlib.Path(text_field(row, "file"))
    if not source_file.is_absolute():
        source_file = directory / source_file
    source_file = source_file.resolve()
    relative = source_file.relative_to(source)
    if len(relative.parts) < 3 or relative.parts[0] != "src":
        raise ValueError(
            f"compile source {source_file}: expected OCCT src/toolkit/file path"
        )
    return CompileContext(
        source_file,
        directory,
        relative.parts[2],
        tuple(compiler_flags(command_argv(row))),
    )


def target_declaration_count(artifact: pathlib.Path, header: pathlib.Path) -> int:
    declarations, seen = inspect_target_artifact(artifact, header)
    if not seen:
        raise ValueError(f"artifact {artifact}: expected target header {header}")
    return declarations


def inspect_target_artifact(
    artifact: pathlib.Path, header: pathlib.Path
) -> tuple[int, bool]:
    target_seen = False
    declarations = 0
    for line in artifact.read_text().splitlines():
        event: JSONValue = json.loads(line)
        if not isinstance(event, dict):
            raise ValueError(f"artifact {artifact}: expected JSON object event")
        source_name = event.get("source")
        if event.get("event") == "header_seen" and isinstance(source_name, str):
            target_seen |= pathlib.Path(source_name).resolve() == header.resolve()
        if event.get("event") == "declaration" and isinstance(source_name, str):
            declarations += pathlib.Path(source_name).resolve() == header.resolve()
    return declarations, target_seen


def read_object(path: pathlib.Path) -> JSONObject:
    value: JSONValue = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected JSON object, got {type(value).__name__}")
    return value


def object_field(value: JSONObject, key: str) -> JSONObject:
    field = value.get(key)
    if not isinstance(field, dict):
        raise ValueError(f"{key}: expected object, got {field!r}")
    return field


def git_revision(source: pathlib.Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(source), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()
