#!/usr/bin/env python3
"""Account every SplitCurve2d request, retaining native crashes as explicit divergences."""

from toolkit_numeric import NumericOracle
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_corpus_split_curve3d import NATIVE_CRASH_REVISION
from tkshhealing_corpus_split_curve2d import split2d_partition
from native_crash_accounting import verify_crash_oracle, validate_crash


def split2d_oracle() -> NumericOracle:
    """Configure isolated accounting evidence; split2d_oracle()."""
    return NumericOracle(
        "TKShHealing-SplitCurve2d",
        TOOLKIT.probe_source,
        TOOLKIT.libraries,
        TOOLKIT.rust_package,
        lambda: "",
    )


def verify_split_curve2d() -> None:
    """Run exact comparisons and required crash replays; verify_split_curve2d()."""
    verify_crash_oracle(split2d_oracle(), split2d_partition, NATIVE_CRASH_REVISION)


if __name__ == "__main__":
    verify_split_curve2d()
