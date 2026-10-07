"""Deterministic request corpus for the TKV3d numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkv3d_probe.cxx and
crates/tkv3d/examples/numeric_probe.rs. Each ported package appends its request families
from its own tkv3d_corpus_<family>.py module, one line per family below. The verifier deals
the requests round-robin to parallel shards (tools/probe_shards.py), so a request prints
exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkv3d_requests(requests: list[str]) -> None:
    """Append every TKV3d request family in a fixed order."""
    # Prs3d tools, arrows, deflection and boxes (tkv3d_corpus_prs3d.py)
    __import__('tkv3d_corpus_prs3d').append_prs3d_requests(requests)
    # V3d view camera operations (tkv3d_corpus_v3d.py)
    __import__('tkv3d_corpus_v3d').append_v3d_requests(requests)
