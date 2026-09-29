// Exact oracle for the TKBRep packages at pinned OCCT 3d097a0.
// Each input line is `operation numbers...`; tools/verify_tkbrep_numeric.py compares the
// output with crates/tkbrep/examples/numeric_probe.rs token for token. Each package adds
// its own tkbrep_probe_<package>.cxx, included below, and its rows in probe_cases.
#include <algorithm>
#include <map>
#include <vector>
#include <iostream>
#include <sstream>
#include <cctype>
#include <cstring>
#include <cstdint>
#include <Standard_Failure.hxx>
#include <Standard_HashUtils.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <TopLoc_Datum3D.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_CompSolid.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Iterator.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include "tkbrep_probe_common.cxx"
#include "tkbrep_probe_graph.cxx"
#include "tkbrep_probe_topods.cxx"
#include "tkbrep_probe_topexp.cxx"
#include "tkbrep_probe_toptools.cxx"
#include "tkbrep_probe_brep.cxx"
#include "tkbrep_probe_brep_report.cxx"
#include "tkbrep_probe_bintools.cxx"
#include "tkbrep_probe_bintools_geometry.cxx"
#include "tkbrep_probe_bintools_shape.cxx"
#include "tkbrep_probe_breptools.cxx"
#include "tkbrep_probe_breptools_modif.cxx"
#include "tkbrep_probe_breptools_reshape.cxx"
#include "tkbrep_probe_brepadaptor.cxx"
#include <Standard_GUID.hxx>
#include "tkbrep_probe_brepgraph.cxx"
#include "tkbrep_probe_brepgraph_views.cxx"
#include "tkbrep_probe_brepgraph_walk.cxx"
#include "tkbrep_probe_brepgraph_explore.cxx"
#include "tkbrep_probe_brepgraph_tool.cxx"
#include "tkbrep_probe_brepgraph_layers.cxx"
#include "tkbrep_probe_brepgraph_cache.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // TopoDS (tkbrep_probe_topods.cxx)
 {"br_topods", br_topods_case},
 {"br_dump", br_dump_case},
 // TopExp (tkbrep_probe_topexp.cxx)
 {"br_explore", br_explore_case},
 {"br_maps", br_maps_case},
 {"br_vertices", br_vertices_case},
 // TopTools (tkbrep_probe_toptools.cxx)
 {"br_shapeset", br_shapeset_case},
 {"br_setread", br_setread_case},
 {"br_locset", br_locset_case},
 {"br_locread", br_locread_case},
 // BRep (tkbrep_probe_brep.cxx, tkbrep_probe_brep_report.cxx)
 {"br_brep", br_brep_case},
 {"br_brep_dump", br_brep_dump_case},
 // BinTools (tkbrep_probe_bintools*.cxx)
 {"br_bin_scalars", br_bin_scalars_case},
 {"br_bin_ostream", br_bin_ostream_case},
 {"br_bin_istream", br_bin_istream_case},
 {"br_bin_geometry", br_bin_geometry_case},
 {"br_bin_shape", br_bin_shape_case},
 {"br_bin_read", br_bin_read_case},
 {"br_bin_truncate", br_bin_truncate_case},
 // BRepTools (tkbrep_probe_breptools*.cxx)
 {"br_tools_text", br_tools_text_case},
 {"br_tools_query", br_tools_query_case},
 {"br_tools_mutate", br_tools_mutate_case},
 {"br_tools_read", br_tools_read_case},
 {"br_tools_modif", br_tools_modif_case},
 {"br_tools_wire", br_tools_wire_case},
 {"br_tools_reshape", br_tools_reshape_case},
 {"br_tools_history", br_tools_history_case},
 {"br_tools_subst", br_tools_subst_case},
 {"br_tools_quilt", br_tools_quilt_case},
 {"br_tools_purge", br_tools_purge_case},
 // BRepAdaptor and BRepLProp (tkbrep_probe_brepadaptor.cxx)
 {"br_adaptor", br_adaptor_case},
 // BRepGraph and BRepGraphInc (tkbrep_probe_brepgraph*.cxx)
 {"br_bgraph", br_bgraph_case},
 {"br_bgraph_walk", br_bgraph_walk_case},
 {"br_blayer", br_blayer_case},
 {"br_bcache", br_bcache_case},
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
