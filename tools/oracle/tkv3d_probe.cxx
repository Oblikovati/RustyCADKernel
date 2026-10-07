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
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // Prs3d tools, arrows, deflection and boxes (tkv3d_probe_prs3d.cxx)
 {"v3_quadric", v3_quadric_case},
 {"v3_arrow", v3_arrow_case},
 {"v3_prs3d", v3_prs3d_case},
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
