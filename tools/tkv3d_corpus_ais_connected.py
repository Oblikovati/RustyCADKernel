"""AIS_ConnectedInteractive and AIS_MultipleConnectedInteractive: connection of a face or an
edge shape, the connection rules (displayed reference, assembly without presentation, chained
instances), selections, display, HLR and disconnection (see
tools/oracle/tkv3d_probe_ais_connected.cxx). A request is `v3_ais_connected Case Arg`.
"""
from __future__ import annotations


def append_ais_connected_requests(requests: list[str]) -> None:
    """Every case with four arguments (the location and the selection mode)."""
    for case in range(3):
        for arg in range(4):
            requests.append(f'v3_ais_connected {case} {arg}')
