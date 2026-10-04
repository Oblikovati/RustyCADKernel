#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
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
  Probe filler;
  auto first = BRepBuilderAPI_MakeEdge(gp_Pnt(-1, 0, -1), gp_Pnt(1, 0, 1)).Edge();
  auto second = BRepBuilderAPI_MakeEdge(gp_Pnt(-0.99959, 0.00041, -1), gp_Pnt(1.00041, 0.00041, 1)).Edge();
  auto third = BRepBuilderAPI_MakeEdge(gp_Pnt(-0.99918, 0.00082, -1), gp_Pnt(1.00082, 0.00082, 1)).Edge();
  auto face = BRepBuilderAPI_MakeFace(gp_Pln(), -2, 2, -2, 2).Face();
  filler.SetFuzzyValue(0.0004);
  filler.AddArgument(first);
  filler.AddArgument(second);
  filler.AddArgument(third);
  filler.AddArgument(face);
  filler.InitOnly();
  filler.EF();
  auto& ds = filler.DS();
  std::cout << std::fixed << std::setprecision(12);
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << ' ' << ds.InterfVE().Length() << '\n';
  for (const auto& record : ds.InterfEF())
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' ' << record.IndexNew() << '\n';
  for (int edge : {0, 3, 6}) {
    for (const auto& block : ds.PaveBlocks(edge)) {
      int firstIndex, lastIndex;
      double firstParameter, lastParameter;
      block->Indices(firstIndex, lastIndex);
      block->Range(firstParameter, lastParameter);
      std::cout << "pb " << edge << ' ' << firstIndex << ' ' << lastIndex << ' '
                << firstParameter << ' ' << lastParameter << '\n';
    }
  }
  for (int i = 0; i < ds.NbShapes(); ++i) {
    if (!ds.IsNewShape(i) || ds.ShapeInfo(i).ShapeType() != TopAbs_VERTEX) continue;
    const auto& vertex = TopoDS::Vertex(ds.Shape(i));
    const auto point = BRep_Tool::Pnt(vertex);
    std::cout << "v " << i << ' ' << point.X() << ' ' << point.Y() << ' ' << point.Z() << ' '
              << BRep_Tool::Tolerance(vertex) << '\n';
  }
}
