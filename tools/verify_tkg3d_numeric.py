#!/usr/bin/env python3
"""Compare the TKG3d (Geom) port with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkg3d_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkg3d_probe.cxx against libTKG3d and the Rust twin
crates/tkg3d/examples/numeric_probe.rs, runs both on tools/tkg3d_corpus.py and
requires identical bits (NaN compared by classification), as the TKMath verifier does.
"""
from __future__ import annotations
from tkg3d_corpus import append_tkg3d_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKG3d request corpus, one request per line."""
    requests: list[str] = []
    append_tkg3d_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKG3d', 'tools/oracle/tkg3d_probe.cxx', ['TKG3d', 'TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkg3d', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
