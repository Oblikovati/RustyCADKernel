#include <BOPAlgo_Splitter.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Tool.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Vertex.hxx>
#include <Geom_Line.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>

#include <iomanip>
#include <iostream>

static void emitEdges(const char* label, const TopoDS_Shape& shape) {
  for (TopExp_Explorer edges(shape, TopAbs_EDGE); edges.More(); edges.Next()) {
    const TopoDS_Edge edge = TopoDS::Edge(edges.Current());
    TopoDS_Vertex first, last;
    TopExp::Vertices(edge, first, last, true);
    const gp_Pnt firstPoint = BRep_Tool::Pnt(first);
    const gp_Pnt lastPoint = BRep_Tool::Pnt(last);
    std::cout << std::setprecision(17) << label << " edge " << firstPoint.X() << ' '
              << firstPoint.Y() << ' ' << lastPoint.X() << ' ' << lastPoint.Y() << " tol "
              << BRep_Tool::Tolerance(first) << ' ' << BRep_Tool::Tolerance(last) << '\n';
  }
}

int main() {
  const occ::handle<Geom_Line> line = new Geom_Line(gp_Pnt(0, 0, 0), gp_Dir(1, 0, 0));
  const TopoDS_Edge reference = BRepBuilderAPI_MakeEdge(line, 0, 10);
  const TopoDS_Vertex split = BRepBuilderAPI_MakeVertex(gp_Pnt(5, 2e-7, 0));
  BOPAlgo_Splitter splitter;
  splitter.AddArgument(reference);
  splitter.AddTool(split);
  splitter.SetRunParallel(false);
  splitter.Perform();
  emitEdges("splitter", splitter.Shape());
  ShapeUpgrade_UnifySameDomain unify(splitter.Shape());
  unify.Build();
  emitEdges("unified", unify.Shape());
  std::cout << "result counts vertices ";
  int vertices = 0, edges = 0;
  for (TopExp_Explorer explore(unify.Shape(), TopAbs_VERTEX); explore.More(); explore.Next()) ++vertices;
  for (TopExp_Explorer explore(unify.Shape(), TopAbs_EDGE); explore.More(); explore.Next()) ++edges;
  std::cout << vertices << " edges " << edges << '\n';
}
