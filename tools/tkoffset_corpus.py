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
    # BRepOffsetAPI_ThruSections (tkoffset_corpus_brepoffsetapi_thru_sections.py)
    __import__('tkoffset_corpus_brepoffsetapi_thru_sections').append_thru_sections_requests(requests)
    # BRepOffsetAPI_MakePipe, MakeEvolved, MakeDraft, MakeFilling, NormalProjection and
    # FindContigousEdges (tkoffset_corpus_brepoffsetapi_wrappers.py)
    __import__('tkoffset_corpus_brepoffsetapi_wrappers').append_wrapper_requests(requests)
    # BRepOffset_Tool (tkoffset_corpus_brepoffset_tool.py)
    __import__('tkoffset_corpus_brepoffset_tool').append_tool_requests(requests)
    # BRepOffset_Offset (tkoffset_corpus_brepoffset_offset.py)
    __import__('tkoffset_corpus_brepoffset_offset').append_offset_requests(requests)
    # BRepOffset_Inter3d (tkoffset_corpus_brepoffset_inter3d.py)
    __import__('tkoffset_corpus_brepoffset_inter3d').append_inter3d_requests(requests)
