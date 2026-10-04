#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRep_Tool.hxx>
#include <Geom_BezierSurface.hxx>
#include <NCollection_Array2.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void InitOnly() { Init(Message_ProgressRange()); }
  void EF() { PerformEF(Message_ProgressRange()); }
  BOPDS_DS& DS() { return *myDS; }
};

int main() {
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles(1, 1) = gp_Pnt(0, 0, 0);
  poles(1, 2) = gp_Pnt(0, 1, 0);
  poles(2, 1) = gp_Pnt(1, 0, 0);
  poles(2, 2) = gp_Pnt(1, 1, 0);
  Handle(Geom_BezierSurface) surface = new Geom_BezierSurface(poles);
  auto edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0.5, 0.5, -1), gp_Pnt(0.5, 0.5, 1)).Edge();
  auto face = BRepBuilderAPI_MakeFace(surface, 0, 1, 0, 1, 1.0e-7).Face();
  Probe filler;
  filler.AddArgument(edge);
  filler.AddArgument(face);
  filler.InitOnly();
  filler.EF();
  auto& ds = filler.DS();
  std::cout << "after " << ds.NbShapes() << ' ' << ds.InterfEF().Length() << '\n';
  for (const auto& record : ds.InterfEF()) {
    std::cout << "ef " << record.Index1() << ' ' << record.Index2() << ' '
              << record.IndexNew() << ' ' << static_cast<int>(record.CommonPart().Type()) << '\n';
  }
  for (int index = 0; index < ds.NbShapes(); ++index) {
    if (ds.ShapeInfo(index).ShapeType() != TopAbs_VERTEX) continue;
    const auto& vertex = TopoDS::Vertex(ds.Shape(index));
    const auto point = BRep_Tool::Pnt(vertex);
    std::cout << std::fixed << std::setprecision(9) << "v " << point.X() << ' '
              << point.Y() << ' ' << point.Z() << '\n';
  }
}
