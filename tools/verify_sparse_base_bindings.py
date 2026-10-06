"""Replay the bounded SparseArrayBase corrections with exact compiler bodies."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from api_binding_transaction import replay_binding_transaction


def replay_sparse_base_bindings(
    root: Path,
    receipt: Path = Path("porting/audits/sparse-base-exchange-and-resources.json"),
) -> int:
    """Replay partial base mappings; `replay_sparse_base_bindings(repo_root)`."""
    return replay_binding_transaction(root, receipt)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--receipt",
        type=Path,
        default=Path("porting/audits/sparse-base-exchange-and-resources.json"),
        help="Explicit audit receipt; historical receipts require their original source checkout.",
    )
    selected = parser.parse_args().receipt
    count = replay_sparse_base_bindings(Path(__file__).resolve().parents[1], selected)
    print(
        json.dumps(
            {
                "receipt": str(selected),
                "verified_partial_bindings": count,
                "implemented_credit_added": 0,
            }
        )
    )
