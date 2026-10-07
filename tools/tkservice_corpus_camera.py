"""Deterministic requests for Graphic3d_Camera (tools/oracle/tkservice_probe_camera.cxx).

`sv_camera` builds cameras of every projection type, depth range, aspect, axial scale, stereo
IOD and focus and tile, and compares the orientation and projection matrices in double and
float, the stereo projections, the point conversions, the frustum planes and points and the
derived values; `sv_camera_fit` compares ZFitAll and FitMinMax over boxes (including points,
lines and boxes behind the eye); `sv_camera_interp` compares Interpolate between cameras.
"""
from __future__ import annotations
import random

NO_TILE = [0, 0, 0, 0, 0, 0, 0]
DEFAULT = [0, 0, 0, 0, -1500, 0, 0, 0, 0, 1, 0, 1, 1, 1, 45.0, 1.0, 0.001, 3000.0, 1000.0, 1, 0.05, 1, 1.0] + NO_TILE


def numbers(values: list[float]) -> str:
    """The fields of a request, with every double spelled exactly."""
    return ' '.join(repr(float(v)) for v in values)


def camera(rng: random.Random) -> list[float]:
    """A random camera: projection, eye, center, up, axial scale, mapping, stereo and tile."""
    proj = rng.randrange(5)
    eye = [round(rng.uniform(-500.0, 500.0), 3) for _ in range(3)]
    center = [round(rng.uniform(-50.0, 50.0), 3) for _ in range(3)]
    up = [round(rng.uniform(-1.0, 1.0), 3) for _ in range(3)]
    axial = [rng.choice([1.0, 1.0, 0.5, 2.0, -1.0]) for _ in range(3)]
    mapping = [rng.choice([45.0, 30.0, 60.0, 10.0]), rng.choice([1.0, 1.5, 0.75, 2.0]),
               rng.choice([0.001, 0.5, 1.0, -10.0]), rng.choice([3000.0, 100.0, 0.25]), rng.uniform(10.0, 2000.0)]
    stereo = [rng.randrange(2), rng.choice([0.05, 0.0, 6.5]), rng.randrange(2), rng.choice([1.0, 0.5, 300.0])]
    tile = rng.choice([NO_TILE, [800, 600, 400, 300, 0, 0, 0], [800, 600, 200, 150, 100, 50, 1]])
    return [proj, rng.randrange(2), *eye, *center, *up, *axial, *mapping, *stereo, *tile]


def append_camera_requests(requests: list[str]) -> None:
    """Append the default camera, then seeded cameras, fits and interpolations."""
    point = [10.0, -20.0, 30.0]
    requests.append('sv_camera ' + numbers(DEFAULT + point))
    rng = random.Random(20261015)
    for _ in range(60):
        requests.append('sv_camera ' + numbers(camera(rng) + [round(rng.uniform(-100.0, 100.0), 3) for _ in range(3)]))
    boxes = [[-10, -10, -10, 10, 10, 10], [0, 0, 0, 0, 0, 0], [0, 0, 0, 10, 0, 0], [100, 200, 300, 400, 500, 600],
             [-5000, -5000, 2000, 5000, 5000, 3000]]
    for _ in range(40):
        model, graphic = rng.choice(boxes), rng.choice(boxes)
        requests.append('sv_camera_fit ' + numbers(camera(rng) + model + graphic + [1e-7, rng.randrange(2), rng.choice([1.0, 2.0, 0.0])]))
    for _ in range(30):
        requests.append('sv_camera_interp ' + numbers(camera(rng) + camera(rng) + [rng.choice([0.0, 1.0, 0.25, 0.5, rng.random()])]))
