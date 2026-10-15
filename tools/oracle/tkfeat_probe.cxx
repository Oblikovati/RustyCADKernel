// Exact numeric oracle for the TKFeat packages (LocOpe and BRepFeat) at pinned OCCT
// 3d097a0. Each input line is `operation numbers...`; tools/verify_tkfeat_numeric.py
// compares the output with crates/tkfeat/examples/numeric_probe.rs token for token.
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
#include "tkfeat_probe_shapes.cxx"
#include "tkfeat_probe_locope.cxx"
#include "tkfeat_probe_sweeps.cxx"
#include "tkfeat_probe_forms.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // LocOpe (tkfeat_probe_locope.cxx)
 {"ft_lo_sweep", ft_lo_sweep_case},
 {"ft_lo_csinter", ft_lo_csinter_case},
 {"ft_lo_find_edges", ft_lo_find_edges_case},
 {"ft_lo_spliter", ft_lo_spliter_case},
 {"ft_lo_gluer", ft_lo_gluer_case},
 // BRepFeat sweeps (tkfeat_probe_sweeps.cxx)
 {"ft_prism", ft_prism_case},
 {"ft_dprism", ft_dprism_case},
 {"ft_revol", ft_revol_case},
 {"ft_pipe", ft_pipe_case},
 // BRepFeat forms, holes, gluing and splitting (tkfeat_probe_forms.cxx)
 {"ft_hole", ft_hole_case},
 {"ft_lform", ft_lform_case},
 {"ft_rform", ft_rform_case},
 {"ft_gluer", ft_gluer_case},
 {"ft_split", ft_split_case},
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
