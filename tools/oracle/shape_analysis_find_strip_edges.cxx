#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Lin.hxx>
#include <gp_Pnt.hxx>

#include <iomanip>
#include <iostream>
#include <vector>

static TopoDS_Edge edge_at(double z, double last, double tolerance) {
  const gp_Lin line(gp_Ax1(gp_Pnt(0, 0, z), gp_Dir(1, 0, 0)));
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(line, 0.0, last).Edge();
  BRep_Builder{}.UpdateEdge(edge, tolerance);
  return edge;
}

static TopoDS_Face face_with_edges(const std::vector<TopoDS_Edge>& edges) {
  BRep_Builder builder;
  TopoDS_Face face;
  TopoDS_Wire wire;
  builder.MakeFace(face);
  builder.MakeWire(wire);
  for (const TopoDS_Edge& edge : edges) builder.Add(wire, edge);
  builder.Add(face, wire);
  return face;
}

static void probe(const char* name, const std::vector<TopoDS_Edge>& edges, double tolerance) {
  ShapeAnalysis_CheckSmallFace analyzer;
  TopoDS_Edge first, second;
  double maximum = -1.0;
  const bool found = analyzer.FindStripEdges(face_with_edges(edges), first, second, tolerance, maximum);
  std::cout << name << ' ' << found << ' ' << analyzer.StatusStrip(ShapeExtend_DONE3) << ' '
            << maximum << ' ' << first.IsNull() << ' ' << second.IsNull() << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  const TopoDS_Edge first = edge_at(0.0, 1.0, 1e-7);
  const TopoDS_Edge close = edge_at(5e-4, 1.0, 1e-7);
  probe("two_strip", {first, close}, 1e-3);
  probe("seam_strip", {first, first, close}, 1e-3);
  probe("too_far", {first, edge_at(2e-3, 1.0, 1e-7)}, 1e-3);
  probe("third_edge", {first, close, edge_at(0.0, 1.0, 1e-7)}, 1e-3);
  probe("small_edge_skipped", {edge_at(0.0, 1e-4, 1e-7), first, close}, 1e-3);
  probe("one_edge", {first}, 1e-3);
}
