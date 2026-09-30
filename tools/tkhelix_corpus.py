"""Deterministic request corpus for the TKHelix numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkhelix_probe.cxx
and crates/tkhelix/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkhelix_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkhelix_requests(requests: list[str]) -> None:
    """Append every TKHelix request family in a fixed order."""
    # HelixGeom_BuilderHelix placement (tkhelix_corpus_placement.py)
    __import__('tkhelix_corpus_placement').append_placement_requests(requests)
    # HelixGeom (tkhelix_corpus_helixgeom.py)
    __import__('tkhelix_corpus_helixgeom').append_helixgeom_requests(requests)
