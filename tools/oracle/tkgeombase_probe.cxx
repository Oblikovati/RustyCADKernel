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
#include "tkgeombase_probe_geomtools.cxx"
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
