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
    # IntSurf (tkgeomalgo_corpus_intsurf.py)
    __import__('tkgeomalgo_corpus_intsurf').append_intsurf_requests(requests)
    # Law (tkgeomalgo_corpus_law.py)
    __import__('tkgeomalgo_corpus_law').append_law_requests(requests)
    # Plate (tkgeomalgo_corpus_plate.py)
    __import__('tkgeomalgo_corpus_plate').append_plate_requests(requests)
    # LocalAnalysis (tkgeomalgo_corpus_localanalysis.py)
    __import__('tkgeomalgo_corpus_localanalysis').append_localanalysis_requests(requests)
    # FairCurve (tkgeomalgo_corpus_faircurve.py)
    __import__('tkgeomalgo_corpus_faircurve').append_faircurve_requests(requests)
    # IntImp (tkgeomalgo_corpus_intimp.py)
    __import__('tkgeomalgo_corpus_intimp').append_intimp_requests(requests)
    # AppBlend (tkgeomalgo_corpus_appblend.py)
    __import__('tkgeomalgo_corpus_appblend').append_appblend_requests(requests)
