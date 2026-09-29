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
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_XY.hxx>
#include <gp_XYZ.hxx>
#include "tkgeomalgo_probe_common.cxx"
#include "tkgeomalgo_probe_intf.cxx"
#include "tkgeomalgo_probe_gccent.cxx"
#include "tkgeomalgo_probe_gccint.cxx"
#include "tkgeomalgo_probe_gccana.cxx"
#include "tkgeomalgo_probe_hatch.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // Intf (tkgeomalgo_probe_intf.cxx)
 {"intf_triangle", ga_intf_triangle_case},
 // GccEnt (tkgeomalgo_probe_gccent.cxx)
 {"gccent_position", ga_gccent_position_case},
 {"gccent_qualified", ga_gccent_qualified_case},
 // GccInt (tkgeomalgo_probe_gccint.cxx)
 {"gccint_bisec", ga_gccint_bisec_case},
 // GccAna (tkgeomalgo_probe_gccana.cxx)
 {"gccana_circ3tan", ga_gccana_circ3tan_case},
 {"gccana_2tanrad", ga_gccana_2tanrad_case},
 {"gccana_2tanon", ga_gccana_2tanon_case},
 {"gccana_tanonrad", ga_gccana_tanonrad_case},
 {"gccana_tancen", ga_gccana_tancen_case},
 {"gccana_lin2tan", ga_gccana_lin2tan_case},
 {"gccana_lintanobl", ga_gccana_lintanobl_case},
 {"gccana_linbisec", ga_gccana_linbisec_case},
 {"gccana_bisec", ga_gccana_bisec_case},
 // Hatch (tkgeomalgo_probe_hatch.cxx)
 {"hatch_hatcher", ga_hatch_hatcher_case},
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
