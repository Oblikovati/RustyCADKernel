"""Deterministic request corpus for the TKTopAlgo numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tktopalgo_probe.cxx
and crates/tktopalgo/examples/numeric_probe.rs. Each ported package appends its request
families from its own tktopalgo_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tktopalgo_requests(requests: list[str]) -> None:
    """Append every TKTopAlgo request family in a fixed order."""
    # BRepBuilderAPI (tktopalgo_corpus_brepbuilderapi.py)
    __import__('tktopalgo_corpus_brepbuilderapi').append_brepbuilderapi_requests(requests)
    # BRepClass (tktopalgo_corpus_brepclass.py, faces of tktopalgo_corpus_faces.py)
    __import__('tktopalgo_corpus_brepclass').append_brepclass_requests(requests)
    # BRepTopAdaptor (tktopalgo_corpus_breptopadaptor.py, faces of tktopalgo_corpus_adaptor_faces.py)
    __import__('tktopalgo_corpus_breptopadaptor').append_breptopadaptor_requests(requests)
    # IntCurvesFace (tktopalgo_corpus_intcurvesface.py, faces of tktopalgo_corpus_adaptor_faces.py)
    __import__('tktopalgo_corpus_intcurvesface').append_intcurvesface_requests(requests)
    # Bisector (tktopalgo_corpus_bisector.py)
    __import__('tktopalgo_corpus_bisector').append_bisector_requests(requests)
    # MAT and MAT2d (tktopalgo_corpus_mat.py)
    __import__('tktopalgo_corpus_mat').append_mat_requests(requests)
    # BRepLib (tktopalgo_corpus_breplib.py and its family modules)
    __import__('tktopalgo_corpus_breplib').append_breplib_requests(requests)
