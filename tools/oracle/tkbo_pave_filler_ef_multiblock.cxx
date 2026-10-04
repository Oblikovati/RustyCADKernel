#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void InitOnly() { Init(Message_ProgressRange()); }
  void VE() { PerformVE(Message_ProgressRange()); }
  void EF() { PerformEF(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};

int main() {
  Probe filler;
  auto edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(0, 0, 2)).Edge();
  auto point = BRepBuilderAPI_MakeVertex(gp_Pnt(0, 0, 0.5)).Vertex();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0, 0, 1), gp_Dir(0, 0, 1)), -1, 1, -1, 1).Face();
  filler.SetFuzzyValue(0.0);
  filler.AddArgument(point);
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.InitOnly();
  filler.VE();
  filler.EF();
  auto& ds = filler.DS();
  const int edgeIndex = ds.Index(edge);
  std::cout << std::fixed << std::setprecision(12);
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfVE().Length() << ' ' << ds.InterfEF().Length() << '\n';
  for (const auto& pave : ds.PaveBlocks(edgeIndex)) {
    int first, last;
    double firstParameter, lastParameter;
    pave->Indices(first, last);
    pave->Range(firstParameter, lastParameter);
    std::cout << "pb " << edgeIndex << ' ' << first << ' ' << last << ' '
              << firstParameter << ' ' << lastParameter << '\n';
  }
  for (const auto& record : ds.InterfVE())
    std::cout << "ve " << record.Index1() << ' ' << record.Index2() << ' ' << record.Parameter() << '\n';
  for (const auto& record : ds.InterfEF())
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' ' << record.IndexNew() << '\n';
}
