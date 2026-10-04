#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <IntTools_Context.hxx>
#include <IntTools_Range.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <cmath>
#include <iomanip>
#include <iostream>

#define private public
#include <IntTools_BeanFaceIntersector.hxx>
#undef private

int main()
{
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0., 0., 0.), gp_Pnt(1., 0., 0.));
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0., 0., 0.), gp_Dir(0., 0., 1.)),
                                                    0.4,
                                                    0.6,
                                                    -1.,
                                                    1.);
  const BRepAdaptor_Curve curve(edge);
  const BRepAdaptor_Surface surface(face, false);
  IntTools_BeanFaceIntersector solver(curve, surface, 0., 1., 0.4, 0.6, -1., 1., 1.e-7, 1.e-7);
  solver.myContext = new IntTools_Context;
  solver.myRangeManager.SetBoundaries(0., 1., 0);
  solver.ComputeUsingExtremum();
  std::cout << std::setprecision(17) << "minimum=" << solver.myMinSqDistance << '\n';
  std::cout << "ranges=" << solver.myRangeManager.Length() << '\n';
  for (int i = 1; i <= solver.myRangeManager.Length(); ++i)
  {
    const IntTools_Range range = solver.myRangeManager.Range(i);
    std::cout << "range " << i << ' ' << solver.myRangeManager.Flag(i) << ' '
              << range.First() << ' ' << range.Last() << '\n';
  }
  const gp_Circ circle(gp_Ax2(gp_Pnt(0., 0., 0.1), gp_Dir(0., 1., 0.), gp_Dir(1., 0., 0.)), 1.);
  const double turn = 2. * std::acos(-1.);
  const TopoDS_Edge circularEdge = BRepBuilderAPI_MakeEdge(circle, 0., turn);
  const TopoDS_Face wideFace = BRepBuilderAPI_MakeFace(
      gp_Pln(gp_Pnt(0., 0., 0.), gp_Dir(0., 0., 1.)), -2., 2., -2., 2.);
  const BRepAdaptor_Curve circularCurve(circularEdge);
  const BRepAdaptor_Surface wideSurface(wideFace, false);
  IntTools_BeanFaceIntersector circularSolver(
      circularCurve, wideSurface, 0., turn, -2., 2., -2., 2., 1.e-7, 1.e-7);
  circularSolver.myContext = new IntTools_Context;
  circularSolver.myRangeManager.SetBoundaries(0., turn, 0);
  circularSolver.ComputeUsingExtremum();
  std::cout << "circle_minimum=" << circularSolver.myMinSqDistance << '\n';
  std::cout << "circle_ranges=" << circularSolver.myRangeManager.Length() << '\n';
  for (int i = 1; i <= circularSolver.myRangeManager.Length(); ++i)
  {
    const IntTools_Range range = circularSolver.myRangeManager.Range(i);
    std::cout << "circle_range " << i << ' ' << circularSolver.myRangeManager.Flag(i) << ' '
              << range.First() << ' ' << range.Last() << '\n';
  }
  return 0;
}
