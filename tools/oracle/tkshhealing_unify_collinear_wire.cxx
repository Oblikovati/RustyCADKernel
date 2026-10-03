#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Tool.hxx>
#include <ShapeUpgrade_UnifySameDomain.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>

#include <iomanip>
#include <iostream>

static TopoDS_Wire makeWire(const gp_Pnt& middle) {
  TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), middle);
  BRepBuilderAPI_MakeWire wireBuilder;
  wireBuilder.Add(first);
  wireBuilder.Add(second);
  return wireBuilder.Wire();
}

static void runCase(const char* name, const TopoDS_Wire& wire, bool unifyEdges) {
  ShapeUpgrade_UnifySameDomain unifier(wire, unifyEdges, false, false);
  unifier.Build();

  const TopoDS_Shape result = unifier.Shape();
  int edgeCount = 0;
  for (TopExp_Explorer explorer(result, TopAbs_EDGE); explorer.More(); explorer.Next()) {
    ++edgeCount;
  }
  std::cout << name << " edge_count " << edgeCount << '\n';
  TopExp_Explorer explorer(result, TopAbs_EDGE);
  if (explorer.More()) {
    TopoDS_Vertex firstVertex, lastVertex;
    TopExp::Vertices(TopoDS::Edge(explorer.Current()), firstVertex, lastVertex, true);
    const gp_Pnt firstPoint = BRep_Tool::Pnt(firstVertex);
    const gp_Pnt lastPoint = BRep_Tool::Pnt(lastVertex);
    std::cout << std::setprecision(17) << name << " endpoints " << firstPoint.X() << ' '
              << firstPoint.Y() << ' ' << firstPoint.Z() << ' ' << lastPoint.X() << ' '
              << lastPoint.Y() << ' ' << lastPoint.Z() << '\n';
  }
}

int main() {
  runCase("linear", makeWire(gp_Pnt(2, 0, 0)), true);
  runCase("disabled", makeWire(gp_Pnt(2, 0, 0)), false);
  runCase("perpendicular", makeWire(gp_Pnt(1, 1, 0)), true);
}
