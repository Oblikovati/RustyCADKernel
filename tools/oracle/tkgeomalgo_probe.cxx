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
#include "tkgeomalgo_probe_gccent.cxx"
#include "tkgeomalgo_probe_gccint.cxx"
#include "tkgeomalgo_probe_gccana.cxx"
#include "tkgeomalgo_probe_hatch.cxx"
#include "tkgeomalgo_probe_intres2d.cxx"
#include "tkgeomalgo_probe_intimppargen.cxx"
#include "tkgeomalgo_probe_intcurve.cxx"
#include "tkgeomalgo_probe_geom2dint.cxx"
#include "tkgeomalgo_probe_geom2dapi.cxx"
#include "tkgeomalgo_probe_ginter.cxx"
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
 // Geom2dInt (tkgeomalgo_probe_geom2dint.cxx)
 {"g2int_tool", ga_g2int_tool_case},
 {"g2int_proj", ga_g2int_proj_case},
 {"g2int_conic_curve", ga_g2int_conic_curve_case},
 {"g2int_functions", ga_g2int_functions_case},
 // Geom2dAPI (tkgeomalgo_probe_geom2dapi.cxx)
 {"g2api_extrema", ga_g2api_extrema_case},
 {"g2api_project", ga_g2api_project_case},
 {"g2api_interpolate", ga_g2api_interpolate_case},
 {"g2api_points_to_bspline", ga_g2api_points_to_bspline_case},
 // Geom2dInt curve-curve, Geom2dAPI_InterCurveCurve (tkgeomalgo_probe_ginter.cxx)
 {"g2int_polygon", ga_g2int_polygon_case},
 {"g2int_exact", ga_g2int_exact_case},
 {"g2int_polypoly", ga_g2int_polypoly_case},
 {"g2int_ginter", ga_g2int_ginter_case},
 {"g2api_intercc", ga_g2api_intercc_case},
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
