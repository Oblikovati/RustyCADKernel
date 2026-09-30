"""Deterministic request corpus for the TKPrim numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkprim_probe.cxx and
crates/tkprim/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkprim_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkprim_requests(requests: list[str]) -> None:
    """Append every TKPrim request family in a fixed order."""
    # The shared shape digest (tkprim_corpus_shape.py)
    __import__('tkprim_corpus_shape').append_shape_requests(requests)
    # Sweep (tkprim_corpus_sweep.py)
    __import__('tkprim_corpus_sweep').append_sweep_requests(requests)
    # BRepPrim (tkprim_corpus_brepprim.py)
    __import__('tkprim_corpus_brepprim').append_brepprim_requests(requests)
    # BRepSweep (tkprim_corpus_brepsweep.py, shapes of tkprim_corpus_sweep_shapes.py)
    __import__('tkprim_corpus_brepsweep').append_brepsweep_requests(requests)
