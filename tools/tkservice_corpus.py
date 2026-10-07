"""Deterministic request corpus for the TKService numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkservice_probe.cxx
and crates/tkservice/examples/numeric_probe.rs. Each ported package appends its request
families from its own tkservice_corpus_<family>.py module, one line per family below.
The verifier deals the requests round-robin to parallel shards (tools/probe_shards.py), so
a request prints exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkservice_requests(requests: list[str]) -> None:
    """Append every TKService request family in a fixed order."""
    # Aspect grids (tkservice_corpus_grid.py)
    __import__('tkservice_corpus_grid').append_grid_requests(requests)
    # WNT_HIDSpaceMouse (tkservice_corpus_hid.py)
    __import__('tkservice_corpus_hid').append_hid_requests(requests)
    # Image_PixMap pixel conversions (tkservice_corpus_pixel.py)
    __import__('tkservice_corpus_pixel').append_pixel_requests(requests)
    # Graphic3d_PBRMaterial and Graphic3d_BSDF (tkservice_corpus_pbr.py)
    __import__('tkservice_corpus_pbr').append_pbr_requests(requests)
    # Graphic3d_MaterialAspect (tkservice_corpus_material.py)
    __import__('tkservice_corpus_material').append_material_requests(requests)
    # Graphic3d_ShaderObject declarations (tkservice_corpus_shader.py)
    __import__('tkservice_corpus_shader').append_shader_requests(requests)
    # Graphic3d textures (tkservice_corpus_texture.py)
    __import__('tkservice_corpus_texture').append_texture_requests(requests)
    # Graphic3d_MarkerImage (tkservice_corpus_marker.py)
    __import__('tkservice_corpus_marker').append_marker_requests(requests)
    # Graphic3d_Aspects (tkservice_corpus_aspects.py)
    __import__('tkservice_corpus_aspects').append_aspects_requests(requests)
    # Graphic3d_ArrayOfPrimitives (tkservice_corpus_array.py)
    __import__('tkservice_corpus_array').append_array_requests(requests)
    # Graphic3d_Camera (tkservice_corpus_camera.py)
    __import__('tkservice_corpus_camera').append_camera_requests(requests)
    # Graphic3d_ClipPlane (tkservice_corpus_clip.py)
    __import__('tkservice_corpus_clip').append_clip_requests(requests)
    # Graphic3d_CLight and Graphic3d_LightSet (tkservice_corpus_light.py)
    __import__('tkservice_corpus_light').append_light_requests(requests)
    # Graphic3d_TransformPers and Graphic3d_TransformUtils (tkservice_corpus_tpers.py)
    __import__('tkservice_corpus_tpers').append_tpers_requests(requests)
    # Graphic3d_RenderingParams, Graphic3d_ZLayerSettings, Graphic3d_PresentationAttributes (tkservice_corpus_params.py)
    __import__('tkservice_corpus_params').append_params_requests(requests)
    # Font_FontMgr, Font_FTFont, Font_TextFormatter (tkservice_corpus_font.py)
    __import__('tkservice_corpus_font').append_font_requests(requests)
    # Graphic3d_CullingTool (tkservice_corpus_culling.py)
    __import__('tkservice_corpus_culling').append_culling_requests(requests)
    # Graphic3d_FrameStats, Graphic3d_FrameStatsData (tkservice_corpus_stats.py)
    __import__('tkservice_corpus_stats').append_stats_requests(requests)
