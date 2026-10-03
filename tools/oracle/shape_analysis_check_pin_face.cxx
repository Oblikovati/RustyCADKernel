#include <BRep_Builder.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Plane.hxx>
#include <ShapeAnalysis_CheckSmallFace.hxx>
#include <ShapeExtend.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Pnt.hxx>
#include <gp_Circ.hxx>
#include <NCollection_DataMap.hxx>
#include <iostream>
#include <cmath>

using EdgeMap = NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>;

static TopoDS_Face make_face() {
  BRep_Builder builder;
  TopoDS_Face face;
  builder.MakeFace(face, new Geom_Plane(gp_Ax3()), 1e-7);
  gp_Circ circle(gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(0, 0, 1)), 1.0);
  TopoDS_Edge firstEdge = BRepBuilderAPI_MakeEdge(circle, -0.1, 0.0);
  TopoDS_Edge secondEdge = BRepBuilderAPI_MakeEdge(circle, -0.1, 0.0);
  TopoDS_Vertex first, common, second, unused;
  TopExp::Vertices(firstEdge, first, common);
  TopExp::Vertices(secondEdge, second, unused);
  builder.UpdateVertex(first, 0.02);
  builder.UpdateVertex(common, 0.02);
  builder.UpdateVertex(second, 0.02);
  TopoDS_Wire wire;
  builder.MakeWire(wire);
  builder.Add(wire, firstEdge);
  builder.Add(wire, secondEdge);
  builder.Add(face, wire);
  return face;
}

int main() {
  ShapeAnalysis_CheckSmallFace analyzer;
  analyzer.SetTolerance(0.02);
  TopoDS_Face face = make_face();
  TopoDS_Edge first, second;
  TopExp_Explorer explorer(face, TopAbs_EDGE);
  first = TopoDS::Edge(explorer.Current()); explorer.Next();
  second = TopoDS::Edge(explorer.Current());
  for (TopExp_Explorer it(face, TopAbs_EDGE); it.More(); it.Next()) {
    TopoDS_Edge edge = TopoDS::Edge(it.Current());
    TopoDS_Vertex v1, v2; TopExp::Vertices(edge, v1, v2);
    std::cout << "edge " << BRep_Tool::Pnt(v1).Distance(BRep_Tool::Pnt(v2)) << ' '
              << BRep_Tool::Tolerance(v1) << ' ' << BRep_Tool::Tolerance(v2) << '\n';
  }
  std::cout << "edgeprobe " << analyzer.CheckPinEdges(first, second, 5.0, 5.0, 0.02) << '\n';
  EdgeMap edges;
  const bool done = analyzer.CheckPinFace(face, edges, 0.02);
  std::cout << done << ' ' << edges.Extent() << ' '
            << analyzer.StatusPinFace(ShapeExtend_DONE) << '\n';
}
