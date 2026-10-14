// Exact numeric oracle for the TKBool packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkbool_numeric.py compares the
// output with crates/tkbool/examples/numeric_probe.rs bit for bit. Each package adds its
// own tkbool_probe_<package>.cxx, included below, and its rows in probe_cases under its
// own package comment.
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
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
// The TKGeomAlgo reader and bit printers (ga_*), shared so every toolkit prints the same
// token stream; the Rust twin includes crates/tkgeomalgo/examples/ga_probe/common.rs.
#include "tkgeomalgo_probe_common.cxx"
// The TKTopAlgo shape digest (tp_shape) and geometry readers of the BRepLib probes; the
// Rust twin includes crates/tktopalgo/examples/tp_probe/brep_lib_common.rs.
#include "tktopalgo_probe_breplib_common.cxx"
// The TKPrim shape digest with face frames (pr_shape); the Rust twin includes
// crates/tkprim/examples/pr_probe/shape.rs.
#include "tkprim_probe_shape.cxx"
// The TKTopAlgo face and shape program readers (tp_surface, tp_shapes); the Rust twin
// includes crates/tktopalgo/examples/tp_probe/face.rs, shapes.rs and shapes_geometry.rs.
#include "tktopalgo_probe_face.cxx"
#include "tktopalgo_probe_shapes.cxx"
#include "tkbool_probe_brepfill_generator.cxx"
#include "tkbool_probe_brepfill_package.cxx"
#include "tkbool_probe_brepfill_compatible.cxx"
#include "tkbool_probe_brepfill_section_law.cxx"
#include "tkbool_probe_brepfill_location_law.cxx"
#include "tkbool_probe_brepfill_section.cxx"
#include "tkbool_probe_brepfill_sweep.cxx"
#include "tkbool_probe_brepfill_pipe_shell.cxx"
#include "tkbool_probe_brepfill_pipe.cxx"
#include "tkbool_probe_brepfill_draft.cxx"
#include "tkbool_probe_brepfill_filling.cxx"
#include "tkbool_probe_brepfill_offset_wire.cxx"
#include "tkbool_probe_brepfill_multi_line.cxx"
#include "tkbool_probe_brepfill_advanced_evolved.cxx"
#include "tkbool_probe_evolved_digest.cxx"
#include "tkbool_probe_brepfill_evolved.cxx"
#include "tkbool_probe_brepalgo.cxx"
#include "tkbool_probe_brepproj.cxx"
#include "tkbool_probe_topopebreptool_curve_tool.cxx"
#include "tkbool_probe_topopebrepds.cxx"
#include "tkbool_probe_curve_explorer.cxx"
#include "tkbool_probe_point_explorer.cxx"
#include "tkbool_probe_interference_tool.cxx"
#include "tkbool_probe_topopebreptool_shapes.cxx"
#include "tkbool_probe_topopebrepds_build.cxx"
#include "tkbool_probe_topopebreptool_classifier.cxx"
#include "tkbool_probe_topopebrepbuild.cxx"
#include "tkbool_probe_topopebreptool_tool.cxx"
#include "tkbool_probe_topopebreptool_regu.cxx"
#include "tkbool_probe_topopebrepds_ops.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // BRepFill (tkbool_probe_brepfill_generator.cxx, tkbool_probe_brepfill_package.cxx,
 // tkbool_probe_brepfill_compatible.cxx, tkbool_probe_brepfill_section_law.cxx,
 // tkbool_probe_brepfill_location_law.cxx, tkbool_probe_brepfill_section.cxx,
 // tkbool_probe_brepfill_sweep.cxx, tkbool_probe_brepfill_pipe_shell.cxx,
 // tkbool_probe_brepfill_pipe.cxx, tkbool_probe_brepfill_draft.cxx,
 // tkbool_probe_brepfill_filling.cxx, tkbool_probe_brepfill_offset_wire.cxx,
 // tkbool_probe_brepfill_multi_line.cxx, tkbool_probe_brepfill_advanced_evolved.cxx)
 {"bf_generator", bl_bf_generator_case},
 {"bf_face", bl_bf_face_case},
 {"bf_shell", bl_bf_shell_case},
 {"bf_axe", bl_bf_axe_case},
 {"bf_search_origin", bl_bf_search_origin_case},
 {"bf_compute_acr", bl_bf_compute_acr_case},
 {"bf_insert_acr", bl_bf_insert_acr_case},
 {"bf_compatible", bl_bf_compatible_case},
 {"bf_shape_law", bl_bf_shape_law_case},
 {"bf_nsections", bl_bf_nsections_case},
 {"bf_location_law", bl_bf_location_law_case},
 {"bf_section", bl_bf_section_case},
 {"bf_section_placement", bl_bf_section_placement_case},
 {"bf_sweep", bl_bf_sweep_case},
 {"bf_pipe_shell", bl_bf_pipe_shell_case},
 {"bf_pipe", bl_bf_pipe_case},
 {"bf_draft", bl_bf_draft_case},
 {"bf_filling", bl_bf_filling_case},
 {"bf_offset_wire", bl_bf_offset_wire_case},
 {"bf_multi_line", bl_bf_multi_line_case},
 {"bf_trim_surface", bl_bf_trim_surface_case},
 {"bf_advanced_evolved", bl_bf_advanced_evolved_case},
 {"bf_evolved", bl_bf_evolved_case},
 {"bf_evolved_empty", bl_bf_evolved_empty_case},
 // BRepAlgo (tkbool_probe_brepalgo.cxx)
 {"ba_concat", bl_ba_concat_case},
 {"ba_concat_c0", bl_ba_concat_c0_case},
 {"ba_convert", bl_ba_convert_case},
 {"ba_valid", bl_ba_valid_case},
 {"ba_face_restrictor", bl_ba_face_restrictor_case},
 {"ba_cut_edge", bl_ba_cut_edge_case},
 {"ba_loop", bl_ba_loop_case},
 // BRepAlgo_NormalProjection and BRepProj (tkbool_probe_brepproj.cxx)
 {"ba_normal_projection", bl_ba_normal_projection_case},
 {"bp_projection", bl_bp_projection_case},
 // TopOpeBRepTool_CurveTool (tkbool_probe_topopebreptool_curve_tool.cxx)
 {"tt_make_curves", bl_tt_make_curves_case},
 {"tt_pcurve_on_face", bl_tt_pcurve_on_face_case},
 // TopOpeBRepDS (tkbool_probe_topopebrepds.cxx)
 {"ds_transition", bl_ds_transition_case},
 {"ds_kind", bl_ds_kind_case},
 {"ds_curve_explorer", bl_curve_explorer_case},
 {"ds_curve_explorer_mutation", bl_curve_explorer_mutation_case},
 {"ds_point_explorer", bl_point_explorer_case},
 {"ds_point_explorer_mutation", bl_point_explorer_mutation_case},
 {"ds_interference_tool", bl_interference_tool_case},
 // TopOpeBRepTool shape helpers (tkbool_probe_topopebreptool_shapes.cxx)
 {"tt_shape_tool", bl_tt_shape_tool_case},
 {"tt_project", bl_tt_project_case},
 {"tt_geometry", bl_tt_geometry_case},
 {"tt_fc2d", bl_tt_fc2d_case},
 {"tt_hbox", bl_tt_hbox_case},
 // TopOpeBRepDS data structure and build tool (tkbool_probe_topopebrepds_build.cxx)
 {"ds_recompute", bl_ds_recompute_case},
 {"ds_same_domain", bl_ds_same_domain_case},
 {"ds_sort", bl_ds_sort_case},
 // TopOpeBRepTool classifiers (tkbool_probe_topopebreptool_classifier.cxx)
 {"tt_solid_classifier", bl_tt_solid_classifier_case},
 {"tt_shape_classifier", bl_tt_shape_classifier_case},
 {"tt_fsc", bl_tt_fsc_case},
 {"tt_tggeom", bl_tt_tggeom_case},
 // TopOpeBRepTool TOOL, TOPOLOGY and transitions (tkbool_probe_topopebreptool_tool.cxx)
 {"tt_tool", bl_tt_tool_case},
 {"tt_topology", bl_tt_topology_case},
 {"tt_make_transition", bl_tt_make_transition_case},
 {"tt_mktondge", bl_tt_mktondge_case},
 // TopOpeBRepTool regularization and UV correction (tkbool_probe_topopebreptool_regu.cxx)
 {"tt_classi", bl_tt_classi_case},
 {"tt_regularize", bl_tt_regularize_case},
 {"tt_corriso", bl_tt_corriso_case},
 {"tt_fuse_edges", bl_tt_fuse_edges_case},
 {"tt_purge_internal", bl_tt_purge_internal_case},
 // TopOpeBRepDS interference processing (tkbool_probe_topopebrepds_ops.cxx)
 {"dsx", bl_dsx_case},
 // TopOpeBRepBuild (tkbool_probe_topopebrepbuild.cxx)
 {"tb_wire_to_face", bl_tb_wire_to_face_case},
 {"tb_face_builder", bl_tb_face_builder_case},
 {"tb_wec", bl_tb_wec_case},
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
