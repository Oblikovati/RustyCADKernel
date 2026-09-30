// Exact numeric oracle for the TKShHealing packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkshhealing_numeric.py compares the
// output with crates/tkshhealing/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkshhealing_probe_<package>.cxx, included below, and its rows in probe_cases
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
// The TKGeomAlgo reader and bit printers (ga_*), shared so every toolkit prints the same
// token stream; the Rust twin includes crates/tkgeomalgo/examples/ga_probe/common.rs.
#include "tkgeomalgo_probe_common.cxx"
#include "tkshhealing_probe_shapeextend.cxx"
// The TKTopAlgo shape programs, face and surface readers and shape digest (tp_*), shared so
// both toolkits build and print shapes the same way; the Rust twin includes their modules.
#include "tktopalgo_probe_face.cxx"
#include "tktopalgo_probe_breplib_common.cxx"
#include "tktopalgo_probe_shapes.cxx"
#include "tkshhealing_probe_wiredata.cxx"
#include "tkshhealing_probe_explorer.cxx"
#include "tkshhealing_probe_registrator.cxx"
#include "tkshhealing_probe_complexcurve.cxx"
#include "tkshhealing_probe_composite.cxx"
#include "tkshhealing_probe_shapebuild.cxx"
#include "tkshhealing_probe_reshape.cxx"
#include "tkshhealing_probe_analysis_basic.cxx"
#include "tkshhealing_probe_fix_basic.cxx"
#include "tkshhealing_probe_shcommon.cxx"
#include "tkshhealing_probe_shapecustom.cxx"
#include "tkshhealing_probe_modification.cxx"
#include "tkshhealing_probe_shapeupgrade.cxx"
#include "tkshhealing_probe_healing_common.cxx"
#include "tkshhealing_probe_analysis_edge.cxx"
#include "tkshhealing_probe_edge_outputs.cxx"
#include "tkshhealing_probe_canonical.cxx"
#include "tkshhealing_probe_shapeconstruct.cxx"
#include "tkshhealing_probe_analysis_surface.cxx"
#include "tkshhealing_probe_project_curve.cxx"
#include "tkshhealing_probe_shell_sewing.cxx"
#include "tkshhealing_probe_free_bounds.cxx"
#include "tkshhealing_probe_split_curve3d.cxx"
#include "tkshhealing_probe_split_curve2d.cxx"
#include "tkshhealing_probe_split_surface.cxx"
#include "tkshhealing_probe_edge_divide.cxx"
#include "tkshhealing_probe_transfer_parameters.cxx"
#include "tkshhealing_probe_transfer_parameters_proj.cxx"
#include "tkshhealing_probe_fix_small_curves.cxx"
#include "tkshhealing_probe_wire_divide.cxx"
#include "tkshhealing_probe_wire_divide_state.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 {"sh_wire_state", sh_wire_state_case},
 {"sh_wire_divide", sh_wire_divide_case},
 {"sh_fix_small_curves", sh_fix_small_curves_case},
 {"sh_transfer_parameters", sh_transfer_parameters_case},
 {"sh_transfer_parameters_proj", sh_transfer_parameters_proj_case},
 {"sh_edge_divide", sh_edge_divide_case},
 {"sh_split_surface", sh_split_surface_case},
 {"sh_split_curve2d", sh_split_curve2d_case},
 {"sh_split_curve3d", sh_split_curve3d_case},
 {"sh_free_bounds", sh_free_bounds_case},
 {"sh_shell_sewing", sh_shell_sewing_case},
 {"sh_project_curve", sh_project_curve_case},
 {"sh_construct", sh_construct_case},
 {"sh_analysis_surface", sh_analysis_surface_case},
 {"sh_canonical", sh_canonical_case},
 {"sh_edgeoutputs", sh_edgeoutputs_case},
 {"sh_edgeana", sh_edgeana_case},
 // ShapeExtend (tkshhealing_probe_shapeextend.cxx)
 {"sh_status", sh_status_case},
 {"sh_msg", sh_message_case},
 // ShapeExtend_WireData (tkshhealing_probe_wiredata.cxx)
 {"sh_wiredata", sh_wiredata_case},
 // ShapeExtend_Explorer (tkshhealing_probe_explorer.cxx)
 {"sh_explorer", sh_explorer_case},
 // ShapeExtend registrators (tkshhealing_probe_registrator.cxx)
 {"sh_registrator", sh_registrator_case},
 // ShapeExtend_ComplexCurve (tkshhealing_probe_complexcurve.cxx)
 {"sh_complex", sh_complex_case},
 // ShapeExtend_CompositeSurface (tkshhealing_probe_composite.cxx)
 {"sh_composite", sh_composite_case},
 // ShapeBuild (tkshhealing_probe_shapebuild.cxx, tkshhealing_probe_reshape.cxx)
 {"sh_vertex", sh_vertex_case},
 {"sh_edge", sh_edge_case},
 {"sh_reshape", sh_reshape_case},
 // ShapeAnalysis_Geom, _ShapeTolerance, _Shell, _WireOrder, _FreeBoundData (tkshhealing_probe_analysis_basic.cxx)
 {"sh_geom", sh_geom_case},
 {"sh_shapetol", sh_shapetol_case},
 {"sh_shell", sh_shell_case},
 {"sh_wireorder", sh_wireorder_case},
 {"sh_fbdata", sh_fbdata_case},
 // ShapeFix_Root, _ShapeTolerance, _EdgeConnect (tkshhealing_probe_fix_basic.cxx)
 {"sh_fixroot", sh_fixroot_case},
 {"sh_fixtol", sh_fixtol_case},
 {"sh_edgeconnect", sh_edgeconnect_case},
 // ShapeCustom (tkshhealing_probe_shapecustom.cxx, tkshhealing_probe_modification.cxx)
 {"sc_curve2d", sc_curve2d_case},
 {"sc_restriction", sc_restriction_case},
 {"sc_modif", sc_modif_case},
 // ShapeUpgrade (tkshhealing_probe_shapeupgrade.cxx)
 {"su_c0split", su_c0split_case},
 {"su_tool", su_tool_case},
 {"su_removeloc", su_removeloc_case},
 {"su_splitcurve", su_splitcurve_case},
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
