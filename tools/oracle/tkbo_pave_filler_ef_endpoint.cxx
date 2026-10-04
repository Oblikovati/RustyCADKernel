#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <TopoDS.hxx>
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
  std::cout << std::fixed << std::setprecision(9);
  Probe filler;
  auto edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 1)).Edge();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), -2, 2, -2, 2).Face();
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.SetFuzzyValue(1.0e-6);
  filler.InitOnly();
  const int edgeIndex = filler.DS().Index(edge);
  const int faceIndex = filler.DS().Index(face);
  filler.EF();
  auto& ds = filler.DS();
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << ' '
            << ds.InterfVF().Length() << '\n';
  for (const auto& record : ds.InterfVF()) {
    double u, v;
    record.UV(u, v);
    std::cout << "vf " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << u << ' ' << v << '\n';
  }
  const auto& blocks = ds.PaveBlocks(edgeIndex);
  std::cout << "blocks " << blocks.Extent() << '\n';
  for (const auto& block : blocks) {
    int first, last;
    double firstParameter, lastParameter;
    block->Indices(first, last);
    block->Range(firstParameter, lastParameter);
    std::cout << "pb " << first << ' ' << last << ' ' << firstParameter << ' '
              << lastParameter << '\n';
  }
  const auto& info = ds.FaceInfo(faceIndex);
  std::cout << "face " << ds.HasFaceInfo(faceIndex) << ' ' << info.VerticesIn().Extent() << '\n';
}
