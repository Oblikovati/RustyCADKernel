"""Deterministic requests for the BRepBuilderAPI makers
(tools/oracle/tktopalgo_probe_brepbuilderapi_makers.cxx).

Each BRepBuilderAPI maker wraps the BRepLib maker of the same arguments, so its requests
are the BRepLib maker requests (tktopalgo_corpus_breplib_*.py) under the `bba_` operation:
every constructor, Init and Add with valid and failing inputs (each error value), later
Init calls on done and failed makers, and the default and changed precisions and planes.
`bba_package` sets and reads the package precision and plane.
"""
from __future__ import annotations
from tktopalgo_corpus_breplib_edge import (
    edge2d_requests, gp_constructor_requests, geom_constructor_requests, pcurve_constructor_requests,
    reinit_requests, segment_requests, vertex_requests)
from tktopalgo_corpus_breplib_face import gp_face_requests, geom_face_requests, wire_face_requests
from tktopalgo_corpus_breplib_package import shell_requests
from tktopalgo_corpus_breplib_records import numbers
from tktopalgo_corpus_breplib_wire import polygon_requests, solid_requests, wire_requests

# The BRepLib operation of each BRepBuilderAPI operation.
RENAMES = {'blm_vertex': 'bba_vertex', 'blm_edge': 'bba_edge', 'blm_edge2d': 'bba_edge2d',
           'blm_polygon': 'bba_polygon', 'blm_wire': 'bba_wire', 'blm_solid': 'bba_solid',
           'blm_face': 'bba_face', 'blp_shell': 'bba_shell'}


def renamed(requests: list[str]) -> list[str]:
    """The BRepLib requests under their BRepBuilderAPI operation."""
    result = []
    for request in requests:
        op, fields = request.split(' ', 1)
        result.append(RENAMES[op] + ' ' + fields)
    return result


def package_requests() -> list[str]:
    """`bba_package prec plane`: the defaults, a precision, a plane, and both."""
    planes = [[0], [1, 0., 0., 1., 0., 1., 0.], [1, 1., 2., 3., 0., 0.6, 0.8]]
    return ['bba_package ' + numbers([prec] + plane) for prec in (0., 1e-3) for plane in planes]


def append_brepbuilderapi_maker_requests(requests: list[str]) -> None:
    """Append the maker requests in a fixed order."""
    requests += package_requests()
    requests += renamed(vertex_requests())
    requests += renamed(segment_requests() + gp_constructor_requests() + geom_constructor_requests())
    requests += renamed(pcurve_constructor_requests() + reinit_requests() + edge2d_requests())
    requests += renamed(polygon_requests() + wire_requests() + solid_requests())
    requests += renamed(gp_face_requests() + geom_face_requests() + wire_face_requests())
    requests += renamed(shell_requests())
