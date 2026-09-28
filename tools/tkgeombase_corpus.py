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
    # GC (tkgeombase_corpus_gc.py, tkgeombase_corpus_gc_surfaces.py)
    __import__('tkgeombase_corpus_gc').append_gc_curve_requests(requests)
    # IntAna2d (tkgeombase_corpus_intana2d.py)
    __import__('tkgeombase_corpus_intana2d').append_intana2d_requests(requests)
    # IntAna (tkgeombase_corpus_intana.py, tkgeombase_corpus_intana_geo.py)
    __import__('tkgeombase_corpus_intana').append_intana_requests(requests)
    # LProp, GeomLProp, GProp and Hermit (tkgeombase_corpus_lprop.py)
    __import__('tkgeombase_corpus_lprop').append_lprop_requests(requests)
