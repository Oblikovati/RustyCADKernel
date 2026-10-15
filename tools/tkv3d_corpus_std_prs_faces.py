"""StdPrs face and surface presentations on the faces of tools/oracle/tkv3d_probe_std_prs_faces.cxx
(plane, full and half cylinder, sphere, cone, torus, B-spline and Bezier patches, infinite plane).

A request is `v3_stdprs_face Op Face NbUIso NbVIso Discretisation Deviation MaxParam Relative
Values...`: op 0 shape tool, 1/3 restricted face isolines (fixed/deflection), 2/4 their point
matching, 5/6 adaptor surface wireframes, 7 shading, 8 pole net, 9 pole curve, 10/11 HLR.
"""
from __future__ import annotations

FACES = range(9)
DRAWERS = ('1 1 5 0.01 100 0', '3 2 9 0.05 50 1', '0 4 2 0.2 1000 0')
POINTS = ('0.5 -1 0.25 0.1', '1.5 0 0.25 0.3', '2.5 -1 1 0.5', '0 0 0 2')
CAMERAS = ('1 10 8 6 0 0 0 0 0 1 1', '0 -5 7 9 1 0 0 0 1 0 2', '1 0 0 10 0 0 0 0 1 0 0.5')


def append_std_prs_face_requests(requests: list[str]) -> None:
    """Every op on every face with three drawers (iso numbers, sampling, deviation)."""
    points = ' '.join(POINTS)
    for face in FACES:
        for index, drawer in enumerate(DRAWERS):
            head = f'{drawer}'
            requests.append(f'v3_stdprs_face 0 {face} {head} {index % 2}')
            for draw in ('1 1', '1 0', '0 1'):
                requests.append(f'v3_stdprs_face 1 {face} {head} {draw} {index + 1} {3 - index}')
                requests.append(f'v3_stdprs_face 3 {face} {head} {draw} {index + 1} {3 - index} {0.01 * (index + 1)}')
            requests.append(f'v3_stdprs_face 2 {face} {head} {points}')
            requests.append(f'v3_stdprs_face 4 {face} {head} {points}')
            for op in (5, 6, 7, 8):
                requests.append(f'v3_stdprs_face {op} {face} {head}')
            requests.append(f'v3_stdprs_face 9 {face} {head} {index % 2} {points}')
            for camera in CAMERAS:
                for op in (10, 11):
                    requests.append(f'v3_stdprs_face {op} {face} {head} {camera}')
