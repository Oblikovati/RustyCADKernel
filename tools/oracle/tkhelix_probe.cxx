// Exact numeric oracle for the TKHelix packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkhelix_numeric.py compares the
// output with crates/tkhelix/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkhelix_probe_<package>.cxx, included below, and its rows in probe_cases
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
#include "tkhelix_probe_placement.cxx"
#include "tkhelix_probe_helixgeom.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // HelixGeom_BuilderHelix placement (tkhelix_probe_placement.cxx)
 {"hx_placement", hx_placement_case},
 // HelixGeom (tkhelix_probe_helixgeom.cxx)
 {"hx_curve", hx_curve_case},
 {"hx_appr_curve3d", hx_appr_curve3d_case},
 {"hx_appr_helix", hx_appr_helix_case},
 {"hx_coil", hx_coil_case},
 {"hx_helix", hx_helix_case},
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
