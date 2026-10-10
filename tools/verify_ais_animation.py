#!/usr/bin/env python3
"""Verify Martensite AIS timelines; python3 tools/verify_ais_animation.py --source OCCT --native-build BUILD."""

from native_build import V3D_LINKED_TOOLKITS
from tkv3d_corpus_ais_animation import append_animation_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """Build independent animation requests; corpus() returns newline-delimited input."""
    requests: list[str] = []
    append_animation_requests(requests)
    return "\n".join(requests) + "\n"


ORACLE = NumericOracle(
    "TKV3d-Animation",
    "tools/oracle/tkv3d_probe.cxx",
    list(V3D_LINKED_TOOLKITS),
    "rustycad-tkv3d",
    corpus,
)

if __name__ == "__main__":
    verify_numeric(ORACLE)
