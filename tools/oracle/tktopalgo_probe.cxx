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
#include "tktopalgo_probe_bisector.cxx"
#include "tktopalgo_probe_mat.cxx"
#include "tktopalgo_probe_mat_parts.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
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
