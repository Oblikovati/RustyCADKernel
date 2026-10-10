#!/usr/bin/env python3
"""Verify Martensite context placement; python3 tools/verify_context_location.py --source OCCT --native-build BUILD."""

from native_build import V3D_LINKED_TOOLKITS
from tkv3d_corpus_context_location import append_location_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """Produce independent placement requests; corpus() returns probe input."""
    requests: list[str] = []
    append_location_requests(requests)
    return "\n".join(requests) + "\n"


ORACLE = NumericOracle(
    "TKV3d-Context-Location",
    "tools/oracle/tkv3d_probe.cxx",
    list(V3D_LINKED_TOOLKITS),
    "rustycad-tkv3d",
    corpus,
)

if __name__ == "__main__":
    verify_numeric(ORACLE)
