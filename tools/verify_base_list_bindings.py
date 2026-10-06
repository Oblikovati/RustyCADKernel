"""Replay BaseList's exact partial API mappings and pinned compiler bodies."""

from __future__ import annotations

import json
from pathlib import Path

from api_binding_transaction import replay_binding_transaction


def replay_base_list_bindings(root: Path) -> int:
    """Replay the recorded base surface; `replay_base_list_bindings(repo_root)`."""
    receipt = Path("porting/audits/base-list-address-current-bindings.json")
    return replay_binding_transaction(root, receipt)


if __name__ == "__main__":
    count = replay_base_list_bindings(Path(__file__).resolve().parents[1])
    print(
        json.dumps({"verified_partial_bindings": count, "implemented_credit_added": 0})
    )
