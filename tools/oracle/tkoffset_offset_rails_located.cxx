#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_Offset.hxx>
#include <GProp_GProps.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <iomanip>
#include <iostream>

int main() {
  auto path = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(0, 0, 5)).Edge();
  auto edge1 = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(1, 0, 5)).Edge();
  auto edge2 = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 1, 0), gp_Pnt(0, 1, 5)).Edge();
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(3, 4, 2));
  path.Move(TopLoc_Location(translation));
  edge1.Move(TopLoc_Location(translation));
  edge2.Move(TopLoc_Location(translation));
  const BRepOffset_Offset offset(path, edge1, edge2, 0.5, false, 1.e-4, GeomAbs_C1);
  const auto face = offset.Face();
  int edges = 0, curves = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    ++edges;
    TopLoc_Location loc;
    double first = 0.0, last = 0.0;
    curves += !BRep_Tool::Curve(TopoDS::Edge(it.Current()), loc, first, last).IsNull();
  }
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(face, properties);
  std::cout << std::setprecision(17) << "status " << offset.Status() << " valid "
            << BRepCheck_Analyzer(face, true).IsValid() << " edges " << edges
            << " curves " << curves << " area " << properties.Mass() << "\n";
}
