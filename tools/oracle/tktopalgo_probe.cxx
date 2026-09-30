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
