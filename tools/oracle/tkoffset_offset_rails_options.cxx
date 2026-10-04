#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepOffset_Offset.hxx>
#include <BRep_Tool.hxx>
#include <GProp_GProps.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierCurve.hxx>
#include <NCollection_Array1.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Edge bezier_edge(double dx, double dy) {
  NCollection_Array1<gp_Pnt> poles(1, 3);
  poles.SetValue(1, gp_Pnt(dx, dy, 0));
  poles.SetValue(2, gp_Pnt(dx, dy + 3, 5));
  poles.SetValue(3, gp_Pnt(dx, dy, 10));
  return BRepBuilderAPI_MakeEdge(new Geom_BezierCurve(poles)).Edge();
}

static void run(const char* label, bool polynomial, double tolerance, GeomAbs_Shape continuity) {
  const auto path = bezier_edge(0, 0);
  const auto rail1 = bezier_edge(1, 0);
  const auto rail2 = bezier_edge(0, 1);
  const BRepOffset_Offset offset(path, rail1, rail2, 0.5, polynomial, tolerance, continuity);
  const auto face = offset.Face();
  int edges = 0, curves = 0;
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    ++edges;
    TopLoc_Location location;
    double first = 0.0, last = 0.0;
    curves += !BRep_Tool::Curve(TopoDS::Edge(it.Current()), location, first, last).IsNull();
  }
  GProp_GProps properties;
  BRepGProp::SurfaceProperties(face, properties);
  const auto spline = Handle(Geom_BSplineSurface)::DownCast(BRep_Tool::Surface(face));
  std::cout << std::setprecision(17) << label << " status " << offset.Status()
            << " valid " << BRepCheck_Analyzer(face, true).IsValid() << " edges " << edges
            << " curves " << curves << " type " << BRep_Tool::Surface(face)->DynamicType()->Name()
            << " poles " << spline->NbUPoles() << " " << spline->NbVPoles()
            << " degree " << spline->UDegree() << " " << spline->VDegree() << " area " << properties.Mass() << " tol " << BRep_Tool::Tolerance(rail1)
            << " " << BRep_Tool::Tolerance(rail2) << "\n";
}

int main() {
  run("default", false, 1.e-4, GeomAbs_C1);
  run("polynomial", true, 1.e-4, GeomAbs_C1);
  run("loose", false, 1.0, GeomAbs_C1);
  run("tight", false, 1.e-12, GeomAbs_C1);
  run("continuity_c0", false, 1.e-4, GeomAbs_C0);
  run("continuity_c2", false, 1.e-4, GeomAbs_C2);
}
