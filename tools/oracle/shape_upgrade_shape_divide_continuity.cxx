#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_Plane.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeUpgrade_ShapeDivideContinuity.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Pln.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Wire.hxx>

#include <iostream>

static occ::handle<Geom_BSplineCurve> c0Curve()
{
  NCollection_Array1<gp_Pnt> poles(1, 5);
  poles(1) = gp_Pnt(0.0, 0.0, 0.0);
  poles(2) = gp_Pnt(0.5, 1.0, 0.0);
  poles(3) = gp_Pnt(1.0, 1.0, 0.0);
  poles(4) = gp_Pnt(1.5, 0.0, 0.0);
  poles(5) = gp_Pnt(2.0, 0.0, 0.0);
  NCollection_Array1<double> knots(1, 3);
  knots(1) = 0.0;
  knots(2) = 0.5;
  knots(3) = 1.0;
  NCollection_Array1<int> mults(1, 3);
  mults(1) = 3;
  mults(2) = 2;
  mults(3) = 3;
  return new Geom_BSplineCurve(poles, knots, mults, 2, false);
}

static TopoDS_Shape c0Wire()
{
  const occ::handle<Geom_BSplineCurve> curve = c0Curve();
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(curve).Edge();
  return BRepBuilderAPI_MakeWire(edge).Shape();
}

static NCollection_Array2<gp_Pnt> c0SurfacePoles()
{
  NCollection_Array2<gp_Pnt> poles(1, 5, 1, 5);
  for (int u = 1; u <= 5; ++u)
  {
    for (int v = 1; v <= 5; ++v)
    {
      const double z = static_cast<double>(std::max(u - 3, 0) + std::max(v - 3, 0));
      poles(u, v) = gp_Pnt((u - 1) / 4.0, (v - 1) / 4.0, z);
    }
  }
  return poles;
}

static TopoDS_Shape c0Face()
{
  const NCollection_Array2<gp_Pnt> poles = c0SurfacePoles();
  NCollection_Array1<double> knots(1, 3);
  knots(1) = 0.0;
  knots(2) = 0.5;
  knots(3) = 1.0;
  NCollection_Array1<int> mults(1, 3);
  mults(1) = 3;
  mults(2) = 2;
  mults(3) = 3;
  occ::handle<Geom_BSplineSurface> surface =
    new Geom_BSplineSurface(poles, knots, knots, mults, mults, 2, 2, false, false);
  return BRepBuilderAPI_MakeFace(surface, 0.0, 1.0, 0.0, 1.0, 1.0e-7).Shape();
}

static occ::handle<Geom2d_BSplineCurve> splitPCurve(const NCollection_Array1<gp_Pnt2d>& poles)
{
  NCollection_Array1<double> knots(1, 3);
  knots(1) = 0.0;
  knots(2) = 0.5;
  knots(3) = 1.0;
  NCollection_Array1<int> mults(1, 3);
  mults(1) = 2;
  mults(2) = 1;
  mults(3) = 2;
  return new Geom2d_BSplineCurve(poles, knots, mults, 1, false);
}

static occ::handle<Geom2d_BSplineCurve> linePCurve(const NCollection_Array1<gp_Pnt2d>& poles)
{
  NCollection_Array1<double> knots(1, 2);
  knots(1) = 0.0;
  knots(2) = 1.0;
  NCollection_Array1<int> mults(1, 2);
  mults(1) = 2;
  mults(2) = 2;
  return new Geom2d_BSplineCurve(poles, knots, mults, 1, false);
}

static occ::handle<Geom2d_BSplineCurve> pcurve(const std::initializer_list<gp_Pnt2d>& points)
{
  NCollection_Array1<gp_Pnt2d> poles(1, static_cast<int>(points.size()));
  int index = 1;
  for (const gp_Pnt2d& point : points)
  {
    poles(index++) = point;
  }
  if (points.size() == 3)
  {
    return splitPCurve(poles);
  }
  return linePCurve(poles);
}

