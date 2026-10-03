// Evidence probe for native SplitWire when a context removes an edge at 3d097a0.
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <Geom_Plane.hxx>
#include <NCollection_HArray2.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeFix_ComposeShell.hxx>
#include <ShapeFix_WireSegment.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <TopLoc_Location.hxx>
#include <iostream>

class ComposeProbe : public ShapeFix_ComposeShell
{
public:
  using ShapeFix_ComposeShell::LoadWires;
  using ShapeFix_ComposeShell::SplitWire;
};

int main()
{
  BRepBuilderAPI_MakeWire wire;
  gp_Pnt points[] = {gp_Pnt(0, 0, 0), gp_Pnt(2, 0, 0), gp_Pnt(2, 2, 0), gp_Pnt(0, 2, 0)};
  for (int i = 0; i < 4; ++i)
    wire.Add(BRepBuilderAPI_MakeEdge(points[i], points[(i + 1) % 4]).Edge());
  TopoDS_Face face = BRepBuilderAPI_MakeFace(gp_Pln(), wire.Wire(), true).Face();
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
    new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_Plane(gp_Pln()));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  Handle(ShapeBuild_ReShape) context = new ShapeBuild_ReShape;
  ComposeProbe tool;
  tool.SetContext(context);
  tool.Init(grid, TopLoc_Location(), face, 1.0e-7);
  NCollection_Sequence<ShapeFix_WireSegment> segments;
  tool.LoadWires(segments);
  context->Remove(segments(1).Edge(1));
  NCollection_Sequence<int> indices, codes;
  NCollection_Sequence<double> values;
  NCollection_Sequence<TopoDS_Shape> vertices;
  std::cerr << "entering SplitWire with a context-removed edge\n" << std::flush;
  tool.SplitWire(segments(1), indices, values, vertices, codes, true, 1);
}
