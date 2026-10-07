#!/usr/bin/env python3
"""Generate porting/parity.json rows for macro-generated and special C++ members.

Covers DEFINE_STANDARD_ALLOC / DEFINE_NCOLLECTION_ALLOC operators, DEFINE_STANDARD_RTTI*
members, DEFINE_STANDARD_EXCEPTION members, destructors, copy/move constructors and
assignments, and class records whose members are already mapped, in every toolkit that
has a Rust crate. Each row is justified by a check (see tools/api_macro_*.py) and must
resolve through the coverage crosswalk. Re-running is idempotent.

Usage:
    python3 tools/generate_api_macro_rows.py --rust target/api-proof/<tag>/rust-api.json \
        --occt-source /path/to/OCCT [--check]
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

from api_coverage_report import read_json, read_jsonl
from api_macro_ledger import GenerationResult, GeneratorInputs, MacroRowGenerator
from api_macro_rust_source import WorkspaceRustSources
from api_macro_source import OcctSourceReader
from api_report_inputs import mark_conflicted_declarations, verify_conflict_artifact

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "porting/api-inventory"


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--rust", type=Path, default=INVENTORY / "rust-api.json.gz")
    parser.add_argument("--cpp", type=Path, default=INVENTORY / "occt-declarations.jsonl.gz")
    parser.add_argument("--cpp-manifest", type=Path, default=INVENTORY / "occt-manifest.json")
    parser.add_argument("--headers", type=Path, default=ROOT / "porting/api-header-manifest.json")
    parser.add_argument("--parity", type=Path, default=ROOT / "porting/parity.json")
    parser.add_argument("--occt-source", type=Path, required=True)
    parser.add_argument("--check", action="store_true", help="exit 1 when the ledger would change")
    return parser.parse_args()


def load_inputs(args: argparse.Namespace) -> GeneratorInputs:
    """Read every input once; conflicted compiler identities are withheld as in the report."""
    declarations = read_jsonl(args.cpp)
    manifest = read_json(args.cpp_manifest)
    conflicts = read_jsonl(verify_conflict_artifact(manifest, args.cpp_manifest.parent))
    mark_conflicted_declarations(declarations, conflicts)
    return GeneratorInputs(
        declarations=declarations,
        headers=read_json(args.headers),
        rust=read_json(args.rust),
        parity=read_json(args.parity),
        occt=OcctSourceReader(args.occt_source),
        rust_sources=WorkspaceRustSources(ROOT),
    )


def ledger_text(parity: dict[str, object]) -> str:
    """The ledger's canonical serialization (2-space indent, UTF-8 kept, final newline)."""
    return json.dumps(parity, indent=2, ensure_ascii=False) + "\n"


def print_summary(result: GenerationResult) -> None:
    """Plain-text run summary: written and skipped counts per toolkit and kind."""
    for label, counts in (("WRITTEN", result.written), ("SKIPPED", result.skipped)):
        for (toolkit, kind, detail), count in sorted(counts.items()):
            print(f"{label} {toolkit} {kind} {detail} {count}")
    print(f"RETIRED macro-spelling rows: {len(result.removed_macro_rows)}")
    for name, evidence in result.class_conflicts.items():
        print(f"CLASS-CONFLICT {name}: {'; '.join(evidence)}")


def main() -> int:
    args = arguments()
    result = MacroRowGenerator(load_inputs(args)).run()
    print_summary(result)
    text = ledger_text(result.parity)
    if args.parity.read_text(encoding="utf-8") == text:
        print("ledger unchanged")
        return 0
    if args.check:
        print("ledger would change; run without --check")
        return 1
    args.parity.write_text(text, encoding="utf-8")
    print(f"ledger written to {args.parity}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(2)
