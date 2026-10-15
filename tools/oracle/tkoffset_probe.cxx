// Exact numeric oracle for the TKOffset packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkoffset_numeric.py compares the
// output with crates/tkoffset/examples/numeric_probe.rs bit for bit. Each package adds its
// own tkoffset_probe_<package>.cxx, included below, and its rows in probe_cases under its
// own package comment.
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
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
// The TKGeomAlgo reader and bit printers (ga_*), shared so every toolkit prints the same
// token stream; the Rust twin includes crates/tkgeomalgo/examples/ga_probe/common.rs.
#include "tkgeomalgo_probe_common.cxx"
// The TKTopAlgo shape digest (tp_shape) and geometry readers of the BRepLib probes; the
// Rust twin includes crates/tktopalgo/examples/tp_probe/brep_lib_common.rs.
#include "tktopalgo_probe_breplib_common.cxx"
// The TKPrim shape digest with face frames (pr_shape); the Rust twin includes
// crates/tkprim/examples/pr_probe/shape.rs.
#include "tkprim_probe_shape.cxx"
// The TKTopAlgo face and shape program readers (tp_surface, tp_shapes); the Rust twin
// includes crates/tktopalgo/examples/tp_probe/face.rs, shapes.rs and shapes_geometry.rs.
#include "tktopalgo_probe_face.cxx"
#include "tktopalgo_probe_shapes.cxx"
#include "tkoffset_probe_brepoffsetapi_make_offset.cxx"
#include "tkoffset_probe_brepoffsetapi_make_pipe_shell.cxx"
#include "tkoffset_probe_brepoffsetapi_thru_sections.cxx"
#include "tkoffset_probe_brepoffsetapi_wrappers.cxx"
#include "tkoffset_probe_brepoffset_tool.cxx"
#include "tkoffset_probe_brepoffset_analyse.cxx"
#include "tkoffset_probe_brepoffset_offset.cxx"
#include "tkoffset_probe_brepoffset_inter3d.cxx"
#include "tkoffset_probe_brepoffset_make_offset.cxx"
#include "tkoffset_probe_brepoffsetapi_middle_path.cxx"
#include "tkoffset_probe_brepoffsetapi_draft_angle.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // BRepOffsetAPI (tkoffset_probe_brepoffsetapi_make_offset.cxx,
 // tkoffset_probe_brepoffsetapi_make_pipe_shell.cxx, tkoffset_probe_brepoffsetapi_thru_sections.cxx)
 {"oa_make_offset", of_oa_make_offset_case},
 {"oa_pipe_shell", of_oa_pipe_shell_case},
 {"oa_thru_sections", of_oa_thru_sections_case},
 // The thin BRepOffsetAPI wrappers (tkoffset_probe_brepoffsetapi_wrappers.cxx)
 {"oa_pipe", of_oa_pipe_case},
 {"oa_evolved", of_oa_evolved_case},
 {"oa_draft", of_oa_draft_case},
 {"oa_filling", of_oa_filling_case},
 {"oa_normal_projection", of_oa_normal_projection_case},
 {"oa_contigous", of_oa_contigous_case},
 // BRepOffset_Tool (tkoffset_probe_brepoffset_tool.cxx)
 {"ot_inter3d", of_ot_inter3d_case},
 {"ot_enlarge", of_ot_enlarge_case},
 {"ot_inter2d", of_ot_inter2d_case},
 {"ot_queries", of_ot_queries_case},
 {"ot_deboucle", of_ot_deboucle_case},
 // BRepOffset_Analyse (tkoffset_probe_brepoffset_analyse.cxx)
 {"oan_analyse", of_oan_analyse_case},
 // BRepOffset_Offset (tkoffset_probe_brepoffset_offset.cxx)
 {"oo_faces", of_oo_faces_case},
 {"oo_pipes", of_oo_pipes_case},
 {"oo_tubes", of_oo_tubes_case},
 // BRepOffset_Inter3d (tkoffset_probe_brepoffset_inter3d.cxx)
 {"oi_inter", of_oi_inter_case},
 // BRepOffset_MakeOffset (tkoffset_probe_brepoffset_make_offset.cxx)
 {"om_offset", of_om_offset_case},
 {"om_simple", of_om_simple_case},
 {"om_face_offsets", of_om_face_offsets_case},
 // BRepOffsetAPI_MiddlePath (tkoffset_probe_brepoffsetapi_middle_path.cxx)
 {"oa_middle", of_oa_middle_case},
 {"oa_middle_prim", of_oa_middle_prim_case},
 // BRepOffsetAPI_DraftAngle and Draft (tkoffset_probe_brepoffsetapi_draft_angle.cxx)
 {"oa_draft_angle", of_oa_draft_angle_case},
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
