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
