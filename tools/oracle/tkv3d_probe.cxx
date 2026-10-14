// Exact numeric oracle for the TKV3d packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkv3d_numeric.py compares the
// output with crates/tkv3d/examples/numeric_probe.rs bit for bit. Each package adds its own
// tkv3d_probe_<family>.cxx, included below, and its rows in probe_cases under its own
// package comment.
// Every request prints exactly one line and never depends on an earlier request: the
// verifier deals the corpus to parallel shards (tools/probe_shards.py).
#include <map>
#include <memory>
#include <string>
#include <vector>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <Standard_Failure.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_XY.hxx>
#include <gp_XYZ.hxx>
// The TKGeomAlgo reader and bit printers (ga_*), shared so every toolkit prints the same
// token stream; the Rust twin includes crates/tkgeomalgo/examples/ga_probe/common.rs.
#include "tkgeomalgo_probe_common.cxx"
// The TKService primitive array printer (sv_array_state); the Rust twin includes
// crates/tkservice/examples/sv_probe/array.rs.
#include "tkservice_probe_array.cxx"
#include "tkv3d_probe_prs3d.cxx"
#include "tkv3d_probe_v3d.cxx"
#include "tkv3d_probe_selection_helpers.cxx"
#include "tkv3d_probe_intersector.cxx"
#include "tkv3d_probe_base_frustum.cxx"
#include "tkv3d_probe_frustum.cxx"
#include "tkv3d_probe_triangular.cxx"
#include "tkv3d_probe_axis.cxx"
#include "tkv3d_probe_rectangular.cxx"
#include "tkv3d_probe_frustum_set.cxx"
#include "tkv3d_probe_selecting_volume.cxx"
#include "tkv3d_probe_selection_ranking.cxx"
#include "tkv3d_probe_sensitive_entity_set.cxx"
#include "tkv3d_probe_selectable_object_set.cxx"
#include "tkv3d_probe_bvh_thread_pool.cxx"
#include "tkv3d_probe_viewer_selector_state.cxx"
#include "tkv3d_probe_selection_manager.cxx"
#include "tkv3d_probe_prebuild_threshold.cxx"
#include "tkv3d_probe_ais_selection.cxx"
#include "tkv3d_probe_ais_status.cxx"
#include "tkv3d_probe_ais_filters.cxx"
#include "tkv3d_probe_selector_geometry.cxx"
#include "tkv3d_probe_ais_context.cxx"
#include "tkv3d_probe_ais_object.cxx"
#include "tkv3d_probe_ais_load.cxx"
#include "tkv3d_probe_ais_display.cxx"
#include "tkv3d_probe_ais_lifecycle.cxx"
#include "tkv3d_probe_selector_overlap.cxx"
#include "tkv3d_probe_selector_traversal.cxx"
#include "tkv3d_probe_selector_pick.cxx"
#include "tkv3d_probe_selection_image.cxx"
#include "tkv3d_probe_sensitive_display.cxx"
#include "tkv3d_probe_ais_detection.cxx"
#include "tkv3d_probe_ais_selection_actions.cxx"
#include "tkv3d_probe_stdprs_curve.cxx"
#include "tkv3d_probe_plane_presentations.cxx"
#include "tkv3d_probe_deflection_curve.cxx"
#include "tkv3d_probe_ais_line.cxx"
#include "tkv3d_probe_ais_circle.cxx"
#include "tkv3d_probe_ais_point.cxx"
#include "tkv3d_probe_ais_point_cloud.cxx"
#include "tkv3d_probe_ais_axis.cxx"
#include "tkv3d_probe_ais_plane_trihedron.cxx"
#include "tkv3d_probe_ais_plane.cxx"
#include "tkv3d_probe_ais_animation.cxx"
#include "tkv3d_probe_animation_camera.cxx"
#include "tkv3d_probe_context_location.cxx"
#include "tkv3d_probe_animation_object.cxx"
#include "tkv3d_probe_ais_trihedron.cxx"
#include "tkv3d_probe_ais_rubber_band.cxx"
#include "tkv3d_probe_brep_font.cxx"
#include "tkv3d_probe_dsg_prs.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // BRepFont regularization and builders (tkv3d_probe_brep_font.cxx)
 {"v3_brepfont_regularize", v3_brepfont_regularize},
 {"v3_brepfont_curve", v3_brepfont_curve},
 {"v3_brepfont_glyph", v3_brepfont_glyph},
 {"v3_brepfont_text", v3_brepfont_text},
 // DsgPrs presentations (tkv3d_probe_dsg_prs.cxx)
 {"v3_dsgprs", v3_dsgprs},
 {"v3_ais_rubber_band", v3_ais_rubber_band},
 {"v3_ais_trihedron", v3_ais_trihedron},
 {"v3_ais_point_cloud", v3_ais_point_cloud},
 {"v3_point_cloud_alias", v3_point_cloud_alias},
 {"v3_point_cloud_owner", v3_point_cloud_owner},
 {"v3_animation_object", v3_animation_object},
 {"v3_base_animation_object", v3_base_animation_object},
 {"v3_context_location", v3_context_location},
 {"v3_ais_animation_edits", v3_ais_animation_edits},
 {"v3_animation_camera", v3_animation_camera},
 {"v3_ais_animation", v3_ais_animation},
 {"v3_ais_animation_timer", v3_ais_animation_timer},
 {"v3_xyz_axis", v3_xyz_axis},
 {"v3_plane_add", v3_plane_add},
 {"v3_ais_plane", v3_ais_plane},
 {"v3_plane_match", v3_plane_match},
 {"v3_deflection_add", v3_deflection_add},
 {"v3_deflection_match", v3_deflection_match},
 {"v3_plane_trihedron", v3_plane_trihedron},
 {"v3_ais_axis", v3_ais_axis},
 {"v3_ais_point", v3_ais_point},
 {"v3_standard_point", v3_standard_point},
 {"v3_ais_circle", v3_ais_circle},
 {"v3_ais_line", v3_ais_line},
    {"v3_stdprs_add",v3_stdprs_add},
 {"v3_stdprs_match",v3_stdprs_match},
 {"v3_selector_pick",v3_selector_pick},
 {"v3_sensitive_display",v3_sensitive_display},
  {"v3_sensitive_prs",v3_sensitive_prs},
  {"v3_selection_image",v3_selection_image},
 {"v3_selector_pixmap",v3_selector_pixmap},
 {"v3_selector_traversal",v3_selector_traversal},
 {"v3_selector_overlap",v3_selector_overlap},
 {"v3_ais_object",v3_ais_object},
 {"v3_ais_load",v3_ais_load},
 {"v3_ais_lifecycle",v3_ais_lifecycle},
 {"v3_ais_display",v3_ais_display},
 {"v3_ais_context_shutdown",v3_ais_context_shutdown},
 {"v3_ais_context",v3_ais_context},
 {"v3_ais_detection",v3_ais_detection},
 {"v3_ais_selection_actions",v3_ais_selection_actions},
 {"v3_selector_clear",v3_selector_clear},
 {"v3_selector_point",v3_selector_point},
 {"v3_selector_frustum",v3_selector_frustum},
 {"v3_selector_layers",v3_selector_layers},
 {"v3_ais_filters",v3_ais_filters},
 {"v3_ais_selection",v3_ais_selection},
 {"v3_ais_status",v3_ais_status},
 {"v3_viewer_selector_state",v3_viewer_selector_state},
 {"v3_selection_manager",v3_selection_manager},
 {"v3_prebuild_threshold",v3_prebuild_threshold},
 {"v3_bvh_thread_pool",v3_bvh_thread_pool},
 {"v3_selectable_object_set",v3_selectable_object_set},
 // Prs3d tools, arrows, deflection and boxes (tkv3d_probe_prs3d.cxx)
 {"v3_quadric", v3_quadric_case},
 {"v3_arrow", v3_arrow_case},
 {"v3_prs3d", v3_prs3d_case},
 // V3d view camera operations (tkv3d_probe_v3d.cxx)
 {"v3_view", v3_view_case},
 {"v3_frustum_builder", v3_frustum_builder_case},
 {"v3_clip_range", v3_clip_range_case},
 {"v3_ray_sphere", v3_ray_sphere_case},
 {"v3_ray_cylinder", v3_ray_cylinder_case},
 {"v3_ray_circle", v3_ray_circle_case},
 {"v3_base_intersector", v3_base_intersector_case},
 {"v3_base_frustum", v3_base_frustum_case},
 {"v3_frustum_core", v3_frustum_core_case},
 {"v3_triangular", v3_triangular_case},
 {"v3_axis", v3_axis_case},
 {"v3_selecting_volume", v3_selecting_volume_case},
 {"v3_tolerance", v3_tolerance_case},
 {"v3_criterion", v3_criterion_case},
 {"v3_sensitive_entity_set", v3_sensitive_entity_set_case},
 {"v3_frustum_set", v3_frustum_set_case},
 {"v3_rectangle", v3_rectangle_case},
 {"v3_boundary_sphere", v3_boundary_sphere_case},

};
int main() {
 std::string line;
 while(std::getline(std::cin,line)) {
  std::istringstream input(line);std::string op;input>>op;
  if(op.empty())continue;
  auto found=probe_cases.find(op);
  if(found==probe_cases.end()) {std::cerr<<"Unknown probe operation: "<<op<<std::endl;return 2;}
  found->second(input);
 }
}
