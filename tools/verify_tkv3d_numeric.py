#!/usr/bin/env python3
"""Compare the TKV3d port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkv3d_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkv3d_probe.cxx against libTKV3d (and the toolkits it depends on) and
the Rust twin crates/tkv3d/examples/numeric_probe.rs, runs both on tools/tkv3d_corpus.py in
64 round-robin shards (tools/probe_shards.py) and requires identical output tokens (doubles
as bits, NaN compared by classification, exception types), through the shared driver
tools/toolkit_numeric.py.
"""
from __future__ import annotations
from native_build import V3D_LINKED_TOOLKITS
from tkv3d_corpus import append_tkv3d_requests
from toolkit_numeric import NumericOracle, verify_numeric
from tkv3d_shutdown_failure import verify_shutdown_failure


def corpus() -> str:
    """The whole TKV3d request corpus, one request per line."""
    requests: list[str] = []
    append_tkv3d_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKV3d', 'tools/oracle/tkv3d_probe.cxx',
                       list(V3D_LINKED_TOOLKITS), 'rustycad-tkv3d', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
    verify_shutdown_failure(ORACLE.evidence, ORACLE.rust_probe)
