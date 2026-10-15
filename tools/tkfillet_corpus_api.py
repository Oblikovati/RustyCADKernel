"""Deterministic requests for the BRepFilletAPI probe (tools/oracle/tkfillet_probe_api.cxx).

`fl_make_fillet` fillets or chamfers one edge, or every edge, of a box, of the pyramid of
a wedge (its apex has four edges), of a cylinder, of a box with a rounded vertical edge,
and of a box with a boss, a pocket, a notch or a step made by the Boolean operations:
constant and evolving fillets, and chamfers of one or two distances or of a distance and
an angle. The result is compared by its face and edge counts, its volume and area bits
and its validity. `fl_fillet_surf` computes or simulates the fillet surfaces of
FilletSurf_Builder on the same solids.
"""
from __future__ import annotations

# The solids of the probe and the number of their edges (the requests also run past the
# last edge, where both kernels refuse the empty contour).
NEDGES = {0: 12, 1: 8, 2: 18, 3: 24, 4: 18, 5: 24, 6: 3, 7: 20}
METHODS = ((0, 0.5, 0), (2, 0.5, 0), (1, 0.3, 0.6), (3, 0.4, 0.7), (4, 0.5, 0.6),
           (0, 2.0, 0), (2, 1.5, 0))


def append_api_requests(requests: list[str]) -> None:
    """Append the BRepFilletAPI request families."""
    for kind, n in NEDGES.items():
        for edge in list(range(1, n + 1)) + [0]:
            for method, d1, d2 in METHODS:
                requests.append('fl_make_fillet %d %d %d %r %r' % (kind, edge, method, d1, d2))
    for method in (5, 6):
        requests.append('fl_make_fillet 0 1 %d 2.0 0.0' % method)
    append_fillet_surf_requests(requests)


def append_fillet_surf_requests(requests: list[str]) -> None:
    """`fl_fillet_surf`: FilletSurf_Builder on each edge (computed and simulated, two
    radii), on runs of two and three consecutive edges of the map (mostly refused as not
    G1 or not sharp), and on no edge."""
    for kind, n in NEDGES.items():
        for e in range(1, n + 1):
            for rad in (1.0, 2.0):
                for sim in (0, 1):
                    requests.append('fl_fillet_surf %d %d 0 0 %r %d' % (kind, e, rad, sim))
        for e in range(1, n):
            requests.append('fl_fillet_surf %d %d %d 0 1.0 0' % (kind, e, e + 1))
            if e + 2 <= n:
                requests.append('fl_fillet_surf %d %d %d %d 1.0 0' % (kind, e, e + 1, e + 2))
        requests.append('fl_fillet_surf %d 0 0 0 1.0 0' % kind)
