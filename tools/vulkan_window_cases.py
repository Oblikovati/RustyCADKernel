"""Compare actual X11 presentation pixels separately from offscreen color/depth captures."""

from pathlib import Path
from vulkan_capture_artifacts import CaptureStore, rgba_png, difference_image

WINDOW_MODES = tuple(
    "window-" + name
    for name in ("base", "resize", "rebind", "vsync", "remap", "erased")
)


def compare_window_pixels(
    width: int, height: int, native: Path, ported: Path, store: CaptureStore
) -> dict[str, object]:
    """Require exact displayed colors; compare_window_pixels(64, 64, gl, vk, store)."""
    left, right = (
        store.read(Path(f"{prefix}.window.rgba")) for prefix in (native, ported)
    )
    for prefix, pixels in ((native, left), (ported, right)):
        store.write(Path(f"{prefix}.window.png"), rgba_png(width, height, pixels))
    diff, bounds = difference_image(width, left, right)
    store.write(Path(f"{ported}.window.diff.png"), rgba_png(width, height, diff))
    return {
        "status": "passed" if left == right else "failed",
        "different_pixels": sum(
            left[i : i + 4] != right[i : i + 4] for i in range(0, len(left), 4)
        ),
        "bounds": bounds,
    }
