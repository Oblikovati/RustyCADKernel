#!/usr/bin/env python3
"""Verify Martensite camera animation; python3 tools/verify_animation_camera.py --source OCCT --native-build BUILD."""

from native_build import V3D_LINKED_TOOLKITS
from tkv3d_corpus_animation_camera import append_camera_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """Produce independent camera lifecycle/endpoint requests; corpus() returns probe input."""
    requests: list[str] = []
    append_camera_requests(requests)
    return "\n".join(requests) + "\n"


ORACLE = NumericOracle(
    "TKV3d-Camera-Animation",
    "tools/oracle/tkv3d_probe.cxx",
    list(V3D_LINKED_TOOLKITS),
    "rustycad-tkv3d",
    corpus,
)

if __name__ == "__main__":
    verify_numeric(ORACLE)
