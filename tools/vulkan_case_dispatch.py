"""Select each scene family's executable without coupling capture comparisons to fixture modules."""

from pathlib import Path

CAPTURE_EXECUTABLES = {
    "lit-": "lighting_capture",
    "ais-": "ais_capture",
    "datum-": "ais_line_capture",
    "text-": "text_capture",
    "plane-": "plane_trihedron_capture",
    "axis-": "ais_axis_capture",
    "window-": "window_capture",
    "point-": "ais_point_capture",
    "circle-": "ais_circle_capture",
}


def capture_executable(default: Path, mode: str) -> Path:
    """Resolve the compiled scene probe; capture_executable(binary, 'circle-full')."""
    for prefix, name in CAPTURE_EXECUTABLES.items():
        if mode.startswith(prefix):
            return default.with_name(name)
    return default
