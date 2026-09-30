// Exact numeric oracle for the TKTopAlgo packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tktopalgo_numeric.py compares the
// output with crates/tktopalgo/examples/numeric_probe.rs bit for bit. Each package adds
// its own tktopalgo_probe_<package>.cxx, included below, and its rows in probe_cases
// under its own package comment.
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
// The TKGeomAlgo reader and bit printers (ga_*), shared so both toolkits print the same
// token stream; the Rust twin includes crates/tkgeomalgo/examples/ga_probe/common.rs.
#include "tkgeomalgo_probe_common.cxx"
#include "tktopalgo_probe_brepbuilderapi.cxx"
#include "tkgeomalgo_probe_intres2d.cxx"
#include "tktopalgo_probe_face.cxx"
#include "tktopalgo_probe_brepclass.cxx"
#include "tktopalgo_probe_breptopadaptor.cxx"
#include "tktopalgo_probe_intcurvesface.cxx"
#include "tktopalgo_probe_bisector.cxx"
#include "tktopalgo_probe_mat.cxx"
#include "tktopalgo_probe_mat_parts.cxx"
#include "tktopalgo_probe_shapes.cxx"
#include "tktopalgo_probe_brepcheck.cxx"
#include "tktopalgo_probe_shape.cxx"
#include "tktopalgo_probe_brepbndlib.cxx"
#include "tktopalgo_probe_brepclass3d.cxx"
// The IntPatch line printers of the TKGeomAlgo probe (ga_ipl_*) with the readers they
// need, for the BRepTopAdaptor domains of IntStart and IntPatch.
#include "tkgeomalgo_probe_intsurf.cxx"
#include "tkgeomalgo_probe_intwalk.cxx"
#include "tkgeomalgo_probe_iwalk.cxx"
#include "tkgeomalgo_probe_intpatch.cxx"
#include "tkgeomalgo_probe_intpatch_report.cxx"
#include "tktopalgo_probe_breptopadaptor_domain.cxx"
#include "tktopalgo_probe_breplib_common.cxx"
#include "tktopalgo_probe_breplib_edge.cxx"
#include "tktopalgo_probe_breplib_wire.cxx"
#include "tktopalgo_probe_breplib_shapes.cxx"
#include "tktopalgo_probe_breplib_package.cxx"
#include "tktopalgo_probe_breplib_face.cxx"
#include "tktopalgo_probe_breplib_fuse.cxx"
#include "tktopalgo_probe_brepgprop.cxx"
#include "tktopalgo_probe_brepgprop_tools.cxx"
#include "tktopalgo_probe_brepapprox.cxx"
#include "tktopalgo_probe_brepapprox_parts.cxx"
#include "tktopalgo_probe_brepapprox_tool.cxx"
#include "tktopalgo_probe_brepintcurvesurface.cxx"
#include "tktopalgo_probe_brepbuilderapi_makers.cxx"
#include "tktopalgo_probe_brepbuilderapi_modify.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // BRepBuilderAPI (tktopalgo_probe_brepbuilderapi.cxx)
 {"tp_make_vertex", tp_make_vertex_case},
 // BRepBuilderAPI makers and modifications (tktopalgo_probe_brepbuilderapi_makers.cxx,
 // tktopalgo_probe_brepbuilderapi_modify.cxx)
 {"bba_vertex", tp_bba_vertex_case},
 {"bba_package", tp_bba_package_case},
 {"bba_edge", tp_bba_edge_case},
 {"bba_edge2d", tp_bba_edge2d_case},
 {"bba_polygon", tp_bba_polygon_case},
 {"bba_wire", tp_bba_wire_case},
 {"bba_solid", tp_bba_solid_case},
 {"bba_face", tp_bba_face_case},
 {"bba_shell", tp_bba_shell_case},
 {"bba_transform", tp_bba_transform_case},
 {"bba_gtransform", tp_bba_gtransform_case},
 {"bba_copy", tp_bba_copy_case},
 {"bba_nurbs", tp_bba_nurbs_case},
 {"bba_modify", tp_bba_modify_case},
 {"bba_findplane", tp_bba_findplane_case},
 {"bba_collect", tp_bba_collect_case},
 // BRepClass (tktopalgo_probe_brepclass.cxx, faces of tktopalgo_probe_face.cxx)
 {"brc_edge", tp_brc_edge_case},
 {"brc_explorer", tp_brc_explorer_case},
 {"brc_segment", tp_brc_segment_case},
 {"brc_checkpoint", tp_brc_checkpoint_case},
 {"brc_intersect", tp_brc_intersect_case},
 {"brc_intersect2", tp_brc_intersect2_case},
 {"brc_classify", tp_brc_classify_case},
 {"brc_classify3d", tp_brc_classify3d_case},
 {"brc_passive", tp_brc_passive_case},
 {"brc_defaults", tp_brc_defaults_case},
 // BRepTopAdaptor (tktopalgo_probe_breptopadaptor.cxx)
 {"bta_fclass2d", tp_bta_fclass2d_case},
 {"bta_destroy", tp_bta_destroy_case},
 {"bta_topol", tp_bta_topol_case},
 {"bta_classify", tp_bta_classify_case},
 {"bta_tool", tp_bta_tool_case},
 {"bta_errors", tp_bta_errors_case},
 // IntCurvesFace (tktopalgo_probe_intcurvesface.cxx)
 {"icf_line", tp_icf_line_case},
 {"icf_curve", tp_icf_curve_case},
 {"icf_shape", tp_icf_shape_case},
 // BRepBuilderAPI (tktopalgo_probe_brepbuilderapi.cxx)
 {"tp_make_vertex", tp_make_vertex_case},
 // Bisector (tktopalgo_probe_bisector.cxx)
 {"tp_bisec_curves", tp_bisec_curves_case},
 {"tp_bisec_curve_point", tp_bisec_curve_point_case},
 {"tp_bisec_points", tp_bisec_points_case},
 {"tp_bisec_ana", tp_bisec_ana_case},
 {"tp_bisec_cc", tp_bisec_cc_case},
 {"tp_bisec_pc", tp_bisec_pc_case},
 {"tp_bisec_inter", tp_bisec_inter_case},
 {"tp_bisec_function_h", tp_bisec_function_h_case},
 {"tp_bisec_function_inter", tp_bisec_function_inter_case},
 {"tp_bisec_poly", tp_bisec_poly_case},
 {"tp_bisec_is_convex", tp_bisec_is_convex_case},
 // MAT and MAT2d (tktopalgo_probe_mat.cxx, tktopalgo_probe_mat_parts.cxx)
 {"tp_mat2d_locus", tp_mat2d_locus_case},
 {"tp_mat_list", tp_mat_list_case},
 {"tp_mat_bisector", tp_mat_bisector_case},
 {"tp_mat_arc", tp_mat_arc_case},
 {"tp_mat2d_connexion", tp_mat2d_connexion_case},
 {"tp_mat2d_cut", tp_mat2d_cut_case},
 {"tp_mat2d_minipath", tp_mat2d_minipath_case},
 {"tp_mat2d_tool", tp_mat2d_tool_case},
 // BRepCheck and BRepLib_ValidateEdge (tktopalgo_probe_brepcheck.cxx, shapes of
 // tktopalgo_probe_shapes.cxx)
 {"bck_status", tp_bck_status_case},
 {"bck_add", tp_bck_add_case},
 {"bck_prec", tp_bck_prec_case},
 {"bck_selfint", tp_bck_selfint_case},
 {"bck_check", tp_bck_check_case},
 {"ble_validate", tp_ble_validate_case},
 {"bck_analyze", tp_bck_analyze_case},
 // BRepBndLib (tktopalgo_probe_brepbndlib.cxx, shapes of tktopalgo_probe_shape.cxx)
 {"bbl_add", tp_bbl_add_case},
 // BRepClass3d (tktopalgo_probe_brepclass3d.cxx, shapes of tktopalgo_probe_shape.cxx)
 {"bc3_classify", tp_bc3_classify_case},
 {"bc3_infinite", tp_bc3_infinite_case},
 {"bc3_sclassifier", tp_bc3_sclassifier_case},
 {"bc3_segment", tp_bc3_segment_case},
 {"bc3_explorer", tp_bc3_explorer_case},
 {"bc3_findpoint", tp_bc3_findpoint_case},
 {"bc3_pointinface", tp_bc3_pointinface_case},
 {"bc3_uvpoint", tp_bc3_uvpoint_case},
 {"bc3_tree", tp_bc3_tree_case},
 {"bc3_inter3d", tp_bc3_inter3d_case},
 // BRepTopAdaptor domains of IntStart and IntPatch (tktopalgo_probe_breptopadaptor_domain.cxx)
 {"bta_sob", tp_bta_sob_case},
 {"bta_inside", tp_bta_inside_case},
 {"bta_intersection", tp_bta_intersection_case},
 // BRepLib (tktopalgo_probe_breplib_*.cxx)
 {"blm_vertex", tp_blm_vertex_case},
 {"blm_edge", tp_blm_edge_case},
 {"blm_edge2d", tp_blm_edge2d_case},
 {"blm_polygon", tp_blm_polygon_case},
 {"blm_wire", tp_blm_wire_case},
 {"blm_solid", tp_blm_solid_case},
 {"blp_build3d", tp_blp_build3d_case},
 {"blp_samerange", tp_blp_samerange_case},
 {"blp_edgetol", tp_blp_edgetol_case},
 {"blp_regularity", tp_blp_regularity_case},
 {"blp_continuity", tp_blp_continuity_case},
 {"blp_inner", tp_blp_inner_case},
 {"blp_sort", tp_blp_sort_case},
 {"blp_bounding", tp_blp_bounding_case},
 {"blp_validrange", tp_blp_validrange_case},
 {"blp_validrange2", tp_blp_validrange2_case},
 {"blp_pcurveplane", tp_blp_pcurveplane_case},
 {"blp_ccos", tp_blp_ccos_case},
 {"blp_shell", tp_blp_shell_case},
 {"blp_normals", tp_blp_normals_case},
 {"blp_deflection", tp_blp_deflection_case},
 {"blp_trinormals", tp_blp_trinormals_case},
 {"blp_tolerances", tp_blp_tolerances_case},
 {"blp_orient", tp_blp_orient_case},
 {"blm_face", tp_blm_face_case},
 {"blm_facedegen", tp_blm_facedegen_case},
 {"blp_findsurface", tp_blp_findsurface_case},
 {"blp_sameparam", tp_blp_sameparam_case},
 {"blp_extendface", tp_blp_extendface_case},
 {"blp_fuseedges", tp_blp_fuseedges_case},
 // BRepGProp (tktopalgo_probe_brepgprop.cxx, tktopalgo_probe_brepgprop_tools.cxx, shapes of
 // tktopalgo_probe_shape.cxx)
 {"bgp_package", tp_bgp_package_case},
 {"bgp_cinert", tp_bgp_cinert_case},
 {"bgp_mesh_cinert", tp_bgp_mesh_cinert_case},
 {"bgp_mesh_nodes", tp_bgp_mesh_nodes_case},
 {"bgp_mesh_props", tp_bgp_mesh_props_case},
 {"bgp_mesh_calc", tp_bgp_mesh_calc_case},
 {"bgp_sinert", tp_bgp_sinert_case},
 {"bgp_vinert", tp_bgp_vinert_case},
 {"bgp_vinert_gk", tp_bgp_vinert_gk_case},
 {"bgp_face", tp_bgp_face_case},
 {"bgp_knots", tp_bgp_knots_case},
 {"bgp_arrays", tp_bgp_arrays_case},
 {"bgp_domain", tp_bgp_domain_case},
 {"bgp_gauss", tp_bgp_gauss_case},
 {"bgp_functions", tp_bgp_functions_case},
 // BRepApprox (tktopalgo_probe_brepapprox*.cxx, faces of tktopalgo_probe_face.cxx)
 {"bra_approx", tp_bra_approx_case},
 {"bra_line", tp_bra_line_case},
 {"bra_mline", tp_bra_mline_case},
 {"bra_sv", tp_bra_sv_case},
 {"bra_zerimp", tp_bra_zerimp_case},
 {"bra_int2s", tp_bra_int2s_case},
 {"bra_tool", tp_bra_tool_case},
 // BRepIntCurveSurface (tktopalgo_probe_brepintcurvesurface.cxx, shapes of
 // tktopalgo_probe_shape.cxx)
 {"bics_line", tp_bics_line_case},
 {"bics_curve", tp_bics_curve_case},
 {"bics_state", tp_bics_state_case},
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
