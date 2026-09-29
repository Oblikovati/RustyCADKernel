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
#include "tkgeomalgo_probe_intres2d.cxx"
#include "tkgeomalgo_probe_intimppargen.cxx"
#include "tkgeomalgo_probe_intcurve.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // Intf (tkgeomalgo_probe_intf.cxx)
 {"intf_triangle", ga_intf_triangle_case},
 // IntRes2d (tkgeomalgo_probe_intres2d.cxx)
 {"intres2d_domain", ga_intres2d_domain_case},
 {"intres2d_insert", ga_intres2d_insert_case},
 {"intres2d_append", ga_intres2d_append_case},
 // IntImpParGen (tkgeomalgo_probe_intimppargen.cxx)
 {"imppar_transition", ga_imppar_transition_case},
 {"imppar_domain", ga_imppar_domain_case},
 // IntCurve (tkgeomalgo_probe_intcurve.cxx)
 {"intcurve_conics", ga_intcurve_conics_case},
 {"intcurve_imppar", ga_intcurve_imppar_case},
 {"intcurve_itool", ga_intcurve_itool_case},
 {"intcurve_ptool", ga_intcurve_ptool_case},
 {"intcurve_function", ga_intcurve_function_case},
 {"intcurve_interval", ga_intcurve_interval_case},
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
