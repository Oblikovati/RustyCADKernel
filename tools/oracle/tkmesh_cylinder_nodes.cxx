// Independent OCCT node/UV and triangle bit streams for every cylinder face.
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cstring>
#include <cstdint>
#include <iomanip>
#include <iostream>

namespace {
std::uint64_t dump_node_words(const Handle(Poly_Triangulation)& triangulation) {
  std::uint64_t hash=0xcbf29ce484222325ULL;
  for(int index=1;index<=triangulation->NbNodes();++index) {
    auto point=triangulation->Node(index);auto uv=triangulation->UVNode(index);
    for(double value:{point.X(),point.Y(),point.Z(),uv.X(),uv.Y()}) {
      std::uint64_t word;std::memcpy(&word,&value,sizeof(word));
      hash=(hash^word)*0x100000001b3ULL;
      std::cout<<std::hex<<std::setw(16)<<std::setfill('0')<<word<<' ';
    }
    std::cout<<'\n';
  }
  return hash;
}

std::uint64_t dump_triangle_indices(const Handle(Poly_Triangulation)& triangulation) {
  std::uint64_t hash=0xcbf29ce484222325ULL;
  for(int index=1;index<=triangulation->NbTriangles();++index) {
    int first,second,third;triangulation->Triangle(index).Get(first,second,third);
    std::cout<<"t "<<std::dec<<first<<' '<<second<<' '<<third<<'\n';
    for(int node:{first,second,third}) hash=(hash^std::uint64_t(node))*0x100000001b3ULL;
  }
  return hash;
}

void dump_cylinder_faces(const TopoDS_Shape& shape) {
  int face_index=0;
  for(TopExp_Explorer explorer(shape,TopAbs_FACE);explorer.More();explorer.Next()) {
    TopLoc_Location location;
    auto triangulation=BRep_Tool::Triangulation(TopoDS::Face(explorer.Current()),location);
    std::cout<<"face "<<std::dec<<face_index<<' '<<triangulation->NbNodes()<<' '<<triangulation->NbTriangles()<<'\n';
    auto node_hash=dump_node_words(triangulation);
    auto triangle_hash=dump_triangle_indices(triangulation);
    std::cout<<"hash "<<std::dec<<face_index<<' '<<std::hex<<node_hash<<' '<<triangle_hash<<'\n';
    ++face_index;
  }
}
}

int main() {
  auto shape=BRepPrimAPI_MakeCylinder(5.,10.).Shape();
  IMeshTools_Parameters parameters;parameters.Deflection=.5;
  BRepMesh_IncrementalMesh mesh(shape,parameters);
  dump_cylinder_faces(shape);
}
