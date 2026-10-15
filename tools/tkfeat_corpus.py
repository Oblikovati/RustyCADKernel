"""Deterministic request corpus for the TKFeat numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkfeat_probe.cxx
and crates/tkfeat/examples/numeric_probe.rs. The verifier deals the requests
round-robin to parallel shards (tools/probe_shards.py), so a request prints exactly one
line and never relies on an earlier request.

LocOpe: the swept shapes (prisms, drafted prisms, revolutions, linear and revolution
forms), the points of lines through a box, the confused edges of stacked boxes, the
splitter and the gluer. BRepFeat: the prism, drafted prism, revolution and pipe features
with each limit method on a box and a cylinder, the cylindrical holes, the linear and
revolution forms (ribs, slots and grooves, sliding or not), the gluer and the splitter.
The sliding revolution grooves are left out: the native kernel walks their section edges
without end and exhausts the memory. A few drafted cases are left out or moved because
libTKFeat itself varies by one unit in the last place between runs on them (the order of
the faces of their results follows addresses); the comments name them.
"""
from __future__ import annotations


def append_locope_requests(requests: list[str]) -> None:
    """The LocOpe families of tools/oracle/tkfeat_probe_locope.cxx."""
    for v in ((0., 0., 5.), (0., 0., -3.), (1., 2., 4.)):
        requests.append('ft_lo_sweep 0 %r %r %r' % v)
    for a, b in ((5., 0.), (4., -2.), (-3., 1.)):
        requests.append('ft_lo_sweep 1 %r %r 0.0' % (a, b))
    for h, ang in ((5., 0.), (5., 0.1), (3., -0.2), (-4., 0.15)):
        requests.append('ft_lo_sweep 2 %r %r 0.0' % (h, ang))
    # A drafted prism of two heights with a draft varies by one unit in the last place of
    # its volume from one native run to the next (its faces are ordered by address), so
    # only the straight one is compared.
    requests.append('ft_lo_sweep 3 3.0 2.0 0.0')
    for ang in (0.5, 1.5707963267948966, 3.141592653589793, 6.283185307179586):
        requests.append('ft_lo_sweep 4 %r 0.0 0.0' % ang)
        requests.append('ft_lo_sweep 7 %r 0.0 0.0' % ang)
    for ang, off in ((1., 0.5), (2., -1.)):
        requests.append('ft_lo_sweep 5 %r %r 0.0' % (ang, off))
    for h in (4., -2.):
        requests.append('ft_lo_sweep 6 %r 0.0 0.0' % h)
    for line in ((5., 5., 12., 0., 0., -1.), (2., 3., 15., 0., 0., -1.), (-2., 5., 5., 1., 0., 0.),
                 (20., 5., 5., 1., 0., 0.), (0., 0., 0., 1., 1., 1.), (5., 5., 5., 0., 1., 0.)):
        requests.append('ft_lo_csinter %r %r %r %r %r %r' % line)
    for ox, dz in ((0., 5.), (2., 5.), (-3., 2.), (10., 1.)):
        requests.append('ft_lo_find_edges %r %r' % (ox, dz))
    for x in (5., 2.5, 0.):
        requests.append('ft_lo_spliter %r' % x)
    for z, h in ((10., 2.), (8., 2.), (9., 3.)):
        requests.append('ft_lo_gluer %r %r' % (z, h))


def append_sweep_requests(requests: list[str]) -> None:
    """The BRepFeat sweep families of tools/oracle/tkfeat_probe_sweeps.cxx."""
    limits = ((0, 4., 0.), (0, 12., 0.), (1, 15., 0.), (1, 3., 0.), (2, 12., 16.), (2, 8., 2.),
              (3, 0., 0.), (4, 0., 0.), (5, 0., 3.), (5, 15., 2.), (6, 4., 0.), (7, 0., 0.))
    for kind in (0, 1):
        for mode, up in ((1, 1), (0, 0), (2, 1)):
            for method, a, b in limits:
                if mode == 0 and method == 0:
                    a = -a if a > 10. else a
                requests.append('ft_prism %d %d %d %d %r %r' % (kind, mode, up, method, a, b))
    for kind in (0, 1):
        for mode, height in ((1, 4.), (0, -4.)):
            # the draft 0.1 varies natively from run to run on the box pocket; 0.12 does not
            for ang in (0., 0.12):
                requests.append('ft_dprism %d %d %r 0 %r 0.0' % (kind, mode, ang, height))
                for method, a, b in limits[2:]:
                    # the drafted pocket between two faces varies by one unit in the last
                    # place of its volume or area between native runs (faces ordered by
                    # address); the port gives one of the native values
                    if (mode, ang, method) == (0, 0.12, 2):
                        continue
                    requests.append('ft_dprism %d %d %r %d %r %r' % (kind, mode, ang, method, a, b))
    for mode in (1, 0, 2):
        for method, a in ((0, 1.5707963267948966), (0, 3.141592653589793), (0, 6.283185307179586),
                          (1, 0.), (2, 0.), (3, 1.0), (4, 0.)):
            requests.append('ft_revol %d %d %r' % (mode, method, a))
    for mode, z1 in ((1, 14.), (0, 6.), (2, 13.)):
        for method in (0, 1, 2):
            requests.append('ft_pipe %d %r %d 12.0' % (mode, z1, method))


def append_form_requests(requests: list[str]) -> None:
    """The BRepFeat form families of tools/oracle/tkfeat_probe_forms.cxx."""
    for kind in (0, 2):
        for x in (5., 20.):
            for method, a, b in ((0, 0., 0.), (1, 0., 0.), (2, 0., 0.), (3, 0., 7.), (3, 4., 9.), (4, 6., 0.), (4, 13., 0.)):
                for cont in (0, 1):
                    requests.append('ft_hole %d %r %d 1.0 %r %r %d' % (kind, x, method, a, b, cont))
    for mode, h in ((1, 3.), (0, -3.)):
        for slide in (0, 1):
            for back in (0, 1):
                requests.append('ft_lform %d %d %d %r' % (mode, slide, back, h))
    for kind in (0, 1):
        for mode, slide in ((1, 0), (1, 1), (0, 0)):
            for h1, h2 in ((1., 1.), (1., 2.)):
                requests.append('ft_rform %d %d %d %r %r' % (kind, mode, slide, h1, h2))
    for z, h in ((10., 2.), (8., 2.), (9., 3.)):
        requests.append('ft_gluer %r %r' % (z, h))
    for x, n in ((5., 1), (2., 2), (0., 1), (3., 3)):
        requests.append('ft_split %r %d' % (x, n))


def append_tkfeat_requests(requests: list[str]) -> None:
    """Append every TKFeat request family in a fixed order."""
    append_locope_requests(requests)
    append_sweep_requests(requests)
    append_form_requests(requests)
