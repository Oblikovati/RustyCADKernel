"""Deterministic requests for Standard memory managers, synchronization, locale, buffered readers, dump and Plugin.

Every operation starts with `services_`. Each group module documents the native inputs it
excludes (undefined behavior is never probed).
"""
import random

from tkernel_standard_services_memory_corpus import memory_requests
from tkernel_standard_services_readers_corpus import readers_requests
from tkernel_standard_services_sync_corpus import sync_requests


def append_standard_services_requests(requests: list[str]) -> None:
    """Append this area's requests with its own seed; append_standard_services_requests(requests)."""
    requests.extend(area_requests(random.Random(20260932)))


def area_requests(rng: random.Random) -> list[str]:
    """Every request of this area, drawn from `rng`; area_requests(random.Random(1))."""
    return memory_requests(rng) + sync_requests() + readers_requests(rng)
