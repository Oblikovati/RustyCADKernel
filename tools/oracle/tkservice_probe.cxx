// Exact numeric oracle for the TKService packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkservice_numeric.py compares the
// output with crates/tkservice/examples/numeric_probe.rs bit for bit. Each package adds
// its own tkservice_probe_<family>.cxx, included below, and its rows in probe_cases
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
// The TKGeomAlgo reader and bit printers (ga_*), shared so every toolkit prints the same
// token stream; the Rust twin includes crates/tkgeomalgo/examples/ga_probe/common.rs.
#include "tkgeomalgo_probe_common.cxx"
#include "tkservice_probe_grid.cxx"
#include "tkservice_probe_hid.cxx"
#include "tkservice_probe_pixel.cxx"
#include "tkservice_probe_pbr.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // Aspect grids (tkservice_probe_grid.cxx)
 {"sv_rgrid", sv_rgrid_case},
 {"sv_cgrid", sv_cgrid_case},
 // WNT_HIDSpaceMouse (tkservice_probe_hid.cxx)
 {"sv_hid", sv_hid_case},
 // Image_PixMap pixel conversions (tkservice_probe_pixel.cxx)
 {"sv_pixel", sv_pixel_case},
 {"sv_rawpixel", sv_rawpixel_case},
 {"sv_half", sv_half_case},
 // Graphic3d_PBRMaterial and Graphic3d_BSDF (tkservice_probe_pbr.cxx)
 {"sv_pbr_lut", sv_pbr_lut_case},
 {"sv_pbr_factor", sv_pbr_factor_case},
 {"sv_pbr_bsdf", sv_pbr_bsdf_case},
 {"sv_fresnel", sv_fresnel_case},
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
