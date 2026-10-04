#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <iostream>
#include <iomanip>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void InitOnly() { Init(Message_ProgressRange()); }
  void EF() { PerformEF(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};

int main() {
  Probe filler;
  auto first = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, -1), gp_Pnt(1, 0, 1)).Edge();
  auto second = BRepBuilderAPI_MakeEdge(gp_Pnt(0, -1, -1), gp_Pnt(0, 1, 1)).Edge();
  auto third = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, -1, -1), gp_Pnt(1, 1, 1)).Edge();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), -2, 2, -2, 2).Face();
  filler.AddArgument(first);
  filler.AddArgument(second);
  filler.AddArgument(third);
  filler.AddArgument(face);
  filler.InitOnly();
  filler.EF();
  auto& ds = filler.DS();
  std::cout << std::fixed << std::setprecision(9);
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << '\n';
  for (const auto& record : ds.InterfEF()) {
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << static_cast<int>(record.CommonPart().Type()) << '\n';
  }
  for (int edge : {0, 3, 6}) {
    const auto& blocks = ds.PaveBlocks(edge);
    std::cout << "blocks " << edge << ' ' << blocks.Extent() << '\n';
    for (const auto& block : blocks) {
      int firstIndex, lastIndex;
      double firstParameter, lastParameter;
      block->Indices(firstIndex, lastIndex);
      block->Range(firstParameter, lastParameter);
      std::cout << "pb " << firstIndex << ' ' << lastIndex << ' '
                << firstParameter << ' ' << lastParameter << '\n';
    }
  }
  std::cout << "face " << ds.HasFaceInfo(9) << ' '
            << ds.FaceInfo(9).VerticesIn().Extent() << '\n';
  const int newVertexIndex = ds.NbShapes() - 1;
  const auto& newVertex = TopoDS::Vertex(ds.Shape(newVertexIndex));
  const auto point = BRep_Tool::Pnt(newVertex);
  std::cout << "v " << newVertexIndex << ' ' << point.X() << ' ' << point.Y() << ' '
            << point.Z() << ' ' << BRep_Tool::Tolerance(newVertex) << '\n';
}
