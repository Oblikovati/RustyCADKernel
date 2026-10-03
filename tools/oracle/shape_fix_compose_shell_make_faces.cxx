#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRep_Builder.hxx>
#include <Geom_Plane.hxx>
#include <NCollection_HArray2.hxx>
#include <ShapeExtend_CompositeSurface.hxx>
#include <ShapeFix_ComposeShell.hxx>
#include <ShapeBuild_ReShape.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <iostream>

static TopoDS_Wire square(double low, double high) {
  const gp_Pnt points[] = {gp_Pnt(low, low, 0), gp_Pnt(high, low, 0),
                           gp_Pnt(high, high, 0), gp_Pnt(low, high, 0)};
  BRepBuilderAPI_MakeWire wire;
  for (int i = 0; i < 4; ++i)
    wire.Add(BRepBuilderAPI_MakeEdge(points[i], points[(i + 1) % 4]).Edge());
  return wire.Wire();
}

int main() {
  const gp_Pln plane;
  TopoDS_Face source = BRepBuilderAPI_MakeFace(plane, square(0.0, 5.0), true).Face();
  BRep_Builder builder;
  TopoDS_Wire hole = square(1.0, 2.0);
  hole.Reverse();
  builder.Add(source, hole);
  Handle(NCollection_HArray2<Handle(Geom_Surface)>) patches =
      new NCollection_HArray2<Handle(Geom_Surface)>(1, 1, 1, 1);
  patches->SetValue(1, 1, new Geom_Plane(plane));
  Handle(ShapeExtend_CompositeSurface) grid = new ShapeExtend_CompositeSurface(patches);
  ShapeFix_ComposeShell composer;
  composer.Init(grid, TopLoc_Location(), source, 1.0e-7);
  composer.ClosedMode() = true;
  composer.SetContext(new ShapeBuild_ReShape);
  const bool changed = composer.Perform();
  const TopoDS_Shape result = composer.Result();
  int faces = 0;
  int wires = 0;
  for (TopExp_Explorer faceIt(result, TopAbs_FACE); faceIt.More(); faceIt.Next()) {
    ++faces;
    for (TopExp_Explorer wireIt(faceIt.Current(), TopAbs_WIRE); wireIt.More(); wireIt.Next())
      ++wires;
  }
  std::cout << "nested_patch changed=" << changed << " faces=" << faces
            << " wires=" << wires << '\n';
}
