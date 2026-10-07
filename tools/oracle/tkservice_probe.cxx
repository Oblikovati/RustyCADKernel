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
#include "tkservice_probe_material.cxx"
#include "tkservice_probe_shader.cxx"
#include "tkservice_probe_texture.cxx"
#include "tkservice_probe_marker.cxx"
#include "tkservice_probe_aspects.cxx"
#include "tkservice_probe_array.cxx"
#include "tkservice_probe_camera.cxx"
#include "tkservice_probe_clip.cxx"
#include "tkservice_probe_light.cxx"
#include "tkservice_probe_tpers.cxx"
#include "tkservice_probe_params.cxx"
#include "tkservice_probe_font.cxx"
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
 // Graphic3d_MaterialAspect (tkservice_probe_material.cxx)
 {"sv_material", sv_material_case},
 {"sv_material_rank", sv_material_rank_case},
 {"sv_material_edit", sv_material_edit_case},
 // Graphic3d_ShaderObject declarations (tkservice_probe_shader.cxx)
 {"sv_shader_decl", sv_shader_decl_case},
 // Graphic3d textures (tkservice_probe_texture.cxx)
 {"sv_cubemap", sv_cubemap_case},
 {"sv_segment", sv_segment_case},
 // Graphic3d_MarkerImage (tkservice_probe_marker.cxx)
 {"sv_marker", sv_marker_case},
 // Graphic3d_Aspects (tkservice_probe_aspects.cxx)
 {"sv_aspects_dump", sv_aspects_dump_case},
 // Graphic3d_ArrayOfPrimitives (tkservice_probe_array.cxx)
 {"sv_array", sv_array_case},
 // Graphic3d_Camera (tkservice_probe_camera.cxx)
 {"sv_camera", sv_camera_case},
 {"sv_camera_fit", sv_camera_fit_case},
 {"sv_camera_interp", sv_camera_interp_case},
 // Graphic3d_ClipPlane (tkservice_probe_clip.cxx)
 {"sv_clip", sv_clip_case},
 {"sv_clip_capping", sv_clip_capping_case},
 // Graphic3d_CLight and Graphic3d_LightSet (tkservice_probe_light.cxx)
 {"sv_light", sv_light_case},
 {"sv_light_set", sv_light_set_case},
 // Graphic3d_TransformPers and Graphic3d_TransformUtils (tkservice_probe_tpers.cxx)
 {"sv_tpers", sv_tpers_case},
 {"sv_tutils", sv_tutils_case},
 // Graphic3d_RenderingParams, Graphic3d_ZLayerSettings, Graphic3d_PresentationAttributes (tkservice_probe_params.cxx)
 {"sv_params", sv_params_case},
 // Font_FontMgr, Font_FTFont, Font_TextFormatter (tkservice_probe_font.cxx)
 {"sv_fontmgr", sv_fontmgr_case},
 {"sv_ftfont", sv_ftfont_case},
 {"sv_textfmt", sv_textfmt_case},
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
