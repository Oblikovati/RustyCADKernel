#!/usr/bin/env python3
"""Check every workspace Rust target without exceeding Windows process arguments.

Example: python3 tools/check_rust_format.py
"""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
from collections.abc import Iterable

# Leave room below CreateProcessW's 32,767 UTF-16 units for executable resolution.
ARGUMENT_LIMIT = 24000


def command_units(arguments: list[str]) -> int:
    """Count quoted Windows command units; command_units(['rustfmt', 'a b.rs'])."""
    return len(subprocess.list2cmdline(arguments).encode("utf-16-le")) // 2 + 1


def format_batches(edition: str, roots: Iterable[str]) -> list[list[str]]:
    """Retain every target within bounded rustfmt commands; format_batches('2024', roots)."""
    prefix = ["rustfmt", "--check", "--edition", edition]
    commands: list[list[str]] = []
    batch = prefix.copy()
    for root in sorted(set(roots)):
        if command_units(prefix + [root]) > ARGUMENT_LIMIT:
            raise ValueError(
                f"Rust target {root!r}: expected command below {ARGUMENT_LIMIT} UTF-16 units"
            )
        if command_units(batch + [root]) > ARGUMENT_LIMIT:
            commands.append(batch)
            batch = prefix.copy()
        batch.append(root)
    if len(batch) > len(prefix):
        commands.append(batch)
    return commands


def workspace_commands(metadata: dict[str, object]) -> list[list[str]]:
    """Group workspace roots by their edition; workspace_commands(cargo_metadata)."""
    members = set(metadata["workspace_members"])
    editions: dict[str, list[str]] = {}
    for package in metadata["packages"]:
        if package["id"] not in members:
            continue
        for target in package["targets"]:
            if Path(target["src_path"]).suffix == ".rs":
                editions.setdefault(target["edition"], []).append(target["src_path"])
    return [
        command
        for edition, roots in sorted(editions.items())
        for command in format_batches(edition, roots)
    ]


def main() -> None:
    """Run the same rustfmt check across all target roots; main()."""
    root = Path(__file__).resolve().parents[1]
    output = subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--locked", "--format-version", "1"],
        cwd=root,
        text=True,
    )
    commands = workspace_commands(json.loads(output))
    for command in commands:
        subprocess.run(command, cwd=root, check=True)
    print(f"Rust formatting passed across {len(commands)} bounded commands.")


if __name__ == "__main__":
    main()
