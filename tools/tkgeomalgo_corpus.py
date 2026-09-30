"""Deterministic request corpus for the TKGeomAlgo numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkgeomalgo_probe.cxx
and crates/tkgeomalgo/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkgeomalgo_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkgeomalgo_requests(requests: list[str]) -> None:
    """Append every TKGeomAlgo request family in a fixed order."""
    # Intf (tkgeomalgo_corpus_intf.py)
    __import__('tkgeomalgo_corpus_intf').append_intf_requests(requests)
    # TopTrans (tkgeomalgo_corpus_toptrans.py)
    __import__('tkgeomalgo_corpus_toptrans').append_toptrans_requests(requests)
    # GccEnt (tkgeomalgo_corpus_gccent.py)
    __import__('tkgeomalgo_corpus_gccent').append_gccent_requests(requests)
    # GccInt (tkgeomalgo_corpus_gccint.py)
    __import__('tkgeomalgo_corpus_gccint').append_gccint_requests(requests)
    # Hatch (tkgeomalgo_corpus_hatch.py)
    __import__('tkgeomalgo_corpus_hatch').append_hatch_requests(requests)
    # GccAna (tkgeomalgo_corpus_gccana.py)
    __import__('tkgeomalgo_corpus_gccana').append_gccana_requests(requests)
    # IntRes2d, IntImpParGen, IntCurve (tkgeomalgo_corpus_intcurve.py)
    __import__('tkgeomalgo_corpus_intcurve').append_intcurve_requests(requests)
    # Geom2dInt, Geom2dAPI (tkgeomalgo_corpus_geom2d.py)
    __import__('tkgeomalgo_corpus_geom2d').append_geom2d_requests(requests)
    # Geom2dInt curve-curve, Geom2dAPI_InterCurveCurve (tkgeomalgo_corpus_ginter.py)
    __import__('tkgeomalgo_corpus_ginter').append_ginter_family_requests(requests)
    # IntSurf (tkgeomalgo_corpus_intsurf.py)
    __import__('tkgeomalgo_corpus_intsurf').append_intsurf_requests(requests)
    # Law (tkgeomalgo_corpus_law.py)
    __import__('tkgeomalgo_corpus_law').append_law_requests(requests)
    # Plate (tkgeomalgo_corpus_plate.py)
    __import__('tkgeomalgo_corpus_plate').append_plate_requests(requests)
    # LocalAnalysis (tkgeomalgo_corpus_localanalysis.py)
    __import__('tkgeomalgo_corpus_localanalysis').append_localanalysis_requests(requests)
    # FairCurve (tkgeomalgo_corpus_faircurve.py)
    __import__('tkgeomalgo_corpus_faircurve').append_faircurve_requests(requests)
    # IntImp (tkgeomalgo_corpus_intimp.py)
    __import__('tkgeomalgo_corpus_intimp').append_intimp_requests(requests)
    # AppBlend (tkgeomalgo_corpus_appblend.py)
    __import__('tkgeomalgo_corpus_appblend').append_appblend_requests(requests)
    # IntWalk (tkgeomalgo_corpus_intwalk.py)
    __import__('tkgeomalgo_corpus_intwalk').append_intwalk_requests(requests)
