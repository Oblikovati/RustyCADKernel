"""Deterministic requests for Standard scalar, character, hashing, time, GUID and atomic helpers."""
import random


def append_standard_values_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_standard_values_requests(requests)."""
    requests.extend(area_requests(random.Random(20260930)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    return []
