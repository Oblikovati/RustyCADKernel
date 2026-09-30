"""Curve splitting range clamping, wrapper recursion, segmentation and native null pieces."""

from itertools import product
from native_crash_accounting import partition_crash_requests
from tkshhealing_corpus_shapeconstruct import construct_curves_3d
from tktopalgo_corpus_breplib_records import (
    numbers,
    geom_bspline,
    geom_trimmed,
    geom_offset,
)


def split3d_record(
    curve: list[float],
    natural: bool,
    bounds: tuple[float, float],
    operations: list[list[float]],
) -> str:
    """Serialize one shared splitter history; split3d_record(curve,False,(0.,1.),[[1,1]])."""
    values = [
        *curve,
        natural,
        *bounds,
        len(operations),
        *(value for operation in operations for value in operation),
    ]
    return "sh_split_curve3d " + numbers(values)


def append_split3d_candidates(requests: list[str]) -> None:
    """Exercise all dynamic curve kinds and partial result state; append_split_curve3d_requests(rows)."""
    curves = construct_curves_3d()
    for curve, bounds, segment in product(
        curves,
        [
            (0.0, 1.0),
            (0.1, 0.9),
            (-0.2, 1.2),
            (0.5, 0.5),
            (1.0, 0.0),
            (0.0, 1.0 + 1e-12),
        ],
        [0, 1],
    ):
        operations = [[0, 3, 0.25, 0.5, 0.75], [1, segment], [1, segment], [2, segment]]
        requests.append(split3d_record(curve, False, bounds, operations))
        requests.append(
            split3d_record(curve, False, bounds, [[1, segment], [4, 2], [1, segment]])
        )
    append_split3d_natural(requests, curves)
    append_split3d_knots(requests)


def append_split3d_natural(requests: list[str], curves: list[list[float]]) -> None:
    """Exercise natural ranges and direct shared-range mutation; append_split3d_natural(rows,curves)."""
    for curve, segment in product(curves, [0, 1]):
        requests.append(
            split3d_record(
                curve, True, (0.0, 1.0), [[1, segment], [0, 2, 0.3, 0.7], [2, segment]]
            )
        )
        requests.append(
            split3d_record(
                curve,
                False,
                (0.0, 1.0),
                [[3, -0.5, 1.5], [0, 3, 0.2, 0.5, 0.8], [1, segment]],
            )
        )


def append_split3d_knots(requests: list[str]) -> None:
    """Probe PConfusion snapping and nested trims/offsets; append_split3d_knots(rows)."""
    curves = split3d_knot_curves()
    for curve, shift, segment in product(
        curves, [-2e-9, -1e-10, 0.0, 1e-10, 2e-9], [0, 1]
    ):
        operations = [
            [0, 2, 0.3 + shift, 0.7 + shift],
            [1, segment],
            [3, 0.2, 0.8],
            [1, segment],
        ]
        requests.append(split3d_record(curve, False, (0.0, 1.0), operations))


def split3d_knot_curves() -> list[list[float]]:
    """A spline with internal knots and nested wrappers; split3d_knot_curves()."""
    poles = [
        (0.0, 0.0, 0.0),
        (1.0, 2.0, 1.0),
        (2.0, 1.0, 0.0),
        (3.0, 0.0, 2.0),
        (4.0, 2.0, 0.0),
    ]
    spline = geom_bspline(2, False, poles, [(0.0, 3), (0.3, 1), (0.7, 1), (1.0, 3)])
    return [
        spline,
        geom_trimmed(0.1, 0.9, spline),
        geom_offset(0.2, (0.0, 0.0, 1.0), spline),
        geom_trimmed(0.1, 0.9, geom_offset(0.2, (0.0, 0.0, 1.0), spline)),
    ]


# OCCT3d097a0 Build(false) leaves multiple null pieces, then offset recursion
# dereferences a null basis in Geom_OffsetCurve. Keep these requests accounted
# separately: Rust returns Standard_ConstructionError, an explicit divergence.
NATIVE_CRASH_REVISION = "3d097a0328e71b826377d4814ab05ec3c3d23871"
NATIVE_CRASH_SHA256 = frozenset(
    [
        "394514813391c0262689a1b8ce439d71fa03176a4be379b5ba1fe055dbd27adb",
        "30e59b95209a4cca0360ee193faaa07f21638b48fe01503f7567250c51d6161b",
        "758f7fddb8772c7ff8f688a655d1f44eb9eae9c9eb9347480c947985c127c449",
        "b90189e40250c73815c94389241bc059c247e272ed4dabffaf5acca9071be0cb",
        "2f9cb497d832395a7e2682d40e613fdbab2025da3987b6170fe63af7baa00965",
        "8e4af5e4c083e2a6a2f8f293774f4ded935857e5b50d7ba79a1f122ef62980c1",
        "704fa056c0676ac70f2b9a43f60f1c6e0d07efaef8987a664fb5900092ac950f",
        "7b7a9e1fe1d1e0388b95284f065f254c51d1d721dc5834a19b76dddbe2413f23",
        "4bff7f38b790e33593a9dc4eb14e6ccf53a4fb0f1c58ed4840f321252c7e0da4",
        "c0a3addf1106cbeaccc57ca8124c196bb7f68460e17e76c4f0434e97d53abb8f",
        "597b7542cab8766d88b507bf21e70fa9e20df6f6683cd8ff6b4fc717036029c8",
        "ea12beac7bd7ec11c1ffe608dae9cf8bd74c1a8916685a8727038432c6374111",
        "9effc20c6999174e54ee3c5c0466d3b700cfab289ae7fa10e236381aa3381cd3",
        "b83bd9ed59b3d4818169d0b0bc04f831b179c2dab9da52d66b5154e8aaa36f20",
        "f51806a563c76709c52d50e84ad168d54041044ab601c5e37fe648ba651b31e5",
        "5176b568098b7352b66d7385c3ec6a208180b08d96c7c855e3449bf6c5da521f",
    ]
)


def split3d_partition() -> tuple[list[str], list[str]]:
    """Account every candidate as exact or reproducible native crash; split3d_partition()."""
    candidates: list[str] = []
    append_split3d_candidates(candidates)
    return partition_crash_requests(candidates, NATIVE_CRASH_SHA256)


def append_split_curve3d_requests(requests: list[str]) -> None:
    """Append only defined exact comparisons; crash cases require verify_split_curve3d.py."""
    exact, _ = split3d_partition()
    requests.extend(exact)
