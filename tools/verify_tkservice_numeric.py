#!/usr/bin/env python3
"""Compare the TKService port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkservice_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkservice_probe.cxx against libTKService (and the toolkits it depends
on) and the Rust twin crates/tkservice/examples/numeric_probe.rs, runs both on
tools/tkservice_corpus.py in 64 round-robin shards (tools/probe_shards.py) and requires
identical output tokens (doubles as bits, NaN compared by classification, exception
types), through the shared driver tools/toolkit_numeric.py.
"""
from __future__ import annotations
from native_build import VISUALIZATION_LINKED_TOOLKITS
from tkservice_corpus import append_tkservice_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKService request corpus, one request per line."""
    requests: list[str] = []
    append_tkservice_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKService', 'tools/oracle/tkservice_probe.cxx',
                       list(VISUALIZATION_LINKED_TOOLKITS), 'rustycad-tkservice', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
