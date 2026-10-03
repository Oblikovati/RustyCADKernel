// Differential oracle for private TempSameRange through FixSameParameter's degenerated-edge path.
#include <BRep_Builder.hxx>
#include <BRepLib_MakeEdge.hxx>
#include <BRep_Tool.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Curve.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Circ2d.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <Precision.hxx>
#include <ShapeFix_Edge.hxx>
#include <gp_Cylinder.hxx>
#include <gp_Pln.hxx>
#include <iomanip>
#include <iostream>

void print_curve_points(const occ::handle<Geom2d_Curve>& curve, double first, double last)
{
  for (double parameter : {first, last})
  {
    gp_Pnt2d point;
    curve->D0(parameter, point);
    std::cout << ' ' << point.X() << ' ' << point.Y();
  }
}

void print_curve(const occ::handle<Geom2d_Curve>& curve, double first, double last)
{
  std::cout << ' ' << curve->DynamicType()->Name();
  print_curve_points(curve, first, last);
}

void print_seam_curve(const TopoDS_Edge& edge, double first, double last)
{
  occ::handle<Geom2d_Curve> second;
  occ::handle<Geom_Surface> seamSurface;
  TopLoc_Location location;
  double secondFirst, secondLast;
  BRep_Tool::CurveOnSurface(edge, second, seamSurface, location, secondFirst, secondLast, 2);
  std::cout << ' ' << second->DynamicType()->Name() << ' ' << secondFirst << ' ' << secondLast;
  print_curve_points(second, first, last);
}

void dump(const char* name, const TopoDS_Edge& edge, const occ::handle<Geom_Surface>& surface)
{
  double edgeFirst, edgeLast, pcurveFirst, pcurveLast;
  BRep_Tool::Range(edge, edgeFirst, edgeLast);
  BRep_Tool::Range(edge, surface, TopLoc_Location(), pcurveFirst, pcurveLast);
  occ::handle<Geom2d_Curve> first = BRep_Tool::CurveOnSurface(
    edge, surface, TopLoc_Location(), pcurveFirst, pcurveLast);
  std::cout << name << ' ' << edgeFirst << ' ' << edgeLast << ' ' << pcurveFirst << ' ' << pcurveLast;
  print_curve(first, edgeFirst, edgeLast);
  const bool seam = BRep_Tool::IsClosed(edge, surface, TopLoc_Location());
  std::cout << ' ' << seam;
  if (seam) print_seam_curve(edge, edgeFirst, edgeLast);
  std::cout << ' ' << BRep_Tool::SameRange(edge) << ' ' << BRep_Tool::SameParameter(edge) << '\n';
}

void pconfusion_case(const char* name, const occ::handle<Geom_Surface>& plane, double delta)
{
  BRep_Builder builder;
  const TopLoc_Location location;
  TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0,0,0), gp_Pnt(1,0,0)).Edge();
  occ::handle<Geom2d_Curve> curve = new Geom2d_Line(gp_Pnt2d(0,0), gp_Dir2d(1,0));
  builder.UpdateEdge(edge, curve, plane, location, 1e-7);
  builder.Range(edge, plane, location, delta, 1.0 + delta);
  builder.Range(edge, 0.0, 1.0, true);
  builder.SameRange(edge, false);
  ShapeFix_Edge fixer;
  fixer.FixSameParameter(edge);
  dump(name, edge, plane);
}

void periodic_case(const occ::handle<Geom_Surface>& cylinder)
{
  BRep_Builder builder;
  const TopLoc_Location location;
  TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0,0,0), gp_Pnt(1,0,0)).Edge();
  occ::handle<Geom2d_Curve> curve = new Geom2d_Circle(gp_Circ2d(gp_Ax2d(gp_Pnt2d(0,0), gp_Dir2d(1,0)), 1.0));
  builder.UpdateEdge(edge, curve, cylinder, location, 1e-7);
  builder.Range(edge, cylinder, location, 7.0, 8.0);
  builder.Range(edge, 0.0, 1.0, true);
  builder.SameRange(edge, false);
  ShapeFix_Edge fixer;
  fixer.FixSameParameter(edge);
  dump("periodic", edge, cylinder);
}

void bezier_case(const occ::handle<Geom_Surface>& plane)
{
  BRep_Builder builder;
  const TopLoc_Location location;
  TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0,0,0), gp_Pnt(1,0,0)).Edge();
  NCollection_Array1<gp_Pnt2d> poles(1,3);
  poles(1)=gp_Pnt2d(0,0); poles(2)=gp_Pnt2d(0.5,1); poles(3)=gp_Pnt2d(1,0);
  occ::handle<Geom2d_Curve> curve = new Geom2d_BezierCurve(poles);
  builder.UpdateEdge(edge, curve, plane, location, 1e-7);
  builder.Range(edge, plane, location, 0.25, 0.75);
  builder.Range(edge, 0.0, 1.0, true);
  builder.SameRange(edge, false);
  ShapeFix_Edge fixer;
  fixer.FixSameParameter(edge);
  dump("bezier", edge, plane);
}

void seam_case(const occ::handle<Geom_Surface>& cylinder)
{
  BRep_Builder builder;
  const TopLoc_Location location;
  TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0,0,0), gp_Pnt(1,0,0)).Edge();
  occ::handle<Geom2d_Curve> first = new Geom2d_Circle(gp_Circ2d(gp_Ax2d(gp_Pnt2d(0,0), gp_Dir2d(1,0)), 1.0));
  occ::handle<Geom2d_Curve> second = new Geom2d_Line(gp_Pnt2d(2.0 * M_PI,0), gp_Dir2d(0,1));
  builder.UpdateEdge(edge, first, second, cylinder, location, 1e-7);
  builder.Range(edge, cylinder, location, 7.0, 8.0);
  builder.Range(edge, 0.0, 1.0, true);
  builder.SameRange(edge, false);
  ShapeFix_Edge fixer;
  fixer.FixSameParameter(edge);
  dump("seam", edge, cylinder);
}

int main()
{
  std::cout << std::setprecision(17);
  const occ::handle<Geom_Surface> plane = new Geom_Plane(gp_Pln(gp::XOY()));
  const occ::handle<Geom_Surface> cylinder = new Geom_CylindricalSurface(gp_Cylinder(gp_Ax3(), 2.0));
  pconfusion_case("pconfusion-under", plane, Precision::PConfusion() * 0.5);
  pconfusion_case("pconfusion-over", plane, Precision::PConfusion() * 2.0);
  periodic_case(cylinder);
  bezier_case(plane);
  seam_case(cylinder);
}
