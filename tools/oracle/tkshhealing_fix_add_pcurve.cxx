// FixAddPCurve oracle for pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRepLib_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <NCollection_Array1.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeFix_Edge.hxx>
#include <ShapeExtend_Status.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <gp_Vec2d.hxx>
#include <iomanip>
#include <iostream>

TopoDS_Edge generator(double x)
{
  return BRepLib_MakeEdge(gp_Pnt(x, 0, 0), gp_Pnt(x, 0, 1)).Edge();
}

void setPCurve(const TopoDS_Edge& edge,
               const occ::handle<Geom2d_Curve>& curve,
               const occ::handle<Geom_Surface>& surface,
               const TopLoc_Location& location)
{
  BRep_Builder builder;
  builder.UpdateEdge(edge, curve, surface, location, 1.e-7);
  builder.Range(edge, surface, location, 0., 1.);
}

int curveKind(const occ::handle<Geom2d_Curve>& curve)
{
  if (!occ::down_cast<Geom2d_Line>(curve).IsNull()) return 1;
  if (!occ::down_cast<Geom2d_BSplineCurve>(curve).IsNull()) return 2;
  return 0;
}

void record(const char* name,
            ShapeFix_Edge& fixer,
            const TopoDS_Edge& edge,
            const occ::handle<Geom_Surface>& surface,
            const TopLoc_Location& location,
            bool changed)
{
  ShapeAnalysis_Edge analyzer;
  std::cout << name << ' ' << changed << ' ' << analyzer.HasPCurve(edge, surface, location)
            << ' ' << analyzer.IsSeam(edge, surface, location);
  for (int flag = ShapeExtend_DONE1; flag <= ShapeExtend_FAIL8; ++flag)
    std::cout << ' ' << fixer.Status(static_cast<ShapeExtend_Status>(flag));
  for (int index : {1, 2})
  {
    occ::handle<Geom2d_Curve> curve;
    occ::handle<Geom_Surface> foundSurface;
    TopLoc_Location foundLocation;
    double first = 0., last = 0.;
    BRep_Tool::CurveOnSurface(edge, curve, foundSurface, foundLocation, first, last, index);
    std::cout << ' ' << curveKind(curve);
    if (curve.IsNull())
    {
      std::cout << " 0 0 0 0 0 0";
      continue;
    }
    gp_Pnt2d start = curve->Value(first), end = curve->Value(last);
    std::cout << ' ' << first << ' ' << last << ' ' << start.X() << ' ' << start.Y()
              << ' ' << end.X() << ' ' << end.Y();
  }
  std::cout << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  occ::handle<Geom_Surface> cylinder = new Geom_CylindricalSurface(gp_Ax3(), 1.);
  TopLoc_Location identity;
  {
    TopoDS_Edge edge = generator(1.);
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, cylinder, 1.e-7);
    ShapeFix_Edge fixer;
    record("cylinder-project", fixer, edge, cylinder, identity,
           fixer.FixAddPCurve(edge, face, false, 1.e-7));
  }
  {
    TopoDS_Edge edge = generator(1.);
    setPCurve(edge, new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(0, 1)), cylinder, identity);
    ShapeFix_Edge fixer;
    record("cylinder-second-seam", fixer, edge, cylinder, identity,
           fixer.FixAddPCurve(edge, cylinder, identity, true, 1.e-7));
  }
  {
    TopoDS_Edge edge = generator(1.);
    occ::handle<Geom_Surface> plane = new Geom_Plane(gp_Pln());
    ShapeFix_Edge fixer;
    record("plane-early-return", fixer, edge, plane, identity,
           fixer.FixAddPCurve(edge, plane, identity, false, 1.e-7));
  }
  {
    TopoDS_Edge edge;
    BRep_Builder().MakeEdge(edge);
    ShapeFix_Edge fixer;
    record("missing-3d-curve", fixer, edge, cylinder, identity,
           fixer.FixAddPCurve(edge, cylinder, identity, false, 1.e-7));
  }
  {
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(3, 0, 0));
    TopLoc_Location location(transform);
    TopoDS_Edge edge = generator(4.);
    ShapeFix_Edge fixer;
    record("located-cylinder", fixer, edge, cylinder, location,
           fixer.FixAddPCurve(edge, cylinder, location, false, 1.e-7));
  }
  occ::handle<Geom_Surface> torus = new Geom_ToroidalSurface(gp_Ax3(), 5., 2.);
  {
    TopoDS_Edge edge = generator(7.);
    setPCurve(edge, new Geom2d_Line(gp_Pnt2d(0, .2), gp_Dir2d(0, 1)), torus, identity);
    ShapeFix_Edge fixer;
    record("torus-line-seam", fixer, edge, torus, identity,
           fixer.FixAddPCurve(edge, torus, identity, true, 1.e-7));
  }
  {
    NCollection_Array1<gp_Pnt2d> poles(1, 2);
    poles(1) = gp_Pnt2d(0, .2); poles(2) = gp_Pnt2d(0, 1.);
    NCollection_Array1<double> knots(1, 2);
    knots(1) = 0.; knots(2) = 1.;
    NCollection_Array1<int> mults(1, 2);
    mults(1) = 2; mults(2) = 2;
    occ::handle<Geom2d_Curve> spline = new Geom2d_BSplineCurve(poles, knots, mults, 1, false);
    TopoDS_Edge edge = generator(7.);
    setPCurve(edge, spline, torus, identity);
    ShapeFix_Edge fixer;
    record("torus-spline-seam", fixer, edge, torus, identity,
           fixer.FixAddPCurve(edge, torus, identity, true, 1.e-7));
  }
}
