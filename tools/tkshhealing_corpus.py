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
    # ShapeAnalysis_Geom, _ShapeTolerance, _Shell, _WireOrder, _FreeBoundData (tkshhealing_corpus_analysis_basic.py)
    __import__('tkshhealing_corpus_analysis_basic').append_analysis_basic_requests(requests)
    # ShapeFix_Root, _ShapeTolerance, _EdgeConnect (tkshhealing_corpus_fix_basic.py)
    __import__('tkshhealing_corpus_fix_basic').append_fix_basic_requests(requests)
    # ShapeCustom (tkshhealing_corpus_shapecustom.py)
    __import__('tkshhealing_corpus_shapecustom').append_shapecustom_requests(requests)
    # ShapeUpgrade (tkshhealing_corpus_shapeupgrade.py)
    __import__('tkshhealing_corpus_shapeupgrade').append_shapeupgrade_requests(requests)
    # ShapeAnalysis_Edge recovery: all overloads and degenerate geometry.
    __import__('tkshhealing_corpus_analysis_edge').append_analysis_edge_requests(requests)
    # ShapeAnalysis_CanonicalRecognition: state, output geometry and fitting seeds.
    __import__('tkshhealing_corpus_canonical').append_canonical_requests(requests)
    # ShapeConstruct_Curve: adjustments, B-spline conversions and OCC90 knot spacing.
    __import__('tkshhealing_corpus_shapeconstruct').append_shapeconstruct_requests(requests)
    # ShapeAnalysis_Surface: singularity state, projection and closure.
    __import__('tkshhealing_corpus_analysis_surface').append_analysis_surface_requests(requests)

    # ShapeConstruct_ProjectCurveOnSurface: geometry, cache and statuses.
    __import__('tkshhealing_corpus_project_curve').append_project_curve_requests(requests)

    # ShapeUpgrade_ShellSewing: topology, tolerance and persistent shell history.
    __import__('tkshhealing_corpus_shell_sewing').append_shell_sewing_requests(requests)

    # ShapeAnalysis_FreeBounds and Wire connectivity: chaining and partial outputs.
    __import__('tkshhealing_corpus_free_bounds').append_free_bounds_requests(requests)

    # SplitCurve3d: defined cases only; all 16 native crashes require verify_split_curve3d.py.
    __import__('tkshhealing_corpus_split_curve3d').append_split_curve3d_requests(requests)
    # Surface sweeps retain 16 crash divergences in verify_split_surface.py.
    __import__('tkshhealing_corpus_split_surface').append_split_surface_requests(requests)
    # Offset recursion retains 70 crash divergences in verify_split_curve2d.py.
    __import__('tkshhealing_corpus_split_curve2d').append_split_curve2d_requests(requests)

    # EdgeDivide criteria and FixSmallCurves base preserve real curve geometry.
    __import__('tkshhealing_corpus_edge_divide').append_edge_divide_requests(requests)
    __import__('tkshhealing_corpus_edge_divide').append_fix_small_curve_requests(requests)
