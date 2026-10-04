#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

int main() {
  const BRepOffset_Offset empty;
  std::cout << "default " << empty.Status() << " " << empty.Face().IsNull() << " "
            << empty.InitialShape().IsNull() << "\n";
  const auto path = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(0, 0, 10)).Edge();
  const auto rail1 = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(1, 0, 10)).Edge();
  const auto rail2 = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 1, 0), gp_Pnt(0, 1, 10)).Edge();
  const BRepOffset_Offset offset(path, rail1, rail2, 0.5, false, 1.e-4, GeomAbs_C1);
  const TopoDS_Face face = offset.Face();
  int edges = 0;
  int vertices = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) ++edges;
  for (TopExp_Explorer it(face, TopAbs_VERTEX); it.More(); it.Next()) ++vertices;
  int edgeIndex = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    TopLoc_Location location;
    double first = 0.0;
    double last = 0.0;
    const auto curve = BRep_Tool::Curve(TopoDS::Edge(it.Current()), location, first, last);
    std::cout << "curve " << edgeIndex++ << " " << !curve.IsNull() << "\n";
  }
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(face, properties);
  std::cout << std::setprecision(17) << "status " << offset.Status() << " valid "
            << BRepCheck_Analyzer(face, true).IsValid() << " edges " << edges
            << " vertices " << vertices << " area " << properties.Mass() << " type "
            << BRep_Tool::Surface(face)->DynamicType()->Name() << " tol "
            << BRep_Tool::Tolerance(rail1) << " " << BRep_Tool::Tolerance(rail2) << "\n";
}
