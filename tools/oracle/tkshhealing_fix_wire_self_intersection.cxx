#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_Plane.hxx>
#include <ShapeExtend_Status.hxx>
#include <ShapeFix_Wire.hxx>
#include <gp_Ax2d.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt2d.hxx>
#include <iostream>

static TopoDS_Edge edge(const Handle(Geom_Surface)& surface, gp_Pnt2d origin, gp_Dir2d direction, double length) {
  Handle(Geom2d_Line) curve = new Geom2d_Line(gp_Ax2d(origin, direction));
  return BRepBuilderAPI_MakeEdge(curve, surface, 0, length);
}

int main() {
  ShapeFix_Wire modes;
  modes.FixSelfIntersectingEdgeMode() = 1;
  modes.FixIntersectingEdgesMode() = 0;
  modes.FixNonAdjacentIntersectingEdgesMode() = -1;
  std::cout << "modes self_edge " << modes.FixSelfIntersectingEdgeMode()
            << " adjacent " << modes.FixIntersectingEdgesMode()
            << " nonadjacent " << modes.FixNonAdjacentIntersectingEdgesMode() << '\n';
  Handle(Geom_Surface) surface = new Geom_Plane(gp_Pln());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(surface, 1e-7);
  TopoDS_Edge first = edge(surface, gp_Pnt2d(0, 0), gp_Dir2d(1, 0), 2);
  TopoDS_Edge second = edge(surface, gp_Pnt2d(2, 0), gp_Dir2d(0, 1), 2);
  TopoDS_Edge third = edge(surface, gp_Pnt2d(0, 2), gp_Dir2d(1, -1), 2.8284271247461903);
  BRepBuilderAPI_MakeWire builder(first, second, third);
  ShapeFix_Wire fixer(builder.Wire(), face, 1e-7);
  const bool fixed = fixer.FixSelfIntersection();
  std::cout << "simple fixed " << fixed << " edges " << fixer.NbEdges()
            << " done " << fixer.StatusSelfIntersection(ShapeExtend_DONE)
            << " fail " << fixer.StatusSelfIntersection(ShapeExtend_FAIL)
            << " removed " << fixer.StatusRemovedSegment() << '\n';
  ShapeFix_Wire empty;
  std::cout << "unready fixed " << empty.FixSelfIntersection()
            << " done " << empty.StatusSelfIntersection(ShapeExtend_DONE)
            << " fail " << empty.StatusSelfIntersection(ShapeExtend_FAIL) << '\n';
}
