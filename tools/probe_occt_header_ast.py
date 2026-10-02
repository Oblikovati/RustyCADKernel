#!/usr/bin/env python3
"""Capture Clang's raw AST for one OCCT header using its toolkit flags."""

from __future__ import annotations

import argparse
import json
import pathlib
import shlex
import subprocess
import sys
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=pathlib.Path, required=True)
    parser.add_argument("--compile-commands", type=pathlib.Path, required=True)
    parser.add_argument("--header", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    return parser.parse_args()


def toolkit_prefix(header: pathlib.Path) -> pathlib.Path:
    parts = header.parts
    if len(parts) < 4 or parts[0] != "src":
        raise ValueError(f"expected src/<module>/<toolkit>/... header path, got {header}")
    return pathlib.Path(*parts[:3])


def compile_command(database: pathlib.Path, prefix: pathlib.Path) -> tuple[pathlib.Path, list[str]]:
    entries = json.loads(database.read_text(encoding="utf-8"))
    matches = [entry for entry in entries if prefix.as_posix() in entry["file"]]
    if not matches:
        raise ValueError(f"no compile command found for toolkit prefix {prefix}")
    entry = matches[0]
    command = entry.get("arguments") or shlex.split(entry["command"])
    return pathlib.Path(entry["directory"]), command


def ast_command(command: list[str], header: pathlib.Path) -> list[str]:
    result = [command[0]]
    skip_next = False
    output_options = {"-o", "-MT", "-MF", "-MQ"}
    for argument in command[1:]:
        if skip_next:
            skip_next = False
            continue
        if argument in output_options:
            skip_next = True
        elif argument in {"-c", "-MD", "-MMD", "-MP"}:
            continue
        elif argument.startswith("-o") and argument != "-openmp":
            continue
        elif argument.endswith((".c", ".cc", ".cxx", ".cpp")):
            continue
        else:
            result.append(argument)
    return result + ["-x", "c++", "-fsyntax-only", "-Xclang", "-ast-dump=json", str(header)]


def count_nodes(root: Any) -> tuple[int, int]:
    pending = [root]
    nodes = 0
    declarations = 0
    while pending:
        node = pending.pop()
        if isinstance(node, list):
            pending.extend(node)
        elif isinstance(node, dict):
            nodes += 1
            declarations += int(node.get("kind", "").endswith("Decl"))
            pending.extend(node.values())
    return nodes, declarations


def capture_ast(arguments: argparse.Namespace) -> tuple[pathlib.Path, dict[str, Any], int, int]:
    source = arguments.source.resolve()
    header = (source / arguments.header).resolve()
    database = arguments.compile_commands.resolve()
    if not header.is_file():
        raise FileNotFoundError(f"OCCT header does not exist: {header}")
    working_dir, command = compile_command(database, toolkit_prefix(arguments.header))
    process = subprocess.run(
        ast_command(command, header),
        cwd=working_dir,
        check=False,
        capture_output=True,
        text=True,
    )
    if process.returncode:
        raise RuntimeError(process.stderr)
    tree = json.loads(process.stdout)
    nodes, declarations = count_nodes(tree)
    return arguments.output.resolve(), tree, nodes, declarations


def main() -> int:
    """Capture the requested raw AST and report its size."""
    output, tree, nodes, declarations = capture_ast(parse_args())
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(tree, separators=(",", ":")) + "\n", encoding="utf-8")
    print(f"Captured raw AST: {output} ({nodes} nodes, {declarations} declaration nodes)")
    print("Raw AST includes transitive headers and is not a normalized API inventory.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
