#!/usr/bin/env python3
"""Compare the TKHLR port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkhlr_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkhlr_probe.cxx against libTKHLR (and the toolkits it depends on)
and the Rust twin crates/tkhlr/examples/numeric_probe.rs, runs both on
tools/tkhlr_corpus.py in 64 round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkhlr_corpus import append_tkhlr_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKHLR request corpus, one request per line."""
    requests: list[str] = []
    append_tkhlr_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKHLR', 'tools/oracle/tkhlr_probe.cxx',
                       ['TKHLR', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d',
                        'TKMath', 'TKernel'],
                       'rustycad-tkhlr', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
