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
#include "tkgeomalgo_probe_intsurf.cxx"
#include "tkgeomalgo_probe_law.cxx"
#include "tkgeomalgo_probe_plate.cxx"
#include "tkgeomalgo_probe_localanalysis.cxx"
#include "tkgeomalgo_probe_faircurve.cxx"
#include "tkgeomalgo_probe_intimp.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // Intf (tkgeomalgo_probe_intf.cxx)
 {"intf_triangle", ga_intf_triangle_case},
 // IntSurf (tkgeomalgo_probe_intsurf.cxx)
 {"intsurf_quadric_point", ga_intsurf_quadric_point_case},
 {"intsurf_quadric_uv", ga_intsurf_quadric_uv_case},
 {"intsurf_transition", ga_intsurf_transition_case},
 {"intsurf_same", ga_intsurf_same_case},
 // Law (tkgeomalgo_probe_law.cxx)
 {"law_bspline", ga_law_bspline_case},
 {"law_edit", ga_law_edit_case},
 {"law_function", ga_law_function_case},
 {"law_mix", ga_law_mix_case},
 {"law_interpolate", ga_law_interpolate_case},
 // Plate (tkgeomalgo_probe_plate.cxx)
 {"plate_solve", ga_plate_solve_case},
 {"plate_constraint", ga_plate_constraint_case},
 // LocalAnalysis (tkgeomalgo_probe_localanalysis.cxx)
 {"la_curve", ga_la_curve_case},
 {"la_surface", ga_la_surface_case},
 // FairCurve (tkgeomalgo_probe_faircurve.cxx)
 {"fc_law", ga_fc_law_case},
 {"fc_density", ga_fc_density_case},
 {"fc_energy", ga_fc_energy_case},
 {"fc_batten", ga_fc_batten_case},
 // IntImp (tkgeomalgo_probe_intimp.cxx)
 {"ii_choix", ga_ii_choix_case},
 {"ii_tangence", ga_ii_tangence_case},
 {"ii_zerpar", ga_ii_zerpar_case},
 {"ii_int2s", ga_ii_int2s_case},
 {"ii_zerimp", ga_ii_zerimp_case},
 {"ii_intcs", ga_ii_intcs_case},
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
