"""Pinned null-wire failure retained by ShapeFix_WireSegment::Load."""

from native_crash_accounting import partition_crash_requests
from tkshhealing_corpus_wire_segment import wire_segment_candidates

NATIVE_CRASH_REVISION = "3d097a0328e71b826377d4814ab05ec3c3d23871"
NATIVE_CRASH_HASHES = frozenset(
    [
        "3b443e70595be6ec8f37f44a79a16fd0b2b2694355f31d1517b945f2a1a40509",
        "9a99d37ebbe07bb45f13ead9e5c53a7069e4398a8bcec2cc1b6e60c25a540c6e",
    ]
)


def wire_segment_partition() -> tuple[list[str], list[str]]:
    """Split exact cases from the native null-wire crash; wire_segment_partition()."""
    return partition_crash_requests(wire_segment_candidates(), NATIVE_CRASH_HASHES)
