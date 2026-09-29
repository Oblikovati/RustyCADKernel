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
    # ExtremaPC2d, GCPnts_TangentialDeflection (tkgeombase_corpus_extrema_pc2d.py)
    __import__('tkgeombase_corpus_extrema_pc2d').append_extrema_pc2d_requests(requests)
    # ExtremaPC (tkgeombase_corpus_extrema_pc.py)
    __import__('tkgeombase_corpus_extrema_pc').append_extrema_pc_requests(requests)
    # GeomTools (tkgeombase_corpus_geomtools.py)
    __import__('tkgeombase_corpus_geomtools').append_geomtools_requests(requests)
    # FEmTool (tkgeombase_corpus_femtool.py)
    __import__('tkgeombase_corpus_femtool').append_femtool_requests(requests)
    # AppParCurves (tkgeombase_corpus_appparcurves.py)
    __import__('tkgeombase_corpus_appparcurves').append_appparcurves_requests(requests)
    # GeomBndLib and BndLib (tkgeombase_corpus_bndlib.py)
    __import__('tkgeombase_corpus_bndlib').append_bndlib_requests(requests)
    # CPnts and GCPnts (tkgeombase_corpus_gcpnts.py)
    __import__('tkgeombase_corpus_gcpnts').append_gcpnts_requests(requests)
    # AppCont (tkgeombase_corpus_appcont.py)
    __import__('tkgeombase_corpus_appcont').append_appcont_requests(requests)
    # AdvApp2Var (tkgeombase_corpus_advapp2var.py)
    __import__('tkgeombase_corpus_advapp2var').append_advapp2var_requests(requests)
    # Geom2dConvert (tkgeombase_corpus_geom2dconvert.py)
    __import__('tkgeombase_corpus_geom2dconvert').append_geom2dconvert_requests(requests)
