"""Deterministic request corpus for the TKShHealing numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkshhealing_probe.cxx
and crates/tkshhealing/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkshhealing_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkshhealing_requests(requests: list[str]) -> None:
    """Append every TKShHealing request family in a fixed order."""
    # ShapeExtend (tkshhealing_corpus_shapeextend.py)
    __import__('tkshhealing_corpus_shapeextend').append_shapeextend_requests(requests)
