#!/usr/bin/env python3
"""Preflight a Martensite fragment before integration; no API ledger is modified."""

from __future__ import annotations

import argparse
import json
from functools import partial
from pathlib import Path

from api_claim_preflight import ClaimFragmentPreflight
from api_coverage_report import read_json, read_jsonl
from api_crosswalk import cpp_indexes, header_owners, rust_crates
from api_inventory.inputs import digest


def claim_source_digest(root: Path, filename: str) -> str:
    """Bind source bytes to a checkout; `claim_source_digest(root, 'crates/x/src/lib.rs')`."""
    source = root / filename
    if not source.is_file():
        raise ValueError(
            f"source={str(source)!r}; expected an existing captured source file"
        )
    return digest(source)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fragment", type=Path)
    parser.add_argument(
        "--workspace", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--rust", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    root = args.workspace.resolve()
    owners = header_owners(read_json(root / "porting/api-header-manifest.json"))
    native = read_jsonl(root / "porting/api-inventory/occt-declarations.jsonl.gz")
    by_name, _, _ = cpp_indexes(native, owners)
    inventory = read_json(
        args.rust or root / "target/api-proof/rust-api-inventory.json"
    )
    preflight = ClaimFragmentPreflight(
        by_name, rust_crates(inventory), set(owners), partial(claim_source_digest, root)
    )
    results = preflight.verify_fragment(json.loads(args.fragment.read_text()))
    if args.output:
        args.output.write_text(json.dumps(results, indent=2) + "\n")
    print(f"Verified {len(results)} claim bindings and current Rust source hashes.")
