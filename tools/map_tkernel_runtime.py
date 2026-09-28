#!/usr/bin/env python3
"""Map all pinned TKernel declarations to observed native runtime expansions.

Usage: python3 tools/map_tkernel_runtime.py
Native statuses do not imply a translated Rust test exists or passes.
"""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
from typing import TypedDict
from porting import ROOT


class NativeCase(TypedDict):
    """An observed GoogleTest case, including its optional type parameter."""
    name: str
    type: str | None
    status: str
    skip_reason: str | None


class Declaration(TypedDict):
    """A source-discovered test declaration, before runtime expansion."""
    name: str
    source: str
    macro: str


class Expansion(TypedDict):
    """A fully accounted declaration and all cases instantiated from it."""
    upstream_id: str
    source: str
    source_sha256: str
    macro: str
    runtime_cases: list[NativeCase]


def declaration_id(case: NativeCase) -> str:
    """Normalize a typed suite index; PackedMap/0.Case maps to PackedMap.Case."""
    suite, name = case['name'].split('.', 1)
    if case['type'] is not None:
        suite, ordinal = suite.rsplit('/', 1)
        if not ordinal.isdecimal():
            raise ValueError(f"Unexpected typed suite index: {case['name']}")
    return suite+'.'+name


def map_expansions(declarations: list[Declaration], cases: list[NativeCase],
                   hashes: dict[str, str]) -> list[Expansion]:
    """Require both directions to be complete; map_expansions(declarations, cases, hashes)."""
    grouped: dict[str, list[NativeCase]] = {row['name']: [] for row in declarations}
    if len(grouped) != len(declarations):
        raise ValueError('Duplicate source declaration identity')
    for case in cases:
        grouped[declaration_id(case)].append(case)
    missing = [name for name, instances in grouped.items() if not instances]
    if missing:
        raise ValueError(f'Declarations without runtime instances: {missing}')
    return [{'upstream_id': row['name'], 'source': row['source'],
             'source_sha256': hashes[row['source']], 'macro': row['macro'],
             'runtime_cases': grouped[row['name']]} for row in declarations]


def main() -> None:
    """Write the declaration expansion ledger from a completed native baseline run."""
    path = ROOT/'target/oracle-tkernel/full/runtime-inventory.json'
    native = json.loads(path.read_text())
    inventory = json.loads((ROOT/'porting/inventory.json').read_text())
    if native['revision'] != inventory['revision']:
        raise ValueError('Native revision differs from pinned source inventory')
    toolkit = next(t for t in inventory['toolkits'] if t['name'] == 'TKernel')
    rows = map_expansions(toolkit['gtest_declaration_candidates'], native['cases'], native['source_hashes'])
    report = {'schema_version': 1, 'toolkit': 'TKernel', 'revision': native['revision'],
              'profile': native['profile'], 'native_inventory_sha256': hashlib.sha256(path.read_bytes()).hexdigest(),
              'declaration_count': len(rows), 'runtime_count': len(native['cases']),
              'declaration_expansion_complete': True, 'rust_acceptance': False, 'declarations': rows}
    (ROOT/'porting/tkernel-runtime.json').write_text(json.dumps(report, indent=2)+'\n')
    print(f"Mapped {len(rows)} declarations to {len(native['cases'])} native runtime cases")


if __name__ == '__main__':
    main()
