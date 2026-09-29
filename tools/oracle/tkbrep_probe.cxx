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
