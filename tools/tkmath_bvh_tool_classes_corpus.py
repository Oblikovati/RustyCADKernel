"""Deterministic requests for the BVH namespace tool classes.

Covers ToBndBox (scalar, 2D, 3D, 4D), VecComp::Get including axes outside the
dimension, UnitVector, SurfaceCalculator::Area with its degenerate fallback,
BVH_AxisSelector::MainAxis with tied and signed components, CenterAxis,
BoxMinMax with signed zeros, and MatrixOp<T,4>::Inverse/Multiply on identity,
affine, singular, zero and projective matrices (including W = 0), for float
and double.
"""
import random


def tool_line(precision: int, values: list[float]) -> str:
    """Format one request; the precision stays an integer token; tool_line(0, [1.0] * 24)."""
    return f'bvh_tool_classes {precision} ' + ' '.join(repr(float(value)) for value in values)


def tool_vector(rng: random.Random, case: int) -> list[float]:
    """Four components: random, tied, zero/signed-zero, tiny, or large."""
    pattern = case % 6
    if pattern == 1: return [2.5, 2.5, 2.5, 2.5]
    if pattern == 2: return [0.0, -0.0, 0.0, -0.0]
    if pattern == 3: return [rng.uniform(-1, 1) * 1e-9 for _ in range(4)]
    if pattern == 4: return [rng.uniform(-1, 1) * 1e30 for _ in range(4)]
    if pattern == 5: return [-3.0, 1.0, 1.0, 7.0]
    return [rng.uniform(-10, 10) for _ in range(4)]


def tool_matrix(rng: random.Random, case: int) -> list[float]:
    """Column-major 4x4: identity, affine, singular, zero, projective or W-zero."""
    identity = [1.0 if row == column else 0.0 for column in range(4) for row in range(4)]
    pattern = case % 6
    if pattern == 0: return identity
    if pattern == 1: return identity[:12] + [rng.uniform(-5, 5) for _ in range(3)] + [1.0]
    if pattern == 2: return [rng.uniform(-2, 2) for _ in range(4)] * 4  # rank one
    if pattern == 3: return [0.0] * 16
    if pattern == 4: return identity[:12] + [0.0, 0.0, 0.0, 0.0]  # W row zero -> 1/0
    return [rng.uniform(-2, 2) for _ in range(16)]


def append_bvh_tool_classes_requests(requests: list[str]) -> None:
    """Append 96 tool-class requests; append_bvh_tool_classes_requests(requests)."""
    rng = random.Random(20131221)
    for case in range(96):
        first, second = tool_vector(rng, case), tool_vector(rng, case // 6 + 1)
        requests.append(tool_line(case % 2, first + second + tool_matrix(rng, case // 2)))
