#!/usr/bin/env python3
"""Remove porting/parity.json API rows that describe private OCCT headers.

Coordinator decision 2026-10-07: `.pxx`/`.gxx`/`.lxx` files that the public header
manifest does not list are implementation detail, not public API. Their Rust ports and
unit tests stay; only the API rows go. Example:
`python3 tools/remove_private_header_api_rows.py --dry-run`.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys

from api_inventory.inputs import JSONValue
from api_private_headers import public_header_paths, split_private_header_rows

ROOT = pathlib.Path(__file__).resolve().parents[1]


def removed_rows(
    parity: dict[str, JSONValue], manifest: dict[str, JSONValue]
) -> list[dict[str, JSONValue]]:
    """Drop private-header API rows from `parity` in place; return what was removed."""
    rows = parity.get("api")
    if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
        raise ValueError(f"parity field 'api' {type(rows).__name__}: expected a list of objects")
    kept, removed = split_private_header_rows(rows, public_header_paths(manifest))
    parity["api"] = kept
    return removed


def removal_report(removed: list[dict[str, JSONValue]]) -> list[str]:
    """One line per removed row: toolkit, status, source and upstream id."""
    return [
        f"{row.get('toolkit')}\t{row.get('status')}\t{row.get('source')}\t{row.get('upstream_id')}"
        for row in removed
    ]


def argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--parity", type=pathlib.Path, default=ROOT / "porting/parity.json")
    parser.add_argument(
        "--headers", type=pathlib.Path, default=ROOT / "porting/api-header-manifest.json"
    )
    parser.add_argument("--dry-run", action="store_true")
    return parser


def main() -> int:
    args = argument_parser().parse_args()
    parity = json.loads(args.parity.read_text(encoding="utf-8"))
    manifest = json.loads(args.headers.read_text(encoding="utf-8"))
    removed = removed_rows(parity, manifest)
    print("\n".join(removal_report(removed)))
    print(f"private-header API rows removed: {len(removed)}")
    if not args.dry_run:
        text = json.dumps(parity, indent=2, ensure_ascii=False) + "\n"
        args.parity.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
