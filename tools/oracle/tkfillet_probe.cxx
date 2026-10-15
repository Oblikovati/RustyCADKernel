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
#include "tkfillet_probe_walking.cxx"
#include "tkfillet_probe_linebuilders.cxx"
#include "tkfillet_probe_chfi2d.cxx"
#include "tkfillet_probe_chfi3d.cxx"
#include "tkfillet_probe_builder0.cxx"
#include "tkfillet_probe_chfikpart.cxx"
#include "tkfillet_probe_api.cxx"
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
 {"fl_rst_rst", fl_rst_rst_case},
 {"fl_app_func", fl_app_func_case},
 {"fl_walking", fl_walking_case},
 {"fl_surf_rst_walk", fl_surf_rst_walk_case},
 {"fl_rst_rst_walk", fl_rst_rst_walk_case},
 // ChFi2d (tkfillet_probe_chfi2d.cxx)
 {"fl_chfi2d_fillet", fl_chfi2d_fillet_case},
 {"fl_chfi2d_chamfer", fl_chfi2d_chamfer_case},
 {"fl_chfi2d_builder", fl_chfi2d_builder_case},
 // ChFi3d (tkfillet_probe_chfi3d.cxx)
 {"fl_hatching_domains", fl_hatching_domains_case},
 {"fl_concave_side", fl_concave_side_case},
 {"fl_search_sing", fl_search_sing_case},
 // ChFi3d_Builder_0 (tkfillet_probe_builder0.cxx)
 {"fl_b0_scalars", fl_b0_scalars_case},
 {"fl_bound_srf", fl_bound_srf_case},
 {"fl_bound_fac", fl_bound_fac_case},
 {"fl_enlarge_box", fl_enlarge_box_case},
 {"fl_b0_faces", fl_b0_faces_case},
 {"fl_b0_spines", fl_b0_spines_case},
 {"fl_inter_plane_edge", fl_inter_plane_edge_case},
 {"fl_b0_pcurves", fl_b0_pcurves_case},
 {"fl_reparam_pcurv", fl_reparam_pcurv_case},
 {"fl_project_pcurv", fl_project_pcurv_case},
 {"fl_b0_bounds", fl_b0_bounds_case},
 {"fl_compute_arete", fl_compute_arete_case},
 {"fl_fil_common_point", fl_fil_common_point_case},
 {"fl_b0_stripes", fl_b0_stripes_case},
 {"fl_b0_tols", fl_b0_tols_case},
 {"fl_int_cs", fl_int_cs_case},
 {"fl_computes_int_pc", fl_computes_int_pc_case},
 {"fl_b0_inter2d", fl_b0_inter2d_case},
 {"fl_b0_topology", fl_b0_topology_case},
 {"fl_b0_smooth", fl_b0_smooth_case},
 {"fl_compute_curves", fl_compute_curves_case},
 {"fl_int_traces", fl_int_traces_case},
 {"fl_el_spine", fl_el_spine_case},
 // ChFiKPart (tkfillet_probe_chfikpart.cxx)
 {"fl_kpart_fcts", fl_kpart_fcts_case},
 {"fl_corner_spine", fl_corner_spine_case},
 {"fl_kpart_make", fl_kpart_make_case},
 {"fl_kpart_corner", fl_kpart_corner_case},
 // BRepFilletAPI (tkfillet_probe_api.cxx)
 {"fl_make_fillet", fl_make_fillet_case},
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
