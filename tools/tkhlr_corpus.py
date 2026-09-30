"""Deterministic request corpus for the TKHLR numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkhlr_probe.cxx
and crates/tkhlr/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkhlr_corpus_<package>.py module, one line per package below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkhlr_requests(requests: list[str]) -> None:
    """Append every TKHLR request family in a fixed order."""
    # HLRAlgo_Projector viewing frame (tkhlr_corpus_frame.py)
    __import__('tkhlr_corpus_frame').append_frame_requests(requests)
    # Intrv (tkhlr_corpus_intrv.py)
    __import__('tkhlr_corpus_intrv').append_intrv_requests(requests)
    # TopBas and TopCnx (tkhlr_corpus_topcnx.py)
    __import__('tkhlr_corpus_topcnx').append_topcnx_requests(requests)
    # HLRAlgo (tkhlr_corpus_hlralgo.py, tkhlr_corpus_hlralgo_poly.py)
    __import__('tkhlr_corpus_hlralgo').append_hlralgo_requests(requests)
    __import__('tkhlr_corpus_hlralgo_poly').append_hlralgo_poly_requests(requests)
    __import__('tkhlr_corpus_contap').append_contap_requests(requests)
    __import__('tkhlr_corpus_contana').append_contana_requests(requests)
    __import__('tkhlr_corpus_contap_surface').append_contap_surface_requests(requests)
    __import__('tkhlr_corpus_hcurve').append_hcurve_requests(requests)
    __import__('tkhlr_corpus_hcont').append_hcont_requests(requests)
    __import__('tkhlr_corpus_function').append_function_requests(requests)
    __import__('tkhlr_corpus_arc_function').append_arc_function_requests(requests)
    __import__('tkhlr_corpus_iwline').append_iwline_requests(requests)
    __import__('tkhlr_corpus_search_records').append_search_record_requests(requests)
    __import__('tkhlr_corpus_search_inside').append_search_inside_requests(requests)
    __import__('tkhlr_corpus_search_boundary').append_boundary_requests(requests)
