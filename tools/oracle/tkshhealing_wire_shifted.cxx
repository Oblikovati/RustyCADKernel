#include <array>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <Geom_Plane.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom2d_Line.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeExtend_WireData.hxx>
#include <ShapeFix_Wire.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_Circle.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Edge.hxx>
#include <ShapeExtend_Status.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Circ.hxx>
#include <gp.hxx>
#include <iostream>
#include <cmath>

static void check_cyclic_stop_revisit();
static void check_periodic_shift();
static void check_revolution_basis_period();
static void check_singularity_uses_fixer_precision();
static void check_spherical_bimeridian();
static void check_spherical_v_period();

int main()
{
  check_periodic_shift();
  check_spherical_v_period();
  check_spherical_bimeridian();
  check_cyclic_stop_revisit();
  check_singularity_uses_fixer_precision();
  check_revolution_basis_period();
}

static void check_spherical_v_period()
{
  BRep_Builder builder;
  Handle(Geom_SphericalSurface) sphere = new Geom_SphericalSurface(gp_Ax3(), 1.0);
  TopoDS_Face face;
  builder.MakeFace(face, sphere, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  for (const auto& bounds : {std::array<double, 4>{0.2, 1.4, 0.4, 1.5},
                             std::array<double, 4>{0.4, -1.5, 0.6, -1.4}})
  {
    TopoDS_Edge edge;
    builder.MakeEdge(edge);
    const double du = bounds[2] - bounds[0];
    const double dv = bounds[3] - bounds[1];
    Handle(Geom2d_Line) pcurve = new Geom2d_Line(
      gp_Ax2d(gp_Pnt2d(bounds[0], bounds[1]), gp_Dir2d(du, dv)));
    const double last = std::sqrt(du * du + dv * dv);
    builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
    builder.Range(edge, face, 0.0, last);
    TopoDS_Vertex first;
    TopoDS_Vertex second;
    builder.MakeVertex(first, sphere->Value(bounds[0], bounds[1]), 1.0e-7);
    builder.MakeVertex(second, sphere->Value(bounds[2], bounds[3]), 1.0e-7);
    builder.Add(edge, first);
    second.Reverse();
    builder.Add(edge, second);
    data->Add(edge);
  }
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  const bool changed = fixer.FixShifted();
  ShapeAnalysis_Edge edgeAnalysis;
  std::cout << "sphere=" << changed << " done1="
            << fixer.LastFixStatus(ShapeExtend_DONE1) << " done2="
            << fixer.LastFixStatus(ShapeExtend_DONE2);
  for (int index = 1; index <= data->NbEdges(); ++index)
  {
    Handle(Geom2d_Curve) curve;
    double first;
    double last;
    edgeAnalysis.PCurve(data->Edge(index), face, curve, first, last, true);
    const gp_Pnt2d a = curve->Value(first);
    const gp_Pnt2d b = curve->Value(last);
    std::cout << " e" << index << "=" << a.X() << ',' << a.Y() << ',' << b.X() << ','
              << b.Y();
  }
  std::cout << '\n';
}
static TopoDS_Edge make_sphere_edge(BRep_Builder& builder,
                                    const TopoDS_Face& face,
                                    const Handle(Geom_SphericalSurface)& sphere,
                                    const gp_Pnt2d& firstPoint,
                                    const gp_Pnt2d& lastPoint)
{
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  const double du = lastPoint.X() - firstPoint.X();
  const double dv = lastPoint.Y() - firstPoint.Y();
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(
    gp_Ax2d(firstPoint, gp_Dir2d(du, dv)));
  const double last = std::sqrt(du * du + dv * dv);
  builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
  builder.Range(edge, face, 0.0, last);
  TopoDS_Vertex first;
  TopoDS_Vertex second;
  builder.MakeVertex(first, sphere->Value(firstPoint.X(), firstPoint.Y()), 1.0e-7);
  builder.MakeVertex(second, sphere->Value(lastPoint.X(), lastPoint.Y()), 1.0e-7);
  first.Orientation(TopAbs_FORWARD);
  second.Orientation(TopAbs_REVERSED);
  builder.Add(edge, first);
  builder.Add(edge, second);
  return edge;
}
static void check_spherical_bimeridian()
{
  BRep_Builder builder;
  Handle(Geom_SphericalSurface) sphere = new Geom_SphericalSurface(gp_Ax3(), 1.0);
  TopoDS_Face face;
  builder.MakeFace(face, sphere, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(0.0, 1.3), gp_Pnt2d(1.0, 1.5)));
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(5.0, 1.5), gp_Pnt2d(5.5, 1.3)));
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(4.5, -1.3), gp_Pnt2d(5.0, -1.5)));
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(1.0, -1.5), gp_Pnt2d(1.5, -1.3)));
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  const bool changed = fixer.FixShifted();
  ShapeAnalysis_Edge edgeAnalysis;
  std::cout << "bimeridian=" << changed << " done1="
            << fixer.LastFixStatus(ShapeExtend_DONE1) << " done2="
            << fixer.LastFixStatus(ShapeExtend_DONE2);
  for (int index = 1; index <= data->NbEdges(); ++index)
  {
    Handle(Geom2d_Curve) curve;
    double first;
    double last;
    edgeAnalysis.PCurve(data->Edge(index), face, curve, first, last, true);
    const gp_Pnt2d a = curve->Value(first);
    const gp_Pnt2d b = curve->Value(last);
    std::cout << " e" << index << "=" << a.X() << ',' << a.Y() << ',' << b.X() << ','
              << b.Y();
  }
  std::cout << '\n';
}
static TopoDS_Edge make_cylinder_u_edge(BRep_Builder& builder,
                                        const TopoDS_Face& face,
                                        const Handle(Geom_CylindricalSurface)& cylinder,
                                        double firstU,
                                        double lastU)
{
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  const double du = lastU - firstU;
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(
    gp_Ax2d(gp_Pnt2d(firstU, 0.0), gp_Dir2d(du, 0.0)));
  builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
  builder.Range(edge, face, 0.0, std::abs(du));
  TopoDS_Vertex first;
  TopoDS_Vertex last;
  builder.MakeVertex(first, cylinder->Value(firstU, 0.0), 1.0e-7);
  builder.MakeVertex(last, cylinder->Value(lastU, 0.0), 1.0e-7);
  first.Orientation(TopAbs_FORWARD);
  last.Orientation(TopAbs_REVERSED);
  builder.Add(edge, first);
  builder.Add(edge, last);
  return edge;
}
static void check_cyclic_stop_revisit()
{
  BRep_Builder builder;
  Handle(Geom_CylindricalSurface) cylinder = new Geom_CylindricalSurface(gp_Ax3(), 1.0);
  TopoDS_Face face;
  builder.MakeFace(face, cylinder, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(make_cylinder_u_edge(builder, face, cylinder, 2.5, 2.6));
  data->Add(make_cylinder_u_edge(builder, face, cylinder, 0.1, 0.2));
  data->Add(make_cylinder_u_edge(builder, face, cylinder, 6.1, 0.0));
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  const bool changed = fixer.FixShifted();
  ShapeAnalysis_Edge edgeAnalysis;
  Handle(Geom2d_Curve) curve;
  double first;
  double last;
  std::cout << "cyclic_stop=" << changed << " done1="
            << fixer.LastFixStatus(ShapeExtend_DONE1) << " done2="
            << fixer.LastFixStatus(ShapeExtend_DONE2);
  for (int index = 1; index <= data->NbEdges(); ++index)
  {
    edgeAnalysis.PCurve(data->Edge(index), face, curve, first, last, true);
    const gp_Pnt2d start = curve->Value(first);
    const gp_Pnt2d end = curve->Value(last);
    std::cout << " e" << index << '=' << start.X() << ',' << end.X();
  }
  std::cout << '\n';
}
static void check_singularity_uses_fixer_precision()
{
  BRep_Builder builder;
  Handle(Geom_SphericalSurface) sphere = new Geom_SphericalSurface(gp_Ax3(), 1.0);
  TopoDS_Face face;
  builder.MakeFace(face, sphere, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(0.0, 1.3), gp_Pnt2d(1.0, 1.5706)));
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(5.0, 1.5706), gp_Pnt2d(5.5, 1.3)));
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(4.5, 1.3), gp_Pnt2d(5.0, 1.5706)));
  data->Add(make_sphere_edge(builder, face, sphere, gp_Pnt2d(1.0, 1.5706), gp_Pnt2d(1.5, 1.3)));
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  fixer.SetPrecision(1.0e-3);
  const bool changed = fixer.FixShifted();
  std::cout << "singularity_precision=" << changed << " done1="
            << fixer.LastFixStatus(ShapeExtend_DONE1) << " done2="
            << fixer.LastFixStatus(ShapeExtend_DONE2) << '\n';
}
static void check_revolution_basis_period()
{
  BRep_Builder builder;
  Handle(Geom_Circle) basis = new Geom_Circle(
    gp_Ax2(gp_Pnt(2.0, 0.0, 0.0), gp_Dir(1.0, 0.0, 0.0)), 1.0);
  Handle(Geom_SurfaceOfRevolution) revolution = new Geom_SurfaceOfRevolution(
    basis, gp_Ax1(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0)));
  TopoDS_Face face;
  builder.MakeFace(face, revolution, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  for (const auto& bounds : {std::array<double, 4>{0.2, 5.8, 0.4, 6.0},
                             std::array<double, 4>{0.4, -0.2, 0.6, 0.0}})
  {
    TopoDS_Edge edge;
    builder.MakeEdge(edge);
    const double du = bounds[2] - bounds[0];
    const double dv = bounds[3] - bounds[1];
    Handle(Geom2d_Line) pcurve = new Geom2d_Line(
      gp_Ax2d(gp_Pnt2d(bounds[0], bounds[1]), gp_Dir2d(du, dv)));
    const double last = std::sqrt(du * du + dv * dv);
    builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
    builder.Range(edge, face, 0.0, last);
    TopoDS_Vertex first;
    TopoDS_Vertex second;
    builder.MakeVertex(first, revolution->Value(bounds[0], bounds[1]), 1.0e-7);
    builder.MakeVertex(second, revolution->Value(bounds[2], bounds[3]), 1.0e-7);
    first.Orientation(TopAbs_FORWARD);
    second.Orientation(TopAbs_REVERSED);
    builder.Add(edge, first);
    builder.Add(edge, second);
    data->Add(edge);
  }
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  const bool changed = fixer.FixShifted();
  ShapeAnalysis_Edge edgeAnalysis;
  std::cout << "revolution=" << changed << " done1="
            << fixer.LastFixStatus(ShapeExtend_DONE1) << " done2="
            << fixer.LastFixStatus(ShapeExtend_DONE2);
  for (int index = 1; index <= data->NbEdges(); ++index)
  {
    Handle(Geom2d_Curve) curve;
    double first;
    double last;
    edgeAnalysis.PCurve(data->Edge(index), face, curve, first, last, true);
    const gp_Pnt2d a = curve->Value(first);
    const gp_Pnt2d b = curve->Value(last);
    std::cout << " e" << index << "=" << a.X() << ',' << a.Y() << ',' << b.X() << ','
              << b.Y();
  }
  std::cout << '\n';
}
static TopoDS_Edge make_periodic_edge(BRep_Builder& builder,
                                      const TopoDS_Face& face,
                                      const Handle(Geom_CylindricalSurface)& cylinder,
                                      double firstU,
                                      double lastU)
{
  TopoDS_Edge edge;
  builder.MakeEdge(edge);
  Handle(Geom2d_Line) pcurve = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(firstU, 0.), gp_Dir2d(1., 0.)));
  builder.UpdateEdge(edge, pcurve, face, 1.0e-7);
  builder.Range(edge, face, 0., lastU - firstU);
  TopoDS_Vertex first;
  TopoDS_Vertex last;
  builder.MakeVertex(first, cylinder->Value(firstU, 0.), 1.0e-7);
  builder.MakeVertex(last, cylinder->Value(lastU, 0.), 1.0e-7);
  TopoDS_Vertex orientedFirst = first;
  orientedFirst.Orientation(TopAbs_FORWARD);
  TopoDS_Vertex orientedLast = last;
  orientedLast.Orientation(TopAbs_REVERSED);
  builder.Add(edge, orientedFirst);
  builder.Add(edge, orientedLast);
  return edge;
}
static void check_periodic_shift()
{
  BRep_Builder builder;
  Handle(Geom_CylindricalSurface) cylinder = new Geom_CylindricalSurface(gp_Ax3(), 1.0);
  TopoDS_Face face;
  builder.MakeFace(face, cylinder, 1.0e-7);
  Handle(ShapeExtend_WireData) data = new ShapeExtend_WireData;
  data->Add(make_periodic_edge(builder, face, cylinder, 5.8, 6.0));
  data->Add(make_periodic_edge(builder, face, cylinder, -0.2, 0.0));
  Handle(ShapeAnalysis_Wire) analyzer = new ShapeAnalysis_Wire;
  analyzer->Init(data, face, 1.0e-7);
  ShapeFix_Wire fixer;
  fixer.Init(analyzer);
  bool changed = fixer.FixShifted();
  ShapeAnalysis_Edge edgeAnalysis;
  Handle(Geom2d_Curve) firstCurve;
  double first;
  double last;
  edgeAnalysis.PCurve(data->Edge(1), face, firstCurve, first, last, true);
  gp_Pnt2d firstStart = firstCurve->Value(first);
  gp_Pnt2d firstEnd = firstCurve->Value(last);
  edgeAnalysis.PCurve(data->Edge(2), face, firstCurve, first, last, true);
  gp_Pnt2d secondStart = firstCurve->Value(first);
  gp_Pnt2d secondEnd = firstCurve->Value(last);
  std::cout << "shift=" << changed << " done1=" << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " done2=" << fixer.LastFixStatus(ShapeExtend_DONE2) << " uv=" << firstStart.X()
            << ',' << firstEnd.X() << ',' << secondStart.X() << ',' << secondEnd.X() << '\n';
}
