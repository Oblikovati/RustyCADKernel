"""Deterministic request corpus for the TKFillet numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkfillet_probe.cxx
and crates/tkfillet/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkfillet_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkfillet_requests(requests: list[str]) -> None:
    """Append every TKFillet request family in a fixed order."""
    # ChFiDS (tkfillet_corpus_chfids.py)
    __import__('tkfillet_corpus_chfids').append_chfids_requests(requests)
