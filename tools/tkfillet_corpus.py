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
    # BlendFunc (tkfillet_corpus_blendfunc.py)
    __import__('tkfillet_corpus_blendfunc').append_blendfunc_requests(requests)
    # BRepBlend (tkfillet_corpus_brepblend.py)
    __import__('tkfillet_corpus_brepblend').append_brepblend_requests(requests)
    # BRepBlend_Walking (tkfillet_corpus_walking.py)
    __import__('tkfillet_corpus_walking').append_walking_requests(requests)
    # BRepBlend line builders (tkfillet_corpus_linebuilders.py)
    __import__('tkfillet_corpus_linebuilders').append_linebuilder_requests(requests)
    # ChFi2d (tkfillet_corpus_chfi2d.py)
    __import__('tkfillet_corpus_chfi2d').append_chfi2d_requests(requests)
    # ChFi3d (tkfillet_corpus_chfi3d.py)
    __import__('tkfillet_corpus_chfi3d').append_chfi3d_requests(requests)
    # ChFi3d_Builder_0 (tkfillet_corpus_builder0.py)
    __import__('tkfillet_corpus_builder0').append_builder0_requests(requests)
    # ChFiKPart (tkfillet_corpus_chfikpart.py)
    __import__('tkfillet_corpus_chfikpart').append_chfikpart_requests(requests)
    # BRepFilletAPI (tkfillet_corpus_api.py)
    __import__('tkfillet_corpus_api').append_api_requests(requests)
