#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineSurface.hxx>
#include <IntTools_BeanFaceIntersector.hxx>
#include <NCollection_Array1.hxx>
#include <NCollection_Array2.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>
#include <iomanip>
#include <iostream>

static TopoDS_Face MakeBezierFace(bool located)
{
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles.SetValue(1, 1, gp_Pnt(0., 0., 0.));
  poles.SetValue(2, 1, gp_Pnt(1., 0., 0.));
  poles.SetValue(1, 2, gp_Pnt(0., 1., 0.));
  poles.SetValue(2, 2, gp_Pnt(1., 1., 0.));
  occ::handle<Geom_BezierSurface> surface = new Geom_BezierSurface(poles);
  TopoDS_Face face = BRepBuilderAPI_MakeFace(surface, 0., 1., 0., 1., 1.e-7);
  if (located)
  {
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(10., -4., 3.));
    face.Location(TopLoc_Location(transform));
  }
  return face;
}

static TopoDS_Face MakeBSplineFace(bool located)
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
  occ::handle<Geom_BSplineSurface> surface = new Geom_BSplineSurface(
    poles, knots, knots, mults, mults, 3, 3, false, false);
  TopoDS_Face face = BRepBuilderAPI_MakeFace(surface, 0., 1., 0., 1., 1.e-7);
  if (located)
  {
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(10., -4., 3.));
    face.Location(TopLoc_Location(transform));
  }
  return face;
}

static TopoDS_Edge MakeCrossingEdge(bool located)
{
  const gp_Vec offset = located ? gp_Vec(10., -4., 3.) : gp_Vec(0., 0., 0.);
  return BRepBuilderAPI_MakeEdge(gp_Pnt(0.5, 0.5, -1.).Translated(offset),
                                 gp_Pnt(0.5, 0.5, 1.).Translated(offset));
}

static void Run(const char* name, TopoDS_Face face, bool located)
{
  const TopoDS_Edge edge = MakeCrossingEdge(located);
  IntTools_BeanFaceIntersector solver(edge, face);
  solver.SetBeanParameters(0., 2.);
  solver.SetSurfaceParameters(0., 1., 0., 1.);
  for (int pass = 0; pass < 2; ++pass)
  {
    solver.Perform();
    std::cout << name << ' ' << pass << ' ' << solver.IsDone() << ' '
              << solver.Result().Length();
    for (const IntTools_Range& range : solver.Result())
    {
      std::cout << ' ' << range.First() << ' ' << range.Last();
    }
    std::cout << '\n';
  }
}

int main()
{
  std::cout << std::setprecision(17);
  Run("bezier", MakeBezierFace(false), false);
  Run("bezier_located", MakeBezierFace(true), true);
  Run("bspline", MakeBSplineFace(false), false);
  Run("bspline_located", MakeBSplineFace(true), true);
  return 0;
}
