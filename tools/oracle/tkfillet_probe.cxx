// Exact numeric oracle for the TKFillet packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkfillet_numeric.py compares the
// output with crates/tkfillet/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkfillet_probe_<package>.cxx, included below, and its rows in probe_cases
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
#include "tkfillet_probe_chfids.cxx"
#include "tkfillet_probe_blendfunc.cxx"
#include "tkfillet_probe_brepblend.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // ChFiDS (tkfillet_probe_chfids.cxx)
 {"fl_spine", fl_spine_case},
 {"fl_arc_spine", fl_arc_spine_case},
 {"fl_fil_law", fl_fil_law_case},
 // BlendFunc (tkfillet_probe_blendfunc.cxx)
 {"fl_const_rad", fl_const_rad_case},
 {"fl_const_rad_inv", fl_const_rad_inv_case},
 {"fl_evol_rad", fl_evol_rad_case},
 {"fl_evol_rad_inv", fl_evol_rad_inv_case},
 {"fl_chamfer", fl_chamfer_case},
 {"fl_chamfer_inv", fl_chamfer_inv_case},
 // BRepBlend (tkfillet_probe_brepblend.cxx)
 {"fl_rad_inv", fl_rad_inv_case},
 {"fl_blend_tool", fl_blend_tool_case},
 {"fl_surf_rst", fl_surf_rst_case},
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
