"""Deterministic request corpus for the TKGeomAlgo numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkgeomalgo_probe.cxx
and crates/tkgeomalgo/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkgeomalgo_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkgeomalgo_requests(requests: list[str]) -> None:
    """Append every TKGeomAlgo request family in a fixed order."""
    # Intf (tkgeomalgo_corpus_intf.py)
    __import__('tkgeomalgo_corpus_intf').append_intf_requests(requests)
    # TopTrans (tkgeomalgo_corpus_toptrans.py)
    __import__('tkgeomalgo_corpus_toptrans').append_toptrans_requests(requests)
    # IntRes2d, IntImpParGen, IntCurve (tkgeomalgo_corpus_intcurve.py)
    __import__('tkgeomalgo_corpus_intcurve').append_intcurve_requests(requests)
    # Geom2dInt, Geom2dAPI (tkgeomalgo_corpus_geom2d.py)
    __import__('tkgeomalgo_corpus_geom2d').append_geom2d_requests(requests)
