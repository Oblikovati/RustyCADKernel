#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static TopoDS_Wire makeWire(bool reverseOrder) {
  const TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(1, 0, 0));
  const TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(1, 0, 0), gp_Pnt(2, 0, 0));
  BRepBuilderAPI_MakeWire builder;
  if (reverseOrder) {
    builder.Add(second);
    builder.Add(first);
  } else {
    builder.Add(first);
    builder.Add(second);
  }
  return builder.Wire();
}

static int countEdges(const TopoDS_Shape& shape) {
  int count = 0;
  for (TopExp_Explorer edge(shape, TopAbs_EDGE); edge.More(); edge.Next()) ++count;
  return count;
}

static void runCase(const char* name, bool reverseOrder) {
  ShapeFix_Wire fixer;
  const TopoDS_Wire wire = makeWire(reverseOrder);
  fixer.Load(wire);
  const bool fixed = fixer.Perform();
  std::cout << name << " fixed " << fixed << " edges " << countEdges(fixer.Wire())
            << " reorder_ok " << fixer.StatusReorder(ShapeExtend_OK)
            << " reorder_done " << fixer.StatusReorder(ShapeExtend_DONE)
            << " connected_ok " << fixer.StatusConnected(ShapeExtend_OK)
            << " connected_done " << fixer.StatusConnected(ShapeExtend_DONE)
            << " edge_curves_ok " << fixer.StatusEdgeCurves(ShapeExtend_OK)
            << " edge_curves_done " << fixer.StatusEdgeCurves(ShapeExtend_DONE)
            << " degenerated_ok " << fixer.StatusDegenerated(ShapeExtend_OK)
            << " degenerated_done " << fixer.StatusDegenerated(ShapeExtend_DONE)
            << " self_ok " << fixer.StatusSelfIntersection(ShapeExtend_OK)
            << " self_done " << fixer.StatusSelfIntersection(ShapeExtend_DONE)
            << " lacking_ok " << fixer.StatusLacking(ShapeExtend_OK)
            << " lacking_done " << fixer.StatusLacking(ShapeExtend_DONE)
            << " notches_ok " << fixer.StatusNotches(ShapeExtend_OK)
            << " notches_done " << fixer.StatusNotches(ShapeExtend_DONE)
            << " tails_ok " << fixer.StatusFixTails(ShapeExtend_OK)
            << " tails_done " << fixer.StatusFixTails(ShapeExtend_DONE)
            << " last_ok " << fixer.LastFixStatus(ShapeExtend_OK) << '\n';
}

int main() {
  ShapeFix_Wire unloaded;
  const bool unloadedFixed = unloaded.Perform();
  std::cout << "unloaded fixed " << unloadedFixed << " edges " << unloaded.NbEdges()
            << " last_ok " << unloaded.LastFixStatus(ShapeExtend_OK) << '\n';
  runCase("ordered", false);
  runCase("reversed_order", true);
}
