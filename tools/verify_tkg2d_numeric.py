#!/usr/bin/env python3
"""Compare the TKG2d port (Geom2d, Adaptor2d, Geom2dAdaptor, Geom2dEval, Geom2dGridEval, Geom2dHash) with the pinned kernel on a deterministic corpus.

Usage: python3 tools/verify_tkg2d_numeric.py --source OCCT --native-build BUILD
Builds tools/oracle/tkg2d_probe.cxx against libTKG2d and the Rust twin
crates/tkg2d/examples/numeric_probe.rs, runs both on tools/tkg2d_corpus.py and
requires identical bits (NaN compared by classification), as the TKMath verifier does.
"""
from __future__ import annotations
from tkg2d_adaptor_corpus import append_tkg2d_adaptor_requests
from tkg2d_corpus import append_tkg2d_requests
from tkg2d_eval_corpus import append_tkg2d_eval_requests
from tkg2d_grid_corpus import append_tkg2d_grid_requests
from toolkit_numeric import NumericOracle, verify_numeric


def corpus() -> str:
    """The whole TKG2d request corpus, one request per line."""
    requests: list[str] = []
    append_tkg2d_requests(requests)
    append_tkg2d_adaptor_requests(requests)
    append_tkg2d_eval_requests(requests)
    append_tkg2d_grid_requests(requests)
    return '\n'.join(requests)+'\n'


ORACLE = NumericOracle('TKG2d', 'tools/oracle/tkg2d_probe.cxx', ['TKG2d', 'TKMath', 'TKernel'],
                       'rustycad-tkg2d', corpus)


if __name__ == '__main__':
    verify_numeric(ORACLE)
