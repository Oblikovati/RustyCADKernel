#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static TopoDS_Wire makeWire(bool reverseSecond) {
  const TopoDS_Edge first = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0));
  TopoDS_Edge second = BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0));
  if (reverseSecond) second.Reverse();
  BRepBuilderAPI_MakeWire builder(first, second);
  return builder.Wire();
}

static TopoDS_Wire makeRemovableTailWire() {
  const TopoDS_Edge edges[] = {
    BRepBuilderAPI_MakeEdge(gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0)),
    BRepBuilderAPI_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(2, 1, 0)),
    BRepBuilderAPI_MakeEdge(gp_Pnt(2, 1, 0), gp_Pnt(2, 0, 0)),
    BRepBuilderAPI_MakeEdge(gp_Pnt(2, 0, 0), gp_Pnt(0, 0, 0)),
  };
  BRepBuilderAPI_MakeWire builder;
  for (const TopoDS_Edge& edge : edges) builder.Add(edge);
  return builder.Wire();
}

static int edgeCount(const TopoDS_Shape& shape) {
  int count = 0;
  for (TopExp_Explorer edge(shape, TopAbs_EDGE); edge.More(); edge.Next()) ++count;
  return count;
}

static void probe(const char* name, bool reverseSecond) {
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln()).Face();
  const TopoDS_Wire wire = makeWire(reverseSecond);
  ShapeFix_Wire fixer(wire, face, 1e-7);
  fixer.SetMaxTailWidth(0.1);
  const bool fixed = fixer.FixTails();
  std::cout << name << " fixed " << fixed << " edges " << edgeCount(fixer.Wire())
            << " nb_edges " << fixer.NbEdges()
            << " loaded " << fixer.IsLoaded() << " ready " << fixer.IsReady()
            << " notches_ok " << fixer.StatusNotches(ShapeExtend_OK)
            << " notches_done " << fixer.StatusNotches(ShapeExtend_DONE)
            << " tails_ok " << fixer.StatusFixTails(ShapeExtend_OK)
            << " tails_done " << fixer.StatusFixTails(ShapeExtend_DONE)
            << " last_ok " << fixer.LastFixStatus(ShapeExtend_OK) << '\n';
}

static void probeRemovableTail() {
  const TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln()).Face();
  const TopoDS_Wire wire = makeRemovableTailWire();
  ShapeFix_Wire fixer(wire, face, 1e-7);
  fixer.SetMaxTailWidth(0.1);
  const bool fixed = fixer.FixTails();
  std::cout << "removable_tail fixed " << fixed << " edges " << edgeCount(fixer.Wire())
            << " nb_edges " << fixer.NbEdges()
            << " notches_done " << fixer.StatusNotches(ShapeExtend_DONE)
            << " last_done " << fixer.LastFixStatus(ShapeExtend_DONE) << '\n';
}

int main() {
  probe("same_direction", false);
  probe("opposite_direction", true);
  probeRemovableTail();
}
