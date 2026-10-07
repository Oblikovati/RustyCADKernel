"""Deterministic request corpus for the TKService numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkservice_probe.cxx
and crates/tkservice/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkservice_corpus_<family>.py module, one line per family below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkservice_requests(requests: list[str]) -> None:
    """Append every TKService request family in a fixed order."""
    # Aspect grids (tkservice_corpus_grid.py)
    __import__('tkservice_corpus_grid').append_grid_requests(requests)
    # WNT_HIDSpaceMouse (tkservice_corpus_hid.py)
    __import__('tkservice_corpus_hid').append_hid_requests(requests)
    # Image_PixMap pixel conversions (tkservice_corpus_pixel.py)
    __import__('tkservice_corpus_pixel').append_pixel_requests(requests)
    # Graphic3d_PBRMaterial and Graphic3d_BSDF (tkservice_corpus_pbr.py)
    __import__('tkservice_corpus_pbr').append_pbr_requests(requests)
