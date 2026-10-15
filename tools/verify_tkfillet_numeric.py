#!/usr/bin/env python3
"""Compare the TKFillet port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkfillet_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkfillet_probe.cxx against libTKFillet (and the toolkits it depends
on, TKBool among them) and the Rust twin crates/tkfillet/examples/numeric_probe.rs, runs both on
tools/tkfillet_corpus.py in round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkfillet_corpus import append_tkfillet_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKFillet request corpus, one request per line."""
    requests: list[str] = []
    append_tkfillet_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKFillet', 'tools/oracle/tkfillet_probe.cxx',
                       ['TKFillet', 'TKBool', 'TKBO', 'TKPrim', 'TKShHealing', 'TKTopAlgo', 'TKGeomAlgo',
                        'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkfillet', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
