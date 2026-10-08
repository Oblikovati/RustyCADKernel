#!/usr/bin/env python3
"""Compare the TKExpress port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkexpress_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkexpress_probe.cxx against libTKExpress and libTKernel and the Rust
twin crates/tkexpress/examples/numeric_probe.rs, runs both on tools/tkexpress_corpus.py
in round-robin shards (tools/probe_shards.py) and requires identical output tokens:
the text of every expression byte for byte, doubles as bits (NaN compared by
classification) and exception types, through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from tkexpress_corpus import append_tkexpress_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKExpress request corpus, one request per line."""
    requests: list[str] = []
    append_tkexpress_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKExpress', 'tools/oracle/tkexpress_probe.cxx', ['TKExpress', 'TKernel'],
                       'rustycad-tkexpress', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
