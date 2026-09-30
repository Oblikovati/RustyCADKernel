#!/usr/bin/env python3
"""Account every projection request, including the two native null-curve crashes."""

from toolkit_numeric import NumericOracle
from verify_tkshhealing_numeric import ORACLE as TOOLKIT
from tkshhealing_transfer_projection_crashes import NATIVE_CRASH_REVISION
from tkshhealing_corpus_transfer_proj import transfer_projection_partition
from native_crash_accounting import verify_crash_oracle


def transfer_projection_oracle() -> NumericOracle:
    """Configure isolated accounting evidence; transfer_projection_oracle()."""
    return NumericOracle(
        "TKShHealing-TransferProjection",
        TOOLKIT.probe_source,
        TOOLKIT.libraries,
        TOOLKIT.rust_package,
        lambda: "",
    )


def verify_transfer_projection() -> None:
    """Require exact cases and native crashes with Rust null-object failures."""
    verify_crash_oracle(
        transfer_projection_oracle(),
        transfer_projection_partition,
        NATIVE_CRASH_REVISION,
        "Standard_NullObject",
    )


if __name__ == "__main__":
    verify_transfer_projection()
