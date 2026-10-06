"""Replay BaseSequence's exact partial API mappings and pinned compiler bodies."""

from __future__ import annotations

import json
from pathlib import Path

from api_binding_transaction import replay_binding_transaction


def replay_base_sequence_bindings(root: Path) -> int:
    """Replay the recorded base surface; `replay_base_sequence_bindings(repo_root)`."""
    receipt = Path("porting/audits/base-sequence-api-reconciliation.json")
    return replay_binding_transaction(root, receipt)


if __name__ == "__main__":
    count = replay_base_sequence_bindings(Path(__file__).resolve().parents[1])
    print(
        json.dumps({"verified_partial_bindings": count, "implemented_credit_added": 0})
    )
