#include <BRepAdaptor_Curve.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <IntTools_Range.hxx>
#include <IntTools_MarkedRangeSet.hxx>
#include <IntTools_Context.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>

#define private public
#include <IntTools_BeanFaceIntersector.hxx>
#undef private

int main()
{
  const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0., 0., 1.e-7), gp_Pnt(1., 0., 1.e-7));
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0., 0., 0.), gp_Dir(0., 0., 1.)), -1., 2., -1., 1.);
  const BRepAdaptor_Curve curve(edge);
  const BRepAdaptor_Surface surface(face, false);
  IntTools_BeanFaceIntersector solver(curve, surface, 0., 1., -1., 2., -1., 1., 1.e-7, 1.e-7);
  solver.myContext = new IntTools_Context;
  solver.myRangeManager.SetBoundaries(0., 1., 0);
  solver.ComputeNearRangeBoundaries();
  std::cout << std::setprecision(17) << "ranges=" << solver.myRangeManager.Length() << '\n';
  for (int i = 1; i <= solver.myRangeManager.Length(); ++i)
  {
    const IntTools_Range range = solver.myRangeManager.Range(i);
    std::cout << "range " << i << ' ' << solver.myRangeManager.Flag(i) << ' '
              << range.First() << ' ' << range.Last() << '\n';
  }
  return 0;
}
