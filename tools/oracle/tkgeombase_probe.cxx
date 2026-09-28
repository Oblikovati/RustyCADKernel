// Exact numeric oracle for the TKGeomBase packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkgeombase_numeric.py compares the
// output with crates/tkgeombase/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkgeombase_probe_<package>.cxx, included below, and its rows in probe_cases.
#include <map>
#include <memory>
#include <vector>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <Standard_Failure.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Vec.hxx>
#include <gp_Vec2d.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Cone.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Elips.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Lin.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Parab.hxx>
#include <gp_Parab2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
#include <gp_Trsf2d.hxx>
#include "tkgeombase_probe_common.cxx"
#include "tkgeombase_probe_gce.cxx"
#include "tkgeombase_probe_gc.cxx"
#include "tkgeombase_probe_intana2d.cxx"
#include "tkgeombase_probe_intana.cxx"
#include "tkgeombase_probe_lprop.cxx"
#include "tkgeombase_probe_gprop.cxx"
#include "tkgeombase_probe_hermit.cxx"
#include "tkgeombase_probe_extrema_pc2d.cxx"
#include "tkgeombase_probe_extrema_pc.cxx"
#include "tkgeombase_probe_geomtools.cxx"
#include "tkgeombase_probe_femtool.cxx"
#include "tkgeombase_probe_femtool_curve.cxx"
#include "tkgeombase_probe_appparcurves.cxx"
#include "tkgeombase_probe_appparcurves_fit.cxx"
#include "tkgeombase_probe_bndlib.cxx"
#include "tkgeombase_probe_gcpnts.cxx"
#include "tkgeombase_probe_extrema_el.cxx"
#include "tkgeombase_probe_extrema_pc_classic.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // gce (tkgeombase_probe_gce.cxx)
 {"gce_circ", gb_gce_circ_case},
 {"gce_circ2d", gb_gce_circ2d_case},
 {"gce_cone", gb_gce_cone_case},
 {"gce_cylinder", gb_gce_cylinder_case},
 {"gce_dir", gb_gce_dir_case},
 {"gce_dir2d", gb_gce_dir2d_case},
 {"gce_elips", gb_gce_elips_case},
 {"gce_elips2d", gb_gce_elips2d_case},
 {"gce_hypr", gb_gce_hypr_case},
 {"gce_hypr2d", gb_gce_hypr2d_case},
 {"gce_lin", gb_gce_lin_case},
 {"gce_lin2d", gb_gce_lin2d_case},
 {"gce_parab", gb_gce_parab_case},
 {"gce_parab2d", gb_gce_parab2d_case},
 {"gce_pln", gb_gce_pln_case},
 {"gce_mirror", gb_gce_mirror_case},
 {"gce_mirror2d", gb_gce_mirror2d_case},
 {"gce_rotation", gb_gce_rotation_case},
 {"gce_rotation2d", gb_gce_rotation2d_case},
 {"gce_scale", gb_gce_scale_case},
 {"gce_scale2d", gb_gce_scale2d_case},
 {"gce_translation", gb_gce_translation_case},
 {"gce_translation2d", gb_gce_translation2d_case},
 {"gce_status", gb_gce_status_case},
 // GC (tkgeombase_probe_gc.cxx)
 {"gc_arc_circle", gb_gc_arc_circle_case},
 {"gc_arc_ellipse", gb_gc_arc_ellipse_case},
 {"gc_arc_hyperbola", gb_gc_arc_hyperbola_case},
 {"gc_arc_parabola", gb_gc_arc_parabola_case},
 {"gc_arc_circle2d", gb_gc_arc_circle2d_case},
 {"gc_arc_ellipse2d", gb_gc_arc_ellipse2d_case},
 {"gc_arc_hyperbola2d", gb_gc_arc_hyperbola2d_case},
 {"gc_arc_parabola2d", gb_gc_arc_parabola2d_case},
 {"gc_circle", gb_gc_circle_case},
 {"gc_circle2d", gb_gc_circle2d_case},
 {"gc_ellipse", gb_gc_ellipse_case},
 {"gc_hyperbola", gb_gc_hyperbola_case},
 {"gc_ellipse2d", gb_gc_ellipse2d_case},
 {"gc_hyperbola2d", gb_gc_hyperbola2d_case},
 {"gc_parabola2d", gb_gc_parabola2d_case},
 {"gc_line", gb_gc_line_case},
 {"gc_line2d", gb_gc_line2d_case},
 {"gc_segment", gb_gc_segment_case},
 {"gc_segment2d", gb_gc_segment2d_case},
 {"gc_plane", gb_gc_plane_case},
 {"gc_conical", gb_gc_conical_case},
 {"gc_cylindrical", gb_gc_cylindrical_case},
 {"gc_trimmed_cone", gb_gc_trimmed_cone_case},
 {"gc_trimmed_cylinder", gb_gc_trimmed_cylinder_case},
 {"gc_mirror", gb_gc_mirror_case},
 {"gc_mirror2d", gb_gc_mirror2d_case},
 {"gc_rotation", gb_gc_rotation_case},
 {"gc_rotation2d", gb_gc_rotation2d_case},
 {"gc_scale", gb_gc_scale_case},
 {"gc_scale2d", gb_gc_scale2d_case},
 {"gc_translation", gb_gc_translation_case},
 {"gc_translation2d", gb_gc_translation2d_case},
 // IntAna2d (tkgeombase_probe_intana2d.cxx)
 {"ia2_lines", gb_ia2_lines_case},
 {"ia2_circles", gb_ia2_circles_case},
 {"ia2_line_circle", gb_ia2_line_circle_case},
 {"ia2_conic", gb_ia2_conic_case},
 {"ia2_conic_eval", gb_ia2_conic_eval_case},
 {"ia2_roots", gb_ia2_roots_case},
 {"ia2_outils", gb_ia2_outils_case},
 {"ia2_int_point", gb_ia2_int_point_case},
 // IntAna (tkgeombase_probe_intana.cxx)
 {"ia_quadric", gb_ia_quadric_case},
 {"ia_conic_quad", gb_ia_conic_quad_case},
 {"ia_conic_plane", gb_ia_conic_plane_case},
 {"ia_lin_torus", gb_ia_lin_torus_case},
 {"ia_qqgeo", gb_ia_qqgeo_case},
 {"ia_qq", gb_ia_qq_case},
 {"ia_curve", gb_ia_curve_case},
 // LProp and GeomLProp (tkgeombase_probe_lprop.cxx)
 {"lprop_cl2d", gb_lprop_cl2d_case},
 {"lprop_cl3d", gb_lprop_cl3d_case},
 {"lprop_cl3d_adaptor", gb_lprop_cl3d_adaptor_case},
 {"lprop_sl", gb_lprop_sl_case},
 {"lprop_curinf2d", gb_lprop_curinf2d_case},
 {"lprop_curandinf", gb_lprop_curandinf_case},
 {"lprop_continuity", gb_lprop_continuity_case},
 // GProp (tkgeombase_probe_gprop.cxx)
 {"gprop_points", gb_gprop_points_case},
 {"gprop_pequation", gb_gprop_pequation_case},
 {"gprop_elementary", gb_gprop_elementary_case},
 {"gprop_compose", gb_gprop_compose_case},
 {"gprop_hoperator", gb_gprop_hoperator_case},
 // Hermit (tkgeombase_probe_hermit.cxx)
 {"hermit", gb_hermit_case},
 // ExtremaPC2d, GCPnts_TangentialDeflection (tkgeombase_probe_extrema_pc2d.cxx)
 {"pc2d_curve", gb_pc2d_curve_case},
 {"pc2d_grid", gb_pc2d_grid_case},
 {"pc2d_uniform", gb_pc2d_uniform_case},
 {"pc2d_distance", gb_pc2d_distance_case},
 {"pc2d_endpoints", gb_pc2d_endpoints_case},
 {"gcpnts_td2d", gb_gcpnts_td2d_case},
 // ExtremaPC (tkgeombase_probe_extrema_pc.cxx)
 {"pc_curve", gb_pc_curve_case},
 {"pc_grid", gb_pc_grid_case},
 {"pc_distance", gb_pc_distance_case},
 {"pc_endpoints", gb_pc_endpoints_case},
 {"pc_project", gb_pc_project_case},
 {"gcpnts_td3d", gb_gcpnts_td3d_case},
 // GeomTools (tkgeombase_probe_geomtools*.cxx)
 {"gt_curve", gb_gt_curve_case},
 {"gt_curve2d", gb_gt_curve2d_case},
 {"gt_surface", gb_gt_surface_case},
 {"gt_read", gb_gt_read_case},
 {"gt_get_real", gb_gt_get_real_case},
 {"gt_set", gb_gt_set_case},
 {"gt_set_null", gb_gt_set_null_case},
 {"gt_set_read", gb_gt_set_read_case},
 {"gt_progress", gb_gt_progress_case},
 {"gt_debug_dump", gb_gt_debug_dump_case},
 {"gt_handler", gb_gt_handler_case},
 // FEmTool (tkgeombase_probe_femtool.cxx, tkgeombase_probe_femtool_curve.cxx)
 {"fem_elements", gb_fem_elements_case},
 {"fem_criterion", gb_fem_criterion_case},
 {"fem_profile", gb_fem_profile_case},
 {"fem_curve", gb_fem_curve_case},
 {"fem_assembly", gb_fem_assembly_case},
 // AppParCurves (tkgeombase_probe_appparcurves.cxx, tkgeombase_probe_appparcurves_fit.cxx)
 {"apc_bernstein", gb_apc_bernstein_case},
 {"apc_spline", gb_apc_spline_case},
 {"apc_multipoint", gb_apc_multipoint_case},
 {"apc_multicurve", gb_apc_multicurve_case},
 {"apc_ls", gb_apc_ls_case},
 {"apc_resol", gb_apc_resol_case},
 {"apc_function", gb_apc_function_case},
 {"apc_bspfunction", gb_apc_bspfunction_case},
 {"apc_gradient", gb_apc_gradient_case},
 {"apc_bspgradient", gb_apc_bspgradient_case},
 // GeomBndLib and BndLib (tkgeombase_probe_bndlib.cxx)
 {"bnd_curve3d", gb_bnd_curve3d_case},
 {"bnd_curve2d", gb_bnd_curve2d_case},
 {"bnd_surface", gb_bnd_surface_case},
 {"bnd_gp", gb_bnd_gp_case},
 {"bnd_helpers", gb_bnd_helpers_case},
 // CPnts and GCPnts (tkgeombase_probe_gcpnts.cxx)
 {"gcp_length", gb_gcp_length_case},
 {"cpnts_abscissa", gb_cpnts_abscissa_case},
 {"cpnts_udefl", gb_cpnts_udefl_case},
 {"gcp_abscissa", gb_gcp_abscissa_case},
 {"gcp_uabs", gb_gcp_uabs_case},
 {"gcp_quabs", gb_gcp_quabs_case},
 {"gcp_udefl", gb_gcp_udefl_case},
 {"gcp_qudefl", gb_gcp_qudefl_case},
 {"gcp_tdefl", gb_gcp_tdefl_case},
 {"gcp_arc_step", gb_gcp_arc_step_case},
 {"gcp_dist", gb_gcp_dist_case},
 // Extrema elementary closed forms (tkgeombase_probe_extrema_el.cxx)
 {"ex_pon", gb_ex_pon_case},
 {"ex_pelc", gb_ex_pelc_case},
 {"ex_pelc2d", gb_ex_pelc2d_case},
 {"ex_elc", gb_ex_elc_case},
 {"ex_elc2d", gb_ex_elc2d_case},
 {"ex_pels", gb_ex_pels_case},
 {"ex_elcs", gb_ex_elcs_case},
 {"ex_elss", gb_ex_elss_case},
 // Extrema point-curve classes (tkgeombase_probe_extrema_pc_classic.cxx)
 {"ex_pc", gb_ex_pc_case},
 {"ex_pc2d", gb_ex_pc2d_case},
 {"ex_locpc", gb_ex_locpc_case},
 {"ex_locpc2d", gb_ex_locpc2d_case},
 {"ex_genpc", gb_ex_genpc_case},
 {"ex_genpc2d", gb_ex_genpc2d_case},
 {"ex_pcf", gb_ex_pcf_case},
 {"ex_pcf2d", gb_ex_pcf2d_case},
 {"ex_cloc", gb_ex_cloc_case},
 {"ex_cloc2d", gb_ex_cloc2d_case},
 {"ex_ctool", gb_ex_ctool_case},
 {"ex_ctool2d", gb_ex_ctool2d_case},
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
