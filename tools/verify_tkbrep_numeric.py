#!/usr/bin/env python3
"""Compare the TKBRep port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkbrep_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkbrep_probe.cxx against libTKBRep (and the toolkits it depends
on) and the Rust twin crates/tkbrep/examples/numeric_probe.rs, runs both on
tools/tkbrep_corpus.py and requires identical output tokens (doubles as bits, NaN
compared by classification, exception types, texts as bytes), through the shared
driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkbrep_corpus import append_tkbrep_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKBRep request corpus, one request per line."""
    requests: list[str] = []
    append_tkbrep_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKBRep', 'tools/oracle/tkbrep_probe.cxx',
                       ['TKBRep', 'TKGeomBase', 'TKG3d', 'TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkbrep', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
