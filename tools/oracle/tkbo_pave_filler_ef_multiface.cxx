#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_CommonBlock.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void InitOnly() { Init(Message_ProgressRange()); }
  void EF() { PerformEF(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};

int main() {
  auto edge = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, 0), gp_Pnt(1, 0, 0)).Edge();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), -2, 2, -2, 2).Face();
  BRep_Builder().UpdateFace(face, 0.001);
  auto face2 = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), -2, 2, -2, 2).Face();
  BRep_Builder().UpdateFace(face2, 0.0005);
  Probe filler;
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.AddArgument(face2);
  filler.InitOnly();
  auto& ds = filler.DS();
  const int edgeIndex = ds.Index(edge);
  const int faceIndex = ds.Index(face);
  const int faceIndex2 = ds.Index(face2);
  TopoDS_Vertex first, last;
  TopExp::Vertices(edge, first, last);
  const int firstIndex = ds.Index(first);
  const int lastIndex = ds.Index(last);
  for (int index : {faceIndex, faceIndex2}) {
    auto& faceInfo = ds.ChangeFaceInfo(index);
    faceInfo.ChangeVerticesOn().Add(firstIndex);
    faceInfo.ChangeVerticesOn().Add(lastIndex);
  }
  filler.EF();
  const auto& blocks = ds.PaveBlocks(edgeIndex);
  std::cout << std::fixed << std::setprecision(9);
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << '\n';
  std::cout << "faces " << faceIndex << ' ' << faceIndex2 << '\n';
  for (int index : {faceIndex, faceIndex2}) {
    std::cout << "face " << index << ' ' << ds.HasFaceInfo(index) << ' '
              << ds.FaceInfo(index).PaveBlocksOn().Extent() << '\n';
  }
  for (const auto& block : blocks) {
    if (!ds.IsCommonBlock(block)) continue;
    const auto& common = ds.CommonBlock(block);
    std::cout << "pb " << block->OriginalEdge() << '\n';
    std::cout << "cb " << common->Faces().Extent() << ' ' << common->Tolerance() << '\n';
    for (int commonFace : common->Faces()) std::cout << "cf " << commonFace << '\n';
  }
  for (int index : {firstIndex, lastIndex}) {
    const auto& vertex = TopoDS::Vertex(ds.Shape(index));
    std::cout << "v " << index << ' ' << BRep_Tool::Tolerance(vertex) << '\n';
  }
}
