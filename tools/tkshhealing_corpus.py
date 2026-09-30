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
    # ShapeExtend_WireData (tkshhealing_corpus_wiredata.py)
    __import__('tkshhealing_corpus_wiredata').append_wiredata_requests(requests)
    # ShapeExtend_Explorer (tkshhealing_corpus_explorer.py)
    __import__('tkshhealing_corpus_explorer').append_explorer_requests(requests)
    # ShapeExtend registrators (tkshhealing_corpus_registrator.py)
    __import__('tkshhealing_corpus_registrator').append_registrator_requests(requests)
    # ShapeExtend_ComplexCurve (tkshhealing_corpus_complexcurve.py)
    __import__('tkshhealing_corpus_complexcurve').append_complexcurve_requests(requests)
    # ShapeExtend_CompositeSurface (tkshhealing_corpus_composite.py)
    __import__('tkshhealing_corpus_composite').append_composite_requests(requests)
    # ShapeBuild (tkshhealing_corpus_shapebuild.py)
    __import__('tkshhealing_corpus_shapebuild').append_shapebuild_requests(requests)
    # ShapeCustom (tkshhealing_corpus_shapecustom.py)
    __import__('tkshhealing_corpus_shapecustom').append_shapecustom_requests(requests)
    # ShapeUpgrade (tkshhealing_corpus_shapeupgrade.py)
    __import__('tkshhealing_corpus_shapeupgrade').append_shapeupgrade_requests(requests)
