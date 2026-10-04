#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Line.hxx>
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
  run_parallel_projection();
  run_trimmed_surface_clip();
}
