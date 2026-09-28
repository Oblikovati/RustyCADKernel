"""Deterministic request corpus for the TKGeomBase numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkgeombase_probe.cxx
and crates/tkgeombase/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkgeombase_corpus_<package>.py module, one line per package below.
"""
from __future__ import annotations


def append_tkgeombase_requests(requests: list[str]) -> None:
    """Append every TKGeomBase request family in a fixed order."""
    # gce (tkgeombase_corpus_gce.py)
    __import__('tkgeombase_corpus_gce').append_gce_requests(requests)
