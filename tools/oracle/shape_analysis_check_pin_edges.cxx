#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Line.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <BRep_Builder.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>

#include <iomanip>
#include <iostream>

static void probe_circle() {
  const double tau = 6.283185307179586476925286766559;
  const auto circle = new Geom_Circle(gp_Circ(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1));
  BRep_Builder builder;
  TopoDS_Vertex shared, first_start, second_start;
  builder.MakeVertex(shared, circle->Value(tau), 1e-3);
  builder.MakeVertex(first_start, circle->Value(tau - 0.1), 1e-3);
  builder.MakeVertex(second_start, circle->Value(tau - 0.2), 1e-3);
  const TopoDS_Edge first = BRepBuilderAPI_MakeEdge(circle, first_start, shared, tau - 0.1, tau).Edge();
  const TopoDS_Edge second = BRepBuilderAPI_MakeEdge(circle, second_start, shared, tau - 0.2, tau).Edge();
  ShapeAnalysis_CheckSmallFace analyzer;
  std::cout << "circle " << analyzer.CheckPinEdges(first, second, 100.0, 200.0, 1e-3) << '\n';
}

static void probe_line() {
  BRep_Builder builder;
  TopoDS_Vertex shared, first_start, second_start;
  builder.MakeVertex(shared, gp_Pnt(0, 0, 0), 1e-3);
  builder.MakeVertex(first_start, gp_Pnt(-0.1, 0, 0), 1e-3);
  builder.MakeVertex(second_start, gp_Pnt(-0.2, 0, 0), 1e-3);
  const auto line = new Geom_Line(gp_Ax1(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0)));
  const TopoDS_Edge first = BRepBuilderAPI_MakeEdge(line, first_start, shared, -0.1, 0.0).Edge();
  const TopoDS_Edge second = BRepBuilderAPI_MakeEdge(line, second_start, shared, -0.2, 0.0).Edge();
  ShapeAnalysis_CheckSmallFace analyzer;
  std::cout << "line " << analyzer.CheckPinEdges(first, second, 100.0, 200.0, 1e-3) << '\n';
}

int main() {
  std::cout << std::setprecision(17);
  probe_circle();
  probe_line();
}
