"""Deterministic requests for BRepBuilderAPI_Sewing (tools/oracle/tktopalgo_probe_sewing.cxx).

Each shape of tools/tktopalgo_corpus_sewing_shapes.py is sewn with the default options,
then with each option or mode changed: without cutting, non-manifold, without analysis,
analysis only, without SameParameter, without the face mode, with the floating edges
mode, with minimal and maximal tolerances, and through Add of the whole shape, Load, and
Load then Add, with an own context.
"""
from __future__ import annotations
from tktopalgo_corpus_breplib_records import numbers
from tktopalgo_corpus_sewing_shapes import (
    bezier_bulge, bowed_fin, faces_and_edge, fin_on_cylinder, floating_edges, floating_nodes, glued_chain, holed_square,
    located_squares, loose_box, open_cylinder, open_torus, reversed_pair, sliver_junction, small_pieces, split_fin,
    squares, t_junction, tiny_gap)
from tktopalgo_corpus_solids import box, cylinder, sphere, torus

# tol, o1 sewing, o2 analysis, o3 cutting, o4 non-manifold, face mode, floating mode,
# SameParameter mode, min tolerance, max tolerance, local tolerances, mode, context.
DEFAULT = dict(tol=1e-6, o1=1, o2=1, o3=1, o4=0, face=1, floating=0, samepar=1, mintol=-1.0, maxtol=-1.0,
               local=0, mode=0, context=0, summary=0)
KEYS = list(DEFAULT)


def request(shape: list[float], nm: list[tuple[int, float]], **options: float) -> str:
    fields = {**DEFAULT, **options}
    nm_fields = [len(nm)] + [v for pair in nm for v in pair]
    return 'bba_sew ' + numbers(shape + nm_fields + [fields[k] for k in KEYS])


def variants(tol: float) -> list[dict[str, float]]:
    """The option sets every shape is sewn with."""
    return [dict(tol=tol), dict(tol=tol, o3=0), dict(tol=tol, o4=1), dict(tol=tol, o2=0), dict(tol=tol, o1=0),
            dict(tol=tol, samepar=0), dict(tol=tol, face=0), dict(tol=tol, floating=1), dict(tol=tol, mode=1),
            dict(tol=tol, mode=2), dict(tol=tol, mode=3), dict(tol=tol, context=1), dict(tol=tol, o4=1, o3=0),
            dict(tol=tol, local=1, mintol=tol * 0.5), dict(tol=tol, maxtol=tol * 1e-3)]


def shapes() -> list[tuple[list[float], float, int]]:
    """The shapes to sew, the tolerance of their requests and whether the sewn shape is
    summarised without order (floating edges are quilted in TShape address order)."""
    offsets = [(0., 0., 0.), (2e-4, 0., 0.), (0., 1e-4, 0.), (0., -1e-4, 1e-4), (1e-4, 1e-4, 0.), (0., 0., -2e-4)]
    ordered = [(loose_box(10.0, offsets), 1e-3), (loose_box(10.0, offsets), 1e-5), (loose_box(1.0, [(0., 0., 0.)]), 1e-6),
               (squares([(0., 0., 1., 1., 0.), (1. + 1e-4, 0., 2., 1., 0.)]), 1e-3),
               (squares([(0., 0., 1., 1., 0.), (1. + 1e-2, 0., 2., 1., 0.)]), 1e-3),
               (squares([(0., 0., 1., 1., 0.), (1., 0., 2., 1., 1e-4)]), 1e-3),
               (squares([(0., 0., 2., 1., 0.), (0., 1., 1., 2., 0.), (1., 1. + 1e-5, 2., 2., 0.)]), 1e-4),
               (squares([(0., 0., 3., 1., 0.), (0., 1., 1., 2., 0.), (1., 1., 2., 2., 0.), (2., 1., 3., 2., 0.)]), 1e-6),
               (t_junction(), 1e-6), (t_junction(1e-5), 1e-4), (small_pieces(1e-8, 5e-8), 1e-3),
               (open_cylinder(2.0, 3.0), 1e-6), (open_cylinder(2.0, 3.0, 1e-5), 1e-4),
               (located_squares(1.0), 1e-6), (located_squares(1.0 + 1e-5), 1e-4), (faces_and_edge(1e-5), 1e-4),
               (box([0., 0., 0.], [10., 10., 10.]), 1e-6), (cylinder([0., 0., 0.], 2.0, 3.0), 1e-6),
               (sphere([0., 0., 0.], 3.0), 1e-6), (torus([0., 0., 0.], 3.0, 1.0), 1e-6)]
    ordered += [(bezier_bulge(1e-3), 8.2e-4), (bezier_bulge(1e-3), 1e-3), (sliver_junction(1e-5, 2e-5), 1e-4),
                (sliver_junction(1e-5, 0.0), 1e-4), (fin_on_cylinder(2.0, 3.0), 1e-6), (glued_chain(5e-8), 1e-3),
                (bowed_fin(2.0, 3.0, 3e-7), 1e-6), (reversed_pair(1e-4), 1e-3),
                (squares([(0., 0., 2., 1., 0.), (0., 1., 2., 2., 0.), (0., 1., 1., 2., 0.), (1., 1., 2., 2., 0.)]), 1e-6),
                (squares([(0., 0., 1., 1e-5, 0.), (0., -1., 1., 0., 0.), (0., 1e-5, 1., 1., 0.)]), 1e-4),
                (open_torus(3.0, 1.0), 1e-6), (split_fin(2.0, 3.0), 1e-6), (holed_square(1e-5), 1e-4),
                (glued_chain(5e-8, (0, 1, 3, 2, 4, 5, 6)), 1e-3), (tiny_gap(5e-4), 1e-3), (tiny_gap(2e-4), 1e-3)]
    floating = [(floating_edges(1e-5).numbers(), 1e-4, 1), (floating_nodes(1e-4).numbers(), 0.9e-4, 1)]
    return [(spec.numbers(), tol, 0) for spec, tol in ordered] + floating


def nm_requests() -> list[str]:
    """Shapes with INTERNAL vertices on their edges (the non-manifold vertices kept on
    the merged and cut edges)."""
    offsets = [(0., 0., 0.), (2e-4, 0., 0.), (0., 1e-4, 0.), (0., -1e-4, 1e-4), (1e-4, 1e-4, 0.), (0., 0., -2e-4)]
    cut = squares([(0., 0., 2., 1., 0.), (0., 1., 1., 2., 0.), (1., 1. + 1e-5, 2., 2., 0.)]).numbers()
    cases = [(loose_box(10.0, offsets).numbers(), [(0, 5.0), (5, 2.5), (9, 0.0)], 1e-3),
             (cut, [(2, 0.5), (2, 1.5), (4, 0.25)], 1e-4),
             (squares([(0., 0., 1., 1., 0.), (1. + 1e-4, 0., 2., 1., 0.)]).numbers(), [(1, 0.5), (7, 0.5)], 1e-3)]
    requests = []
    for shape, nm, tol in cases:
        for options in (dict(tol=tol), dict(tol=tol, o4=1), dict(tol=tol, o3=0), dict(tol=tol, o1=0)):
            requests.append(request(shape, nm, **options))
    return requests


def append_sewing_requests(requests: list[str]) -> None:
    """Append the Sewing requests in a fixed order."""
    for shape, tol, summary in shapes():
        for options in variants(tol):
            requests.append(request(shape, [], summary=summary, **options))
    requests += nm_requests()
