#!/usr/bin/env python3
"""Compare the TKHelix port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkhelix_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkhelix_probe.cxx against libTKHelix (and the toolkits it depends
on) and the Rust twin crates/tkhelix/examples/numeric_probe.rs, runs both on
tools/tkhelix_corpus.py in 64 round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkhelix_corpus import append_tkhelix_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKHelix request corpus, one request per line."""
    requests: list[str] = []
    append_tkhelix_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKHelix', 'tools/oracle/tkhelix_probe.cxx',
                       ['TKHelix', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d',
                        'TKMath', 'TKernel'],
                       'rustycad-tkhelix', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
