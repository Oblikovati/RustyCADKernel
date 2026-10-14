//! Exact TKV3d numeric oracle counterpart of tools/oracle/tkv3d_probe.cxx:
//! reads `operation numbers...` lines and prints the same token stream.
use std::io::{self, BufRead};
#[path = "../tests/support/abstract_intersector.rs"]
mod abstract_intersector;
mod v3_probe;
use v3_probe::{prs3d, v3d};

/// One probe operation over the numeric fields of a request.
type ProbeCase = fn(&[f64]);

/// The dispatch table of tools/oracle/tkv3d_probe.cxx (`probe_cases`); each package adds its
/// operations under its own comment, as in the native table.
const PROBE_CASES: &[(&str, ProbeCase)] = &[
    // BRepFont regularization and builders (v3_probe/brep_font.rs)
    (
        "v3_brepfont_regularize",
        v3_probe::brep_font::regularize_case,
    ),
    ("v3_brepfont_curve", v3_probe::brep_font::curve_case),
    ("v3_brepfont_glyph", v3_probe::brep_font::glyph_case),
    ("v3_brepfont_text", v3_probe::brep_font::text_case),
    // DsgPrs presentations (v3_probe/dsg_prs.rs)
    ("v3_dsgprs", v3_probe::dsg_prs::dsg_prs_case),
    // AIS_MediaPlayer (v3_probe/ais_media_player.rs)
    (
        "v3_ais_media_player",
        v3_probe::ais_media_player::media_player_case,
    ),
    // AIS_Triangulation (v3_probe/ais_triangulation.rs)
    (
        "v3_ais_triangulation",
        v3_probe::ais_triangulation::triangulation_case,
    ),
    // AIS connected objects (v3_probe/ais_connected.rs)
    ("v3_ais_connected", v3_probe::ais_connected::connected_case),
    // AIS_InteractiveContext attribute members (v3_probe/ais_context_attributes.rs)
    (
        "v3_ais_ctx_attr",
        v3_probe::ais_context_attributes::context_attributes_case,
    ),
    // AIS_Shape (v3_probe/ais_shape.rs)
    ("v3_ais_shape", v3_probe::ais_shape::ais_shape_case),
    // AIS filters and graphic tool (v3_probe/ais_attribute_filters.rs)
    (
        "v3_ais_attr_filter",
        v3_probe::ais_attribute_filters::attribute_filter_case,
    ),
    // StdPrs face and surface presentations (v3_probe/std_prs_faces.rs)
    ("v3_stdprs_face", v3_probe::std_prs_faces::std_prs_face_case),
    // StdSelect owner filters (v3_probe/std_select_filters.rs)
    (
        "v3_stdsel_filter",
        v3_probe::std_select_filters::filter_case,
    ),
    (
        "v3_ais_rubber_band",
        v3_probe::ais_rubber_band::rubber_band_case,
    ),
    ("v3_ais_trihedron", v3_probe::ais_trihedron::trihedron_case),
    ("v3_ais_plane", v3_probe::ais_plane::plane_case),
    ("v3_ais_point_cloud", v3_probe::ais_point_cloud::cloud_case),
    (
        "v3_point_cloud_owner",
        v3_probe::ais_point_cloud::owner_case,
    ),
    (
        "v3_point_cloud_alias",
        v3_probe::ais_point_cloud::alias_case,
    ),
    (
        "v3_plane_add",
        v3_probe::plane_presentations::plane_add_case,
    ),
    (
        "v3_plane_match",
        v3_probe::plane_presentations::plane_match_case,
    ),
    (
        "v3_animation_object",
        v3_probe::animation_object::object_case,
    ),
    (
        "v3_base_animation_object",
        v3_probe::animation_object::base_case,
    ),
    (
        "v3_animation_axis_rotation",
        v3_probe::animation_object::axis_rotation_case,
    ),
    (
        "v3_context_location",
        v3_probe::context_location::location_case,
    ),
    (
        "v3_animation_camera",
        v3_probe::animation_camera::camera_case,
    ),
    (
        "v3_ais_animation_edits",
        v3_probe::ais_animation::edits_case,
    ),
    ("v3_ais_animation", v3_probe::ais_animation::animation_case),
    (
        "v3_ais_animation_timer",
        v3_probe::ais_animation::timer_case,
    ),
    ("v3_xyz_axis", v3_probe::ais_axis::xyz_axis_case),
    (
        "v3_plane_trihedron",
        v3_probe::ais_plane_trihedron::plane_trihedron_case,
    ),
    ("v3_ais_axis", v3_probe::ais_axis::ais_axis_case),
    ("v3_ais_point", v3_probe::ais_point::ais_point_case),
    (
        "v3_standard_point",
        v3_probe::ais_point::standard_point_case,
    ),
    ("v3_ais_circle", v3_probe::ais_circle::circle_case),
    ("v3_ais_line", v3_probe::ais_line::ais_line_case),
    ("v3_deflection_add", v3_probe::deflection_curve::add_case),
    (
        "v3_deflection_match",
        v3_probe::deflection_curve::match_case,
    ),
    ("v3_stdprs_add", v3_probe::stdprs_curve::curve_add_case),
    ("v3_stdprs_match", v3_probe::stdprs_curve::curve_match_case),
    (
        "v3_ais_selection_actions",
        v3_probe::ais_selection_actions::ais_selection_actions_case,
    ),
    (
        "v3_ais_detection",
        v3_probe::ais_detection::ais_detection_case,
    ),
    (
        "v3_sensitive_display",
        v3_probe::sensitive_display::sensitive_display_case,
    ),
    (
        "v3_sensitive_prs",
        v3_probe::sensitive_display::sensitive_prs_case,
    ),
    ("v3_selection_image", v3_probe::selection_image::filler_case),
    ("v3_selector_pixmap", v3_probe::selection_image::pixmap_case),
    (
        "v3_selector_pick",
        v3_probe::selector_pick::selector_pick_case,
    ),
    (
        "v3_selector_traversal",
        v3_probe::selector_traversal::selector_traversal_case,
    ),
    (
        "v3_selector_overlap",
        v3_probe::selector_overlap::selector_overlap_case,
    ),
    (
        "v3_ais_context_shutdown",
        v3_probe::ais_object::context_shutdown_case,
    ),
    (
        "v3_ais_lifecycle",
        v3_probe::ais_lifecycle::ais_lifecycle_case,
    ),
    ("v3_ais_display", v3_probe::ais_display::ais_display_case),
    ("v3_ais_load", v3_probe::ais_load::ais_load_case),
    ("v3_ais_object", v3_probe::ais_object::ais_object_case),
    ("v3_ais_context", v3_probe::ais_context::ais_context_case),
    (
        "v3_selector_clear",
        v3_probe::selector_geometry::selector_clear_case,
    ),
    (
        "v3_selector_point",
        v3_probe::selector_geometry::selector_point_case,
    ),
    (
        "v3_selector_frustum",
        v3_probe::selector_geometry::selector_frustum_case,
    ),
    (
        "v3_selector_layers",
        v3_probe::selector_geometry::selector_layers_case,
    ),
    ("v3_ais_filters", v3_probe::ais_filters::ais_filters_case),
    (
        "v3_ais_selection",
        v3_probe::ais_selection::ais_selection_case,
    ),
    ("v3_ais_status", v3_probe::ais_status::ais_status_case),
    (
        "v3_prebuild_threshold",
        v3_probe::prebuild_threshold::prebuild_threshold_case,
    ),
    (
        "v3_selection_manager",
        v3_probe::selection_manager::selection_manager_case,
    ),
    (
        "v3_viewer_selector_state",
        v3_probe::viewer_selector_state::viewer_selector_state_case,
    ),
    (
        "v3_bvh_thread_pool",
        v3_probe::bvh_thread_pool::bvh_thread_pool_case,
    ),
    (
        "v3_selectable_object_set",
        v3_probe::selectable_object_set::selectable_object_set_case,
    ),
    ("v3_axis", v3_probe::axis::axis_case),
    (
        "v3_selecting_volume",
        v3_probe::selecting_volume::selecting_volume_case,
    ),
    ("v3_tolerance", v3_probe::selection_ranking::tolerance_case),
    ("v3_criterion", v3_probe::selection_ranking::criterion_case),
    (
        "v3_sensitive_entity_set",
        v3_probe::sensitive_entity_set::sensitive_entity_set_case,
    ),
    ("v3_frustum_set", v3_probe::frustum_set::frustum_set_case),
    ("v3_rectangle", v3_probe::rectangular::rectangular_case),
    ("v3_triangular", v3_probe::triangular::triangular_case),
    ("v3_frustum_core", v3_probe::frustum_core::frustum_core_case),
    ("v3_base_frustum", v3_probe::base_frustum::base_frustum_case),
    (
        "v3_boundary_sphere",
        v3_probe::base_frustum::boundary_sphere_case,
    ),
    ("v3_ray_sphere", v3_probe::intersector::ray_sphere_case),
    ("v3_ray_cylinder", v3_probe::intersector::ray_cylinder_case),
    ("v3_ray_circle", v3_probe::intersector::ray_circle_case),
    (
        "v3_base_intersector",
        v3_probe::intersector::base_intersector_case,
    ),
    // Prs3d tools, arrows, deflection and boxes (v3_probe/prs3d.rs)
    ("v3_quadric", prs3d::quadric_case),
    ("v3_arrow", prs3d::arrow_case),
    ("v3_prs3d", prs3d::prs3d_case),
    // V3d view camera operations (v3_probe/v3d.rs)
    ("v3_view", v3d::view_case),
    (
        "v3_frustum_builder",
        v3_probe::selection_helpers::frustum_builder_case,
    ),
    (
        "v3_clip_range",
        v3_probe::selection_helpers::clip_range_case,
    ),
];

fn main() {
    for line in io::stdin().lock().lines() {
        let line: String = line.expect("probe input line");
        let mut fields = line.split_whitespace();
        let Some(op) = fields.next() else { continue };
        let values: Vec<f64> = fields
            .map(|field| field.parse().expect("numeric probe field"))
            .collect();
        let Some((_, case)) = PROBE_CASES.iter().find(|(name, _)| *name == op) else {
            eprintln!("Unknown probe operation: {op}");
            std::process::exit(2);
        };
        case(&values);
    }
}
