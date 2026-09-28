"""Deterministic requests for TCollection_AsciiString, TCollection_HAsciiString and TCollection."""
import random


def append_ascii_string_api_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_ascii_string_api_requests(requests)."""
    requests.extend(area_requests(random.Random(20260928)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    return []
