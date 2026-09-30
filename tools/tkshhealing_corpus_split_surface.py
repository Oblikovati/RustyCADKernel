"""Surface splitter histories retain wrapper geometry and partial state."""

from itertools import product
from tktopalgo_corpus_breplib_records import numbers
from tkshhealing_corpus_analysis_surface import analysis_surfaces


def split_surface_record(
    surface: list[float],
    natural: bool,
    bounds: tuple[float, float, float, float],
    area: float,
    operations: list[list[float]],
) -> str:
    """Serialize geometry and shared-state operations; split_surface_record(s,False,(0,1,0,1),0,[[1,1]])."""
    return "sh_split_surface " + numbers(
        [
            *surface,
            natural,
            *bounds,
            area,
            len(operations),
            *(value for op in operations for value in op),
        ]
    )


SPLIT_SURFACE_RANGES = [
    (0.0, 1.0, 0.0, 1.0),
    (0.1, 0.9, 0.2, 0.8),
    (-0.5, 1.5, -0.2, 1.2),
    (0.0, 0.0, 0.0, 0.0),
    (1.0, 0.0, 1.0, 0.0),
    (7.0, 8.0, -3.0, -2.0),
]


def append_split_surface_candidates(requests: list[str]) -> None:
    """Retain all candidates, including native crashes; append_split_surface_candidates(rows)."""
    for surface, bounds, segment in product(
        analysis_surfaces(), SPLIT_SURFACE_RANGES, [0, 1]
    ):
        operations = [
            [0, 1, 3, 0.25, 0.5, 0.75],
            [0, 0, 2, 0.3, 0.7],
            [1, segment],
            [1, segment],
            [2, segment],
        ]
        single = [[1, segment], [4, 2], [1, segment], [5, segment]]
        requests.append(split_surface_record(surface, False, bounds, 0.0, operations))
        requests.append(split_surface_record(surface, False, bounds, 0.0, single))
    append_surface_histories(requests)


def append_surface_histories(requests: list[str]) -> None:
    """Preserve natural-range, area and empty-value histories; append_surface_histories(rows)."""
    bounds = (0.0, 1.0, 0.0, 1.0)
    for surface, segment in product(analysis_surfaces(), [0, 1]):
        natural = [[1, segment], [0, 1, 1, 0.5], [0, 0, 1, 0.5], [1, segment]]
        area = [[1, segment], [6], [1, segment]]
        empty = [[3, 1, 0], [1, segment]]
        requests.append(split_surface_record(surface, True, bounds, 0.0, natural))
        requests.append(split_surface_record(surface, False, bounds, 2.0, area))
        requests.append(split_surface_record(surface, False, bounds, 0.0, empty))


def split_surface_knots() -> list[float]:
    """Construct nonsquare spline with internal U knots; split_surface_knots()."""
    from tkshhealing_corpus_analysis_surface import SURFACE_ANALYSIS_FRAME

    poles = [
        coordinate
        for u, v in product(range(5), range(3))
        for coordinate in [float(u), float(v), float(u * v) / 10.0]
    ]
    u_knots = [4, 0.0, 3, 0.3, 1, 0.7, 1, 1.0, 3]
    v_knots = [2, 0.0, 3, 1.0, 3]
    return [7, *SURFACE_ANALYSIS_FRAME, 2, 2, 5, 3, *poles, *u_knots, *v_knots]


def append_split_surface_boundaries(requests: list[str]) -> None:
    """Exercise knot thresholds, reordered values and shifted periodic domains."""
    append_surface_knot_thresholds(requests)
    append_surface_mutations(requests)
    append_surface_periodic_ranges(requests)


def append_surface_knot_thresholds(requests: list[str]) -> None:
    """Probe PConfusion around interior knots with offset recursion."""
    spline = split_surface_knots()
    for surface, shift, segment in product(
        [[0, *spline], [1, 0.1, *spline]], [-2e-9, -1e-10, 0.0, 1e-10, 2e-9], [0, 1]
    ):
        operations = [
            [0, 1, 2, 0.3 + shift, 0.7 + shift],
            [0, 0, 2, 0.25, 0.75],
            [1, segment],
            [3, 1, 3, 0.1, 0.5, 0.9],
            [1, segment],
            [6],
            [1, segment],
        ]
        requests.append(
            split_surface_record(surface, False, (0.0, 1.0, 0.0, 1.0), 0.0, operations)
        )


