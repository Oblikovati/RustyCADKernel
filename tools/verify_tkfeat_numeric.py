#!/usr/bin/env python3
"""Compare the TKFeat port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkfeat_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkfeat_probe.cxx against libTKFeat (and the toolkits it depends on)
and the Rust twin crates/tkfeat/examples/numeric_probe.rs, runs both on
tools/tkfeat_corpus.py in round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkfeat_corpus import append_tkfeat_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKFeat request corpus, one request per line."""
    requests: list[str] = []
    append_tkfeat_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKFeat', 'tools/oracle/tkfeat_probe.cxx',
                       ['TKFeat', 'TKBool', 'TKBO', 'TKPrim', 'TKShHealing', 'TKTopAlgo', 'TKGeomAlgo',
                        'TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkfeat', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
