"""Replay the bounded SparseArrayBase corrections with exact compiler bodies."""

from __future__ import annotations

import json
from pathlib import Path

from api_binding_transaction import replay_binding_transaction


def replay_sparse_base_bindings(root: Path) -> int:
    """Replay partial base mappings; `replay_sparse_base_bindings(repo_root)`."""
    receipt = Path("porting/audits/sparse-base-exchange-and-resources.json")
    return replay_binding_transaction(root, receipt)


if __name__ == "__main__":
    count = replay_sparse_base_bindings(Path(__file__).resolve().parents[1])
    print(
        json.dumps({"verified_partial_bindings": count, "implemented_credit_added": 0})
    )
