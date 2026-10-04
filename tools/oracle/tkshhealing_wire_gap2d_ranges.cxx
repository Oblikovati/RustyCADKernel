#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Line.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <ShapeFix_Wire.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeAnalysis_Surface.hxx>
#include <ShapeAnalysis_Edge.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <TopExp_Explorer.hxx>
#include <gp_Pln.hxx>
#include <gp_Elips.hxx>
#include <NCollection_Array1.hxx>
#include <iomanip>
#include <iostream>

void print_pcurve(const TopoDS_Edge& edge, const TopoDS_Face& face) {
  double first = 0.0, last = 0.0;
  const auto pcurve = BRep_Tool::CurveOnSurface(edge, face, first, last);
  const gp_Pnt2d start = pcurve->Value(first), end = pcurve->Value(last);
  std::cout << ' ' << std::setprecision(17) << first << ' ' << last << ' ' << start.X() << ' '
            << start.Y() << ' ' << end.X() << ' ' << end.Y();
}

void print_result(const char* name, ShapeFix_Wire& tool, bool changed) {
  std::cout << name << ' ' << changed << ' ' << tool.LastFixStatus(ShapeExtend_DONE1) << ' '
            << tool.LastFixStatus(ShapeExtend_FAIL) << ' ' << tool.Context().IsNull();
  for (TopExp_Explorer edge(tool.Wire(), TopAbs_EDGE); edge.More(); edge.Next()) {
    print_pcurve(TopoDS::Edge(edge.Current()), tool.Face());
  }
  std::cout << '\n';
}

void run_crossing_lines() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0.2, 0), gp_Pnt(2, 1.2, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("line_cross", tool, tool.FixGap2d(2, false));
}

void run_reversed_crossing_lines() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0.2, 0), gp_Pnt(2, 1.2, 0));
  first.Reverse();
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("reversed_line_cross", tool, tool.FixGap2d(2, false));
}

void run_parallel_projection() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1.1, 0.2, 0), gp_Pnt(2.1, 0.2, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("parallel_projection", tool, tool.FixGap2d(2, false));
}

void run_single_circular_arc() {
  BRep_Builder builder;
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.0), 0.0, 1.0);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  double f = 0.0, l = 0.0;
  const auto pc = BRep_Tool::CurveOnSurface(edge, face, f, l);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  const bool changed = tool.FixGap2d(1, false);
  double wf = 0.0, wl = 0.0;
  const TopoDS_Edge wd = tool.WireData()->Edge(1);
  const auto wpc = BRep_Tool::CurveOnSurface(wd, face, wf, wl);
  const gp_Pnt2d data_start = wpc->Value(wf), data_end = wpc->Value(wl);
  std::cout << "single_circular_arc_wiredata " << changed << ' '
            << tool.LastFixStatus(ShapeExtend_DONE1) << ' ' << tool.LastFixStatus(ShapeExtend_FAIL)
            << ' ' << tool.Context().IsNull() << ' ' << std::setprecision(17) << wf << ' ' << wl
            << ' ' << data_start.X() << ' ' << data_start.Y() << ' ' << data_end.X() << ' '
            << data_end.Y() << '\n';
  print_result("single_circular_arc", tool, changed);
}

void run_single_elliptical_arc() {
  BRep_Builder builder;
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Elips(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 3.0, 1.0), 0.0, 1.0);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, edge);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("single_elliptical_arc", tool, tool.FixGap2d(1, false));
}

void run_intersecting_circles() {
  BRep_Builder builder;
  gp_Dir normal(0, 0, 1);
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0, 0, 0), normal), 1.0), 0.0, 0.1);
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(1.8, 0, 0), normal), 1.0), M_PI, M_PI + 0.1);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("intersecting_circles", tool, tool.FixGap2d(2, false));
}

void run_disjoint_circles_extrema() {
  BRep_Builder builder;
  gp_Dir normal(0, 0, 1);
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(0, 0, 0), normal), 1.0), 0.05, 0.1);
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Circ(gp_Ax2(gp_Pnt(2.1, 0, 0), normal), 1.0), M_PI - 0.1, M_PI - 0.05);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("disjoint_circles_extrema", tool, tool.FixGap2d(2, false));
}

void run_trimmed_line_pcurves() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0.2, 0), gp_Pnt(2, 1.2, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  Handle(Geom2d_Curve) firstLine = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)));
  Handle(Geom2d_Curve) secondLine = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(1, 0.2), gp_Dir2d(1, 1)));
  builder.UpdateEdge(first, new Geom2d_TrimmedCurve(firstLine, 0., 1.), face, 1e-7);
  builder.Range(first, face, 0., 1.);
  builder.UpdateEdge(second, new Geom2d_TrimmedCurve(secondLine, 0., 1.4142135623730951), face, 1e-7);
  builder.Range(second, face, 0., 1.4142135623730951);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("trimmed_line_pcurves", tool, tool.FixGap2d(2, false));
}

void run_offset_line_pcurves() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0.2, 0), gp_Pnt(2, 1.2, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  Handle(Geom2d_Curve) firstBase = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0, 0.2), gp_Dir2d(1, 0)));
  Handle(Geom2d_Curve) secondBase = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0.3, -0.2), gp_Dir2d(1, 1)));
  Handle(Geom2d_Curve) firstCurve = new Geom2d_OffsetCurve(firstBase, 0.2, false);
  Handle(Geom2d_Curve) secondCurve = new Geom2d_OffsetCurve(secondBase, 0.2, false);
  builder.UpdateEdge(first, firstCurve, face, 1e-7);
  builder.Range(first, face, 0., 1.);
  builder.UpdateEdge(second, secondCurve, face, 1e-7);
  builder.Range(second, face, 0., 1.4142135623730951);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("offset_line_pcurves", tool, tool.FixGap2d(2, false));
}

