"""Deterministic request corpus for the TKV3d numeric oracle.

Each request is one `operation numbers...` line read by tools/oracle/tkv3d_probe.cxx and
crates/tkv3d/examples/numeric_probe.rs. Each ported package appends its request families
from its own tkv3d_corpus_<family>.py module, one line per family below. The verifier deals
the requests round-robin to parallel shards (tools/probe_shards.py), so a request prints
exactly one line and never relies on an earlier request.
"""
from __future__ import annotations


def append_tkv3d_requests(requests: list[str]) -> None:
    __import__("tkv3d_corpus_animation_object").append_object_requests(requests)
    __import__('tkv3d_corpus_context_location').append_location_requests(requests)
    __import__('tkv3d_corpus_animation_camera').append_camera_requests(requests)
    __import__('tkv3d_corpus_ais_animation').append_animation_requests(requests)
    """Append every TKV3d request family in a fixed order."""
    __import__("tkv3d_corpus_ais_circle").append_ais_circle_requests(requests)
    __import__("tkv3d_corpus_deflection_curve").append_deflection_curve_requests(requests)
    __import__("tkv3d_corpus_plane_trihedron").append_plane_trihedron_requests(requests)
    __import__("tkv3d_corpus_ais_trihedron").append_trihedron_requests(requests)
    __import__("tkv3d_corpus_ais_rubber_band").append_rubber_band_requests(requests)
    __import__("tkv3d_corpus_ais_axis").append_ais_axis_requests(requests)
    __import__("tkv3d_corpus_ais_point").append_ais_point_requests(requests)
    __import__("tkv3d_corpus_ais_point_cloud").append_point_cloud_requests(requests)
    __import__("tkv3d_corpus_plane_presentations").append_plane_presentation_requests(requests)
    __import__("tkv3d_corpus_ais_plane").append_ais_plane_requests(requests)
    __import__("tkv3d_corpus_ais_line").append_ais_line_requests(requests)
    __import__("tkv3d_corpus_stdprs_curve").append_stdprs_curve_requests(requests)
    __import__("tkv3d_corpus_ais").append_ais_requests(requests)
    __import__("tkv3d_corpus_ais_detection").append_ais_detection_requests(requests)
    __import__("tkv3d_corpus_ais_selection_actions").append_ais_selection_actions_requests(requests)
    __import__("tkv3d_corpus_ais_context").append_ais_context_requests(requests)
    __import__("tkv3d_corpus_ais_load").append_ais_load_requests(requests)
    __import__("tkv3d_corpus_ais_display").append_ais_display_requests(requests)
    __import__("tkv3d_corpus_ais_lifecycle").append_ais_lifecycle_requests(requests)
    __import__("tkv3d_corpus_ais_context").append_ais_object_requests(requests)
    __import__("tkv3d_corpus_selector_geometry").append_selector_geometry_requests(requests)
    __import__("tkv3d_corpus_selector_overlap").append_selector_overlap_requests(requests)
    __import__("tkv3d_corpus_selector_traversal").append_selector_traversal_requests(requests)
    __import__("tkv3d_corpus_selection_image").append_selection_image_requests(requests)
    __import__("tkv3d_corpus_sensitive_display").append_sensitive_display_requests(requests)
    __import__("tkv3d_corpus_selector_pick").append_selector_pick_requests(requests)
    # Prs3d tools, arrows, deflection and boxes (tkv3d_corpus_prs3d.py)
    __import__('tkv3d_corpus_prs3d').append_prs3d_requests(requests)
    # V3d view camera operations (tkv3d_corpus_v3d.py)
    __import__('tkv3d_corpus_v3d').append_v3d_requests(requests)
    __import__('tkv3d_corpus_selection_helpers').append_selection_helper_requests(requests)
    __import__('tkv3d_corpus_intersector').append_intersector_requests(requests)
    __import__('tkv3d_corpus_base_frustum').append_base_frustum_requests(requests)
    __import__('tkv3d_corpus_frustum').append_frustum_requests(requests)
    __import__('tkv3d_corpus_triangular').append_triangular_requests(requests)
    __import__('tkv3d_corpus_axis').append_axis_requests(requests)
    __import__('tkv3d_corpus_rectangular').append_rectangular_requests(requests)
    __import__('tkv3d_corpus_frustum_set').append_frustum_set_requests(requests)
    __import__('tkv3d_corpus_selecting_volume').append_selecting_volume_requests(requests)
    __import__('tkv3d_corpus_selection_ranking').append_selection_ranking_requests(requests)
    __import__('tkv3d_corpus_sensitive_entity_set').append_sensitive_entity_set_requests(requests)
    __import__('tkv3d_corpus_selectable_object_set').append_selectable_object_set_requests(requests)
    __import__('tkv3d_corpus_bvh_thread_pool').append_bvh_thread_pool_requests(requests)
    __import__('tkv3d_corpus_brep_font').append_brep_font_requests(requests)
    __import__('tkv3d_corpus_dsg_prs').append_dsg_prs_requests(requests)
    __import__('tkv3d_corpus_std_select_filters').append_std_select_filter_requests(requests)
    __import__('tkv3d_corpus_viewer_selector_state').append_viewer_selector_state_requests(requests)
    __import__('tkv3d_corpus_selection_manager').append_selection_manager_requests(requests)
