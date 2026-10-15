"""Deterministic requests for BiTgte_Blend (tools/oracle/tkoffset_probe_bitgte.cxx; the record is
documented there).

The rolling ball along edges of a box (one edge, two edges, with a stop face), between
two adjacent box faces, along the circular edge of a cylinder, with and without the
blended shape, and between the two cylindrical faces of OCC29861 (brollingball and
rollingball).
"""
from __future__ import annotations
from tkoffset_corpus_brepoffset_tool import BOX, CYLINDER


def request(shape: str, radius: float, build: int, pairs=(), edges=(), stops=(), tol: float = 1.0e-4) -> str:
    """One blend: `shape` is `0 <primitive>` or `1` (OCC29861)."""
    body = f'{len(pairs)} ' + ' '.join(f'{a} {b}' for a, b in pairs)
    body += f' {len(edges)} ' + ' '.join(map(str, edges))
    body += f' {len(stops)} ' + ' '.join(map(str, stops))
    return ' '.join(f'obt_blend {shape} {radius} {tol} {build} {body}'.split())


def append_bitgte_requests(requests: list[str]) -> None:
    """The blends in a fixed order."""
    box = f'0 {BOX}'
    cylinder = f'0 {CYLINDER}'
    for build in (0, 1):
        requests.append(request(box, 0.5, build, edges=[0]))
        requests.append(request(box, 0.5, build, edges=[0, 1]))
        requests.append(request(box, 0.5, build, edges=[0], stops=[4]))
        requests.append(request(box, 0.4, build, pairs=[(0, 2)]))
        requests.append(request(cylinder, 0.3, build, edges=[0]))
        requests.append(request('1', 30, build, pairs=[(0, 1)]))
        requests.append(request(box, -0.5, build, edges=[0]))
        requests.append(request(box, -0.4, build, pairs=[(0, 2)]))
