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

static void snapshot(Probe& filler, int mode, int pass, int edgeIndex, int faceIndex) {
  auto& ds = filler.DS();
  std::cout << "case " << mode << ' ' << pass << ' ' << ds.NbShapes() << ' '
            << ds.InterfEF().Length() << '\n';
  for (const auto& record : ds.InterfEF()) {
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << static_cast<int>(record.CommonPart().Type()) << '\n';
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
  std::cout << "face " << ds.HasFaceInfo(faceIndex) << ' '
            << ds.FaceInfo(faceIndex).VerticesIn().Extent() << '\n';
}

static void run(int mode, bool nondestructive, double fuzzy) {
  Probe filler;
  auto edge = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, -1), gp_Pnt(1, 0, 1)).Edge();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), -2, 2, -2, 2).Face();
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.SetFuzzyValue(fuzzy);
  filler.SetNonDestructive(nondestructive);
  filler.InitOnly();
  const int edgeIndex = filler.DS().Index(edge);
  const int faceIndex = filler.DS().Index(face);
  for (int pass = 0; pass < 2; ++pass) {
    filler.EF();
    snapshot(filler, mode, pass, edgeIndex, faceIndex);
  }
}

int main() {
  std::cout << std::fixed << std::setprecision(9);
  run(0, false, 1.0e-6);
  run(1, true, 1.0e-6);
  run(2, false, 0.0);
}
