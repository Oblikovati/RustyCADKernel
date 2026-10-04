#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <iostream>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void InitOnly() { Init(Message_ProgressRange()); }
  void EF() { PerformEF(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};

int main() {
  Probe filler;
  auto edge = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, 0), gp_Pnt(1, 0, 0)).Edge();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), -2, 2, -2, 2).Face();
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.InitOnly();
  auto& ds = filler.DS();
  const int edgeIndex = ds.Index(edge);
  const int faceIndex = ds.Index(face);
  filler.EF();
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << '\n';
  for (const auto& record : ds.InterfEF()) {
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << static_cast<int>(record.CommonPart().Type()) << '\n';
  }
  const auto& info = ds.FaceInfo(faceIndex);
  std::cout << "face " << ds.HasFaceInfo(faceIndex) << ' ' << info.PaveBlocksOn().Extent()
            << ' ' << info.VerticesIn().Extent() << '\n';
  std::cout << "edge-blocks " << ds.PaveBlocks(edgeIndex).Extent() << '\n';
}
