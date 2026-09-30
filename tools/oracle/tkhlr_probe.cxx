// Exact numeric oracle for the TKHLR packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkhlr_numeric.py compares the
// output with crates/tkhlr/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkhlr_probe_<package>.cxx, included below, and its rows in probe_cases
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
#include "tkhlr_probe_frame.cxx"
#include "tkhlr_probe_intrv.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // HLRAlgo_Projector viewing frame (tkhlr_probe_frame.cxx)
 {"hl_frame", hl_frame_case},
 // Intrv (tkhlr_probe_intrv.cxx)
 {"iv_interval", hl_iv_interval_case},
 {"iv_edit", hl_iv_edit_case},
 {"iv_relate", hl_iv_relate_case},
 {"iv_set", hl_iv_set_case},
 {"iv_value", hl_iv_value_case},
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
