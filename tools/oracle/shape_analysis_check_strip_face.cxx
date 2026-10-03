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

#include <iostream>
#include <vector>

static TopoDS_Edge loose_edge(double z) {
  const auto line = new Geom_Line(gp_Ax1(gp_Pnt(0, 0, z), gp_Dir(1, 0, 0)));
  return BRepBuilderAPI_MakeEdge(line, 0.0, 1.0).Edge();
}

static TopoDS_Edge shared_edge(double z, const TopoDS_Vertex& first, const TopoDS_Vertex& last) {
  const auto line = new Geom_Line(gp_Ax1(gp_Pnt(0, 0, z), gp_Dir(1, 0, 0)));
  return BRepBuilderAPI_MakeEdge(line, first, last, 0.0, 1.0).Edge();
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

static void probe(const char* name, const TopoDS_Face& input, const TopoDS_Edge& expected_first,
                  const TopoDS_Edge& expected_second) {
  ShapeAnalysis_CheckSmallFace analyzer;
  TopoDS_Edge first, second;
  const bool found = analyzer.CheckStripFace(input, first, second, 1e-3);
  std::cout << name << ' ' << found << ' ' << analyzer.StatusStrip(ShapeExtend_DONE3) << ' '
            << first.IsNull() << ' ' << second.IsNull() << ' '
            << (!first.IsNull() && first.IsSame(expected_first)) << ' '
            << (!second.IsNull() && second.IsSame(expected_second)) << '\n';
}

int main() {
  BRep_Builder builder;
  TopoDS_Vertex first, last;
  builder.MakeVertex(first, gp_Pnt(0, 0, 0), 3e-3);
  builder.MakeVertex(last, gp_Pnt(1, 0, 0), 3e-3);
  const TopoDS_Edge shared_first = shared_edge(0.0, first, last);
  const TopoDS_Edge shared_second = shared_edge(5e-4, first, last);
  probe("single_path", face({shared_first, shared_second}), shared_first, shared_second);
  const TopoDS_Edge loose_first = loose_edge(0.0);
  const TopoDS_Edge loose_second = loose_edge(5e-4);
  probe("fallback_path", face({loose_first, loose_second}), loose_first, loose_second);
  probe("one_edge", face({loose_first}), loose_first, TopoDS_Edge());
}
