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
#include "tkshhealing_probe_shcommon.cxx"
#include "tkshhealing_probe_shapecustom.cxx"
#include "tkshhealing_probe_modification.cxx"
#include "tkshhealing_probe_shapeupgrade.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
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
