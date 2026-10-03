#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Line.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <TopoDS_Edge.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Lin.hxx>
#include <gp_Pnt.hxx>

#include <iomanip>
#include <iostream>

static TopoDS_Edge edge_at(double z, double first, double last, double tolerance) {
  const gp_Lin line(gp_Ax1(gp_Pnt(0, 0, z), gp_Dir(1, 0, 0)));
  BRepBuilderAPI_MakeEdge maker(line, first, last);
  TopoDS_Edge edge = maker.Edge();
  BRep_Builder builder;
  builder.UpdateEdge(edge, tolerance);
  return edge;
}

static void probe(const char* name, const TopoDS_Edge& first, const TopoDS_Edge& second,
                  double tolerance) {
  ShapeAnalysis_CheckSmallFace analyzer;
  double maximum = -1.0;
  const bool found = analyzer.CheckStripEdges(first, second, tolerance, maximum);
  std::cout << name << ' ' << found << ' ' << maximum << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  probe("coincident", edge_at(0.0, 0.0, 1.0, 1e-7), edge_at(0.0, 0.0, 1.0, 1e-7), 1e-3);
  probe("within", edge_at(0.0, 0.0, 1.0, 1e-7), edge_at(5e-4, 0.0, 1.0, 1e-7), 1e-3);
  probe("outside", edge_at(0.0, 0.0, 1.0, 1e-7), edge_at(2e-3, 0.0, 1.0, 1e-7), 1e-3);
  probe("exact_limit", edge_at(0.0, 0.0, 1.0, 1e-7), edge_at(1e-3, 0.0, 1.0, 1e-7), 1e-3);
  probe("native_tolerance", edge_at(0.0, 0.0, 1.0, 2e-3), edge_at(1.5e-3, 0.0, 1.0, 2e-3), -1.0);
}
