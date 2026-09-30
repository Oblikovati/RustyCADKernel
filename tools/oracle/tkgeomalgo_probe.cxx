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
#include "tkgeomalgo_probe_intsurf.cxx"
#include "tkgeomalgo_probe_law.cxx"
#include "tkgeomalgo_probe_plate.cxx"
#include "tkgeomalgo_probe_localanalysis.cxx"
#include "tkgeomalgo_probe_faircurve.cxx"
#include "tkgeomalgo_probe_intimp.cxx"
#include "tkgeomalgo_probe_appblend.cxx"
#include "tkgeomalgo_probe_hatchgen.cxx"
#include "tkgeomalgo_probe_geom2dhatch.cxx"
#include "tkgeomalgo_probe_geom2dhatcher.cxx"
#include <type_traits>
#include "tkgeomalgo_probe_geom2dgcc.cxx"
#include "tkgeomalgo_probe_geom2dgcc_circles.cxx"
#include "tkgeomalgo_probe_intwalk.cxx"
#include "tkgeomalgo_probe_iwalk.cxx"
#include "tkgeomalgo_probe_intcurvesurface.cxx"
#include "tkgeomalgo_probe_intcurvesurface_inter.cxx"
#include "tkgeomalgo_probe_gfill.cxx"
#include "tkgeomalgo_probe_gfill_fill.cxx"
#include "tkgeomalgo_probe_gfill_cf.cxx"
#include "tkgeomalgo_probe_intstart.cxx"
#include "tkgeomalgo_probe_intpolyh.cxx"
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
 // AppBlend (tkgeomalgo_probe_appblend.cxx)
 {"ab_appsurf", ga_ab_appsurf_case},
 {"ab_context", ga_ab_context_case},
 // HatchGen (tkgeomalgo_probe_hatchgen.cxx)
 {"hgen_element", ga_hgen_element_case},
 {"hgen_hatching", ga_hgen_hatching_case},
 {"hgen_domain", ga_hgen_domain_case},
 // Geom2dHatch (tkgeomalgo_probe_geom2dhatch*.cxx)
 {"g2hatch_local", ga_g2hatch_local_case},
 {"g2hatch_perform", ga_g2hatch_perform_case},
 {"g2hatch_intersect", ga_g2hatch_intersect_case},
 {"g2hatch_elements", ga_g2hatch_elements_case},
 {"g2hatch_classify", ga_g2hatch_classify_case},
 {"g2hatch_fclass", ga_g2hatch_fclass_case},
 {"g2hatch_hatcher", ga_g2hatch_hatcher_case},
 {"g2hatch_hatching", ga_g2hatch_hatching_case},
 // Geom2dGcc (tkgeomalgo_probe_geom2dgcc*.cxx)
 {"g2gcc_qualified", ga_g2gcc_qualified_case},
 {"g2gcc_tool", ga_g2gcc_tool_case},
 {"g2gcc_func1", ga_g2gcc_func1_case},
 {"g2gcc_func2", ga_g2gcc_func2_case},
 {"g2gcc_func3", ga_g2gcc_func3_case},
 {"g2gcc_func4", ga_g2gcc_func4_case},
 {"g2gcc_lin2tan", ga_g2gcc_lin2tan_case},
 {"g2gcc_lin2tan_iter", ga_g2gcc_lin2tan_iter_case},
 {"g2gcc_lintanobl", ga_g2gcc_lintanobl_case},
 {"g2gcc_lintanobl_iter", ga_g2gcc_lintanobl_iter_case},
 {"g2gcc_tancen", ga_g2gcc_tancen_case},
 {"g2gcc_tancen_geo", ga_g2gcc_tancen_geo_case},
 {"g2gcc_2tanrad", ga_g2gcc_2tanrad_case},
 {"g2gcc_2tanrad_geo", ga_g2gcc_2tanrad_geo_case},
 {"g2gcc_3tan", ga_g2gcc_3tan_case},
 {"g2gcc_3tan_iter", ga_g2gcc_3tan_iter_case},
 {"g2gcc_2tanon", ga_g2gcc_2tanon_case},
 {"g2gcc_2tanon_geo", ga_g2gcc_2tanon_geo_case},
 {"g2gcc_2tanon_iter", ga_g2gcc_2tanon_iter_case},
 // IntWalk (tkgeomalgo_probe_intwalk.cxx, tkgeomalgo_probe_iwalk.cxx)
 {"pw_walk", ga_iw_pw_walk_case},
 {"pw_first", ga_iw_pw_first_case},
 {"pw_edit", ga_iw_pw_edit_case},
 {"pw_min", ga_iw_pw_min_case},
 {"iw_walk", ga_iw_iw_walk_case},
 // IntCurveSurface (tkgeomalgo_probe_intcurvesurface*.cxx)
 {"ics_records",ga_ics_records_case},
 {"ics_intersection",ga_ics_intersection_case},
 {"ics_hcurve",ga_ics_hcurve_case},
 {"ics_polygon",ga_ics_polygon_case},
 {"ics_polyhedron",ga_ics_polyhedron_case},
 {"ics_utils",ga_ics_utils_case},
 {"ics_interference",ga_ics_interference_case},
 {"ics_exact",ga_ics_exact_case},
 {"ics_quadcurv",ga_ics_quadcurv_case},
 {"ics_hinter",ga_ics_hinter_case},
 {"ics_hinter_protected",ga_ics_hinter_protected_case},
 {"ics_estlim",ga_ics_estlim_case},
 {"ics_project",ga_ics_project_case},
 {"ics_interutils",ga_ics_interutils_case},
 // GeomFill filling part (tkgeomalgo_probe_gfill*.cxx)
 {"gfill_line", ga_gfill_line_case},
 {"gfill_profiler", ga_gfill_profiler_case},
 {"gfill_secgen", ga_gfill_secgen_case},
 {"gfill_appsurf", ga_gfill_appsurf_case},
 {"gfill_generator", ga_gfill_generator_case},
 {"gfill_scheme", ga_gfill_scheme_case},
 {"gfill_bezier", ga_gfill_bezier_case},
 {"gfill_bspline", ga_gfill_bspline_case},
 {"gfill_bound", ga_gfill_bound_case},
 {"gfill_corner", ga_gfill_corner_case},
 {"gfill_patch", ga_gfill_patch_case},
 {"gfill_cfill", ga_gfill_cfill_case},
 // IntStart (tkgeomalgo_probe_intstart.cxx)
 {"is_sob", ga_is_sob_case},
 {"is_inside", ga_is_inside_case},
 // IntPolyh (tkgeomalgo_probe_intpolyh.cxx)
 {"ph_inter", ga_ph_inter_case},
 {"ph_maillage", ga_ph_maillage_case},
 {"ph_tricontact", ga_ph_tricontact_case},
 {"ph_edgecontact", ga_ph_edgecontact_case},
 {"ph_point", ga_ph_point_case},
 {"ph_triangle", ga_ph_triangle_case},
 {"ph_startpoint", ga_ph_startpoint_case},
 {"ph_sectionline", ga_ph_sectionline_case},
 {"ph_array", ga_ph_array_case},
 {"ph_tools", ga_ph_tools_case},
 {"ph_couple", ga_ph_couple_case},
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
