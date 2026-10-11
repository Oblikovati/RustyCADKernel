"""Fingerprint renderer source and paired native fixtures for integration receipts."""

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RECEIPT_ORACLE_PATHS = (
    "tools/oracle/martensite_vulkan_ais_rubber_band.cxx",
    "tools/oracle/martensite_vulkan_ais_trihedron.cxx",
    "tools/verify_vulkan.py",
    "tools/vulkan_capture_compare.py",
    "tools/oracle/martensite_vulkan_reference.cxx",
    "tools/oracle/martensite_vulkan_capping.cxx",
    "tools/oracle/martensite_vulkan_lighting.cxx",
    "tools/vulkan_lighting_cases.py",
    "tools/vulkan_capping_cases.py",
    "tools/oracle/martensite_vulkan_ais.cxx",
    "tools/oracle/martensite_vulkan_surfaces.cxx",
    "tools/oracle/martensite_vulkan_bound_array.hxx",
    "tools/oracle/martensite_vulkan_lines.cxx",
    "tools/oracle/martensite_stdprs_geometry.hxx",
    "tools/oracle/martensite_vulkan_stdprs.cxx",
    "tools/oracle/martensite_vulkan_ais_line.cxx",
    "tools/oracle/martensite_vulkan_ais_point.cxx",
    "tools/oracle/martensite_vulkan_text.cxx",
    "tools/oracle/martensite_vulkan_ais_axis.cxx",
    "tools/oracle/martensite_vulkan_ais_circle.cxx",
    "tools/oracle/martensite_vulkan_camera_animation.cxx",
    "tools/oracle/martensite_vulkan_object_animation.cxx",
    "tools/oracle/martensite_vulkan_group_flipping.cxx",
    "tools/oracle/martensite_vulkan_ais_text_label.cxx",
    "tools/oracle/martensite_vulkan_plane_presentations.cxx",
    "tools/oracle/martensite_vulkan_ais_plane.cxx",
    "tools/oracle/martensite_vulkan_point_cloud.cxx",
    "tools/oracle/martensite_vulkan_quadrangles.cxx",
    "tools/oracle/martensite_vulkan_plane_trihedron.cxx",
    "tools/oracle/martensite_vulkan_window.cxx",
    "tools/oracle/martensite_vulkan_persistence.cxx",
    "tools/oracle/martensite_vulkan_clipping.cxx",
    "tools/oracle/martensite_vulkan_homogeneous.cxx",
    "tools/oracle/martensite_vulkan_raster_control.cxx",
)


def source_hashes() -> dict[str, str]:
    """Fingerprint the actual renderer and transitive source inputs; e.g. source_hashes()."""
    paths = [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    for crate in sorted(p.name for p in (ROOT / "crates").iterdir() if p.is_dir()):
        paths.extend(p for p in (ROOT / "crates" / crate).rglob("*") if p.is_file())
    paths.extend(ROOT / p for p in RECEIPT_ORACLE_PATHS)
    paths.extend(ROOT.glob("tools/vulkan_*.py"))
    paths.extend(ROOT.glob("tools/tests/test_vulkan_*.py"))
    return {
        str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in sorted(paths)
    }
