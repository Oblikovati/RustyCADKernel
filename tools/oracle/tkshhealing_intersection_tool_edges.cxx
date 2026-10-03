#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Builder.hxx>
#include <BRep_Tool.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeFix_IntersectionTool.hxx>
#include <TopExp.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Vertex.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Geom_Line.hxx>
#include <Geom_Plane.hxx>
#include <gp_Ax1.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Ax3.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_Pln.hxx>

#include <iomanip>
#include <iostream>

int main() {
  std::cout << std::setprecision(17);
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln()).Face();
  TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0)).Edge();
  TopoDS_Vertex split = BRepBuilderAPI_MakeVertex(gp_Pnt(1, 0, 0)).Vertex();
  ShapeFix_IntersectionTool tool(new ShapeBuild_ReShape, 1.e-7);
  TopoDS_Edge first, second;
  bool split_done = tool.SplitEdge(edge, 1.0, split, face, first, second, 1.e-7);
  double first_a = 0, first_b = 0, second_a = 0, second_b = 0;
  if (split_done) {
    BRep_Tool::Range(first, first_a, first_b);
    BRep_Tool::Range(second, second_a, second_b);
  }
  TopoDS_Vertex first_start, first_last, second_first, second_last;
  if (split_done) {
    TopExp::Vertices(first, first_start, first_last, true);
    TopExp::Vertices(second, second_first, second_last, true);
  }
  std::cout << "split " << split_done << " ranges " << first_a << ' ' << first_b << ' '
            << second_a << ' ' << second_b << " same_middle "
            << (split_done && first_last.IsSame(second_first)) << " flags "
            << (split_done && BRep_Tool::SameRange(first)) << ' '
            << (split_done && BRep_Tool::SameParameter(first)) << " vertex_tol "
            << BRep_Tool::Tolerance(split) << '\n';

  TopoDS_Vertex endpoint, other_endpoint;
  TopExp::Vertices(edge, endpoint, other_endpoint, true);
  TopoDS_Edge sentinel = BRepBuilderAPI_MakeEdge(gp_Pnt(5, 0, 0), gp_Pnt(6, 0, 0)).Edge();
  first = sentinel;
  second = sentinel;
  const bool rejected = tool.SplitEdge(edge, 1.0, endpoint, face, first, second, 1.e-7);
  std::cout << "endpoint_rejected " << !rejected << " outputs_preserved "
            << (first.IsSame(sentinel) && second.IsSame(sentinel)) << '\n';

  TopoDS_Edge cut_edge = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0)).Edge();
  bool cut_line = true;
  bool cut_done = tool.CutEdge(cut_edge, 0.0, 0.75, face, cut_line);
  double cut_first = 0, cut_last = 0;
  BRep_Tool::Range(cut_edge, cut_first, cut_last);
  std::cout << "cut " << cut_done << " cut_line " << cut_line << " range " << cut_first << ' '
            << cut_last << '\n';

  BRep_Builder builder;
  TopoDS_Edge trimmed;
  builder.MakeEdge(trimmed, new Geom_Line(gp_Ax1(gp_Pnt(), gp_Dir(1, 0, 0))), 1.e-7);
  builder.UpdateEdge(trimmed,
                     new Geom2d_TrimmedCurve(
                       new Geom2d_Line(gp_Ax2d(gp_Pnt2d(), gp_Dir2d(1, 0))), 0, 1.e9),
                     face,
                     1.e-7);
  builder.Range(trimmed, face, 0, 1.e9);
  builder.Range(trimmed, 0, 1, true);
  builder.SameParameter(trimmed, false);
  bool trimmed_line = false;
  const double trimmed_cut = 1.e9 - 1.e-4;
  const bool trimmed_done = tool.CutEdge(trimmed, 0, trimmed_cut, face, trimmed_line);
  BRep_Tool::Range(trimmed, cut_first, cut_last);
  std::cout << "trimmed_cut " << trimmed_done << " cut_line " << trimmed_line << " range "
            << cut_first << ' ' << cut_last << '\n';
}
