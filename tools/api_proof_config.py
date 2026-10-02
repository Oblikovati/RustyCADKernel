#!/usr/bin/env python3
"""Record or verify the OCCT compiler database used by API inventory probes."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
from typing import Any


def read_json(path: pathlib.Path) -> Any:
    """Read the checked JSON input used by the inventory setup."""
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: pathlib.Path) -> str:
    """Return a stable digest for a file captured by the setup."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_toolkits(entries: list[dict[str, Any]]) -> set[str]:
    return {
        entry["file"].split("/src/", 1)[1].split("/")[1]
        for entry in entries
        if "/src/" in entry["file"]
    }


def source_revision(path: pathlib.Path) -> str:
    return subprocess.check_output(
        ["git", "-C", str(path), "rev-parse", "HEAD"], text=True
    ).strip()


def toolkit_coverage(inventory: Any, entries: list[dict[str, Any]]) -> dict[str, Any]:
    expected = {toolkit["name"] for toolkit in inventory["toolkits"]}
    actual = source_toolkits(entries)
    return {
        "expected_toolkit_count": len(expected),
        "toolkits_with_compile_commands": len(expected & actual),
        "missing_toolkits": sorted(expected - actual),
    }


def configuration_record(args: argparse.Namespace) -> dict[str, Any]:
    config = read_json(args.toolchain_config)
    inventory = read_json(args.inventory)
    database = args.build / "compile_commands.json"
    entries = read_json(database)
    cache = args.build / "CMakeCache.txt"
    return {
        "schema_version": 1,
        "occt_revision": source_revision(args.source),
        "toolchain_config_sha256": sha256(args.toolchain_config),
        "inventory_sha256": sha256(args.inventory),
        "compile_commands_sha256": sha256(database),
        "compile_command_count": len(entries),
        **toolkit_coverage(inventory, entries),
        "enabled_modules": config["modules"],
        "cmake_cache_sha256": sha256(cache),
        "build_directory": str(args.build.resolve()),
    }


def record_configuration(args: argparse.Namespace) -> int:
    record = configuration_record(args)
    destination = args.build / "api-proof-config.json"
    destination.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(f"Compiler database: {args.build / 'compile_commands.json'} ({record['compile_command_count']} translation units)")
    missing = record["missing_toolkits"]
    if missing:
        print(f"Missing toolkit compile commands ({len(missing)}): {', '.join(missing)}")
    return 0


def verify_configuration(args: argparse.Namespace) -> int:
    record = read_json(args.record)
    entries = read_json(args.database)
    if not entries or record["compile_command_count"] != len(entries):
        raise ValueError("compile database is empty or has a stale recorded count")
    if sha256(args.database) != record["compile_commands_sha256"]:
        raise ValueError("compile database changed after configuration")
    if sha256(args.toolchain_config) != record["toolchain_config_sha256"]:
        raise ValueError("toolchain configuration changed after configuration")
    if source_revision(args.source) != record["occt_revision"]:
        raise ValueError("OCCT revision changed after configuration")
    if record["missing_toolkits"]:
        missing = ", ".join(record["missing_toolkits"])
        raise ValueError(f"all-toolkit compile database incomplete; missing: {missing}")
    print(f"Verified complete compile database: {len(entries)} translation units")
    return 0


def argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    record = commands.add_parser("record")
    record.add_argument("toolchain_config", type=pathlib.Path)
    record.add_argument("inventory", type=pathlib.Path)
    record.add_argument("source", type=pathlib.Path)
    record.add_argument("build", type=pathlib.Path)
    verify = commands.add_parser("verify")
    verify.add_argument("database", type=pathlib.Path)
    verify.add_argument("record", type=pathlib.Path)
    verify.add_argument("toolchain_config", type=pathlib.Path)
    verify.add_argument("source", type=pathlib.Path)
    return parser


def main() -> int:
    """Dispatch the record or validation command."""
    args = argument_parser().parse_args()
    return record_configuration(args) if args.command == "record" else verify_configuration(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (
        OSError,
        ValueError,
        KeyError,
        IndexError,
        subprocess.CalledProcessError,
        json.JSONDecodeError,
    ) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
