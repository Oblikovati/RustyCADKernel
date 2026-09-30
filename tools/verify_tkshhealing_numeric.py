#!/usr/bin/env python3
"""Compare the TKShHealing port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkshhealing_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkshhealing_probe.cxx against libTKShHealing (and the toolkits it
depends on) and the Rust twin crates/tkshhealing/examples/numeric_probe.rs, runs both on
tools/tkshhealing_corpus.py in 64 round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkshhealing_corpus import append_tkshhealing_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKShHealing request corpus, one request per line."""
    requests: list[str] = []
    append_tkshhealing_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKShHealing', 'tools/oracle/tkshhealing_probe.cxx',
                       ['TKShHealing', 'TKTopAlgo', 'TKGeomAlgo', 'TKBRep', 'TKGeomBase', 'TKG3d',
                        'TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkshhealing', corpus,
                       generators=('tools/generate_shmessage_data.py',))


if __name__ == '__main__':
    verify_numeric(ORACLE)
