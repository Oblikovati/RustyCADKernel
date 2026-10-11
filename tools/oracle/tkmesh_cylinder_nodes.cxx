// Independent OCCT node/UV bit stream for the cylinder seam regression.
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <cstring>
#include <cstdint>
#include <iomanip>
#include <iostream>
int main() {
  auto shape=BRepPrimAPI_MakeCylinder(5.,10.).Shape();
  IMeshTools_Parameters parameters; parameters.Deflection=.5;
  BRepMesh_IncrementalMesh mesh(shape,parameters);
  TopExp_Explorer explorer(shape,TopAbs_FACE);
  TopLoc_Location location;
  auto triangulation=BRep_Tool::Triangulation(TopoDS::Face(explorer.Current()),location);
  std::uint64_t hash=0xcbf29ce484222325ULL;
  std::cout << triangulation->NbNodes() << ' ' << triangulation->NbTriangles() << '\n';
  for(int index=1;index<=triangulation->NbNodes();++index) {
    auto point=triangulation->Node(index);auto uv=triangulation->UVNode(index);
    for(double value:{point.X(),point.Y(),point.Z(),uv.X(),uv.Y()}) {
      std::uint64_t word;std::memcpy(&word,&value,sizeof(word));
      hash=(hash^word)*0x100000001b3ULL;
      std::cout<<std::hex<<std::setw(16)<<std::setfill('0')<<word<<' ';
    }
    std::cout<<'\n';
  }
  std::cout << "hash " << std::hex << hash << '\n';
}
