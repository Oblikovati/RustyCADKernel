#!/usr/bin/env python3
"""Compare the TKPrim port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkprim_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkprim_probe.cxx against libTKPrim (and the toolkits it depends on)
and the Rust twin crates/tkprim/examples/numeric_probe.rs, runs both on
tools/tkprim_corpus.py in 64 round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkprim_corpus import append_tkprim_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKPrim request corpus, one request per line."""
    requests: list[str] = []
    append_tkprim_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKPrim', 'tools/oracle/tkprim_probe.cxx',
                       ['TKPrim', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d',
                        'TKMath', 'TKernel'],
                       'rustycad-tkprim', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
