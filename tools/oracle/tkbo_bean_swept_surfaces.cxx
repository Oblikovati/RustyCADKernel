#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_Circle.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <IntTools_BeanFaceIntersector.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <iomanip>
#include <iostream>

static void PrintPass(const char* name, IntTools_BeanFaceIntersector& solver, int pass);

static void Run(const char* name,
                const occ::handle<Geom_Surface>& surface,
                const TopoDS_Edge& edge,
                double first,
                double last,
                double uMin,
                double uMax,
                double vMin,
                double vMax)
{
  TopoDS_Face face = BRepBuilderAPI_MakeFace(surface, uMin, uMax, vMin, vMax, 1.e-7);
  IntTools_BeanFaceIntersector solver(edge, face);
  solver.SetBeanParameters(first, last);
  solver.SetSurfaceParameters(uMin, uMax, vMin, vMax);
  for (int pass = 0; pass < 2; ++pass) PrintPass(name, solver, pass);
}

static void PrintPass(const char* name, IntTools_BeanFaceIntersector& solver, int pass)
{
  solver.Perform();
  std::cout << name << ' ' << pass << ' ' << solver.IsDone() << ' ' << solver.Result().Length();
  for (const IntTools_Range& range : solver.Result())
    std::cout << ' ' << range.First() << ' ' << range.Last();
  std::cout << '\n';
}

static occ::handle<Geom_Surface> MakeOffsetPlane()
{
  occ::handle<Geom_Plane> plane = new Geom_Plane(gp_Pln(gp::XOY()));
  return new Geom_OffsetSurface(plane, 0.5);
}

static occ::handle<Geom_Surface> MakeRevolutionTorus()
{
  const gp_Ax2 circleFrame(gp_Pnt(2., 0., 0.), gp_Dir(0., 1., 0.), gp_Dir(1., 0., 0.));
  occ::handle<Geom_Circle> circle = new Geom_Circle(circleFrame, 0.5);
  return new Geom_SurfaceOfRevolution(circle, gp_Ax1(gp::Origin(), gp::DZ()));
}

static occ::handle<Geom_Surface> MakeOffsetSphere()
{
  occ::handle<Geom_SphericalSurface> sphere = new Geom_SphericalSurface(gp_Ax3(), 2.0);
  return new Geom_OffsetSurface(sphere, 0.5);
}

static occ::handle<Geom_Surface> MakeExtrudedCircle()
{
  occ::handle<Geom_Circle> circle = new Geom_Circle(gp::XOY(), 1.0);
  return new Geom_SurfaceOfLinearExtrusion(circle, gp::DZ());
}

int main()
{
  std::cout << std::setprecision(17);
  Run("offset_plane", MakeOffsetPlane(),
      BRepBuilderAPI_MakeEdge(gp_Pnt(0., 0., -1.), gp_Pnt(0., 0., 1.)), 0., 4.,
      -1., 1., -1., 1.);
  Run("revolution_torus", MakeRevolutionTorus(),
      BRepBuilderAPI_MakeEdge(gp_Pnt(2., 0., -1.), gp_Pnt(2., 0., 1.)), 0., 4.,
      0., 2. * M_PI, -M_PI, M_PI);
  Run("extruded_cylinder", MakeExtrudedCircle(),
      BRepBuilderAPI_MakeEdge(gp_Pnt(-2., 0., 0.5), gp_Pnt(2., 0., 0.5)), 0., 4.,
      0., 2. * M_PI, -1., 1.);
  Run("offset_sphere", MakeOffsetSphere(),
      BRepBuilderAPI_MakeEdge(gp_Pnt(0.5, 0., -3.), gp_Pnt(0.5, 0., 3.)), 0., 6.,
      0., 2. * M_PI, -M_PI / 2., M_PI / 2.);
  return 0;
}
