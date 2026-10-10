"""Native image bytes from every filler, including deferred flush and partial failures."""
from __future__ import annotations
import itertools


def filler_request(kind: int, fmt: int, top: int, padding: int, mode: int, depths: tuple[float, ...]) -> str:
    """Build a palette/depth trace over duplicate entities and two runtime types."""
    tags = (0, 1, 3, 9, 0, -1, 6, 2)
    ranks = (1, 1, 1, 1, 2, 1, 1, 1)
    normals = ((0, 0, 0), (1, 2, 3), (-1, 0, 0), (0, 0, 4), (1, -2, 3), (0, 0, 0), (1e20, 0, 0), (1e-30, 0, 0))
    pixels = [' '.join(map(str, (tag, rank, depth, *normal))) for tag, rank, depth, normal in zip(tags, ranks, depths, normals)]
    return f"v3_selection_image {kind} {fmt} 4 2 {top} {padding} {mode} {len(pixels)} " + ' '.join(pixels)


def append_selection_image_requests(requests: list[str]) -> None:
    """Exercise all formats and palettes, then depth rounding and public ToPixMap."""
    for kind, fmt, top, mode in itertools.product(range(10), (1, 3, 7, 8, 9, 14, 18, 19), (0, 1), (-1, 0, 1, 9, 30, 256)):
        requests.append(filler_request(kind, fmt, top, 8, mode, (0., .25, .5, 1., 2., -2., .75, .125)))
    append_image_depth_requests(requests)
    for kind, fmt, size, top, mode, rank in itertools.product(range(10), (7, 9, 14), ((1, 1), (4, 3), (7, 5)), (0, 1), (0, 9), (0, 1, 2)):
        width, height = size
        requests.append(f"v3_selector_pixmap {kind} {fmt} {width} {height} {top} 8 {mode} {rank} -1")
    for kind, failing in itertools.product(range(10), (0, 3, 4, 5)):
        requests.append(f"v3_selector_pixmap {kind} 14 7 5 0 0 9 1 {failing}")
    for kind in range(10):
        requests.extend((f"v3_selection_image {kind} 14 0 0 0 0 9 0", f"v3_selection_image {kind} 14 3 2 0 0 9 0", f"v3_selector_pixmap {kind} 14 0 0 0 0 9 1 -1"))


def append_image_depth_requests(requests: list[str]) -> None:
    """Keep double extrema but independently rounded float bounds and pixel storage."""
    maximum = 3.4028234663852886e38
    profiles = ((.25,) * 8, (.50000001, .50000002) * 4, (0., 2**-24) * 4,
                (-maximum, maximum, 0., 1., .5, -1., 2., -2.),
                tuple(i / 10 for i in range(8)), (.5,) * 6)
    for kind, fmt, top, padding, depths in itertools.product((0, 1, 2), (9, 14), (0, 1), (0, 8), profiles):
        requests.append(filler_request(kind, fmt, top, padding, 9, depths))
