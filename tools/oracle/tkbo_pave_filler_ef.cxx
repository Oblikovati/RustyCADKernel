#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_PaveBlock.hxx>
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
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, -1), gp_Pnt(1, 0, 1));
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), -2, 2, -2, 2);
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.SetFuzzyValue(1.0e-6);
  filler.InitOnly();
  const int edgeIndex = filler.DS().Index(edge);
  const int faceIndex = filler.DS().Index(face);
  std::cout << std::fixed << std::setprecision(9);
  std::cout << "before " << filler.DS().NbShapes() << ' ' << edgeIndex << ' ' << faceIndex << '\n';
  filler.EF();
  auto& ds = filler.DS();
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << ' '
            << ds.InterfVE().Length() << '\n';
  for (const auto& record : ds.InterfEF()) {
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' ' << record.IndexNew()
              << ' ' << static_cast<int>(record.CommonPart().Type()) << ' '
              << record.CommonPart().VertexParameter1() << '\n';
  }
  const auto& blocks = ds.PaveBlocks(edgeIndex);
  std::cout << "blocks " << blocks.Extent() << '\n';
  for (const auto& block : blocks) {
    int first, last;
    double firstParameter, lastParameter;
    block->Indices(first, last);
    block->Range(firstParameter, lastParameter);
    std::cout << "pb " << first << ' ' << last << ' ' << firstParameter << ' ' << lastParameter << '\n';
  }
  const auto& info = ds.FaceInfo(faceIndex);
  std::cout << "face " << ds.HasFaceInfo(faceIndex) << ' ' << info.VerticesIn().Extent() << '\n';
  const int newVertexIndex = ds.NbShapes() - 1;
  const auto& newVertex = TopoDS::Vertex(ds.Shape(newVertexIndex));
  const auto point = BRep_Tool::Pnt(newVertex);
  std::cout << "v " << newVertexIndex << ' ' << point.X() << ' ' << point.Y() << ' '
            << point.Z() << ' ' << BRep_Tool::Tolerance(newVertex) << '\n';
}
