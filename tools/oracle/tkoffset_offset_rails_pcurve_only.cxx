#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge pcurve_edge(const gp_Pln& plane) {
  const auto surface = new Geom_Plane(plane);
  const auto pcurve = new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0));
  return BRepBuilderAPI_MakeEdge(pcurve, surface, 0, 5).Edge();
}

int main() {
  const auto path = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(0, 0, 5)).Edge();
  const auto edge1 = pcurve_edge(gp_Pln(gp_Ax3(gp_Pnt(1, 0, 0), gp_Dir(0, 1, 0), gp_Dir(0, 0, 1))));
  const auto edge2 = pcurve_edge(gp_Pln(gp_Ax3(gp_Pnt(0, 1, 0), gp_Dir(1, 0, 0), gp_Dir(0, 0, 1))));
  TopLoc_Location location;
  double first = 0.0, last = 0.0;
  std::cout << "source_curve_null " << BRep_Tool::Curve(edge1, location, first, last).IsNull() << "\n";
  occ::handle<Geom2d_Curve> pc; occ::handle<Geom_Surface> surf;
  BRep_Tool::CurveOnSurface(edge1, pc, surf, location, first, last);
  std::cout << "pcurve_range " << first << " " << last << "\n" << std::flush;
  const BRepOffset_Offset offset(path, edge1, edge2, 0.5, false, 1.e-4, GeomAbs_C1);
  const auto face = offset.Face();
  int edges = 0, curves = 0, vertices = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    ++edges;
    auto curve = BRep_Tool::Curve(TopoDS::Edge(it.Current()), location, first, last);
    curves += !curve.IsNull();
  }
  for (TopExp_Explorer it(face, TopAbs_VERTEX); it.More(); it.Next()) ++vertices;
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(face, properties);
  std::cout << std::setprecision(17) << "status " << offset.Status() << " valid "
            << BRepCheck_Analyzer(face, true).IsValid() << " edges " << edges
            << " curves " << curves << " vertices " << vertices
            << " area " << properties.Mass() << " source_edges "
            << BRep_Tool::Tolerance(edge1) << " " << BRep_Tool::Tolerance(edge2) << "\n";
}
