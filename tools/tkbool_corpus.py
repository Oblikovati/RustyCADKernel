"""Deterministic request corpus for the TKBool numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkbool_probe.cxx and
crates/tkbool/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkbool_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkbool_requests(requests: list[str]) -> None:
    """Append every TKBool request family in a fixed order."""
    # BRepFill_Generator (tkbool_corpus_brepfill_generator.py)
    __import__('tkbool_corpus_brepfill_generator').append_generator_requests(requests)
    # The BRepFill package functions (tkbool_corpus_brepfill_package.py)
    __import__('tkbool_corpus_brepfill_package').append_package_requests(requests)
    # BRepFill_CompatibleWires (tkbool_corpus_brepfill_compatible.py)
    __import__('tkbool_corpus_brepfill_compatible').append_compatible_requests(requests)
