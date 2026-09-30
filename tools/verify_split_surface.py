#!/usr/bin/env python3
"""Account every SplitSurface request, retaining native crashes as explicit divergences."""

from toolkit_numeric import NumericOracle
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_corpus_split_curve3d import NATIVE_CRASH_REVISION
from tkshhealing_corpus_split_surface import split_surface_partition
from native_crash_accounting import verify_crash_oracle, validate_crash


def surface_oracle() -> NumericOracle:
    """Configure isolated accounting evidence; surface_oracle()."""
    return NumericOracle(
        "TKShHealing-SplitSurface",
        TOOLKIT.probe_source,
        TOOLKIT.libraries,
        TOOLKIT.rust_package,
        lambda: "",
    )


def verify_split_surface() -> None:
    """Run exact comparisons and required crash replays; verify_split_surface()."""
    verify_crash_oracle(
        surface_oracle(), split_surface_partition, NATIVE_CRASH_REVISION
    )


if __name__ == "__main__":
    verify_split_surface()
