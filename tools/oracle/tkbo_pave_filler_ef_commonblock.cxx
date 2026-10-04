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
  Probe filler;
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.InitOnly();
  auto& ds = filler.DS();
  const int edgeIndex = ds.Index(edge);
  const int faceIndex = ds.Index(face);
  TopoDS_Vertex first, last;
  TopExp::Vertices(edge, first, last);
  const int firstIndex = ds.Index(first);
  const int lastIndex = ds.Index(last);
  auto& faceInfo = ds.ChangeFaceInfo(faceIndex);
  faceInfo.ChangeVerticesOn().Add(firstIndex);
  faceInfo.ChangeVerticesOn().Add(lastIndex);
  filler.EF();
  const auto& blocks = ds.PaveBlocks(edgeIndex);
  std::cout << std::fixed << std::setprecision(9);
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << '\n';
  for (const auto& record : ds.InterfEF()) {
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << static_cast<int>(record.CommonPart().Type()) << '\n';
  }
  std::cout << "face " << ds.HasFaceInfo(faceIndex) << ' '
            << ds.FaceInfo(faceIndex).PaveBlocksOn().Extent() << '\n';
  for (const auto& block : blocks) {
    std::cout << "pb " << block->OriginalEdge() << ' ' << ds.IsCommonBlock(block) << '\n';
    if (!ds.IsCommonBlock(block)) continue;
    const auto& common = ds.CommonBlock(block);
    std::cout << "cb " << common->Faces().Extent() << ' ' << common->Tolerance() << '\n';
    for (int commonFace : common->Faces()) std::cout << "cf " << commonFace << '\n';
  }
  for (int index : {firstIndex, lastIndex}) {
    const auto& vertex = TopoDS::Vertex(ds.Shape(index));
    std::cout << "v " << index << ' ' << BRep_Tool::Tolerance(vertex) << '\n';
  }
}
