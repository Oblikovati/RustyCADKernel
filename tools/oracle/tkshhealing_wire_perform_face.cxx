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
  Handle(Geom_Surface) surface = new Geom_Plane(gp_Pln());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(surface, 1e-7);
  TopoDS_Edge first = edge(surface, gp_Pnt2d(0, 0), gp_Dir2d(1, 0), 2);
  TopoDS_Edge second = edge(surface, gp_Pnt2d(2, 0), gp_Dir2d(0, 1), 2);
  TopoDS_Edge third = edge(surface, gp_Pnt2d(0, 2), gp_Dir2d(1, -1), 2.8284271247461903);
  BRepBuilderAPI_MakeWire builder(first, second, third);
  ShapeFix_Wire fixer(builder.Wire(), face, 1e-7);
  const bool fixed = fixer.Perform();
  std::cout << "face fixed " << fixed << " edges " << fixer.NbEdges()
            << " reorder_done " << fixer.StatusReorder(ShapeExtend_DONE)
            << " small_done " << fixer.StatusSmall(ShapeExtend_DONE)
            << " connected_done " << fixer.StatusConnected(ShapeExtend_DONE)
            << " edge_curves_done " << fixer.StatusEdgeCurves(ShapeExtend_DONE)
            << " degenerated_done " << fixer.StatusDegenerated(ShapeExtend_DONE)
            << " notches_done " << fixer.StatusNotches(ShapeExtend_DONE)
            << " tails_done " << fixer.StatusFixTails(ShapeExtend_DONE)
            << " self_done " << fixer.StatusSelfIntersection(ShapeExtend_DONE)
            << " lacking_done " << fixer.StatusLacking(ShapeExtend_DONE)
            << " closed_done " << fixer.StatusClosed(ShapeExtend_DONE) << '\n';
}
