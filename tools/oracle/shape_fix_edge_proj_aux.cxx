// ShapeFix_EdgeProjAux oracle for pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRepLib_MakeEdge.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <ShapeFix_EdgeProjAux.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <iomanip>
#include <iostream>

class Probe final : public ShapeFix_EdgeProjAux
{
public:
  Probe(const TopoDS_Face& face, const TopoDS_Edge& edge)
      : ShapeFix_EdgeProjAux(face, edge)
  {
  }

  void init_2d(double tolerance) { Init2d(tolerance); }
  void init_3d(double tolerance) { Init3d(tolerance); }
  void update_param_2d(const occ::handle<Geom2d_Curve>& curve, double first, double last)
  {
    myFirstParam = first;
    myLastParam = last;
    UpdateParam2d(curve);
  }
  void report_state(const char* name)
  {
    std::cout << name << ' ' << IsFirstDone() << ' ' << IsLastDone() << ' '
              << FirstParam() << ' ' << LastParam() << '\n';
  }
};

void report(const char* name, const TopoDS_Face& face, const TopoDS_Edge& edge)
{
  ShapeFix_EdgeProjAux projection(face, edge);
  projection.Compute(1.e-7);
  std::cout << name << ' ' << projection.IsFirstDone() << ' ' << projection.IsLastDone()
            << ' ' << projection.FirstParam() << ' ' << projection.LastParam() << '\n';
}

void add_pcurve(const TopoDS_Edge& edge, const TopoDS_Face& face,
                const occ::handle<Geom2d_Curve>& curve)
{
  BRep_Builder builder;
  builder.UpdateEdge(edge, curve, face, 1.e-7);
  builder.Range(edge, face, curve->FirstParameter(), curve->LastParameter());
}

int main()
{
  std::cout << std::setprecision(17);
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    add_pcurve(edge, face, new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)));
    report("plane-line", face, edge);
    Probe probe(face, edge);
    probe.init_2d(1.e-7);
    probe.report_state("direct-init-2d");
  }
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(4, 0, 0), gp_Pnt(5, 0, 0));
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    add_pcurve(edge, face, new Geom2d_Line(gp_Pnt2d(4, 0), gp_Dir2d(1, 0)));
    report("plane-line-at-four", face, edge);
  }
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    add_pcurve(edge, face, new Geom2d_Line(gp_Pnt2d(1, 0), gp_Dir2d(-1, 0)));
    report("plane-reversed-line", face, edge);
  }
  {
    occ::handle<Geom_Surface> surface = new Geom_CylindricalSurface(gp_Ax3(), 1.);
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    occ::handle<Geom_Curve> circle = new Geom_Circle(gp_Circ(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.));
    TopoDS_Edge edge = BRepLib_MakeEdge(circle, 0., 2. * 3.14159265358979323846);
    add_pcurve(edge, face, new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)));
    report("cylinder-periodic-loop", face, edge);
  }
  {
    occ::handle<Geom_Surface> surface = new Geom_CylindricalSurface(gp_Ax3(), 1.);
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(1, 0, 1));
    report("cylinder-missing-pcurve", face, edge);
  }
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    occ::handle<Geom2d_Curve> curve = new Geom2d_TrimmedCurve(
        new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 0., 1.);
    add_pcurve(edge, face, curve);
    Probe probe(face, edge);
    probe.init_3d(1.e-7);
    probe.report_state("direct-init-3d");
  }
  {
    Probe probe{TopoDS_Face(), TopoDS_Edge()};
    occ::handle<Geom2d_Curve> curve = new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0));
    probe.update_param_2d(curve, 0., 0.);
    const gp_Dir2d direction = occ::down_cast<Geom2d_Line>(curve)->Direction();
    std::cout << "direct-update-param-2d " << probe.FirstParam() << ' ' << probe.LastParam()
              << ' ' << direction.X() << ' ' << direction.Y() << '\n';
  }
}
