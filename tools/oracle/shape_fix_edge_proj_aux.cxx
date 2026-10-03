// ShapeFix_EdgeProjAux oracle for pinned OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <BRep_Builder.hxx>
#include <BRepLib_MakeEdge.hxx>
#include <Adaptor3d_CurveOnSurface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Extrema_ExtPC.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_Circle.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_Plane.hxx>
#include <ShapeFix_EdgeProjAux.hxx>
#include <Standard_Failure.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Circ.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <NCollection_Array1.hxx>
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
  void seed_params(double first, double last)
  {
    myFirstParam = first;
    myLastParam = last;
  }
  void report_updated_curve(const char* name, const occ::handle<Geom2d_Curve>& curve)
  {
    const gp_Pnt2d first = curve->Value(curve->FirstParameter());
    const gp_Pnt2d last = curve->Value(curve->LastParameter());
    std::cout << name << ' ' << FirstParam() << ' ' << LastParam() << ' '
              << first.X() << ' ' << last.X() << '\n';
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
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
    edge.Reverse();
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    add_pcurve(edge, face, new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)));
    Probe probe(face, edge);
    probe.init_2d(1.e-7);
    probe.report_state("init-2d-reversed-edge");
  }
  {
    BRep_Builder builder;
    TopoDS_Edge edge;
    builder.MakeEdge(edge);
    TopoDS_Vertex first, last;
    builder.MakeVertex(first, gp_Pnt(0, 0, 0), 1.e-7);
    builder.MakeVertex(last, gp_Pnt(1, 0, 0), 1.e-7);
    builder.Add(edge, first.Oriented(TopAbs_FORWARD));
    builder.Add(edge, last.Oriented(TopAbs_REVERSED));
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    builder.MakeFace(face, surface, 1.e-7);
    Probe probe(face, edge);
    probe.seed_params(7., 8.);
    probe.init_2d(1.e-7);
    probe.report_state("init-2d-missing-pcurve-preserves-params");
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
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
    edge.Reverse();
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    occ::handle<Geom2d_Curve> curve = new Geom2d_TrimmedCurve(
        new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 0., 1.);
    add_pcurve(edge, face, curve);
    Probe probe(face, edge);
    probe.init_3d(1.e-7);
    probe.report_state("init-3d-reversed-edge");
  }
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(3, 0, 0));
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    occ::handle<Geom2d_Curve> curve = new Geom2d_TrimmedCurve(
        new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 0., 1.);
    add_pcurve(edge, face, curve);
    occ::handle<Adaptor2d_Curve2d> curve_adaptor = new Geom2dAdaptor_Curve(curve);
    occ::handle<Adaptor3d_Surface> surface_adaptor = new GeomAdaptor_Surface(surface);
    Adaptor3d_CurveOnSurface curve_on_surface(curve_adaptor, surface_adaptor);
    try
    {
      Extrema_ExtPC extrema(gp_Pnt(2, 0, 0), curve_on_surface, 0., 1., -1.);
      std::cout << "init-3d-extrema-failure-result no-throw " << extrema.IsDone() << '\n';
    }
    catch (Standard_Failure const&)
    {
      std::cout << "init-3d-extrema-failure-result threw\n";
    }
    Probe probe(face, edge);
    probe.init_3d(-1.);
    probe.report_state("init-3d-extrema-failure");
    Extrema_ExtPC extrema(gp_Pnt(2, 0, 0), curve_on_surface, 0., 1., 1.e-7);
    std::cout << "init-3d-no-extrema-count " << extrema.IsDone();
    if (extrema.IsDone())
    {
      std::cout << ' ' << extrema.NbExt();
    }
    std::cout << '\n';
    Probe no_extrema(face, edge);
    no_extrema.init_3d(1.e-7);
    no_extrema.report_state("init-3d-no-extrema-fallback");
  }
  {
    BRep_Builder builder;
    TopoDS_Edge edge;
    builder.MakeEdge(edge);
    TopoDS_Vertex vertex;
    builder.MakeVertex(vertex, gp_Pnt(0, 0, 0), 1.e-7);
    builder.Add(edge, vertex.Oriented(TopAbs_FORWARD));
    builder.Add(edge, vertex.Oriented(TopAbs_REVERSED));
    builder.Degenerated(edge, true);
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    builder.MakeFace(face, surface, 1.e-7);
    occ::handle<Geom2d_Curve> curve = new Geom2d_Circle(gp_Circ2d(gp_Ax2d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 1.));
    builder.UpdateEdge(edge, curve, face, 1.e-7);
    builder.Range(edge, face, 0., 2. * 3.14159265358979323846);
    occ::handle<Adaptor2d_Curve2d> curve_adaptor = new Geom2dAdaptor_Curve(curve);
    occ::handle<Adaptor3d_Surface> surface_adaptor = new GeomAdaptor_Surface(surface);
    Adaptor3d_CurveOnSurface curve_on_surface(curve_adaptor, surface_adaptor);
    Extrema_ExtPC extrema(gp_Pnt(0, 0, 0), curve_on_surface, 0.,
                          2. * 3.14159265358979323846, 1.e-7);
    std::cout << "init-3d-no-minimum-extrema " << extrema.IsDone();
    if (extrema.IsDone())
    {
      std::cout << ' ' << extrema.NbExt();
    }
    std::cout << '\n';
    Probe probe(face, edge);
    probe.init_3d(1.e-7);
    probe.report_state("init-3d-no-minimum");
  }
  {
    TopoDS_Edge edge = BRepLib_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(3, 0, 0));
    occ::handle<Geom_Surface> surface = new Geom_Plane(gp_Pln(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)));
    TopoDS_Face face;
    BRep_Builder().MakeFace(face, surface, 1.e-7);
    NCollection_Array1<gp_Pnt2d> poles(1, 2);
    poles.SetValue(1, gp_Pnt2d(0, 0));
    poles.SetValue(2, gp_Pnt2d(0, 0));
    occ::handle<Geom2d_Curve> curve = new Geom2d_BezierCurve(poles);
    add_pcurve(edge, face, curve);
    Probe probe(face, edge);
    probe.init_3d(1.e-7);
    probe.report_state("init-3d-no-stationary-root");
  }
  {
    Probe probe{TopoDS_Face(), TopoDS_Edge()};
    occ::handle<Geom2d_Curve> curve = new Geom2d_Line(gp_Pnt2d(0, 0), gp_Dir2d(1, 0));
    probe.update_param_2d(curve, 0., 0.);
    const gp_Dir2d direction = occ::down_cast<Geom2d_Line>(curve)->Direction();
    std::cout << "direct-update-param-2d " << probe.FirstParam() << ' ' << probe.LastParam()
              << ' ' << direction.X() << ' ' << direction.Y() << '\n';
  }
  {
    Probe probe{TopoDS_Face(), TopoDS_Edge()};
    occ::handle<Geom2d_Curve> basis =
        new Geom2d_Circle(gp_Circ2d(gp_Ax2d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 1.));
    occ::handle<Geom2d_Curve> circle = new Geom2d_TrimmedCurve(
        basis, 0., 2. * 3.14159265358979323846, true, true);
    probe.update_param_2d(circle, 0., 0.);
    probe.report_updated_curve("direct-update-periodic-trimmed-circle", circle);
  }
  {
    Probe probe{TopoDS_Face(), TopoDS_Edge()};
    NCollection_Array1<gp_Pnt2d> poles(1, 2);
    poles.SetValue(1, gp_Pnt2d(0, 0));
    poles.SetValue(2, gp_Pnt2d(1, 0));
    NCollection_Array1<double> knots(1, 2);
    knots.SetValue(1, 0.);
    knots.SetValue(2, 1.);
    NCollection_Array1<int> multiplicities(1, 2);
    multiplicities.SetValue(1, 2);
    multiplicities.SetValue(2, 2);
    occ::handle<Geom2d_Curve> spline = new Geom2d_BSplineCurve(poles, knots, multiplicities, 1, false);
    probe.update_param_2d(spline, 0., 0.);
    probe.report_updated_curve("direct-update-nonclosed-bspline", spline);
  }
}
