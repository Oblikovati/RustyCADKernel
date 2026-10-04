#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_BSplineSurface.hxx>
#include <IntTools_Context.hxx>
#include <IntTools_SurfaceRangeLocalizeData.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

#define private public
#include <IntTools_BeanFaceIntersector.hxx>
#undef private

static TopoDS_Face MakeSurface()
{
  NCollection_Array2<gp_Pnt> poles(1, 6, 1, 6);
  for (int u = 1; u <= 6; ++u)
  {
    for (int v = 1; v <= 6; ++v)
    {
      const double x = static_cast<double>(u - 1) / 5.;
      const double y = static_cast<double>(v - 1) / 5.;
      poles.SetValue(u, v, gp_Pnt(x, y, 0.08 * x * y));
    }
  }
  NCollection_Array1<double> knots(1, 3);
  NCollection_Array1<int> mults(1, 3);
  knots.SetValue(1, 0.); knots.SetValue(2, 0.5); knots.SetValue(3, 1.);
  mults.SetValue(1, 4); mults.SetValue(2, 2); mults.SetValue(3, 4);
  occ::handle<Geom_BSplineSurface> geometry = new Geom_BSplineSurface(
    poles, knots, knots, mults, mults, 3, 3, false, false);
  return BRepBuilderAPI_MakeFace(geometry, 0., 1., 0., 1., 1.e-7);
}

static void Dump(const char* name,
                 bool localized,
                 IntTools_BeanFaceIntersector& solver,
                 IntTools_SurfaceRangeLocalizeData& cache,
                 const IntTools_SurfaceRangeSample& rootSurface)
{
  std::cout << name << ' ' << localized << ' ' << cache.IsRangeOut(rootSurface) << ' '
            << cache.GetRangeUGrid() << ' ' << cache.GetRangeVGrid() << ' '
            << cache.GetGridDeflection() << ' ' << solver.myRangeManager.Length();
  for (int i = 1; i <= solver.myRangeManager.Length(); ++i)
  {
    const IntTools_Range range = solver.myRangeManager.Range(i);
    std::cout << ' ' << solver.myRangeManager.Flag(i) << ' '
              << range.First() << ' ' << range.Last();
  }
  std::cout << '\n';
}

int main()
{
  std::cout << std::setprecision(17);
  const TopoDS_Face face = MakeSurface();
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(
    gp_Pnt(0.5, 0.5, -1.), gp_Pnt(0.5, 0.5, 1.));
  const BRepAdaptor_Curve curve(edge);
  const BRepAdaptor_Surface surface(face, true);
  IntTools_BeanFaceIntersector solver(curve, surface, 0., 2., 0., 1., 0., 1., 1.e-7, 1.e-7);
  solver.myContext = new IntTools_Context;
  solver.myRangeManager.SetBoundaries(0., 2., 2);
  const IntTools_SurfaceRangeSample rootSurface(0, 0, 0, 0);
  IntTools_SurfaceRangeLocalizeData& cache = solver.myContext->SurfaceData(solver.mySurface.Face());
  cache.AddOutRange(rootSurface);
  const bool localized = solver.ComputeLocalized();
  Dump("premarked", localized, solver, cache, rootSurface);
  solver.SetBeanParameters(1., 1.0000001);
  solver.myRangeManager.SetBoundaries(1., 1.0000001, 0);
  Dump("short_range", solver.ComputeLocalized(), solver, cache, rootSurface);
  return 0;
}
