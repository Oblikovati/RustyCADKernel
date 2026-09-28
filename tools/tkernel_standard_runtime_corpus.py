"""Deterministic requests for Standard handles, transients, type descriptors, failures, error handler and TColStd handles."""
import random


def append_standard_runtime_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_standard_runtime_requests(requests)."""
    requests.extend(area_requests(random.Random(20260931)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    return []
