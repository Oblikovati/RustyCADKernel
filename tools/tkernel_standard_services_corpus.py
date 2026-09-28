"""Deterministic requests for Standard memory managers, synchronization, locale, buffered readers, dump and Plugin."""
import random


def append_standard_services_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_standard_services_requests(requests)."""
    requests.extend(area_requests(random.Random(20260932)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    return []
