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
    # GccEnt (tkgeomalgo_corpus_gccent.py)
    __import__('tkgeomalgo_corpus_gccent').append_gccent_requests(requests)
    # GccInt (tkgeomalgo_corpus_gccint.py)
    __import__('tkgeomalgo_corpus_gccint').append_gccint_requests(requests)
    # Hatch (tkgeomalgo_corpus_hatch.py)
    __import__('tkgeomalgo_corpus_hatch').append_hatch_requests(requests)
    # GccAna (tkgeomalgo_corpus_gccana.py)
    __import__('tkgeomalgo_corpus_gccana').append_gccana_requests(requests)
