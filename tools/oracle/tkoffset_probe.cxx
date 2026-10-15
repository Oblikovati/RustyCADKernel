// Exact numeric oracle for the TKOffset packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkoffset_numeric.py compares the
// output with crates/tkoffset/examples/numeric_probe.rs bit for bit. Each package adds its
// own tkoffset_probe_<package>.cxx, included below, and its rows in probe_cases under its
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
#include "tkoffset_probe_brepoffsetapi_make_offset.cxx"
#include "tkoffset_probe_brepoffsetapi_make_pipe_shell.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // BRepOffsetAPI (tkoffset_probe_brepoffsetapi_make_offset.cxx,
 // tkoffset_probe_brepoffsetapi_make_pipe_shell.cxx)
 {"oa_make_offset", of_oa_make_offset_case},
 {"oa_pipe_shell", of_oa_pipe_shell_case},
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
