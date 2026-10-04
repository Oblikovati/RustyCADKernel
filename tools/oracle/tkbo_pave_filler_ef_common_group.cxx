#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_CommonBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void InitOnly() { Init(Message_ProgressRange()); }
  void VF() { PerformVF(Message_ProgressRange()); }
  void EE() { PerformEE(Message_ProgressRange()); }
  void EF() { PerformEF(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};

int main() {
  Probe filler;
  auto firstVertex = BRepBuilderAPI_MakeVertex(gp_Pnt(-1, 0, 0)).Vertex();
  auto lastVertex = BRepBuilderAPI_MakeVertex(gp_Pnt(1, 0, 0)).Vertex();
  auto first = BRepBuilderAPI_MakeEdge(firstVertex, lastVertex).Edge();
  auto second = BRepBuilderAPI_MakeEdge(firstVertex, lastVertex).Edge();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(), -2, 2, -2, 2).Face();
  filler.SetFuzzyValue(1.0e-6);
  filler.AddArgument(first);
  filler.AddArgument(second);
  filler.AddArgument(face);
  filler.InitOnly();
  filler.VF();
  filler.EE();
  filler.EF();
  auto& ds = filler.DS();
  std::cout << std::fixed << std::setprecision(12);
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEE().Length() << ' '
            << ds.InterfEF().Length() << '\n';
  for (int edge : {ds.Index(first), ds.Index(second)}) {
    for (const auto& block : ds.PaveBlocks(edge)) {
      int firstIndex, lastIndex;
      double firstParameter, lastParameter;
      block->Indices(firstIndex, lastIndex);
      block->Range(firstParameter, lastParameter);
      std::cout << "pb " << edge << ' ' << firstIndex << ' ' << lastIndex << ' '
                << firstParameter << ' ' << lastParameter << ' ' << ds.IsCommonBlock(block) << '\n';
      if (!ds.IsCommonBlock(block)) continue;
      const auto& common = ds.CommonBlock(block);
      std::cout << "cb " << common->PaveBlocks().Extent() << ' ' << common->Faces().Extent() << ' '
                << common->Tolerance() << '\n';
      for (const auto& member : common->PaveBlocks())
        std::cout << "member " << member->OriginalEdge() << '\n';
      for (int commonFace : common->Faces())
        std::cout << "face " << commonFace << '\n';
    }
  }
}
