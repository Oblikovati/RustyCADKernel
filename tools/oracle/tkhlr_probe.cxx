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
#include "tkhlr_probe_topcnx.cxx"
#include "tkhlr_probe_hlralgo.cxx"
#include "tkhlr_probe_hlralgo_projector.cxx"
#include "tkhlr_probe_hlralgo_poly.cxx"
#include "tkhlr_probe_hlralgo_internal.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // HLRAlgo_Projector viewing frame (tkhlr_probe_frame.cxx)
 {"hl_frame", hl_frame_case},
 // Intrv (tkhlr_probe_intrv.cxx)
 {"iv_interval", hl_iv_interval_case},
 {"iv_edit", hl_iv_edit_case},
 {"iv_relate", hl_iv_relate_case},
 {"iv_set", hl_iv_set_case},
 {"iv_value", hl_iv_value_case},
 // TopBas and TopCnx (tkhlr_probe_topcnx.cxx)
 {"tb_interference", hl_tb_interference_case},
 {"tc_transition", hl_tc_transition_case},
 // HLRAlgo (tkhlr_probe_hlralgo.cxx, tkhlr_probe_hlralgo_projector.cxx,
 // tkhlr_probe_hlralgo_poly.cxx, tkhlr_probe_hlralgo_internal.cxx)
 {"ha_minmax", hl_ha_minmax_case},
 {"ha_encode", hl_ha_encode_case},
 {"ha_bipoint", hl_ha_bipoint_case},
 {"ha_edges", hl_ha_edges_case},
 {"ha_wires", hl_ha_wires_case},
 {"ha_status", hl_ha_status_case},
 {"ha_records", hl_ha_records_case},
 {"ha_projector", hl_ha_projector_case},
 {"ha_polydata", hl_ha_polydata_case},
 {"ha_polyalgo", hl_ha_polyalgo_case},
 {"ha_internal", hl_ha_internal_case},
 {"ha_grow", hl_ha_grow_case},
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
