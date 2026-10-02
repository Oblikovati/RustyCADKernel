#!/usr/bin/env python3
"""Compare ShapeFix_WireSegment with pinned OCCT and account for its null-wire crash."""

from toolkit_numeric import NumericOracle
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_wire_segment_crashes import NATIVE_CRASH_REVISION
from tkshhealing_wire_segment_crashes import wire_segment_partition
from native_crash_accounting import verify_crash_oracle


def wire_segment_oracle() -> NumericOracle:
    """Configure isolated wire-segment evidence; wire_segment_oracle()."""
    return NumericOracle(
        "TKShHealing-WireSegment",
        TOOLKIT.probe_source,
        TOOLKIT.libraries,
        TOOLKIT.rust_package,
        lambda: "",
    )


def verify_wire_segment() -> None:
    """Require exact defined cases and replay the native null-wire crash."""
    verify_crash_oracle(
        wire_segment_oracle(),
        wire_segment_partition,
        NATIVE_CRASH_REVISION,
        "Standard_NullObject",
    )


if __name__ == "__main__":
    verify_wire_segment()
