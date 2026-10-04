#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <IntTools_Range.hxx>
#include <IntTools_MarkedRangeSet.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

#define private public
#include <IntTools_BeanFaceIntersector.hxx>
#undef private

int main()
{
  const TopoDS_Shape sphere = BRepPrimAPI_MakeSphere(1.0).Shape();
  const TopoDS_Face face = TopoDS::Face(TopExp_Explorer(sphere, TopAbs_FACE).Current());
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(-2.0, 0.0, 0.0), gp_Pnt(2.0, 0.0, 0.0));
  const BRepAdaptor_Curve curve(edge);
  const BRepAdaptor_Surface surface(face, false);
  IntTools_BeanFaceIntersector solver(curve, surface, -2.0, 2.0, 0.0, 2.0 * M_PI,
                                      -0.5 * M_PI, 0.5 * M_PI, 1.0e-7, 1.0e-7);
  solver.myRangeManager.SetBoundaries(-2.0, 2.0, 0);
  solver.ComputeAroundExactIntersection();
  std::cout << std::setprecision(17) << "ranges=" << solver.myRangeManager.Length()
            << " min_sq_distance=" << solver.myMinSqDistance << '\n';
  for (int i = 1; i <= solver.myRangeManager.Length(); ++i)
  {
    const IntTools_Range range = solver.myRangeManager.Range(i);
    std::cout << "range " << i << ' ' << solver.myRangeManager.Flag(i) << ' '
              << range.First() << ' ' << range.Last() << '\n';
  }
  return 0;
}
