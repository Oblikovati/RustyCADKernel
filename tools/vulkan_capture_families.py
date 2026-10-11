"""Explicit fixture families preserve the complete legacy renderer corpus."""

from vulkan_rubber_band_cases import RUBBER_BAND_MODES
from vulkan_line_cases import LINE_MODES
from vulkan_ais_trihedron_cases import AIS_TRIHEDRON_MODES
from vulkan_stdprs_cases import STDPRS_MODES
from vulkan_ais_line_cases import AIS_LINE_MODES
from vulkan_ais_point_cases import AIS_POINT_MODES
from vulkan_text_cases import TEXT_MODES
from vulkan_ais_axis_cases import AIS_AXIS_MODES
from vulkan_plane_trihedron_cases import PLANE_TRIHEDRON_MODES
from vulkan_window_cases import WINDOW_MODES, compare_window_pixels
from vulkan_circle_cases import CIRCLE_MODES
from vulkan_clipping_cases import CLIPPING_MODES
from vulkan_homogeneous_cases import HOMOGENEOUS_MODES
from vulkan_capping_cases import CAPPING_MODES
from vulkan_lighting_cases import LIGHTING_MODES, LIT_LINE_MODES, LIT_ALPHA_MODES
from vulkan_scene_cases import SURFACE_MODES, PERSISTENCE_MODES, AIS_MODES
from vulkan_camera_animation_cases import CAMERA_ANIMATION_MODES
from vulkan_object_animation_cases import OBJECT_ANIMATION_MODES
from vulkan_group_flipping_cases import GROUP_FLIPPING_MODES
from vulkan_ais_text_label_cases import AIS_TEXT_LABEL_MODES

from vulkan_plane_presentation_cases import PLANE_PRESENTATION_MODES

from vulkan_ais_plane_cases import AIS_PLANE_MODES, QUAD_MODES

from vulkan_point_cloud_cases import POINT_CLOUD_MODES

ALL_CAPTURE_MODES = (
    *RUBBER_BAND_MODES,
    *AIS_TRIHEDRON_MODES,
    *POINT_CLOUD_MODES,
    *AIS_PLANE_MODES,
    *QUAD_MODES,
    *PLANE_PRESENTATION_MODES,
    "default",
    "full",
    *AIS_MODES,
    *CAMERA_ANIMATION_MODES,
    *OBJECT_ANIMATION_MODES,
    *GROUP_FLIPPING_MODES,
    *AIS_TEXT_LABEL_MODES,
    *SURFACE_MODES,
    *PERSISTENCE_MODES,
    *LINE_MODES,
    *STDPRS_MODES,
    *AIS_LINE_MODES,
    *AIS_POINT_MODES,
    *TEXT_MODES,
    *AIS_AXIS_MODES,
    *PLANE_TRIHEDRON_MODES,
    *WINDOW_MODES,
    *CIRCLE_MODES,
    *CLIPPING_MODES,
    *HOMOGENEOUS_MODES,
    *CAPPING_MODES,
    *LIGHTING_MODES,
    *LIT_LINE_MODES,
    *LIT_ALPHA_MODES,
)

FAMILY_CAPTURE_MODES = {
    "rubber-band": RUBBER_BAND_MODES,
    "ais-trihedron": AIS_TRIHEDRON_MODES,
    "point-cloud": POINT_CLOUD_MODES,
    "ais-plane": (*AIS_PLANE_MODES, *QUAD_MODES),
    "plane-presentations": PLANE_PRESENTATION_MODES,
    "text-label": AIS_TEXT_LABEL_MODES,
    "group-flipping": GROUP_FLIPPING_MODES,
    "object-animation": OBJECT_ANIMATION_MODES,
    "camera-animation": CAMERA_ANIMATION_MODES,
    "capping": CAPPING_MODES,
    "lighting": LIGHTING_MODES,
    "lit-lines": LIT_LINE_MODES,
    "lit-transparency": LIT_ALPHA_MODES,
}


def capture_modes(case_family: str) -> tuple[str, ...]:
    """Select an explicit fixture family without relaxing gates; capture_modes('text-label')."""
    if case_family == "all":
        return ALL_CAPTURE_MODES
    if case_family in FAMILY_CAPTURE_MODES:
        return FAMILY_CAPTURE_MODES[case_family]
    raise ValueError(
        f"received case family {case_family!r}; expected all, lighting, capping, camera-animation, object-animation or group-flipping or text-label or plane-presentations or ais-plane or point-cloud or ais-trihedron or rubber-band or lit-lines or lit-transparency"
    )