def append_surface_mutations(requests: list[str]) -> None:
    """Preserve partial output for malformed public sequences."""
    sequences = [[0.2], [0.9, 0.3, 0.1], [-0.2, 0.5, 1.2], [0.5, 0.5]]
    for surface, values in product(analysis_surfaces(), sequences):
        operations = [[3, 1, len(values), *values], [1, 1]]
        requests.append(
            split_surface_record(surface, False, (0.0, 1.0, 0.0, 1.0), 0.0, operations)
        )


def append_surface_periodic_ranges(requests: list[str]) -> None:
    """Compare range shifts shorter and longer than a natural period."""
    ranges = [
        (7.0, 13.0, -7.0, -1.0),
        (-0.5, 7.0, -0.5, 7.0),
        (-15.0, -14.0, 13.0, 14.0),
    ]
    for surface, bounds in product(analysis_surfaces()[1:7], ranges):
        operations = [[1, 1], [0, 1, 1, 7.5], [0, 0, 1, 13.5], [1, 1]]
        requests.append(split_surface_record(surface, False, bounds, 0.0, operations))


# OCCT 3d097a0 sweeps dereference the null pieces produced by SplitCurve3d
# Build(false). These remain explicit native-crash/Rust-error divergences.
SPLIT_SURFACE_CRASH_SHA256 = frozenset(
    [
        "0263c51fca7abf5f5a70cb459a963d5779a4b16f74c6bfb36ca8e764d5f98c2c",
        "a40889612af609fa6c216c4f5b1bca59377fd3cfc1e757ca4e8df3e3e6aee7d9",
        "f6aa39996db88af4768988c8a892406a4fcfcdb76023e063c097442ced16365d",
        "10b008e7da772493c83d933c54590a0cbf7159b1aad9f28e86b4dfe035867600",
        "01200570e51def413ea1fda01ce327e05fdcfe9094574d0dcede897502d5d0d3",
        "81d82c39910f7e243d916578ec4a0eb050977eff70adb227f0f5493f805b14b8",
        "09dbcc1ea269d6447420da4dfaea0a2de7213e57a4cf3b680c871383e5b59991",
        "71263bb3b9f5696065283146326b30543b165dec3c4ce5efbe57aa95b605e056",
        "3f8c6276fa999273a92a2a2361959cf32594b5a877533671ea238503d0e4a99b",
        "05bcc2d364faadf87dfdd3d7b30155e503f5c947605f6203ae8f4e9c44211b21",
        "cd91b36e67e785fde3cbe64180e0b1aaa210994b4d919ab841cfab1b014fd1c2",
        "9415e2314b8f69f31dd361b9518685f58ae856d59ce8624436ba8a83f1eff3f6",
        "ddb14390026e5ce16cf3b168ebd6c0c4368429ad6d584cf7841c719cde414867",
        "eac7f531eb4fc88e71a308640720ca2c2176d6f4bd40328895e8b5028b6917ec",
        "539a199027a90207483df41788bf663dc0cf0dc91b073cb743e0036277e0865b",
        "81e17767f156b52d303277ce5e38b41bd34aa37afc3b598fc690239dc14b5e08",
    ]
)


def split_surface_partition() -> tuple[list[str], list[str]]:
    """Require every declared crash request to remain in the candidate inventory."""
    from native_crash_accounting import partition_crash_requests

    candidates: list[str] = []
    append_split_surface_candidates(candidates)
    append_split_surface_boundaries(candidates)
    return partition_crash_requests(candidates, SPLIT_SURFACE_CRASH_SHA256)


def append_split_surface_requests(requests: list[str]) -> None:
    """Append exact comparisons; the other requests require verify_split_surface.py."""
    exact, _ = split_surface_partition()
    requests.extend(exact)
