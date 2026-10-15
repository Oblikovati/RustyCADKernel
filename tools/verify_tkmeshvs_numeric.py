#!/usr/bin/env python3
"""Compare the TKMeshVS port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkmeshvs_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkmeshvs_probe.cxx against libTKMeshVS (and the toolkits it depends on)
and the Rust twin crates/tkmeshvs/examples/numeric_probe.rs, runs both on
tools/tkmeshvs_corpus.py in round-robin shards (tools/probe_shards.py) and requires identical
output tokens (doubles as bits, NaN compared by classification, exception types), through the
shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from native_build import MESHVS_LINKED_TOOLKITS
from tkmeshvs_corpus import append_tkmeshvs_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKMeshVS request corpus, one request per line."""
    requests: list[str] = []
    append_tkmeshvs_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKMeshVS', 'tools/oracle/tkmeshvs_probe.cxx',
                       list(MESHVS_LINKED_TOOLKITS), 'martensite-tkmeshvs', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
