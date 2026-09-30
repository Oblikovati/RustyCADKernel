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
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // BRepBuilderAPI (tktopalgo_probe_brepbuilderapi.cxx)
 {"tp_make_vertex", tp_make_vertex_case},
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
