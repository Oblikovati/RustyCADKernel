#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <IntTools_Context.hxx>
#include <IntTools_Range.hxx>
#include <NCollection_Array2.hxx>
#include <NCollection_Array1.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

#define private public
#include <IntTools_BeanFaceIntersector.hxx>
#undef private

static void Dump(IntTools_BeanFaceIntersector& solver, const char* name)
{
  solver.myRangeManager.SetBoundaries(solver.myFirstParameter, solver.myLastParameter, 0);
  const bool localized = solver.ComputeLocalized();
  std::cout << name << " localized=" << localized << " minimum=" << solver.myMinSqDistance
            << " ranges=" << solver.myRangeManager.Length() << '\n';
  for (int i = 1; i <= solver.myRangeManager.Length(); ++i)
  {
    const IntTools_Range range = solver.myRangeManager.Range(i);
    std::cout << "range " << i << ' ' << solver.myRangeManager.Flag(i) << ' '
              << range.First() << ' ' << range.Last() << '\n';
  }
}

int main()
{
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles.SetValue(1, 1, gp_Pnt(0., 0., 0.));
  poles.SetValue(2, 1, gp_Pnt(1., 0., 0.));
  poles.SetValue(1, 2, gp_Pnt(0., 1., 0.));
  poles.SetValue(2, 2, gp_Pnt(1., 1., 0.));
  const occ::handle<Geom_BezierSurface> surfaceGeometry = new Geom_BezierSurface(poles);
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(surfaceGeometry, 0., 1., 0., 1., 1.e-7);
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0.5, 0.5, -1.), gp_Pnt(0.5, 0.5, 1.));
  const BRepAdaptor_Curve curve(edge);
  const BRepAdaptor_Surface surface(face, true);
  IntTools_BeanFaceIntersector solver(curve, surface, 0., 2., 0., 1., 0., 1., 1.e-7, 1.e-7);
  solver.myContext = new IntTools_Context;
  std::cout << std::setprecision(17);
  Dump(solver, "flat_bezier");

  NCollection_Array2<gp_Pnt> splinePoles(1, 6, 1, 6);
  for (int u = 1; u <= 6; ++u)
  {
    for (int v = 1; v <= 6; ++v)
    {
      const double x = static_cast<double>(u - 1) / 5.;
      const double y = static_cast<double>(v - 1) / 5.;
      splinePoles.SetValue(u, v, gp_Pnt(x, y, 0.08 * x * y));
    }
  }
  NCollection_Array1<double> uKnots(1, 3), vKnots(1, 3);
  NCollection_Array1<int> uMults(1, 3), vMults(1, 3);
  uKnots.SetValue(1, 0.); uKnots.SetValue(2, 0.5); uKnots.SetValue(3, 1.);
  vKnots.SetValue(1, 0.); vKnots.SetValue(2, 0.5); vKnots.SetValue(3, 1.);
  uMults.SetValue(1, 4); uMults.SetValue(2, 2); uMults.SetValue(3, 4);
  vMults.SetValue(1, 4); vMults.SetValue(2, 2); vMults.SetValue(3, 4);
  const occ::handle<Geom_BSplineSurface> splineGeometry =
    new Geom_BSplineSurface(splinePoles, uKnots, vKnots, uMults, vMults, 3, 3, false, false);
  const TopoDS_Face splineFace = BRepBuilderAPI_MakeFace(splineGeometry, 0., 1., 0., 1., 1.e-7);
  const BRepAdaptor_Surface splineAdaptor(splineFace, true);
  IntTools_BeanFaceIntersector splineSolver(curve, splineAdaptor, 0., 2., 0., 1., 0., 1., 1.e-7, 1.e-7);
  splineSolver.myContext = new IntTools_Context;
  Dump(splineSolver, "cubic_bspline");
  return 0;
}
