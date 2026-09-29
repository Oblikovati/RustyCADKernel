// Exact numeric oracle for the TKGeomAlgo packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkgeomalgo_numeric.py compares the
// output with crates/tkgeomalgo/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkgeomalgo_probe_<package>.cxx, included below, and its rows in probe_cases
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
#include <array>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_XY.hxx>
#include <gp_XYZ.hxx>
#include "tkgeomalgo_probe_common.cxx"
#include "tkgeomalgo_probe_intf.cxx"
#include "tkgeomalgo_probe_intf_tool.cxx"
#include "tkgeomalgo_probe_intf_poly.cxx"
#include "tkgeomalgo_probe_toptrans.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // Intf (tkgeomalgo_probe_intf*.cxx)
 {"intf_triangle", ga_intf_triangle_case},
 {"intf_spoint", ga_intf_spoint_case},
 {"intf_sline", ga_intf_sline_case},
 {"intf_tzone", ga_intf_tzone_case},
 {"intf_interference", ga_intf_interference_case},
 {"intf_tool2d", ga_intf_tool2d_case},
 {"intf_tool3d", ga_intf_tool3d_case},
 {"intf_ipoly2d", ga_intf_ipoly2d_case},
 {"intf_ipp", ga_intf_ipp_case},
 // TopTrans (tkgeomalgo_probe_toptrans.cxx)
 {"tt_curve", ga_tt_curve_case},
 {"tt_surface", ga_tt_surface_case},
 {"tt_states", ga_tt_states_case},
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
