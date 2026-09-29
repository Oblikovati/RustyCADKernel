#!/usr/bin/env python3
"""Compare the TKGeomAlgo port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkgeomalgo_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkgeomalgo_probe.cxx against libTKGeomAlgo (and the toolkits it
depends on) and the Rust twin crates/tkgeomalgo/examples/numeric_probe.rs, runs both on
tools/tkgeomalgo_corpus.py and requires identical output tokens (doubles as bits, NaN
compared by classification, exception types), through the shared driver
tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkgeomalgo_corpus import append_tkgeomalgo_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKGeomAlgo request corpus, one request per line."""
    requests: list[str] = []
    append_tkgeomalgo_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKGeomAlgo', 'tools/oracle/tkgeomalgo_probe.cxx',
                       ['TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkgeomalgo', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
