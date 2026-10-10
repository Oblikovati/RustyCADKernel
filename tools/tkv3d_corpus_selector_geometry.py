"""Native point construction, scale-only cache identity and ordered depth-write layers."""
from __future__ import annotations
import random


def append_selector_geometry_requests(requests: list[str]) -> None:
    """Exercise all branches plus float normalization thresholds and general transforms."""
    rng = random.Random(729316)
    for kind in range(4):
        for projection in range(3):
            for pick in range(4):
                for normal in [(0,0,0),(3,4,0),(0.0001,0,0),(0.00034526698,0,0),(1e20,1e20,1e20)]:
                    for tolerance_kind in range(3):
                        transform, initial = rng.randrange(5), rng.randrange(-1,5)
                        requests.append(f'v3_selector_point {kind} {projection} {pick} {transform} {initial} {rng.choice([1,3,33])} {tolerance_kind} '+
                                        ' '.join(map(str,normal))+f' {rng.choice([-1,0,2.5])} {rng.choice([0,0.1,2.])} {rng.choice([-3.,0.,7.])}')
    for kind in range(4):
        # OCCT 3d097a0 cacheVertexProjections dereferences Camera() during
        # rectangular scaling/transformation. Null-camera point updates remain
        # covered above; rebuilding a rectangle requires a retained camera.
        for projection in ([1,2] if kind in (1,2) else range(3)):
            for initial in [-1,0,1,2,3,4]:
                for transform in range(5):
                    for custom in [-1,0,2]:
                        requests.append(f'v3_selector_frustum {kind} {projection} {initial} {transform} {rng.choice([1,3,12])} {rng.choice([2,10,20])} {custom}')
    for seed in range(1024):
        requests.append(f'v3_selector_layers {seed}')

    for kind in (1,2):
        requests.append(f"v3_selector_frustum {kind} 0 -1 0 1 2 -1")

    for count in range(5):
        for view in range(2):
            requests.append(f"v3_selector_clear {count} {view}")
