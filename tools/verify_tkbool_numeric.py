#!/usr/bin/env python3
"""Compare the TKBool port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkbool_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkbool_probe.cxx against libTKBool (and the toolkits it depends on)
and the Rust twin crates/tkbool/examples/numeric_probe.rs, runs both on
tools/tkbool_corpus.py in 64 round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkbool_corpus import append_tkbool_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKBool request corpus, one request per line."""
    requests: list[str] = []
    append_tkbool_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKBool', 'tools/oracle/tkbool_probe.cxx',
                       ['TKBool', 'TKBO', 'TKShHealing', 'TKPrim', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep',
                        'TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkbool', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
