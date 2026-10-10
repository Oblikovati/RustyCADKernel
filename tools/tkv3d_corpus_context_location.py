"""Context location placement and identity-chain cases; append_location_requests(rows)."""

from itertools import product


def append_location_requests(requests: list[str]) -> None:
    """Cover display, parent and highlight paths; append_location_requests([])."""
    for flags, operation, initial, final in product(
        range(32), range(4), [-3.0, 0.0, 2.0], [-4.0, 0.0, 5.0]
    ):
        requests.append(f"v3_context_location {flags} {operation} {initial} {final}")
