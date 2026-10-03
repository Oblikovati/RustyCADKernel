#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeFix_Wire.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt2d.hxx>
#include <iostream>

static TopoDS_Edge edge(const Handle(Geom_Surface)& surface, double x, double direction, double length) {
  Handle(Geom2d_Line) curve = new Geom2d_Line(gp_Ax2d(gp_Pnt2d(x, 0), gp_Dir2d(direction, 0)));
  return BRepBuilderAPI_MakeEdge(curve, surface, 0, length);
}

int main() {
  ShapeFix_Wire modes;
  modes.FixSmallMode() = 1;
  modes.FixConnectedMode() = 2;
  modes.FixEdgeCurvesMode() = 3;
  modes.FixDegeneratedMode() = 4;
  modes.FixSelfIntersectionMode() = 5;
  modes.FixLackingMode() = 6;
  std::cout << "modes small " << modes.FixSmallMode() << " connected " << modes.FixConnectedMode()
            << " edge_curves " << modes.FixEdgeCurvesMode() << " degenerated "
            << modes.FixDegeneratedMode() << " self_intersection " << modes.FixSelfIntersectionMode()
            << " lacking " << modes.FixLackingMode() << " removed " << modes.StatusRemovedSegment()
            << '\n';
  Handle(Geom_Surface) surface = new Geom_Plane(gp_Pln());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(surface, 1e-7);
  TopoDS_Edge first = edge(surface, 0, 1, 2);
  TopoDS_Edge overlap = edge(surface, 1, 1, 1);
  overlap.Reverse();
  TopoDS_Edge close = edge(surface, 1, -1, 1);
  BRepBuilderAPI_MakeWire builder(first, overlap, close);
  std::cout << "wire_done " << builder.IsDone() << '\n';
  TopoDS_Wire wire = builder.Wire();
  ShapeFix_Wire fixer(wire, face, 1e-7);
  const bool fixed = fixer.FixNotchedEdges();
  int edges = 0;
  for (TopExp_Explorer it(fixer.Wire(), TopAbs_EDGE); it.More(); it.Next()) ++edges;
  std::cout << "notch fixed " << fixed << " edges " << edges
            << " nb_edges " << fixer.NbEdges()
            << " done1 " << fixer.LastFixStatus(ShapeExtend_DONE1)
            << " done2 " << fixer.LastFixStatus(ShapeExtend_DONE2)
            << " notches_done " << fixer.StatusNotches(ShapeExtend_DONE)
            << " last_ok " << fixer.LastFixStatus(ShapeExtend_OK) << '\n';
}
