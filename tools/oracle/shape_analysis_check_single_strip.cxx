#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Line.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

#include <iostream>
#include <vector>

static TopoDS_Edge edge(double z, const TopoDS_Vertex& first, const TopoDS_Vertex& last) {
  const auto line = new Geom_Line(gp_Ax1(gp_Pnt(0, 0, z), gp_Dir(1, 0, 0)));
  return BRepBuilderAPI_MakeEdge(line, first, last, 0.0, 1.0).Edge();
}

static TopoDS_Edge edge_between(const gp_Pnt& first, const gp_Pnt& last,
                               const TopoDS_Vertex& first_vertex,
                               const TopoDS_Vertex& last_vertex) {
  const gp_Dir direction(gp_Vec(first, last));
  const auto line = new Geom_Line(gp_Ax1(first, direction));
  return BRepBuilderAPI_MakeEdge(line, first_vertex, last_vertex,
                                 0.0, first.Distance(last)).Edge();
}

static TopoDS_Face face(const std::vector<TopoDS_Edge>& edges) {
  BRep_Builder builder;
  TopoDS_Wire wire;
  TopoDS_Face result;
  builder.MakeWire(wire);
  builder.MakeFace(result);
  for (const auto& item : edges) builder.Add(wire, item);
  builder.Add(result, wire);
  return result;
}

static void probe(const char* name, const std::vector<TopoDS_Edge>& edges) {
  ShapeAnalysis_CheckSmallFace analyzer;
  TopoDS_Edge first, second;
  const bool found = analyzer.CheckSingleStrip(face(edges), first, second, 1e-3);
  std::cout << name << ' ' << found << ' ' << analyzer.StatusStrip(ShapeExtend_DONE3) << ' ' 
            << first.IsNull() << ' ' << second.IsNull();
  if (!first.IsNull()) std::cout << ' ' << first.IsSame(edges[0]);
  if (!second.IsNull()) std::cout << ' ' << second.IsSame(edges[1]);
  std::cout << std::endl;
}

int main() {
  BRep_Builder builder;
  TopoDS_Vertex first, last, extra;
  builder.MakeVertex(first, gp_Pnt(0, 0, 0), 3e-3);
  builder.MakeVertex(last, gp_Pnt(1, 0, 0), 3e-3);
  builder.MakeVertex(extra, gp_Pnt(1, 1, 0), 3e-3);
  const TopoDS_Edge lower = edge(0.0, first, last);
  const TopoDS_Edge upper = edge(5e-4, first, last);
  probe("two_strip", {lower, upper});
  probe("one_edge", {lower});
  probe("three_vertices", {lower, edge_between(gp_Pnt(0, 0, 0), gp_Pnt(1, 1, 0), first, extra)});
  probe("three_edges", {lower, upper, lower});
  probe("too_far", {lower, edge(2e-3, first, last)});
}