static TopoDS_Wire c0PcurveWire(const occ::handle<Geom_Surface>& surface)
{
  const occ::handle<Geom2d_BSplineCurve> curves[] = {
    pcurve({gp_Pnt2d(0.0, 0.0), gp_Pnt2d(0.5, 0.2), gp_Pnt2d(1.0, 0.0)}),
    pcurve({gp_Pnt2d(1.0, 0.0), gp_Pnt2d(1.0, 1.0)}),
    pcurve({gp_Pnt2d(1.0, 1.0), gp_Pnt2d(0.0, 1.0)}),
    pcurve({gp_Pnt2d(0.0, 1.0), gp_Pnt2d(0.0, 0.0)}),
  };
  BRepBuilderAPI_MakeWire wire;
  for (const occ::handle<Geom2d_BSplineCurve>& curve : curves)
  {
    wire.Add(BRepBuilderAPI_MakeEdge(curve, surface, 0.0, 1.0).Edge());
  }
  return wire.Wire();
}

static TopoDS_Shape c0PcurveFace()
{
  occ::handle<Geom_Plane> plane = new Geom_Plane(gp_Pln());
  occ::handle<Geom_Surface> surface = plane;
  return BRepBuilderAPI_MakeFace(surface, c0PcurveWire(surface), true).Shape();
}

static int count(const TopoDS_Shape& shape, const TopAbs_ShapeEnum kind)
{
  int result = 0;
  for (TopExp_Explorer explorer(shape, kind); explorer.More(); explorer.Next())
  {
    ++result;
  }
  return result;
}

static void observe(const int id, const int criterion, const double tolerance)
{
  ShapeUpgrade_ShapeDivideContinuity divider(c0Wire());
  divider.SetBoundaryCriterion(static_cast<GeomAbs_Shape>(criterion));
  divider.SetTolerance(tolerance);
  const bool changed = divider.Perform();
  const TopoDS_Shape result = divider.Result();
  std::cout << id << ' ' << changed << ' ' << count(result, TopAbs_WIRE) << ' '
            << count(result, TopAbs_EDGE) << ' ' << count(result, TopAbs_VERTEX) << ' '
            << divider.Status(ShapeExtend_DONE1) << ' ' << divider.Status(ShapeExtend_DONE2) << ' '
            << divider.Status(ShapeExtend_FAIL1) << ' ' << divider.Status(ShapeExtend_FAIL2) << ' '
            << divider.Status(ShapeExtend_FAIL3) << '\n';
}

static void observeSurface(const int id, const int criterion)
{
  ShapeUpgrade_ShapeDivideContinuity divider(c0Face());
  divider.SetSurfaceCriterion(static_cast<GeomAbs_Shape>(criterion));
  const bool changed = divider.Perform();
  const TopoDS_Shape result = divider.Result();
  std::cout << id << ' ' << changed << ' ' << count(result, TopAbs_FACE) << ' '
            << count(result, TopAbs_EDGE) << ' ' << divider.Status(ShapeExtend_DONE1) << ' '
            << divider.Status(ShapeExtend_DONE2) << ' ' << divider.Status(ShapeExtend_FAIL1) << ' '
            << divider.Status(ShapeExtend_FAIL2) << ' ' << divider.Status(ShapeExtend_FAIL3) << '\n';
}

static void observePCurve(const int id, const double tolerance)
{
  ShapeUpgrade_ShapeDivideContinuity divider(c0PcurveFace());
  divider.SetBoundaryCriterion(GeomAbs_C0);
  divider.SetPCurveCriterion(GeomAbs_C1);
  divider.SetSurfaceCriterion(GeomAbs_C0);
  divider.SetTolerance(1.0e-7);
  if (tolerance >= 0.0)
  {
    divider.SetTolerance2d(tolerance);
  }
  const bool changed = divider.Perform();
  const TopoDS_Shape result = divider.Result();
  std::cout << id << ' ' << changed << ' ' << count(result, TopAbs_FACE) << ' '
            << count(result, TopAbs_WIRE) << ' ' << count(result, TopAbs_EDGE) << ' '
            << count(result, TopAbs_VERTEX) << ' ' << divider.Status(ShapeExtend_DONE1) << ' '
            << divider.Status(ShapeExtend_DONE2) << ' ' << divider.Status(ShapeExtend_FAIL1) << ' '
            << divider.Status(ShapeExtend_FAIL2) << ' ' << divider.Status(ShapeExtend_FAIL3) << '\n';
}

int main()
{
  observe(0, 0, 1.0e-7); // C0
  observe(1, 2, 1.0e-7); // C1
  observe(2, 4, 1.0e-7); // C2
  observe(3, 2, 2.0); // C1 with loose tolerance
  observeSurface(4, 0); // C0
  observeSurface(5, 2); // C1
  observeSurface(6, 4); // C2
  observePCurve(7, -1.0);
  observePCurve(8, 2.0);
}
