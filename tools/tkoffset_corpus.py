"""Deterministic request corpus for the TKOffset numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkoffset_probe.cxx and
crates/tkoffset/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkoffset_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkoffset_requests(requests: list[str]) -> None:
    """Append every TKOffset request family in a fixed order."""
    # BRepOffsetAPI_MakeOffset (tkoffset_corpus_brepoffsetapi_make_offset.py)
    __import__('tkoffset_corpus_brepoffsetapi_make_offset').append_make_offset_requests(requests)
    # BRepOffsetAPI_MakePipeShell (tkoffset_corpus_brepoffsetapi_make_pipe_shell.py)
    __import__('tkoffset_corpus_brepoffsetapi_make_pipe_shell').append_pipe_shell_requests(requests)
