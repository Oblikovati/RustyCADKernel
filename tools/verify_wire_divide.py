#!/usr/bin/env python3
"""Account every wire request, including two native null-handle crashes."""

from toolkit_numeric import NumericOracle
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_wire_divide_crashes import NATIVE_CRASH_REVISION
from tkshhealing_corpus_wire_divide import wire_divide_partition
from native_crash_accounting import verify_crash_oracle


def wire_divide_oracle() -> NumericOracle:
    """Configure isolated accounting evidence; wire_divide_oracle()."""
    return NumericOracle(
        "TKShHealing-WireDivide",
        TOOLKIT.probe_source,
        TOOLKIT.libraries,
        TOOLKIT.rust_package,
        lambda: "",
    )


def verify_wire_divide() -> None:
    """Require exact cases and native crashes with Rust null-object failures."""
    verify_crash_oracle(
        wire_divide_oracle(),
        wire_divide_partition,
        NATIVE_CRASH_REVISION,
        "Standard_NullObject",
    )


if __name__ == "__main__":
    verify_wire_divide()
