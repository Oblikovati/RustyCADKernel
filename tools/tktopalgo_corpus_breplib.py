"""Deterministic requests for BRepLib (tools/oracle/tktopalgo_probe_breplib_*.cxx).

Each family lives in its own tktopalgo_corpus_breplib_<family>.py module, with the record
builders of tktopalgo_corpus_breplib_records.py; this module appends them in a fixed order.
"""
from __future__ import annotations
from tktopalgo_corpus_breplib_edge import append_breplib_edge_requests
from tktopalgo_corpus_breplib_face import append_breplib_face_requests
from tktopalgo_corpus_breplib_fuse import append_breplib_fuse_requests
from tktopalgo_corpus_breplib_package import append_breplib_package_requests
from tktopalgo_corpus_breplib_wire import append_breplib_wire_requests


def append_breplib_requests(requests: list[str]) -> None:
    """Append every BRepLib request family in a fixed order."""
    append_breplib_edge_requests(requests)
    append_breplib_wire_requests(requests)
    append_breplib_package_requests(requests)
    append_breplib_face_requests(requests)
    append_breplib_fuse_requests(requests)