void run_bezier_line_pcurves() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(0.3, -0.2, 0), gp_Pnt(1.3, 0.8, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  NCollection_Array1<gp_Pnt2d> firstPoles(1, 2), secondPoles(1, 2);
  firstPoles.SetValue(1, gp_Pnt2d(0, 0));
  firstPoles.SetValue(2, gp_Pnt2d(1, 0));
  secondPoles.SetValue(1, gp_Pnt2d(0.3, -0.2));
  secondPoles.SetValue(2, gp_Pnt2d(1.3, 0.8));
  builder.UpdateEdge(first, new Geom2d_BezierCurve(firstPoles), face, 1e-7);
  builder.Range(first, face, 0., 1.);
  builder.UpdateEdge(second, new Geom2d_BezierCurve(secondPoles), face, 1e-7);
  builder.Range(second, face, 0., 1.);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("bezier_line_pcurves", tool, tool.FixGap2d(2, false));
}

void run_bspline_line_pcurves() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(0.3, -0.2, 0), gp_Pnt(1.3, 0.8, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  NCollection_Array1<gp_Pnt2d> firstPoles(1, 2), secondPoles(1, 2);
  NCollection_Array1<double> knots(1, 2);
  NCollection_Array1<int> multiplicities(1, 2);
  firstPoles.SetValue(1, gp_Pnt2d(0, 0));
  firstPoles.SetValue(2, gp_Pnt2d(1, 0));
  secondPoles.SetValue(1, gp_Pnt2d(0.3, -0.2));
  secondPoles.SetValue(2, gp_Pnt2d(1.3, 0.8));
  knots.SetValue(1, 0.);
  knots.SetValue(2, 1.);
  multiplicities.SetValue(1, 2);
  multiplicities.SetValue(2, 2);
  builder.UpdateEdge(first, new Geom2d_BSplineCurve(firstPoles, knots, multiplicities, 1), face, 1e-7);
  builder.Range(first, face, 0., 1.);
  builder.UpdateEdge(second, new Geom2d_BSplineCurve(secondPoles, knots, multiplicities, 1), face, 1e-7);
  builder.Range(second, face, 0., 1.);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("bspline_line_pcurves", tool, tool.FixGap2d(2, false));
}

void run_tangent_parabola_line() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0.25, 0), gp_Pnt(1, 0.25, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  NCollection_Array1<gp_Pnt2d> parabolaPoles(1, 3);
  parabolaPoles.SetValue(1, gp_Pnt2d(0, 0));
  parabolaPoles.SetValue(2, gp_Pnt2d(0.5, 0.5));
  parabolaPoles.SetValue(3, gp_Pnt2d(1, 0));
  builder.UpdateEdge(first, new Geom2d_BezierCurve(parabolaPoles), face, 1e-7);
  builder.Range(first, face, 0., 1.);
  Handle(Geom2d_Curve) line = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0, 0.25), gp_Dir2d(1, 0)));
  builder.UpdateEdge(second, line, face, 1e-7);
  builder.Range(second, face, 0., 1.);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("tangent_parabola_line", tool, tool.FixGap2d(2, false));
}

void run_tangent_bspline_line() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0.25, 0), gp_Pnt(1, 0.25, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Pln()), 1e-7);
  builder.Add(face, wire);
  NCollection_Array1<gp_Pnt2d> parabolaPoles(1, 3);
  NCollection_Array1<double> knots(1, 2);
  NCollection_Array1<int> multiplicities(1, 2);
  parabolaPoles.SetValue(1, gp_Pnt2d(0, 0));
  parabolaPoles.SetValue(2, gp_Pnt2d(0.5, 0.5));
  parabolaPoles.SetValue(3, gp_Pnt2d(1, 0));
  knots.SetValue(1, 0.);
  knots.SetValue(2, 1.);
  multiplicities.SetValue(1, 3);
  multiplicities.SetValue(2, 3);
  builder.UpdateEdge(first, new Geom2d_BSplineCurve(parabolaPoles, knots, multiplicities, 2), face, 1e-7);
  builder.Range(first, face, 0., 1.);
  Handle(Geom2d_Curve) line = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(0, 0.25), gp_Dir2d(1, 0)));
  builder.UpdateEdge(second, line, face, 1e-7);
  builder.Range(second, face, 0., 1.);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("tangent_bspline_line", tool, tool.FixGap2d(2, false));
}

void run_trimmed_surface_clip() {
  BRep_Builder builder;
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1.1, 0.2, 0), gp_Pnt(2.1, 0.2, 0));
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, first);
  builder.Add(wire, second);
  TopoDS_Face face;
  Handle(Geom_Surface) plane = new Geom_Plane(gp_Pln());
  Handle(Geom_Surface) surface = new Geom_RectangularTrimmedSurface(plane, 0., 1., 0., 1., true, true);
  builder.MakeFace(face, surface, 1e-7);
  builder.Add(face, wire);
  ShapeFix_Wire tool;
  tool.Load(wire);
  tool.SetFace(face);
  print_result("trimmed_surface_clip", tool, tool.FixGap2d(2, false));
}

int main() {
  run_crossing_lines();
  run_reversed_crossing_lines();
  run_single_circular_arc();
  run_single_elliptical_arc();
  run_intersecting_circles();
  run_disjoint_circles_extrema();
  run_trimmed_line_pcurves();
  run_offset_line_pcurves();
  run_bezier_line_pcurves();
  run_bspline_line_pcurves();
  run_tangent_parabola_line();
  run_tangent_bspline_line();
  run_parallel_projection();
  run_trimmed_surface_clip();
}
