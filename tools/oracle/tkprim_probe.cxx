// Exact numeric oracle for the TKPrim packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkprim_numeric.py compares the
// output with crates/tkprim/examples/numeric_probe.rs bit for bit. Each package adds its
// own tkprim_probe_<package>.cxx, included below, and its rows in probe_cases under its
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
#include "tkprim_probe_shape.cxx"
#include "tkprim_probe_sweep.cxx"
#include "tkprim_probe_brepprim.cxx"
#include "tkprim_probe_brepprim_oneaxis.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // The shared shape digest (tkprim_probe_shape.cxx)
 {"pr_digest_vertex", pr_digest_vertex_case},
 // Sweep (tkprim_probe_sweep.cxx)
 {"sw_numshape", pr_sw_numshape_case},
 {"sw_reinit", pr_sw_reinit_case},
 {"sw_init", pr_sw_init_case},
 // BRepPrim (tkprim_probe_brepprim.cxx, tkprim_probe_brepprim_oneaxis.cxx)
 {"bp_wedge", pr_bp_wedge_case},
 {"bp_builder", pr_bp_builder_case},
 {"bp_oneaxis", pr_bp_oneaxis_case},
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
